#!/usr/bin/env bash
# deploy-devnet.sh — Orchestrator: preflight → fund-check → deploy → init → smoke.
#
# Chains the devnet deployment steps together. Does NOT duplicate deploy logic
# (that lives in deploy.sh and init-devnet.ts).
#
# Usage:
#   ./scripts/deploy-devnet.sh
#
# Environment (all optional with sensible defaults):
#   DEPLOY_KEYPAIR      Path to durable deploy keypair (default: ~/.config/solana/id.json)
#   ORACLE_KEYPAIR      Path to persisted oracle keypair (default: .devnet/oracle-keypair.json)
#   VALIDATOR_KEYPAIRS  Comma-separated validator keypair paths
#   CLUSTER             Target cluster (default: devnet)
#   SPRINTS_DIR         Directory for smoke transcripts (default: sprints/)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

CLUSTER="${CLUSTER:-devnet}"
DEPLOY_KEYPAIR="${DEPLOY_KEYPAIR:-$HOME/.config/solana/id.json}"
ORACLE_KEYPAIR="${ORACLE_KEYPAIR:-.devnet/oracle-keypair.json}"
VALIDATOR_KEYPAIRS="${VALIDATOR_KEYPAIRS:-}"
SPRINTS_DIR="${SPRINTS_DIR:-sprints}"

fail() { echo "ERROR: $*" >&2; exit 1; }

# --- 1. Preflight ----------------------------------------------------------
echo "==> Step 1: preflight"
./preflight-devnet.sh "$CLUSTER"

# --- 2. Fund-check loop ----------------------------------------------------
echo "==> Step 2: fund-check"
if ! command -v solana >/dev/null 2>&1; then
    fail "solana CLI not found in PATH"
fi

solana config set --url "$CLUSTER" 2>&1

BALANCE=$(solana balance "$DEPLOY_KEYPAIR" 2>&1 | head -1)
echo "==> Deployer balance: $BALANCE"

# Rough estimate: ~15.7 SOL total rent for both programs + accounts (~$2.8k at $180)
# Each airdrop on devnet gives ~2 SOL. Rotate faucet URLs if needed.
FAUCET_URLS="${FAUCET_URLS:-https://api.devnet.solana.com}"
FAUCET_LIST=()
IFS=',' read -ra FAUCET_LIST <<< "$FAUCET_URLS"

attempt=0
max_attempts=20
while true; do
    BALANCE_LAMPORTS=$(solana balance "$DEPLOY_KEYPAIR" --lamports 2>/dev/null | head -1 || echo "0")
    if [[ "$BALANCE_LAMPORTS" -gt 15000000000 ]]; then
        echo "==> Sufficient balance: $(solana balance "$DEPLOY_KEYPAIR" 2>&1 | head -1)"
        break
    fi
    attempt=$((attempt + 1))
    if [[ $attempt -gt $max_attempts ]]; then
        fail "Insufficient deployer balance after $max_attempts faucet attempts. Current: $BALANCE_LAMPORTS lamports"
    fi
    faucet_url="${FAUCET_LIST[$((attempt % ${#FAUCET_LIST[@]}))]}"
    echo "==> Funding attempt $attempt via $faucet_url (balance: $BALANCE_LAMPORTS lamports)"
    # requestAirdrop via JSON-RPC to the faucet URL
    resp=$(curl -sf -X POST -H 'Content-Type: application/json' \
        --data '{"jsonrpc":"2.0","id":1,"method":"requestAirdrop","params":["'"$DEPLOY_KEYPAIR"'", 2000000000]}' \
        "$faucet_url" 2>/dev/null || echo '{}')
    if echo "$resp" | grep -q '"result"'; then
        echo "  Airdrop accepted, waiting for confirmation..."
        sleep 8
    else
        echo "  Faucet $faucet_url declined, rotating..."
        sleep 3
    fi
done

# --- 3. Deploy -------------------------------------------------------------
echo "==> Step 3: deploy programs"
./deploy.sh "$CLUSTER"

