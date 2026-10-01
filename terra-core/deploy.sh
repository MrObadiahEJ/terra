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

# --- Program-ID sanity ----------------------------------------------------
# anchor deploy installs each program at the address of
# target/deploy/<name>-keypair.json. If that keypair does not match
# declare_id! (e.g. it was regenerated), the program lands on an address
# that matches neither the source nor Anchor.toml — every PDA derivation
# (crate::ID seeds) and every client using the IDL then points at nothing.
# Fail fast, before spending build time.
verify_program_keypair() {
    local name="$1"
    local declared actual anchor_ids

    declared=$(sed -n 's/.*declare_id!("\([^"]*\)").*/\1/p' "programs/$name/src/lib.rs")
    if [[ -z "$declared" ]]; then
        echo "ERROR: declare_id! not found in programs/$name/src/lib.rs"
        exit 1
    fi

    # Anchor.toml must agree with declare_id! for every cluster section.
    anchor_ids=$(grep -E "^${name} = \"" Anchor.toml | sed 's/.*"\(.*\)"/\1/' | sort -u)
    if [[ -n "$anchor_ids" && "$anchor_ids" != "$declared" ]]; then
        echo "ERROR: Anchor.toml ${name} id(s) [$anchor_ids] != declare_id! ($declared)"
        exit 1
    fi

    local kp="target/deploy/${name}-keypair.json"
    if [[ ! -f "$kp" ]]; then
        echo "ERROR: $kp is missing."
        echo "  Provide the original deploy keypair for $declared"
        echo "  (anchor build generates a FRESH keypair here, which is wrong)."
        if [[ "$name" == "terra_registry" && -f deploy-backup/terra_registry-keypair.json ]]; then
            echo "  fix: cp deploy-backup/terra_registry-keypair.json $kp"
        fi
        exit 1
    fi

    actual=$(solana-keygen pubkey "$kp")
    if [[ "$actual" != "$declared" ]]; then
        echo "ERROR: $kp belongs to $actual but programs/$name declares $declared."
        echo "  Deploying with it would install the program at the WRONG address."
        if [[ "$name" == "terra_registry" && -f deploy-backup/terra_registry-keypair.json ]]; then
            echo "  fix: cp deploy-backup/terra_registry-keypair.json $kp"
        else
            echo "  Restore the original keypair for $declared and re-run."
        fi
        exit 1
    fi
    echo "==> ${name} keypair matches declare_id! ($actual)"
}

echo "==> Verifying program keypairs..."
verify_program_keypair terra_registry
verify_program_keypair terra_identity
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
echo "    terra_identity: HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p"
echo ""
echo "==> Verify with: solana program show <PROGRAM_ID> --url $CLUSTER"
