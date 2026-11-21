#!/usr/bin/env bash
# Generate a local, DEVELOPMENT-ONLY Groth16 setup for the membership circuit
# and print the environment the API needs.
#
# These keys give whoever holds them the ability to forge proofs. Use them only
# on a laptop/devnet. Production requires an independent multi-party ceremony
# and an audit (docs/book-of-intents/OPEN_QUESTIONS.md Q-009).
#
# Usage:  ./scripts/zk-dev-setup.sh [output-dir]
#         (default: terra-core/.zk, already gitignored)
set -euo pipefail

cd "$(dirname "$0")/.."

out_dir="${1:-${TERRA_ZK_DIR:-$PWD/.zk}}"
mkdir -p "$out_dir"
out_dir="$(cd "$out_dir" && pwd)"

pk="$out_dir/membership.pk"
vk="$out_dir/membership.vk"

if [[ -e "$pk" || -e "$vk" ]]; then
  echo "refusing to overwrite existing setup in $out_dir" >&2
  echo "remove $pk and $vk first if you really want a new setup" >&2
  exit 1
fi

echo "==> generating Groth16 keys (release build; this takes a few minutes)" >&2
cargo run --quiet --release -p terra-zk-membership --bin setup-dev -- "$pk" "$vk"

echo "==> verification key identity" >&2
info="$(cargo run --quiet --release -p terra-zk-membership --bin zk-info -- --vk "$vk" --json)"
vk_hash="$(printf '%s' "$info" | sed -E 's/.*"verification_key_hash":"([0-9a-f]+)".*/\1/')"

env_file="$out_dir/env.sh"
cat > "$env_file" <<EOF
# Generated $(date -u +%Y-%m-%dT%H:%M:%SZ). DEVELOPMENT-ONLY keys.
export TERRA_ZK_MODE=development
export TERRA_ZK_PROVING_KEY_PATH="$pk"
export TERRA_ZK_VERIFYING_KEY_PATH="$vk"
export TERRA_ZK_VERIFICATION_KEY_HASH="$vk_hash"
EOF

cat <<EOF

Development ZK setup ready.

  proving key:            $pk
  verifying key:          $vk
  verification key hash:  $vk_hash

Load it into the API shell:

  source $env_file

Pin the same hash on-chain before accepting proofs (zone authority):

  update_verification_key_hash(<${vk_hash}>)

Keep PRODUCTION deployments at TERRA_ZK_MODE=production (verification only) or
unset (disabled). Never use these keys on mainnet: the setup is not trustworthy.
EOF