# --- 4. Initialize ---------------------------------------------------------
echo "==> Step 4: initialize protocol (init-devnet.ts)"

# Ensure .devnet directory exists for persisted oracle/validator keypairs
mkdir -p .devnet

# Generate persisted oracle keypair if absent (durable across sessions)
if [[ ! -f "$ORACLE_KEYPAIR" ]]; then
    echo "==> Generating oracle keypair at $ORACLE_KEYPAIR"
    solana-keygen new --no-passphrase -o "$ORACLE_KEYPAIR" --silent
fi

# Generate validator keypairs if absent
ORACLE_PUBKEY=$(solana-keygen pubkey "$ORACLE_KEYPAIR")
VALIDATOR_KEYPAIR_ARGS=""
if [[ -n "$VALIDATOR_KEYPAIRS" ]]; then
    IFS=',' read -ra VK_ARRAY <<< "$VALIDATOR_KEYPAIRS"
    for vk in "${VK_ARRAY[@]}"; do
        vk=$(echo "$vk" | xargs)
        if [[ ! -f "$vk" ]]; then
            echo "==> Generating validator keypair at $vk"
            solana-keygen new --no-passphrase -o "$vk" --silent
        fi
        VALIDATOR_KEYPAIR_ARGS="$VALIDATOR_KEYPAIRS"
    done
else
    # Default: create 2 validator keypairs in .devnet/
    for i in 1 2; do
        vk=".devnet/validator-${i}-keypair.json"
        if [[ ! -f "$vk" ]]; then
            echo "==> Generating validator keypair at $vk"
            solana-keygen new --no-passphrase -o "$vk" --silent
        fi
        VALIDATOR_KEYPAIR_ARGS="$VALIDATOR_KEYPAIR_ARGS,$vk"
    done
    VALIDATOR_KEYPAIR_ARGS="${VALIDATOR_KEYPAIR_ARGS#,}"
fi

echo "==> Oracle pubkey: $ORACLE_PUBKEY"
echo "==> Validator keypairs: $VALIDATOR_KEYPAIR_ARGS"

# Run init-devnet.ts with durable keys
DEPLOY_KEYPAIR="$DEPLOY_KEYPAIR" \
    ORACLE_KEYPAIR="$ORACLE_KEYPAIR" \
    VALIDATOR_KEYPAIRS="$VALIDATOR_KEYPAIR_ARGS" \
    TERRA_RPC_URL="$CLUSTER" \
    npx ts-node scripts/init-devnet.ts

# --- 5. Smoke transcript ---------------------------------------------------
echo "==> Step 5: smoke transcript"

mkdir -p "$SPRINTS_DIR"
SMOKE_FILE="$SPRINTS_DIR/devnet-smoke-$(date -u +%Y%m%d-%H%M%S).md"

cat > "$SMOKE_FILE" <<SMOKEEOF
# Devnet Smoke Transcript
Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Cluster: $CLUSTER
Deployer: $(solana-keygen pubkey "$DEPLOY_KEYPAIR")
Oracle: $ORACLE_PUBKEY
Validators: $VALIDATOR_KEYPAIR_ARGS

## Steps completed
1. Preflight: PASSED
2. Fund-check: PASSED (balance: $BALANCE_LAMPORTS lamports)
3. Deploy: terra_identity + terra_registry deployed via deploy.sh $CLUSTER
4. Initialize: init-devnet.ts ran (registry, VRF oracle, validators, fee policy, coverage incentive)

## Verification commands
solana program show GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage --url $CLUSTER
solana program show HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p --url $CLUSTER

## Environment variables for API
export VRF_ORACLE_PUBKEY=$ORACLE_PUBKEY
export TERRA_RPC_URL=$CLUSTER
export DEPLOY_KEYPAIR=$DEPLOY_KEYPAIR

SMOKEEOF

echo "==> Devnet deployment complete!"
echo "    Smoke transcript: $SMOKE_FILE"
echo "    VRF_ORACLE_PUBKEY=$ORACLE_PUBKEY (set this in the API environment)"
echo "    Runbooks: see runbooks/deployment/main.tx for post-deploy ops"
