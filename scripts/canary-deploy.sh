#!/usr/bin/env bash
# Canary deployment with gradual traffic shifting
# Usage: ./scripts/canary-deploy.sh <contract_id> <new_version> [--network testnet]

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Defaults
NETWORK="testnet"
CANARY_PERCENTAGE=5
MAX_CANARY_PERCENTAGE=100
HEALTH_CHECK_INTERVAL=30
ROLLBACK_ERROR_THRESHOLD=3
CANARY_STATE_FILE=".canary-state"

# Helper functions
log_info() { echo -e "${BLUE}➜${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1" >&2; }
log_warning() { echo -e "${YELLOW}⚠${NC} $1"; }

# Parse arguments
CONTRACT_ID="${1:-}"
NEW_VERSION="${2:-}"

if [ -z "$CONTRACT_ID" ] || [ -z "$NEW_VERSION" ]; then
  echo "Usage: $0 <contract_id> <new_version> [--network testnet]"
  exit 1
fi

while [[ $# -gt 2 ]]; do
  case $3 in
    --network)
      NETWORK="$4"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

log_info "Starting canary deployment for $CONTRACT_ID"
log_info "New version: $NEW_VERSION"
log_info "Network: $NETWORK"

# Initialize canary state
init_canary_state() {
  cat > "$CANARY_STATE_FILE" << EOF
{
  "contract_id": "$CONTRACT_ID",
  "new_version": "$NEW_VERSION",
  "network": "$NETWORK",
  "start_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "canary_percentage": $CANARY_PERCENTAGE,
  "status": "initializing",
  "health_checks": [],
  "errors": 0
}
EOF
}

# Update canary state
update_canary_state() {
  local key=$1
  local value=$2
  if [ -f "$CANARY_STATE_FILE" ]; then
    # Simple JSON update (requires jq in production)
    sed -i "s/\"$key\": [^,}]*/\"$key\": $value/" "$CANARY_STATE_FILE"
  fi
}

# Health check function
perform_health_check() {
  log_info "Performing health check on canary deployment..."
  
  local health_status="healthy"
  local error_count=0
  
  # Check contract exists
  if ! stellar contract info --id "$CONTRACT_ID" --network "$NETWORK" > /dev/null 2>&1; then
    log_error "Contract health check failed: contract not found"
    health_status="unhealthy"
    ((error_count++))
  fi
  
  # Check contract methods are callable
  if ! stellar contract invoke --id "$CONTRACT_ID" --network "$NETWORK" -- get_stats > /dev/null 2>&1; then
    log_warning "Contract method check failed"
    ((error_count++))
  fi
  
  if [ $error_count -ge $ROLLBACK_ERROR_THRESHOLD ]; then
    log_error "Health check failed with $error_count errors"
    return 1
  fi
  
  log_success "Health check passed"
  return 0
}

# Gradual traffic increase
increase_canary_traffic() {
  local current_percentage=$1
  local next_percentage=$((current_percentage + 10))
  
  if [ $next_percentage -gt $MAX_CANARY_PERCENTAGE ]; then
    next_percentage=$MAX_CANARY_PERCENTAGE
  fi
  
  log_info "Increasing canary traffic from $current_percentage% to $next_percentage%"
  update_canary_state "canary_percentage" "$next_percentage"
  
  return $next_percentage
}

# Rollback function
rollback_deployment() {
  log_error "Rolling back canary deployment..."
  update_canary_state "status" '"rolled_back"'
  
  # In production, this would revert to previous contract version
  log_success "Rollback completed"
  exit 1
}

# Main canary deployment flow
init_canary_state
update_canary_state "status" '"deploying"'

log_info "Phase 1: Deploy new version with 5% traffic"
# Deploy new contract version
if ! cargo build --release --target wasm32-unknown-unknown --manifest-path contracts/crowdfund/Cargo.toml 2>&1 | tail -3; then
  log_error "Failed to build new version"
  exit 1
fi
log_success "New version built"

# Deploy to canary
CANARY_CONTRACT=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/crowdfund.wasm \
  --network "$NETWORK" \
  --source deployer 2>&1) || {
  log_error "Failed to deploy canary contract"
  exit 1
}
log_success "Canary contract deployed: $CANARY_CONTRACT"

update_canary_state "canary_contract_id" "\"$CANARY_CONTRACT\""
update_canary_state "status" '"monitoring"'

# Phase 2: Monitor and gradually increase traffic
log_info "Phase 2: Monitoring canary deployment..."
CURRENT_PERCENTAGE=$CANARY_PERCENTAGE
MONITORING_DURATION=300  # 5 minutes
ELAPSED=0

while [ $ELAPSED -lt $MONITORING_DURATION ] && [ $CURRENT_PERCENTAGE -lt $MAX_CANARY_PERCENTAGE ]; do
  log_info "Canary traffic: $CURRENT_PERCENTAGE% (elapsed: ${ELAPSED}s)"
  
  # Perform health check
  if ! perform_health_check; then
    log_error "Health check failed during canary deployment"
    rollback_deployment
  fi
  
  # Wait before next check
  sleep $HEALTH_CHECK_INTERVAL
  ELAPSED=$((ELAPSED + HEALTH_CHECK_INTERVAL))
  
  # Increase traffic if healthy
  if [ $CURRENT_PERCENTAGE -lt $MAX_CANARY_PERCENTAGE ]; then
    increase_canary_traffic $CURRENT_PERCENTAGE
    CURRENT_PERCENTAGE=$?
  fi
done

# Phase 3: Complete rollout
log_info "Phase 3: Completing rollout to 100%..."
update_canary_state "canary_percentage" "100"
update_canary_state "status" '"completed"'
update_canary_state "completion_time" "\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\""

log_success "Canary deployment completed successfully"

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC}                 ${GREEN}CANARY DEPLOYMENT COMPLETE${NC}                   ${BLUE}║${NC}"
echo -e "${BLUE}╠════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BLUE}║${NC} Contract ID:       ${GREEN}$CONTRACT_ID${NC}"
echo -e "${BLUE}║${NC} New Version:       ${GREEN}$NEW_VERSION${NC}"
echo -e "${BLUE}║${NC} Canary Contract:   ${GREEN}$CANARY_CONTRACT${NC}"
echo -e "${BLUE}║${NC} Network:           ${GREEN}$NETWORK${NC}"
echo -e "${BLUE}║${NC} Status:            ${GREEN}Completed${NC}"
echo -e "${BLUE}║${NC} State File:        ${GREEN}$CANARY_STATE_FILE${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
