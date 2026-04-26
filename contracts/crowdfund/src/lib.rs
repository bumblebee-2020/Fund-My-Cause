#![no_std]
#![allow(missing_docs)]
#![allow(clippy::too_many_arguments)]

mod errors;
mod storage;
mod types;

pub use errors::ContractError;
pub use storage::{CONTRACT_VERSION, KEY_ADMIN, KEY_CONTRIBS, KEY_CREATOR, KEY_DEADLINE, KEY_DESC, KEY_GOAL, KEY_MIN, KEY_PLATFORM, KEY_SOCIAL, KEY_STATUS, KEY_TITLE, KEY_TOKEN, KEY_TOTAL};
pub use types::{CampaignInfo, CampaignStats, DataKey, ExtensionProposal, PlatformConfig, RecurringPlan, Status};

use soroban_sdk::{contract, contractimpl, token, Address, Env, String, Vec};

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct CrowdfundContract;

#[contractimpl]
impl CrowdfundContract {
    /// Initializes a new crowdfunding campaign.
    ///
    /// Creates a campaign with the specified parameters. Can only be called once per contract instance.
    /// The creator must authorize this transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `creator` - The campaign creator's Stellar address (must authorize)
    /// * `token` - The token address for contributions (e.g., native XLM or custom token)
    /// * `goal` - The funding goal in stroops (must be > 0)
    /// * `deadline` - Unix timestamp (seconds) when the campaign ends (must be > current ledger time)
    /// * `min_contribution` - Minimum contribution amount in stroops (must be >= 0)
    /// * `title` - Campaign title
    /// * `description` - Campaign description
    /// * `social_links` - Optional list of social media URLs
    /// * `platform_config` - Optional platform fee configuration (address and fee_bps)
    /// * `accepted_tokens` - Optional whitelist of accepted token addresses
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::AlreadyInitialized)` if campaign already initialized
    /// * `Err(ContractError::InvalidGoal)` if goal <= 0
    /// * `Err(ContractError::InvalidDeadline)` if deadline <= current time
    /// * `Err(ContractError::InvalidFee)` if platform fee_bps > 10,000
    ///
    /// # Example
    /// ```ignore
    /// initialize(
    ///     env,
    ///     creator_address,
    ///     token_address,
    ///     1_000_000_000,  // 100 XLM goal
    ///     1704067200,     // deadline timestamp
    ///     1_000_000,      // 0.1 XLM minimum
    ///     String::from_str(&env, "My Campaign"),
    ///     String::from_str(&env, "Help fund my project"),
    ///     None,
    ///     None,
    ///     None,
    /// )
    /// ```
    pub fn initialize(
        env: Env,
        creator: Address,
        token: Address,
        goal: i128,
        deadline: u64,
        min_contribution: i128,
        title: String,
        description: String,
        social_links: Option<Vec<String>>,
        platform_config: Option<PlatformConfig>,
        accepted_tokens: Option<Vec<Address>>,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&KEY_CREATOR) {
            return Err(ContractError::AlreadyInitialized);
        }
        creator.require_auth();

        if goal <= 0 {
            return Err(ContractError::InvalidGoal);
        }
        if deadline <= env.ledger().timestamp() {
            return Err(ContractError::InvalidDeadline);
        }
        if min_contribution < 0 {
            return Err(ContractError::BelowMinimum);
        }

        if let Some(ref config) = platform_config {
            if config.fee_bps > 10_000 {
                return Err(ContractError::InvalidFee);
            }
            env.storage().instance().set(&KEY_PLATFORM, config);
        }

        env.storage().instance().set(&KEY_ADMIN, &creator);
        env.storage().instance().set(&KEY_CREATOR, &creator);
        env.storage().instance().set(&KEY_TOKEN, &token);
        env.storage().instance().set(&KEY_GOAL, &goal);
        env.storage().instance().set(&KEY_DEADLINE, &deadline);
        env.storage().instance().set(&KEY_MIN, &min_contribution);
        env.storage().instance().set(&KEY_TITLE, &title);
        env.storage().instance().set(&KEY_DESC, &description);
        env.storage().instance().set(&KEY_TOTAL, &0i128);
        env.storage().instance().set(&KEY_STATUS, &Status::Active);
        env.storage().instance().set(&DataKey::ContributorCount, &0u32);
        env.storage().instance().set(&DataKey::LargestContribution, &0i128);

        if let Some(links) = social_links {
            env.storage().instance().set(&KEY_SOCIAL, &links);
        }

        env.storage().instance().set(&DataKey::ContributorCount, &0u32);
        env.storage().instance().set(&DataKey::LargestContribution, &0i128);

        if let Some(tokens) = accepted_tokens {
            env.storage().instance().set(&DataKey::AcceptedTokens, &tokens);
        }

        let empty: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&KEY_CONTRIBS, &empty);

        env.events().publish(("campaign", "initialized"), ());
        Ok(())
    }

    /// Submits a contribution to the campaign.
    ///
    /// Allows a contributor to pledge tokens before the campaign deadline.
    /// The contributor must authorize this transaction and have sufficient token balance.
    /// Uses a pull-based refund model: contributors claim refunds individually if the goal is not met.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's Stellar address (must authorize)
    /// * `amount` - Contribution amount in stroops (must be >= min_contribution)
    /// * `token` - The token address being contributed (must match campaign token or be in whitelist)
    /// * `message` - Optional message/memo attached to the contribution (max 256 chars)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::BelowMinimum)` if amount < min_contribution
    /// * `Err(ContractError::CampaignPaused)` if campaign is paused
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    /// * `Err(ContractError::CampaignEnded)` if current time >= deadline
    /// * `Err(ContractError::TokenNotAccepted)` if token not in whitelist
    /// * `Err(ContractError::Overflow)` if total raised would overflow
    /// * `Err(ContractError::MessageTooLong)` if message exceeds 256 characters
    ///
    /// # Side Effects
    /// - Transfers tokens from contributor to contract
    /// - Updates contributor's total contribution amount
    /// - Stores contribution message if provided
    /// - Increments contributor count if this is their first contribution
    /// - Updates largest contribution if applicable
    /// - Publishes "contributed" event
    pub fn contribute(env: Env, contributor: Address, amount: i128, token: Address, message: Option<String>) -> Result<(), ContractError> {
        contributor.require_auth();

        if let Some(ref msg) = message {
            if msg.len() > 256 {
                return Err(ContractError::MessageTooLong);
            }
        }

        let min: i128 = env.storage().instance().get(&KEY_MIN).unwrap();
        if amount < min {
            return Err(ContractError::BelowMinimum);
        }

        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        if status == Status::Paused {
            return Err(ContractError::CampaignPaused);
        }
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }

        let deadline: u64 = env.storage().instance().get(&KEY_DEADLINE).unwrap();
        if env.ledger().timestamp() >= deadline {
            return Err(ContractError::CampaignEnded);
        }

        // Validate token against whitelist if one is set, otherwise fall back to default token
        let default_token: Address = env.storage().instance().get(&KEY_TOKEN).unwrap();
        if let Some(whitelist) = env.storage().instance().get::<_, Vec<Address>>(&DataKey::AcceptedTokens) {
            if !whitelist.contains(&token) {
                return Err(ContractError::TokenNotAccepted);
            }
        } else if token != default_token {
            return Err(ContractError::TokenNotAccepted);
        }

        token::Client::new(&env, &token)
            .transfer(&contributor, &env.current_contract_address(), &amount);

        let key = DataKey::Contribution(contributor.clone());
        let prev: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        let new_amount = prev.checked_add(amount).ok_or(ContractError::Overflow)?;
        env.storage().persistent().set(&key, &new_amount);
        env.storage().persistent().extend_ttl(&key, 100, 100);

        if let Some(msg) = message {
            let msg_key = DataKey::ContributionMessage(contributor.clone());
            env.storage().persistent().set(&msg_key, &msg);
            env.storage().persistent().extend_ttl(&msg_key, 100, 100);
        }

        let total: i128 = env.storage().instance().get(&KEY_TOTAL).unwrap();
        let new_total = total.checked_add(amount).ok_or(ContractError::Overflow)?;
        env.storage().instance().set(&KEY_TOTAL, &new_total);

        let presence_key = DataKey::ContributorPresence(contributor.clone());
        let is_present: bool = env.storage().persistent().get(&presence_key).unwrap_or(false);
        if !is_present {
            env.storage().persistent().set(&presence_key, &true);
            env.storage().persistent().extend_ttl(&presence_key, 100, 100);
            let count: u32 = env.storage().instance().get(&DataKey::ContributorCount).unwrap();
            env.storage().instance().set(&DataKey::ContributorCount, &(count + 1));

            let mut contributors: Vec<Address> = env
                .storage()
                .persistent()
                .get(&KEY_CONTRIBS)
                .unwrap_or_else(|| Vec::new(&env));
            contributors.push_back(contributor.clone());
            env.storage().persistent().set(&KEY_CONTRIBS, &contributors);
            env.storage().persistent().extend_ttl(&KEY_CONTRIBS, 100, 100);
        }

        let largest: i128 = env.storage().instance().get(&DataKey::LargestContribution).unwrap();
        if new_amount > largest {
            env.storage().instance().set(&DataKey::LargestContribution, &new_amount);
        }

        let mut contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&KEY_CONTRIBS)
            .unwrap_or_else(|| Vec::new(&env));
        if !contributors.contains(&contributor) {
            contributors.push_back(contributor.clone());
            env.storage().persistent().set(&KEY_CONTRIBS, &contributors);
            env.storage().persistent().extend_ttl(&KEY_CONTRIBS, 100, 100);
        }

        env.storage().instance().extend_ttl(17280, 518400);
        env.events().publish(("campaign", "contributed"), (contributor, amount));
        Ok(())
    }

    /// Withdraws raised funds to the campaign creator after a successful campaign.
    ///
    /// Can only be called after the deadline has passed and the goal has been reached.
    /// The creator must authorize this transaction.
    /// If a platform fee is configured, it is deducted from the total before payout.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    /// * `Err(ContractError::CampaignStillActive)` if current time < deadline
    /// * `Err(ContractError::GoalNotReached)` if total_raised < goal
    ///
    /// # Side Effects
    /// - Transfers platform fee to platform address (if configured)
    /// - Transfers remaining funds to creator
    /// - Sets campaign status to Successful
    /// - Resets total_raised to 0
    /// - Publishes "withdrawn" event
    ///
    /// # Platform Fee Calculation
    /// If platform_config is set:
    /// ```ignore
    /// fee = total_raised * platform_fee_bps / 10_000
    /// creator_payout = total_raised - fee
    /// ```
    pub fn withdraw(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }

        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        let deadline: u64 = env.storage().instance().get(&KEY_DEADLINE).unwrap();
        if env.ledger().timestamp() < deadline {
            return Err(ContractError::CampaignStillActive);
        }

        let goal: i128 = env.storage().instance().get(&KEY_GOAL).unwrap();
        let total: i128 = env.storage().instance().get(&KEY_TOTAL).unwrap();
        if total < goal {
            return Err(ContractError::GoalNotReached);
        }

        let token_address: Address = env.storage().instance().get(&KEY_TOKEN).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        let payout = if let Some(config) = env.storage().instance().get::<_, PlatformConfig>(&KEY_PLATFORM) {
            let fee = total * config.fee_bps as i128 / 10_000;
            token_client.transfer(&env.current_contract_address(), &config.address, &fee);
            total - fee
        } else {
            total
        };

        token_client.transfer(&env.current_contract_address(), &creator, &payout);

        // Extend instance storage TTL after successful withdrawal.
        // This ensures contract metadata remains accessible for historical reference
        // and potential future interactions (e.g., viewing campaign results).
        // Uses same TTL strategy as contribute: threshold 17280, extension 518400 ledgers.
        env.storage().instance().extend_ttl(17280, 518400);

        env.storage().instance().set(&KEY_TOTAL, &0i128);
        env.storage().instance().set(&KEY_STATUS, &Status::Successful);
        env.storage().instance().extend_ttl(17280, 518400);
        env.events().publish(("campaign", "withdrawn"), (creator, total));
        Ok(())
    }

    /// Updates campaign metadata (title, description, social links).
    ///
    /// Can only be called while the campaign is in Active status.
    /// The creator must authorize this transaction.
    /// Any field can be omitted (None) to leave it unchanged.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `title` - New campaign title (optional)
    /// * `description` - New campaign description (optional)
    /// * `social_links` - New social media links (optional)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    ///
    /// # Side Effects
    /// - Updates specified metadata fields in storage
    /// - Publishes "metadata_updated" event
    pub fn update_metadata(
        env: Env,
        title: Option<String>,
        description: Option<String>,
        social_links: Option<Vec<String>>,
    ) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        if let Some(t) = title { env.storage().instance().set(&KEY_TITLE, &t); }
        if let Some(d) = description { env.storage().instance().set(&KEY_DESC, &d); }
        if let Some(l) = social_links { env.storage().instance().set(&KEY_SOCIAL, &l); }

        env.events().publish(("campaign", "metadata_updated"), ());
        Ok(())
    }

    /// Extends the campaign deadline to a later time.
    ///
    /// Can only be called while the campaign is in Active status.
    /// The creator must authorize this transaction.
    /// The new deadline must be strictly greater than the current deadline.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `new_deadline` - New Unix timestamp (seconds) for campaign end
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    /// * `Err(ContractError::InvalidDeadline)` if new_deadline <= current_deadline
    ///
    /// # Side Effects
    /// - Updates deadline in storage
    /// - Publishes "deadline_extended" event with new deadline
    pub fn extend_deadline(env: Env, new_deadline: u64) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        let current_deadline: u64 = env.storage().instance().get(&KEY_DEADLINE).unwrap();
        if new_deadline <= current_deadline {
            return Err(ContractError::InvalidDeadline);
        }
        env.storage().instance().set(&KEY_DEADLINE, &new_deadline);
        env.events().publish(("campaign", "deadline_extended"), new_deadline);
        Ok(())
    }

    /// Cancels the campaign, allowing all contributors to claim refunds.
    ///
    /// Can only be called while the campaign is in Active status.
    /// The creator must authorize this transaction.
    /// After cancellation, contributors can call `refund_single` to claim their refunds.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    ///
    /// # Side Effects
    /// - Sets campaign status to Cancelled
    /// - Publishes "cancelled" event
    pub fn cancel_campaign(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();
        env.storage().instance().set(&KEY_STATUS, &Status::Cancelled);
        env.events().publish(("campaign", "cancelled"), ());
        Ok(())
    }

    /// Claims a refund for a single contributor (pull-based refund model).
    ///
    /// A contributor can claim their refund if:
    /// - The campaign was cancelled, OR
    /// - The deadline has passed AND the goal was not reached
    ///
    /// This implements a pull-based refund model where each contributor individually
    /// claims their refund, avoiding the gas cost and failure risk of a single
    /// transaction refunding all contributors.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's Stellar address claiming the refund
    ///
    /// # Returns
    /// * `Ok(())` on success (even if contributor has no refund)
    /// * `Err(ContractError::CampaignStillActive)` if deadline not passed and not cancelled
    /// * `Err(ContractError::GoalReached)` if goal was reached and campaign not cancelled
    ///
    /// # Side Effects
    /// - Transfers refund amount to contributor (if > 0)
    /// - Sets contributor's contribution to 0
    /// - Publishes "refunded" event
    pub fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();

        if status != Status::Cancelled {
            let deadline: u64 = env.storage().instance().get(&KEY_DEADLINE).unwrap();
            if env.ledger().timestamp() < deadline {
                return Err(ContractError::CampaignStillActive);
            }
            let goal: i128 = env.storage().instance().get(&KEY_GOAL).unwrap();
            let total: i128 = env.storage().instance().get(&KEY_TOTAL).unwrap();
            if total >= goal {
                return Err(ContractError::GoalReached);
            }
        }

        let key = DataKey::Contribution(contributor.clone());
        let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if amount > 0 {
            let token_address: Address = env.storage().instance().get(&KEY_TOKEN).unwrap();
            token::Client::new(&env, &token_address)
                .transfer(&env.current_contract_address(), &contributor, &amount);
            env.storage().persistent().set(&key, &0i128);
            env.events().publish(("campaign", "refunded"), (contributor, amount));
        }
        Ok(())
    }

    /// Refunds multiple contributors in a single transaction (batch refund).
    ///
    /// Processes refunds for a list of contributors. Stops early if the batch
    /// limit is reached to avoid exceeding resource limits.
    /// Each contributor is only refunded if eligible (same conditions as `refund_single`).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributors` - List of contributor addresses to refund
    ///
    /// # Returns
    /// * `Ok(u32)` - Number of contributors successfully refunded
    /// * `Err(ContractError::CampaignStillActive)` if deadline not passed and not cancelled
    /// * `Err(ContractError::GoalReached)` if goal was reached and campaign not cancelled
    pub fn refund_batch(env: Env, contributors: Vec<Address>) -> Result<u32, ContractError> {
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();

        if status != Status::Cancelled {
            let deadline: u64 = env.storage().instance().get(&KEY_DEADLINE).unwrap();
            if env.ledger().timestamp() < deadline {
                return Err(ContractError::CampaignStillActive);
            }
            let goal: i128 = env.storage().instance().get(&KEY_GOAL).unwrap();
            let total: i128 = env.storage().instance().get(&KEY_TOTAL).unwrap();
            if total >= goal {
                return Err(ContractError::GoalReached);
            }
        }

        let token_address: Address = env.storage().instance().get(&KEY_TOKEN).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        // Cap batch size to avoid resource exhaustion
        const MAX_BATCH: u32 = 25;
        let limit = contributors.len().min(MAX_BATCH);
        let mut refunded: u32 = 0;

        for i in 0..limit {
            let contributor = contributors.get(i).unwrap();
            let key = DataKey::Contribution(contributor.clone());
            let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
            if amount > 0 {
                token_client.transfer(&env.current_contract_address(), &contributor, &amount);
                env.storage().persistent().set(&key, &0i128);
                env.events().publish(("campaign", "refunded"), (contributor, amount));
                refunded += 1;
            }
        }

        Ok(refunded)
    }

    /// Pauses the campaign, preventing new contributions.
    ///
    /// Can only be called while the campaign is in Active status.
    /// The admin (creator) must authorize this transaction.
    /// While paused, contributors cannot make new contributions.
    /// The campaign can be resumed with `unpause`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Active status
    ///
    /// # Side Effects
    /// - Sets campaign status to Paused
    /// - Publishes "paused" event
    pub fn pause(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        if status != Status::Active {
            return Err(ContractError::NotActive);
        }
        let admin: Address = env.storage().instance().get(&KEY_ADMIN).unwrap();
        admin.require_auth();
        env.storage().instance().set(&KEY_STATUS, &Status::Paused);
        env.events().publish(("campaign", "paused"), ());
        Ok(())
    }

    /// Resumes a paused campaign, allowing contributions again.
    ///
    /// Can only be called while the campaign is in Paused status.
    /// The admin (creator) must authorize this transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::NotActive)` if campaign is not in Paused status
    ///
    /// # Side Effects
    /// - Sets campaign status to Active
    /// - Publishes "unpaused" event
    pub fn unpause(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        if status != Status::Paused {
            return Err(ContractError::NotActive);
        }
        let admin: Address = env.storage().instance().get(&KEY_ADMIN).unwrap();
        admin.require_auth();
        env.storage().instance().set(&KEY_STATUS, &Status::Active);
        env.events().publish(("campaign", "unpaused"), ());
        Ok(())
    }

    /// Sets up a recurring contribution plan for a contributor.
    ///
    /// Allows a contributor to schedule automatic contributions at regular intervals.
    /// The contributor must authorize this transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's Stellar address (must authorize)
    /// * `amount` - Amount to contribute each interval in stroops
    /// * `interval` - Interval in seconds between contributions
    /// * `end_date` - Unix timestamp when recurring contributions should stop
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::InvalidRecurringPlan)` if parameters are invalid
    ///
    /// # Side Effects
    /// - Stores recurring plan in persistent storage
    /// - Publishes "recurring_setup" event
    pub fn setup_recurring(env: Env, contributor: Address, amount: i128, interval: u64, end_date: u64) -> Result<(), ContractError> {
        contributor.require_auth();

        if amount <= 0 || interval == 0 || end_date <= env.ledger().timestamp() {
            return Err(ContractError::InvalidRecurringPlan);
        }

        let plan = RecurringPlan {
            amount,
            interval,
            end_date,
            last_executed: env.ledger().timestamp(),
        };

        let key = DataKey::RecurringPlan(contributor.clone());
        env.storage().persistent().set(&key, &plan);
        env.storage().persistent().extend_ttl(&key, 100, 100);

        env.events().publish(("campaign", "recurring_setup"), (contributor, amount, interval));
        Ok(())
    }

    /// Executes pending recurring contributions for a contributor.
    ///
    /// Can be called by anyone to trigger scheduled contributions.
    /// Only executes if the interval has passed since last execution.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::InvalidRecurringPlan)` if no plan exists or plan expired
    pub fn execute_recurring(env: Env, contributor: Address) -> Result<(), ContractError> {
        let key = DataKey::RecurringPlan(contributor.clone());
        let mut plan: RecurringPlan = env.storage().persistent().get(&key)
            .ok_or(ContractError::InvalidRecurringPlan)?;

        let now = env.ledger().timestamp();
        if now > plan.end_date {
            return Err(ContractError::InvalidRecurringPlan);
        }

        if now < plan.last_executed + plan.interval {
            return Err(ContractError::InvalidRecurringPlan);
        }

        plan.last_executed = now;
        env.storage().persistent().set(&key, &plan);

        // Execute contribution
        let token: Address = env.storage().instance().get(&KEY_TOKEN).unwrap();
        token::Client::new(&env, &token)
            .transfer(&contributor, &env.current_contract_address(), &plan.amount);

        let contrib_key = DataKey::Contribution(contributor.clone());
        let prev: i128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);
        let new_amount = prev.checked_add(plan.amount).ok_or(ContractError::Overflow)?;
        env.storage().persistent().set(&contrib_key, &new_amount);

        let total: i128 = env.storage().instance().get(&KEY_TOTAL).unwrap();
        let new_total = total.checked_add(plan.amount).ok_or(ContractError::Overflow)?;
        env.storage().instance().set(&KEY_TOTAL, &new_total);

        env.events().publish(("campaign", "recurring_executed"), (contributor, plan.amount));
        Ok(())
    }

    /// Cancels a recurring contribution plan.
    ///
    /// The contributor must authorize this transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address (must authorize)
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn cancel_recurring(env: Env, contributor: Address) -> Result<(), ContractError> {
        contributor.require_auth();

        let key = DataKey::RecurringPlan(contributor.clone());
        env.storage().persistent().remove(&key);

        env.events().publish(("campaign", "recurring_cancelled"), contributor);
        Ok(())
    }

    /// Proposes a deadline extension and initiates voting.
    ///
    /// Only the creator can propose extensions. Voting period is 7 days.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `new_deadline` - Proposed new deadline (Unix timestamp)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::InvalidDeadline)` if new_deadline <= current_deadline
    pub fn propose_extension(env: Env, new_deadline: u64) -> Result<(), ContractError> {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        creator.require_auth();

        let current_deadline: u64 = env.storage().instance().get(&KEY_DEADLINE).unwrap();
        if new_deadline <= current_deadline {
            return Err(ContractError::InvalidDeadline);
        }

        let now = env.ledger().timestamp();
        let proposal = ExtensionProposal {
            new_deadline,
            votes_for: 0,
            votes_against: 0,
            created_at: now,
            voting_ends_at: now + 604800, // 7 days
            executed: false,
        };

        env.storage().instance().set(&DataKey::ExtensionProposal, &proposal);
        env.events().publish(("campaign", "extension_proposed"), new_deadline);
        Ok(())
    }

    /// Votes on a pending deadline extension.
    ///
    /// Contributors vote with weight equal to their contribution amount.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address (must authorize)
    /// * `approve` - true to vote for, false to vote against
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::VotingEnded)` if voting period has ended
    pub fn vote_on_extension(env: Env, contributor: Address, approve: bool) -> Result<(), ContractError> {
        contributor.require_auth();

        let mut proposal: ExtensionProposal = env.storage().instance().get(&DataKey::ExtensionProposal)
            .ok_or(ContractError::InvalidRecurringPlan)?;

        if env.ledger().timestamp() > proposal.voting_ends_at {
            return Err(ContractError::VotingEnded);
        }

        let vote_weight: i128 = env.storage()
            .persistent()
            .get(&DataKey::Contribution(contributor.clone()))
            .unwrap_or(0);

        if approve {
            proposal.votes_for = proposal.votes_for.checked_add(vote_weight).ok_or(ContractError::Overflow)?;
        } else {
            proposal.votes_against = proposal.votes_against.checked_add(vote_weight).ok_or(ContractError::Overflow)?;
        }

        env.storage().instance().set(&DataKey::ExtensionProposal, &proposal);
        env.storage().instance().set(&DataKey::ExtensionVote(contributor.clone()), &approve);

        env.events().publish(("campaign", "extension_voted"), (contributor, approve));
        Ok(())
    }

    /// Executes a deadline extension if voting threshold is met.
    ///
    /// Requires >50% of votes to be in favor. Can only be called after voting period ends.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Ok(())` on success
    pub fn execute_extension(env: Env) -> Result<(), ContractError> {
        let mut proposal: ExtensionProposal = env.storage().instance().get(&DataKey::ExtensionProposal)
            .ok_or(ContractError::InvalidRecurringPlan)?;

        if env.ledger().timestamp() <= proposal.voting_ends_at {
            return Err(ContractError::VotingEnded);
        }

        if proposal.executed {
            return Ok(());
        }

        let total_votes = proposal.votes_for.checked_add(proposal.votes_against).ok_or(ContractError::Overflow)?;
        if total_votes > 0 && proposal.votes_for * 2 > total_votes {
            env.storage().instance().set(&KEY_DEADLINE, &proposal.new_deadline);
            env.events().publish(("campaign", "extension_executed"), proposal.new_deadline);
        }

        proposal.executed = true;
        env.storage().instance().set(&DataKey::ExtensionProposal, &proposal);
        Ok(())
    }

    /// Allows a contributor to request a partial refund before campaign ends.
    ///
    /// Limited to 50% of original contribution. Contributor must authorize.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address (must authorize)
    /// * `amount` - Amount to refund in stroops
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ContractError::RefundLimitExceeded)` if amount > 50% of contribution
    pub fn refund_partial(env: Env, contributor: Address, amount: i128) -> Result<(), ContractError> {
        contributor.require_auth();

        let contrib_key = DataKey::Contribution(contributor.clone());
        let total_contrib: i128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);

        if amount > total_contrib / 2 {
            return Err(ContractError::RefundLimitExceeded);
        }

        let token: Address = env.storage().instance().get(&KEY_TOKEN).unwrap();
        token::Client::new(&env, &token)
            .transfer(&env.current_contract_address(), &contributor, &amount);

        let new_contrib = total_contrib - amount;
        env.storage().persistent().set(&contrib_key, &new_contrib);

        let total: i128 = env.storage().instance().get(&KEY_TOTAL).unwrap();
        env.storage().instance().set(&KEY_TOTAL, &(total - amount));

        env.events().publish(("campaign", "partial_refund"), (contributor, amount));
        Ok(())
    }

    // ── View functions ────────────────────────────────────────────────────────

    /// Returns the total amount raised so far in stroops.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Total raised amount (i128), or 0 if not yet initialized
    pub fn total_raised(env: Env) -> i128 {
        env.storage().instance().get(&KEY_TOTAL).unwrap_or(0)
    }

    /// Returns the campaign creator's Stellar address.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Creator's address
    pub fn creator(env: Env) -> Address {
        env.storage().instance().get(&KEY_CREATOR).unwrap()
    }

    /// Returns the current campaign status.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Current Status (Active, Successful, Refunded, Cancelled, or Paused)
    pub fn status(env: Env) -> Status {
        env.storage().instance().get(&KEY_STATUS).unwrap()
    }

    /// Returns the campaign funding goal in stroops.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Goal amount (i128)
    pub fn goal(env: Env) -> i128 {
        env.storage().instance().get(&KEY_GOAL).unwrap()
    }

    /// Returns the campaign deadline as a Unix timestamp (seconds).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Deadline timestamp (u64)
    pub fn deadline(env: Env) -> u64 {
        env.storage().instance().get(&KEY_DEADLINE).unwrap()
    }

    /// Returns the total contribution amount for a specific contributor in stroops.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's Stellar address
    ///
    /// # Returns
    /// Total contribution amount (i128), or 0 if no contributions
    pub fn contribution(env: Env, contributor: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Contribution(contributor))
            .unwrap_or(0)
    }

    /// Checks if an address has made any contributions to the campaign.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - The address to check
    ///
    /// # Returns
    /// true if the address has contributed, false otherwise
    pub fn is_contributor(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get::<_, i128>(&DataKey::Contribution(address))
            .unwrap_or(0)
            > 0
    }

    /// Returns the minimum contribution amount in stroops.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Minimum contribution amount (i128)
    pub fn min_contribution(env: Env) -> i128 {
        env.storage().instance().get(&KEY_MIN).unwrap()
    }

    /// Returns the campaign title.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Campaign title string
    pub fn title(env: Env) -> String {
        env.storage()
            .instance()
            .get(&KEY_TITLE)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    /// Returns the campaign description.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Campaign description string
    pub fn description(env: Env) -> String {
        env.storage()
            .instance()
            .get(&KEY_DESC)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    /// Returns the campaign's social media links.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Vector of social media URLs
    pub fn social_links(env: Env) -> Vec<String> {
        env.storage()
            .instance()
            .get(&KEY_SOCIAL)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns the list of accepted token addresses (whitelist).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Vector of accepted token addresses, or empty if no whitelist is set
    pub fn accepted_tokens(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::AcceptedTokens)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns the platform fee configuration (if set).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Optional PlatformConfig containing address and fee_bps
    pub fn platform_config(env: Env) -> Option<PlatformConfig> {
        env.storage().instance().get(&KEY_PLATFORM)
    }

    /// Returns the contract version number.
    ///
    /// # Arguments
    /// * `_env` - The Soroban environment (unused)
    ///
    /// # Returns
    /// Contract version (u32)
    pub fn version(_env: Env) -> u32 {
        CONTRACT_VERSION
    }

    /// Returns comprehensive campaign statistics.
    ///
    /// Includes total raised, goal, progress percentage, contributor count,
    /// average contribution, and largest single contribution.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// CampaignStats struct with all metrics
    ///
    /// # Progress Calculation
    /// progress_bps = (total_raised * 10_000) / goal, capped at 10_000 (100%)
    pub fn get_stats(env: Env) -> CampaignStats {
        let contributor_count: u32 = env.storage().instance().get(&DataKey::ContributorCount).unwrap_or(0);
        let largest_contribution: i128 = env.storage().instance().get(&DataKey::LargestContribution).unwrap_or(0);
        let total_raised: i128 = env.storage().instance().get(&KEY_TOTAL).unwrap_or(0);
        let goal: i128 = env.storage().instance().get(&KEY_GOAL).unwrap();

        let progress_bps = if goal > 0 {
            let raw = (total_raised * 10_000) / goal;
            if raw > 10_000 { 10_000 } else { raw as u32 }
        } else {
            0
        };

        let average_contribution = if contributor_count == 0 {
            0
        } else {
            total_raised / contributor_count as i128
        };

        CampaignStats {
            total_raised,
            goal,
            progress_bps,
            contributor_count,
            average_contribution,
            largest_contribution,
        }
    }

    /// Returns comprehensive campaign information.
    ///
    /// Includes creator, token, goal, deadline, minimum contribution, metadata,
    /// status, and platform fee configuration.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// CampaignInfo struct with all campaign details
    pub fn get_campaign_info(env: Env) -> CampaignInfo {
        let creator: Address = env.storage().instance().get(&KEY_CREATOR).unwrap();
        let token: Address = env.storage().instance().get(&KEY_TOKEN).unwrap();
        let goal: i128 = env.storage().instance().get(&KEY_GOAL).unwrap();
        let deadline: u64 = env.storage().instance().get(&KEY_DEADLINE).unwrap();
        let min_contribution: i128 = env.storage().instance().get(&KEY_MIN).unwrap();
        let title: String = env.storage()
            .instance()
            .get(&KEY_TITLE)
            .unwrap_or_else(|| String::from_str(&env, ""));
        let description: String = env.storage()
            .instance()
            .get(&KEY_DESC)
            .unwrap_or_else(|| String::from_str(&env, ""));
        let status: Status = env.storage().instance().get(&KEY_STATUS).unwrap();
        
        let platform_config: Option<PlatformConfig> = env.storage()
            .instance()
            .get(&KEY_PLATFORM);

        let (has_platform_config, platform_fee_bps, platform_address) =
            if let Some(config) = env.storage().instance().get::<_, PlatformConfig>(&KEY_PLATFORM) {
                (true, config.fee_bps, config.address)
            } else {
                (false, 0, creator.clone())
            };

        CampaignInfo {
            creator,
            token,
            goal,
            deadline,
            min_contribution,
            title,
            description,
            status,
            has_platform_config,
            platform_fee_bps,
            platform_address,
        }
    }

    /// Returns a paginated list of contributor addresses.
    ///
    /// Useful for iterating through all contributors without loading the entire list.
    /// The limit is capped at 50 to prevent excessive memory usage.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `offset` - Starting index in the contributor list (0-based)
    /// * `limit` - Maximum number of contributors to return (capped at 50)
    ///
    /// # Returns
    /// Vector of contributor addresses for the requested page
    ///
    /// # Example
    /// ```ignore
    /// // Get first 10 contributors
    /// let page1 = contributor_list(env, 0, 10);
    /// // Get next 10 contributors
    /// let page2 = contributor_list(env, 10, 10);
    /// ```
    pub fn contributor_list(env: Env, offset: u32, limit: u32) -> Vec<Address> {
        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&KEY_CONTRIBS)
            .unwrap_or_else(|| Vec::new(&env));

        let total_count = contributors.len();
        if offset >= total_count {
            return Vec::new(&env);
        }

        let capped_limit = if limit > 50 { 50 } else { limit };
        let end = (offset + capped_limit).min(total_count);

        let mut result = Vec::new(&env);
        for i in offset..end {
            result.push_back(contributors.get(i).unwrap());
        }
        result
    }

    /// Returns the contribution message for a contributor.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address
    ///
    /// # Returns
    /// Optional message string, or None if no message was provided
    pub fn get_contribution_message(env: Env, contributor: Address) -> Option<String> {
        env.storage()
            .persistent()
            .get(&DataKey::ContributionMessage(contributor))
    }

    /// Returns the recurring plan for a contributor.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `contributor` - The contributor's address
    ///
    /// # Returns
    /// Optional RecurringPlan, or None if no plan exists
    pub fn get_recurring_plan(env: Env, contributor: Address) -> Option<RecurringPlan> {
        env.storage()
            .persistent()
            .get(&DataKey::RecurringPlan(contributor))
    }

    /// Returns the current extension proposal.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// Optional ExtensionProposal, or None if no proposal exists
    pub fn get_extension_proposal(env: Env) -> Option<ExtensionProposal> {
        env.storage()
            .instance()
            .get(&DataKey::ExtensionProposal)
    }
}

#[cfg(test)]
mod test;
