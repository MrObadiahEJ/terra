#!/usr/bin/env bash
# preflight-devnet.sh - Check everything that must be true before `deploy.sh devnet`.
#
# Read-only. Prints the estimated rent, the current balance, and the shortfall.
# Exits non-zero if a hard prerequisite (keypair mismatch, missing artifact)
# fails, so it can gate CI/manual deploys.
#
# Usage:
#   ./preflight-devnet.sh              # devnet
#   CLUSTER=mainnet ./preflight-devnet.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

CLUSTER="${CLUSTER:-devnet}"
DEPLOY_KEYPAIR="${DEPLOY_KEYPAIR:-$HOME/.config/solana/id.json}"

fail=0
note() { printf '  %s\n' "$*"; }
ok() { printf 'OK   %s\n' "$*"; }
bad() { printf 'FAIL %s\n' "$*"; fail=1; }

echo "==> Preflight for: $CLUSTER"

for bin in solana solana-keygen anchor; do
    if command -v "$bin" >/dev/null 2>&1; then
        ok "$bin found"
    else
        bad "$bin not found in PATH"
    fi
done

if [[ -f "$DEPLOY_KEYPAIR" ]]; then
    ok "deployer keypair: $DEPLOY_KEYPAIR ($(solana-keygen pubkey "$DEPLOY_KEYPAIR"))"
else
    bad "deployer keypair missing: $DEPLOY_KEYPAIR"
fi

echo
echo "==> Program keypairs (must match declare_id!)"
for name in terra_registry terra_identity; do
    declared=$(sed -n 's/.*declare_id!("\([^"]*\)").*/\1/p' "programs/$name/src/lib.rs")
    kp="target/deploy/${name}-keypair.json"
    if [[ ! -f "$kp" ]]; then
        bad "$kp missing (restore from deploy-backup/)"
        continue
    fi
    actual=$(solana-keygen pubkey "$kp")
    if [[ "$actual" == "$declared" ]]; then
        ok "$name -> $actual"
    else
        bad "$name keypair is $actual but declare_id! is $declared"
    fi
done

echo
echo "==> Build artifacts"
total_bytes=0
for name in terra_registry terra_identity; do
    so="target/deploy/${name}.so"
    if [[ -f "$so" ]]; then
        bytes=$(stat -c%s "$so")
        total_bytes=$((total_bytes + bytes))
        ok "$so ($((bytes / 1024)) KiB)"
    else
        bad "$so missing — run: anchor build --skip-lint"
    fi
done

echo
echo "==> Funding estimate"
if [[ -f "$DEPLOY_KEYPAIR" ]] && command -v solana >/dev/null 2>&1; then
    # `solana rent` prints the rent-exempt minimum for a given account size.
    rent_lamports=0
    for name in terra_registry terra_identity; do
        so="target/deploy/${name}.so"
        [[ -f "$so" ]] || continue
        bytes=$(stat -c%s "$so")
        line=$(solana rent "$bytes" 2>/dev/null | head -1 || true)
        lamports=$(sed -n 's/.*rent-exempt minimum: \([0-9.]*\) SOL.*/\1/p' <<<"$line")
        if [[ -n "${lamports:-}" ]]; then
            sol=$(awk -v l="$lamports" 'BEGIN { printf "%.6f", l }')
        else
            # Fallback: 6.96 lamports/byte-year * 2 years = 13.92 lamports/byte,
            # plus ~128 bytes of account overhead.
            sol=$(awk -v b="$bytes" 'BEGIN { printf "%.6f", (b + 128) * 13.92 / 1000000000 }')
        fi
        note "$name rent: ~$sol SOL"
        rent_lamports=$(awk -v a="$rent_lamports" -v s="$sol" 'BEGIN { printf "%.0f", a + s * 1000000000 }')
    done

    balance=$(solana balance "$DEPLOY_KEYPAIR" --url "$CLUSTER" 2>/dev/null | awk '{print $1}' || echo "0")
    note "deployer balance: $balance SOL"
    shortfall=$(awk -v b="$balance" -v r="$rent_lamports" 'BEGIN { s = r / 1000000000 - b; if (s < 0) s = 0; printf "%.4f", s }')
    if [[ "$shortfall" == "0.0000" ]]; then
        ok "balance covers estimated rent"
    else
        note "estimated shortfall: $shortfall SOL"
        note "faucet: https://faucet.solana.com (devnet) or transfer SOL to the deployer"
        fail=1
    fi
else
    note "skipping balance check (missing keypair or solana CLI)"
fi

echo
echo "==> Environment checklist (for the hosted API)"
for var in DATABASE_URL TERRA_RPC_URL API_AUTHORITY_PUBKEY CORS_ALLOWED_ORIGINS; do
    if [[ -n "${!var:-}" ]]; then
        ok "$var is set"
    else
        note "$var is not set in this shell (set it in Render, not in the repo)"
    fi
done
for var in S3_BUCKET S3_ENDPOINT S3_ACCESS_KEY S3_SECRET_KEY; do
    if [[ -n "${!var:-}" ]]; then
        ok "$var is set"
    else
        note "$var is not set (evidence will use ephemeral local disk)"
    fi
done

echo
if [[ "$fail" -ne 0 ]]; then
    echo "Preflight FAILED — resolve the items above before deploying."
    exit 1
fi
echo "Preflight PASSED — run: ./deploy.sh $CLUSTER"
