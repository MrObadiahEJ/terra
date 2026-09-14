#!/usr/bin/env bash
# deploy.sh - Deploy programs to a Solana cluster
#
# Usage:
#   ./deploy.sh devnet          # Deploy to devnet
#   ./deploy.sh mainnet         # Deploy to mainnet (requires --confirm)
#   ./deploy.sh localnet        # Deploy to localnet (requires local validator)
#
# Environment variables:
#   DEPLOY_KEYPAIR  - Path to the deploy keypair (default: ~/.config/solana/id.json)
#   CLUSTER         - Override cluster (default: derived from argument)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

CLUSTER="${1:-devnet}"
DEPLOY_KEYPAIR="${DEPLOY_KEYPAIR:-$HOME/.config/solana/id.json}"

# Safety check for mainnet
if [[ "$CLUSTER" == "mainnet" ]]; then
    if [[ "${CONFIRM_MAINNET:-}" != "yes" ]]; then
        echo "WARNING: You are about to deploy to MAINNET."
        echo "This will cost real SOL and may be irreversible."
        echo ""
        echo "Run with CONFIRM_MAINNET=yes to proceed:"
        echo "  CONFIRM_MAINNET=yes ./deploy.sh mainnet"
        exit 1
    fi
fi

echo "==> Deploying to: $CLUSTER"
echo "==> Deploy keypair: $DEPLOY_KEYPAIR"
echo ""

# Set cluster
solana config set --url "$CLUSTER" 2>&1

# Check balance
BALANCE=$(solana balance "$DEPLOY_KEYPAIR" 2>&1 | head -1)
echo "==> Wallet balance: $BALANCE"
echo ""

# Build first
echo "==> Building programs..."
"$SCRIPT_DIR/build.sh"
echo ""

# Deploy terra_identity first (terra_registry depends on it)
echo "==> Deploying terra_identity..."
anchor deploy --provider.cluster "$CLUSTER" --program-name terra_identity 2>&1
echo ""

# Deploy terra_registry
echo "==> Deploying terra_registry..."
anchor deploy --provider.cluster "$CLUSTER" --program-name terra_registry 2>&1
echo ""

echo "==> Deployment complete!"
echo "    terra_registry: GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage"
echo "    terra_identity: 68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4"
echo ""
echo "==> Verify with: solana program show <PROGRAM_ID> --url $CLUSTER"
