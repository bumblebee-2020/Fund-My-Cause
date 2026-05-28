# Fund-My-Cause Contract API Reference

> **Contract Version:** 4  
> **Network:** Stellar / Soroban  
> **Rustdoc:** [View generated API docs](https://fund-my-cause.github.io/Fund-My-Cause/crowdfund/)

Complete reference for both Soroban smart contracts powering Fund-My-Cause.

---

## Contracts

| Crate | Purpose |
|-------|---------|
| [`crowdfund`](#crowdfund-contract) | Per-campaign crowdfunding logic |
| [`registry`](#registry-contract) | On-chain campaign discovery |

---

## Crowdfund Contract

### Data Types

#### `Status`
Campaign lifecycle state.

```rust
pub enum Status {
    Active,      // Accepting contributions
    Successful,  // Goal reached, funds withdrawn
    Refunded,    // Goal not met, refunds available
    Cancelled,   // Creator cancelled, refunds available
    Paused,      // Temporarily paused, no contributions
}
```

#### `Category`
```rust
pub enum Category { Charity, Technology, Creative, Event, Personal, Other }
```

#### `Visibility`
```rust
pub enum Visibility {
    Public,    // Listed; anyone may contribute
    Private,   // Whitelist-only; not listed in discovery
    Unlisted,  // Anyone may contribute; not listed in discovery
}
```

#### `TemplateType`
```rust
pub enum TemplateType { Charity, Product, Event, Personal, Custom }
```

#### `CampaignStats`
```rust
pub struct CampaignStats {
    pub total_raised: i128,         // Total raised in stroops
    pub goal: i128,                 // Funding goal in stroops
    pub progress_bps: u32,          // Progress 0–10000 (10000 = 100%)
    pub contributor_count: u32,     // Unique contributors
    pub average_contribution: i128, // Average contribution in stroops
    pub largest_contribution: i128, // Largest single contribution
}
```

#### `CampaignInfo`
```rust
pub struct CampaignInfo {
    pub creator: Address,
    pub token: Address,
    pub goal: i128,
    pub deadline: u64,           // Unix timestamp (seconds)
    pub min_contribution: i128,
    pub max_contribution: i128,  // 0 = no limit
    pub title: String,
    pub description: String,
    pub status: Status,
    pub has_platform_config: bool,
    pub platform_fee_bps: u32,
    pub platform_address: Address,
    pub category: Category,
}
```

#### `PlatformConfig`
```rust
pub struct PlatformConfig {
    pub address: Address, // Fee recipient
    pub fee_bps: u32,     // Fee in basis points (max 10 000)
}
```

#### `VestingSchedule`
```rust
pub struct VestingSchedule {
    pub cliff: u64,    // No withdrawal before this Unix timestamp
    pub duration: u64, // Linear vesting duration in seconds after cliff
}
```

#### `RateLimit`
```rust
pub struct RateLimit {
    pub max_amount: i128,    // Max contribution per address per window
    pub window_seconds: u64, // Window length in seconds
}
```

#### `MatchingConfig`
```rust
pub struct MatchingConfig {
    pub sponsor: Address,
    pub match_ratio: u32, // Basis points (10 000 = 1:1)
    pub max_match: i128,  // Maximum total matching in stroops
}
```

#### `InsuranceConfig`
```rust
pub struct InsuranceConfig {
    pub fee_bps: u32,      // Insurance fee in basis points
    pub provider: Address, // Insurance provider address
    pub enabled: bool,
}
```

#### `RecurringPlan`
```rust
pub struct RecurringPlan {
    pub amount: i128,       // Amount per interval in stroops
    pub interval: u64,      // Seconds between contributions
    pub end_date: u64,      // Unix timestamp to stop
    pub last_executed: u64, // Timestamp of last execution
}
```

#### `RewardTier`
```rust
pub struct RewardTier {
    pub min_amount: i128, // Minimum cumulative contribution to qualify
    pub name: String,     // e.g. "Bronze", "Silver", "Gold"
    pub description: String,
}
```

#### `RewardConfig`
```rust
pub struct RewardConfig {
    pub reward_token: Address,
    pub reward_per_unit: i128, // Reward per stroop contributed
    pub enabled: bool,
}
```

#### `ContributionRecord`
```rust
pub struct ContributionRecord {
    pub amount: i128,        // Amount in this contribution
    pub timestamp: u64,      // Ledger timestamp
    pub running_total: i128, // Cumulative total after this contribution
}
```

#### `PerformanceMetrics`
```rust
pub struct PerformanceMetrics {
    pub success_rate_bps: u32,
    pub contribution_velocity: i128,  // Stroops per day
    pub trending: i32,                // Positive = increasing
    pub milestones_reached: u32,
    pub total_milestones: u32,
    pub time_elapsed: u64,
    pub estimated_time_to_goal: u64,
    pub average_daily_contribution: i128,
}
```

---

### Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 2 | `CampaignEnded` | Deadline has passed |
| 3 | `CampaignStillActive` | Deadline has not yet passed |
| 4 | `GoalNotReached` | Funding goal not met |
| 5 | `GoalReached` | Goal already reached (cannot refund) |
| 6 | `Overflow` | Arithmetic overflow |
| 7 | `NotActive` | Campaign not in Active status |
| 8 | `InvalidFee` | Fee exceeds 10 000 bps |
| 9 | `BelowMinimum` | Amount below minimum contribution |
| 10 | `InvalidDeadline` | Deadline is invalid |
| 11 | `CampaignPaused` | Campaign is paused |
| 12 | `InvalidGoal` | Goal must be positive |
| 13 | `TokenNotAccepted` | Token not in accepted list |
| 14 | `ExceedsMaximum` | Contribution would exceed per-contributor max |
| 15 | `NotWhitelisted` | Address not on whitelist |
| 16 | `Blacklisted` | Address is blacklisted |
| 17 | `InvalidDelegation` | Invalid delegation parameters |
| 18 | `DelegationNotFound` | No delegation found for address |
| 19 | `InvalidTemplate` | Invalid template type |
| 20 | `VotingEnded` | Voting period has ended |
| 21 | `InvalidRecurringPlan` | Invalid recurring plan parameters |
| 22 | `RefundLimitExceeded` | Partial refund exceeds 50% of contribution |
| 23 | `VestingNotComplete` | Vesting cliff not reached |
| 24 | `EmergencyLocked` | Emergency withdrawal is locked |
| 25 | `RateLimitExceeded` | Contribution exceeds rate limit |
| 26 | `MessageTooLong` | Message exceeds 256 characters |
| 27 | `StringEmpty` | Required string field is empty |
| 28 | `StringTooLong` | String exceeds maximum length |
| 29 | `AmountNotPositive` | Amount must be > 0 |
| 30 | `SelfFeeAddress` | Platform fee address same as creator |
| 31 | `GoalOverflow` | Goal too large (would overflow i128) |
| 32 | `InsufficientFunds` | Insufficient funds in pool |
| 33 | `Unauthorized` | Caller not authorized |
| 34 | `InvalidRateLimit` | Invalid rate limit configuration |
| 35 | `MultiSigNotMet` | Multi-sig approval threshold not met |
| 36 | `ProposalNotFound` | Extension proposal not found |
| 37 | `AlreadyVoted` | Address already voted on proposal |
| 38 | `NoRewardsConfigured` | Rewards not configured |
| 39 | `NotCreator` | Caller is not the campaign creator |

---

### Public Functions

#### Core Lifecycle

##### `initialize`

Creates a new campaign. Can only be called once per contract instance.

```rust
pub fn initialize(
    env: Env,
    creator: Address,          // Must authorize; becomes admin
    token: Address,            // Primary contribution token
    goal: i128,                // Funding goal in stroops (> 0)
    deadline: u64,             // Unix timestamp (> current time)
    min_contribution: i128,    // Minimum per contribution (>= 0)
    max_contribution: i128,    // Per-contributor cap (0 = no limit)
    title: String,             // Max 64 chars
    description: String,       // Max 512 chars
    social_links: Option<Vec<String>>,
    platform_config: Option<PlatformConfig>,
    accepted_tokens: Option<Vec<Address>>,
    category: Category,
    vesting: Option<VestingSchedule>,
    penalty_bps: Option<u32>,
) -> Result<(), ContractError>
```

**Errors:** `AlreadyInitialized`, `InvalidGoal`, `InvalidDeadline`, `InvalidFee`,
`SelfFeeAddress`, `GoalOverflow`, `StringEmpty`, `StringTooLong`

**Events:** `("campaign", "initialized")`

**Example:**
```ignore
contract.initialize(
    env, creator, token,
    1_000_000_000,  // 100 XLM goal
    1_800_000_000,  // deadline timestamp
    10_000_000,     // 1 XLM minimum
    0,              // no per-contributor cap
    String::from_str(&env, "My Campaign"),
    String::from_str(&env, "Help us build something great"),
    None, None, None,
    Category::Technology,
    None, None,
)?;
```

##### `contribute`

Pledges tokens to the campaign before the deadline.

```rust
pub fn contribute(
    env: Env,
    contributor: Address,   // Must authorize
    amount: i128,           // >= min_contribution
    token: Address,         // Must be in accepted tokens
    message: Option<String>, // Optional memo, max 256 chars
) -> Result<(), ContractError>
```

**Errors:** `NotActive`, `CampaignPaused`, `CampaignEnded`, `BelowMinimum`,
`ExceedsMaximum`, `TokenNotAccepted`, `NotWhitelisted`, `Blacklisted`,
`RateLimitExceeded`, `MessageTooLong`, `Overflow`

**Events:** `("campaign", "contributed")`, `("campaign", "contribution_recorded")`,
optionally `("campaign", "tier_assigned")`, `("campaign", "rate_limit_hit")`

**Example:**
```ignore
// Contribute 5 XLM (50_000_000 stroops)
contract.contribute(env, contributor, 50_000_000, xlm_token, None)?;
```

##### `withdraw`

Creator claims funds after a successful campaign (deadline passed + goal met).

```rust
pub fn withdraw(env: Env) -> Result<(), ContractError>
```

**Errors:** `NotActive`, `CampaignStillActive`, `GoalNotReached`, `VestingNotComplete`

**Events:** `("campaign", "withdrawn")`

**Platform fee calculation:**
```
fee    = total_raised * platform_fee_bps / 10_000
payout = total_raised - fee
```

##### `refund_single`

Contributor reclaims their funds after a failed or cancelled campaign.

```rust
pub fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError>
```

**Errors:** `GoalReached`, `CampaignStillActive`

**Events:** `("campaign", "refunded")`

##### `refund_batch`

Batch refund for multiple contributors in one transaction.

```rust
pub fn refund_batch(env: Env, contributors: Vec<Address>) -> Result<(), ContractError>
```

**Events:** `("campaign", "batch_refund_completed")`

##### `refund_partial`

Contributor withdraws up to 50% of their contribution before the deadline.

```rust
pub fn refund_partial(
    env: Env,
    contributor: Address,
    amount: i128,
) -> Result<(), ContractError>
```

**Errors:** `RefundLimitExceeded`, `NotActive`, `CampaignEnded`

**Events:** `("campaign", "partial_refund")`

##### `cancel_campaign`

Creator cancels the campaign; all contributors may then claim refunds.

```rust
pub fn cancel_campaign(env: Env) -> Result<(), ContractError>
```

**Errors:** `NotActive`, `NotCreator`

**Events:** `("campaign", "cancelled")`

#### Metadata & Campaign Management

##### `update_metadata`

Update title, description, or social links while the campaign is Active.

```rust
pub fn update_metadata(
    env: Env,
    title: Option<String>,
    description: Option<String>,
    social_links: Option<Vec<String>>,
) -> Result<(), ContractError>
```

**Events:** `("campaign", "metadata_updated")`, `("campaign", "metadata_versioned")`

##### `extend_deadline`

Creator directly extends the campaign deadline.

```rust
pub fn extend_deadline(env: Env, new_deadline: u64) -> Result<(), ContractError>
```

**Errors:** `InvalidDeadline` (new must be > current)

**Events:** `("campaign", "deadline_extended")`

##### `adjust_goal`

Creator adjusts the funding goal. Recorded in audit history.

```rust
pub fn adjust_goal(env: Env, new_goal: i128) -> Result<(), ContractError>
```

**Events:** `("campaign", "goal_adjusted")`

##### `update_category`

Creator updates the campaign category.

```rust
pub fn update_category(env: Env, new_category: Category) -> Result<(), ContractError>
```

**Events:** `("campaign", "category_updated")`

##### `set_visibility`

Creator changes campaign visibility level.

```rust
pub fn set_visibility(env: Env, visibility: Visibility) -> Result<(), ContractError>
```

**Events:** `("campaign", "visibility_changed")`

##### `pause` / `resume` / `unpause`

Pause or resume the campaign (creator only).

```rust
pub fn pause(env: Env) -> Result<(), ContractError>
pub fn resume(env: Env) -> Result<(), ContractError>
pub fn unpause(env: Env) -> Result<(), ContractError>
```

**Events:** `("campaign", "paused")` / `("campaign", "resumed")`

#### Access Control

##### `add_to_whitelist` / `remove_from_whitelist`

Manage the contributor whitelist (creator only).

```rust
pub fn add_to_whitelist(env: Env, address: Address) -> Result<(), ContractError>
pub fn remove_from_whitelist(env: Env, address: Address) -> Result<(), ContractError>
```

**Events:** `("campaign", "whitelisted")` / `("campaign", "whitelist_removed")`

##### `add_to_blacklist` / `remove_from_blacklist`

Block or unblock specific addresses (creator only).

```rust
pub fn add_to_blacklist(env: Env, address: Address) -> Result<(), ContractError>
pub fn remove_from_blacklist(env: Env, address: Address) -> Result<(), ContractError>
```

**Events:** `("campaign", "blacklisted")` / `("campaign", "blacklist_removed")`

##### `set_whitelist_only`

Toggle whitelist-only contribution mode.

```rust
pub fn set_whitelist_only(env: Env, enabled: bool) -> Result<(), ContractError>
```

**Events:** `("campaign", "whitelist_only_set")`

##### `set_rate_limit`

Configure per-address contribution rate limiting.

```rust
pub fn set_rate_limit(env: Env, rate_limit: Option<RateLimit>) -> Result<(), ContractError>
```

Pass `None` to clear the rate limit.

**Errors:** `InvalidRateLimit`

**Events:** `("campaign", "rate_limit_updated")`

**Example:**
```ignore
// Max 100 XLM per address per hour
contract.set_rate_limit(env, Some(RateLimit {
    max_amount: 1_000_000_000,
    window_seconds: 3600,
}))?;
```

#### Recurring Contributions

##### `setup_recurring`

Set up a scheduled recurring contribution plan.

```rust
pub fn setup_recurring(
    env: Env,
    contributor: Address,
    amount: i128,
    interval: u64,  // Seconds between contributions
    end_date: u64,  // Unix timestamp to stop
) -> Result<(), ContractError>
```

**Errors:** `InvalidRecurringPlan`

**Events:** `("campaign", "recurring_setup")`

##### `execute_recurring`

Execute a pending recurring contribution for a contributor.

```rust
pub fn execute_recurring(env: Env, contributor: Address) -> Result<(), ContractError>
```

**Events:** `("campaign", "recurring_executed")`

##### `cancel_recurring`

Cancel a contributor's recurring plan.

```rust
pub fn cancel_recurring(env: Env, contributor: Address) -> Result<(), ContractError>
```

**Events:** `("campaign", "recurring_cancelled")`

#### Delegation

##### `delegate_contribution`

Authorize a delegate to contribute on your behalf.

```rust
pub fn delegate_contribution(
    env: Env,
    delegator: Address,
    delegate: Address,
    amount: i128,
) -> Result<(), ContractError>
```

**Events:** `("campaign", "delegation_created")`

##### `contribute_on_behalf`

Delegate submits a contribution on behalf of the delegator.

```rust
pub fn contribute_on_behalf(
    env: Env,
    delegate: Address,
    delegator: Address,
    amount: i128,
    token: Address,
) -> Result<(), ContractError>
```

**Events:** `("campaign", "delegated_contribution")`

##### `revoke_delegation`

Delegator revokes an active delegation.

```rust
pub fn revoke_delegation(env: Env, delegator: Address) -> Result<(), ContractError>
```

**Events:** `("campaign", "delegation_revoked")`

#### Deadline Extension Voting

##### `propose_extension`

Creator proposes a deadline extension subject to contributor vote.

```rust
pub fn propose_extension(
    env: Env,
    new_deadline: u64,
    voting_period: u64, // Seconds for voting window
) -> Result<(), ContractError>
```

**Events:** `("campaign", "extension_proposed")`

##### `vote_on_extension`

Contributor votes on the active extension proposal. Vote weight = contribution amount.

```rust
pub fn vote_on_extension(
    env: Env,
    contributor: Address,
    approve: bool,
) -> Result<(), ContractError>
```

**Errors:** `ProposalNotFound`, `VotingEnded`, `AlreadyVoted`

**Events:** `("campaign", "extension_voted")`

##### `execute_extension`

Execute an approved extension proposal after voting ends.

```rust
pub fn execute_extension(env: Env) -> Result<(), ContractError>
```

**Events:** `("campaign", "extension_executed")`

#### Emergency Withdrawal

##### `initiate_emergency_withdrawal`

Initiates a time-locked emergency withdrawal (24-hour lock).

```rust
pub fn initiate_emergency_withdrawal(env: Env) -> Result<(), ContractError>
```

**Events:** `("campaign", "emergency_initiated")`

##### `setup_emergency_multisig`

Configure multi-sig approvers for emergency withdrawal.

```rust
pub fn setup_emergency_multisig(
    env: Env,
    approvers: Vec<Address>,
    required_approvals: u32,
) -> Result<(), ContractError>
```

**Events:** `("campaign", "multisig_configured")`

##### `approve_emergency_withdrawal`

Approver submits their approval for the pending emergency withdrawal.

```rust
pub fn approve_emergency_withdrawal(env: Env, approver: Address) -> Result<(), ContractError>
```

**Events:** `("campaign", "emergency_approved")`

##### `execute_emergency_withdrawal`

Execute emergency withdrawal after lock period and multi-sig threshold met.

```rust
pub fn execute_emergency_withdrawal(env: Env) -> Result<(), ContractError>
```

**Errors:** `EmergencyLocked`, `MultiSigNotMet`

**Events:** `("campaign", "emergency_executed")`

##### `cancel_emergency_withdrawal`

Cancel a pending emergency withdrawal.

```rust
pub fn cancel_emergency_withdrawal(env: Env) -> Result<(), ContractError>
```

#### Insurance

##### `enable_insurance`

Enable contributor insurance for the campaign.

```rust
pub fn enable_insurance(
    env: Env,
    fee_bps: u32,
    provider: Address,
) -> Result<(), ContractError>
```

**Errors:** `InvalidFee`

**Events:** `("insurance", "enabled")`

When insurance is enabled, a portion of each contribution (`fee_bps / 10_000`) is
held in the insurance pool. If the campaign fails, contributors receive their
insurance fee back in addition to their contribution refund.

#### Contribution Matching

##### `setup_matching`

Sponsor configures a contribution matching pool.

```rust
pub fn setup_matching(
    env: Env,
    sponsor: Address,
    match_ratio: u32, // Basis points (10 000 = 1:1 match)
    max_match: i128,
) -> Result<(), ContractError>
```

**Events:** `("campaign", "matching_setup")`

**Example:**
```ignore
// 50% match up to 10 000 XLM
contract.setup_matching(env, sponsor, 5_000, 100_000_000_000)?;
```

#### Reward Tiers

##### `set_reward_tiers`

Define contribution reward tiers (sorted ascending by `min_amount`).

```rust
pub fn set_reward_tiers(env: Env, tiers: Vec<RewardTier>) -> Result<(), ContractError>
```

**Events:** `("campaign", "tiers_set")`

**Example:**
```ignore
let tiers = vec![
    RewardTier { min_amount: 100_000_000,   name: "Bronze".into(), description: "Early supporter".into() },
    RewardTier { min_amount: 1_000_000_000, name: "Silver".into(), description: "Major backer".into() },
    RewardTier { min_amount: 10_000_000_000, name: "Gold".into(), description: "Champion".into() },
];
contract.set_reward_tiers(env, tiers)?;
```

##### `configure_rewards`

Configure token rewards distributed to contributors.

```rust
pub fn configure_rewards(
    env: Env,
    reward_token: Address,
    reward_per_unit: i128,
) -> Result<(), ContractError>
```

**Events:** `("campaign", "rewards_configured")`

##### `distribute_rewards`

Distribute token rewards to a contributor based on their contribution.

```rust
pub fn distribute_rewards(env: Env, contributor: Address) -> Result<(), ContractError>
```

**Errors:** `NoRewardsConfigured`

**Events:** `("campaign", "rewards_distributed")`

#### Templates

##### `initialize_from_template`

Initialize a campaign using a predefined template.

```rust
pub fn initialize_from_template(
    env: Env,
    creator: Address,
    token: Address,
    goal: i128,
    deadline: u64,
    title: String,
    description: String,
    template_type: TemplateType,
) -> Result<(), ContractError>
```

**Events:** `("campaign", "template_applied")`, `("campaign", "initialized")`

##### `clone_campaign`

Clone an existing campaign with new goal and deadline.

```rust
pub fn clone_campaign(
    env: Env,
    new_creator: Address,
    new_goal: i128,
    new_deadline: u64,
) -> Result<(), ContractError>
```

**Events:** `("campaign", "cloned")`

#### Read-Only Functions

| Function | Signature | Returns |
|----------|-----------|---------|
| `get_stats` | `(env)` | `CampaignStats` |
| `get_campaign_info` | `(env)` | `CampaignInfo` |
| `get_performance_metrics` | `(env)` | `PerformanceMetrics` |
| `total_raised` | `(env)` | `i128` |
| `goal` | `(env)` | `i128` |
| `deadline` | `(env)` | `u64` |
| `contribution` | `(env, contributor: Address)` | `i128` |
| `min_contribution` | `(env)` | `i128` |
| `title` | `(env)` | `String` |
| `description` | `(env)` | `String` |
| `social_links` | `(env)` | `Vec<String>` |
| `version` | `(env)` | `u32` (currently `4`) |
| `get_rate_limit` | `(env)` | `Option<RateLimit>` |
| `get_matching_config` | `(env)` | `Option<MatchingConfig>` |
| `get_total_matched` | `(env)` | `i128` |
| `get_contribution_history` | `(env, contributor)` | `Vec<ContributionRecord>` |
| `get_contributor_tier` | `(env, contributor)` | `Option<RewardTier>` |
| `get_goal_history` | `(env)` | `Vec<GoalAdjustment>` |
| `get_metadata_version` | `(env, version: u32)` | `Option<MetadataVersion>` |
| `contributor_list` | `(env, offset: u32, limit: u32)` | `Vec<Address>` |

---

### Events Reference

Every state-changing function publishes a typed event. All events use the topic
format `("campaign", "<event_type>")` except insurance events which use `("insurance", "<type>")`.

| Event | Emitted by | Payload type |
|-------|-----------|--------------|
| `initialized` | `initialize` | `EventInitialized` |
| `contributed` | `contribute` | `EventContributed` |
| `contribution_recorded` | `contribute` | `EventContributionRecorded` |
| `withdrawn` | `withdraw` | `EventWithdrawn` |
| `refunded` | `refund_single` | `EventRefunded` |
| `partial_refund` | `refund_partial` | `EventPartialRefund` |
| `batch_refund_completed` | `refund_batch` | `EventBatchRefundCompleted` |
| `metadata_updated` | `update_metadata` | `EventMetadataUpdated` |
| `metadata_versioned` | `update_metadata` | `EventMetadataVersioned` |
| `deadline_extended` | `extend_deadline` | `EventDeadlineExtended` |
| `goal_adjusted` | `adjust_goal` | `EventGoalAdjusted` |
| `category_updated` | `update_category` | `EventCategoryUpdated` |
| `visibility_changed` | `set_visibility` | `EventVisibilityChanged` |
| `status_changed` | various | `EventStatusChanged` |
| `cancelled` | `cancel_campaign` | `EventCancelled` |
| `paused` | `pause` | `EventPaused` |
| `resumed` | `resume` / `unpause` | `EventResumed` |
| `whitelisted` | `add_to_whitelist` | `EventWhitelisted` |
| `whitelist_removed` | `remove_from_whitelist` | `EventWhitelistRemoved` |
| `blacklisted` | `add_to_blacklist` | `EventBlacklisted` |
| `blacklist_removed` | `remove_from_blacklist` | `EventBlacklistRemoved` |
| `whitelist_only_set` | `set_whitelist_only` | `EventWhitelistOnlySet` |
| `rate_limit_updated` | `set_rate_limit` | `EventRateLimitUpdated` |
| `rate_limit_hit` | `contribute` | `EventRateLimitHit` |
| `recurring_setup` | `setup_recurring` | `EventRecurringSetup` |
| `recurring_executed` | `execute_recurring` | `EventRecurringExecuted` |
| `recurring_cancelled` | `cancel_recurring` | `EventRecurringCancelled` |
| `delegation_created` | `delegate_contribution` | `EventDelegationCreated` |
| `delegated_contribution` | `contribute_on_behalf` | `EventDelegatedContribution` |
| `delegation_revoked` | `revoke_delegation` | `EventDelegationRevoked` |
| `extension_proposed` | `propose_extension` | `EventExtensionProposed` |
| `extension_voted` | `vote_on_extension` | `EventExtensionVoted` |
| `extension_executed` | `execute_extension` | `EventExtensionExecuted` |
| `emergency_initiated` | `initiate_emergency_withdrawal` | `EventEmergencyInitiated` |
| `multisig_configured` | `setup_emergency_multisig` | `EventMultiSigConfigured` |
| `emergency_approved` | `approve_emergency_withdrawal` | `EventEmergencyApproved` |
| `emergency_executed` | `execute_emergency_withdrawal` | `EventEmergencyExecuted` |
| `insurance enabled` | `enable_insurance` | `EventInsuranceEnabled` |
| `insurance payout` | refund flow | `EventInsurancePayout` |
| `matching_setup` | `setup_matching` | `EventMatchingSetup` |
| `tiers_set` | `set_reward_tiers` | `EventTiersSet` |
| `tier_assigned` | `contribute` | `EventTierAssigned` |
| `rewards_configured` | `configure_rewards` | `EventRewardsConfigured` |
| `rewards_distributed` | `distribute_rewards` | `EventRewardsDistributed` |
| `template_applied` | `initialize_from_template` | `EventTemplateApplied` |
| `cloned` | `clone_campaign` | `EventCampaignCloned` |
| `indexed` | `initialize` | `EventCampaignIndexed` |

---

### Storage Layout

#### Instance Storage (campaign-wide state)

| Key | Type | Description |
|-----|------|-------------|
| `CREATOR` | `Address` | Campaign creator / admin |
| `TOKEN` | `Address` | Primary contribution token |
| `GOAL` | `i128` | Funding goal in stroops |
| `DEADLINE` | `u64` | Unix timestamp deadline |
| `TOTAL` | `i128` | Total raised in stroops |
| `STATUS` | `Status` | Current campaign status |
| `MIN` | `i128` | Minimum contribution |
| `MAX` | `i128` | Per-contributor cap (0 = none) |
| `TITLE` | `String` | Campaign title |
| `DESC` | `String` | Campaign description |
| `SOCIAL` | `Vec<String>` | Social links |
| `PLATFORM` | `PlatformConfig` | Optional fee config |
| `ADMIN` | `Address` | Admin address |
| `RATELIMIT` | `RateLimit` | Optional rate limit config |
| `INSURE` | `InsuranceConfig` | Optional insurance config |
| `INSPOOL` | `i128` | Insurance pool balance |
| `CATEGORY` | `Category` | Campaign category |
| `VESTING` | `VestingSchedule` | Optional vesting schedule |
| `GHIST` | `Vec<GoalAdjustment>` | Goal adjustment history |
| `VIS` | `Visibility` | Campaign visibility |
| `METAHIST` | `Vec<MetadataVersion>` | Metadata version history |
| `START` | `u64` | Campaign start timestamp |

#### Persistent Storage (per-contributor data)

| Key | Type | TTL |
|-----|------|-----|
| `Contribution(Address)` | `i128` | 100 ledgers |
| `ContributorPresence(Address)` | `bool` | 100 ledgers |
| `ContributionMessage(Address)` | `String` | 100 ledgers |
| `RecurringPlan(Address)` | `RecurringPlan` | 100 ledgers |
| `ContributionHistory(Address)` | `Vec<ContributionRecord>` | 100 ledgers |
| `ContributorTier(Address)` | `RewardTier` | 100 ledgers |
| `Whitelist(Address)` | `bool` | 100 ledgers |
| `Blacklist(Address)` | `bool` | 100 ledgers |
| `Delegation(Address)` | `Delegation` | 100 ledgers |
| `InsuranceFee(Address)` | `i128` | 100 ledgers |
| `RateLimitTimestamp(Address)` | `u64` | 100 ledgers |
| `RateLimitAmount(Address)` | `i128` | 100 ledgers |
| `CONTRIBS` | `Vec<Address>` | 100 ledgers |

---

## Registry Contract

A lightweight on-chain directory of all deployed campaign contracts.

### Functions

#### `register`

Register a campaign address. No-op if already registered.

```rust
pub fn register(env: Env, campaign_id: Address)
```

**Events:** `("registry", "registered")`

**Example:**
```ignore
// Called immediately after deploying a new CrowdfundContract
registry_client.register(&new_campaign_address);
```

#### `register_with_category`

Register a campaign with a numeric category index for filtered discovery.

```rust
pub fn register_with_category(env: Env, campaign_id: Address, category_id: u32)
```

Category IDs match the `Category` enum discriminants:

| `category_id` | Category |
|---------------|----------|
| 0 | Charity |
| 1 | Technology |
| 2 | Creative |
| 3 | Event |
| 4 | Personal |
| 5 | Other |

#### `list`

Paginated list of all registered campaigns.

```rust
pub fn list(env: Env, offset: u32, limit: u32) -> Vec<Address>
```

**Example:**
```ignore
let page1 = registry_client.list(&0, &20);  // first 20
let page2 = registry_client.list(&20, &20); // next 20
```

#### `get_campaigns_by_category`

Paginated list filtered by category.

```rust
pub fn get_campaigns_by_category(
    env: Env,
    category_id: u32,
    offset: u32,
    limit: u32,
) -> Vec<Address>
```

**Example:**
```ignore
// First 10 Technology campaigns
let tech = registry_client.get_campaigns_by_category(&1, &0, &10);
```

---

## Usage Examples

### Full campaign lifecycle (Rust test style)

```ignore
use soroban_sdk::{testutils::Address as _, Address, Env};

let env = Env::default();
env.mock_all_auths();

let creator = Address::generate(&env);
let contributor = Address::generate(&env);
let token = Address::generate(&env); // mock token

// 1. Initialize
contract.initialize(
    env.clone(), creator.clone(), token.clone(),
    1_000_000_000,  // 100 XLM goal
    env.ledger().timestamp() + 86400, // 1 day deadline
    10_000_000, 0,
    String::from_str(&env, "My Campaign"),
    String::from_str(&env, "A great cause"),
    None, None, None,
    Category::Charity, None, None,
)?;

// 2. Contribute
contract.contribute(env.clone(), contributor.clone(), 500_000_000, token.clone(), None)?;

// 3. Check stats
let stats = contract.get_stats(env.clone());
assert_eq!(stats.total_raised, 500_000_000);

// 4. Withdraw (after deadline + goal met)
contract.withdraw(env.clone())?;
```

### CLI interaction

```bash
# Initialize a campaign
stellar contract invoke \
  --id <CONTRACT_ID> --source <CREATOR_KEY> --network testnet \
  -- initialize \
  --creator <CREATOR_ADDR> --token <TOKEN_ADDR> \
  --goal 1000000000 --deadline 1800000000 \
  --min_contribution 10000000 --max_contribution 0 \
  --title '"My Campaign"' --description '"Help us build"' \
  --social_links null --platform_config null \
  --accepted_tokens null --category Charity \
  --vesting null --penalty_bps null

# Contribute 5 XLM
stellar contract invoke \
  --id <CONTRACT_ID> --source <CONTRIBUTOR_KEY> --network testnet \
  -- contribute \
  --contributor <CONTRIBUTOR_ADDR> --amount 50000000 \
  --token <TOKEN_ADDR> --message null

# Get stats
stellar contract invoke --id <CONTRACT_ID> --network testnet -- get_stats
```

---

## Security Considerations

1. **Authorization** — all mutating functions call `require_auth()` on the relevant address.
2. **Overflow protection** — all arithmetic uses `checked_add` / `checked_mul`.
3. **Pull-based refunds** — contributors claim individually; no single-point-of-failure batch.
4. **Fee cap** — platform fees are hard-capped at 10 000 bps (100%).
5. **Reentrancy** — token transfers use Soroban's safe `token::Client` interface.
6. **TTL management** — persistent storage uses explicit TTL extension to control costs.
7. **Emergency multi-sig** — emergency withdrawals require time-lock + multi-sig approval.
8. **Rate limiting** — per-address contribution rate limits prevent spam/manipulation.

---

*For the generated rustdoc API reference, see the [deployed docs](https://fund-my-cause.github.io/Fund-My-Cause/crowdfund/) or build locally with `cargo doc --workspace --no-deps`.*
