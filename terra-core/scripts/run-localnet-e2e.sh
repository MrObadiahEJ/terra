#!/usr/bin/env bash
# run-localnet-e2e.sh — Sprint A end-to-end lifecycle smoke on Surfpool.
#
# Starts a local Surfnet, deploys both programs from the committed runbook
# (instant deployment), runs `tests/localnet_e2e.ts`, and tears the Surfnet
# down. No solana-test-validator, no AVX requirement.
#
# Inputs (all optional):
#   SURFPOOL_BIN  path to the surfpool binary            (default: surfpool)
#   RPC_PORT      JSON-RPC port                          (default: 8899)
#   WS_PORT       WebSocket port                         (default: 8900)
#   LOG_DIR       directory for surfpool logs            (default: .surfpool/e2e)
#
# Requirements:
#   - target/deploy/{terra_registry,terra_identity}.so (CI artifact or anchor build)
#   - idl/{terra_registry,terra_identity}.json        (committed)
#   - node_modules (@solana/web3.js is used to create a payer keypair if absent)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SURFPOOL_BIN="${SURFPOOL_BIN:-surfpool}"
RPC_PORT="${RPC_PORT:-8899}"
WS_PORT="${WS_PORT:-8900}"
RPC_URL="http://127.0.0.1:${RPC_PORT}"
LOG_DIR="${LOG_DIR:-$ROOT/.surfpool/e2e}"
LOG="$LOG_DIR/surfpool.log"
mkdir -p "$LOG_DIR"

REGISTRY_ID="GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage"
IDENTITY_ID="HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p"

fail() {
  echo "ERROR: $*" >&2
  echo "--- surfpool log (tail) ---" >&2
  tail -120 "$LOG" >&2 || true
  exit 1
}

if ! command -v "$SURFPOOL_BIN" >/dev/null 2>&1; then
  echo "ERROR: surfpool binary not found: $SURFPOOL_BIN" >&2
  echo "  install v1.7.0 or set SURFPOOL_BIN=/path/to/surfpool" >&2
  exit 1
fi

for so in terra_registry terra_identity; do
  [[ -f "target/deploy/${so}.so" ]] || {
    echo "ERROR: target/deploy/${so}.so missing." >&2
    echo "  build with 'anchor build' or download the CI 'program-artifacts' bundle." >&2
    exit 1
  }
done
for idl in terra_registry terra_identity; do
  [[ -f "idl/${idl}.json" ]] || {
    echo "ERROR: idl/${idl}.json missing (committed IDL required by the runbook)." >&2
    exit 1
  }
done

# The runbook reads both the .so and the IDL from ./target.
mkdir -p target/idl
cp idl/terra_registry.json target/idl/terra_registry.json
cp idl/terra_identity.json target/idl/terra_identity.json

# Surfpool's runbook signer is ~/.config/solana/id.json. Create an ephemeral
# one when missing (CI) so deployment has a fee payer/authority.
PAYER_KEYPAIR="$HOME/.config/solana/id.json"
if [[ ! -f "$PAYER_KEYPAIR" ]]; then
  echo "==> Creating ephemeral deploy keypair at $PAYER_KEYPAIR"
  mkdir -p "$(dirname "$PAYER_KEYPAIR")"
  node -e '
    const { Keypair } = require("@solana/web3.js");
    const fs = require("fs");
    const kp = Keypair.generate();
    fs.writeFileSync(process.argv[1], JSON.stringify(Array.from(kp.secretKey)));
    console.log("payer:", kp.publicKey.toBase58());
  ' "$PAYER_KEYPAIR"
  chmod 600 "$PAYER_KEYPAIR"
fi

cleanup() {
  if [[ -n "${SURF_PID:-}" ]] && kill -0 "$SURF_PID" 2>/dev/null; then
    kill "$SURF_PID" 2>/dev/null || true
    wait "$SURF_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

rpc() {
  curl -sf -X POST -H 'Content-Type: application/json' \
    --data "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\",\"params\":${2:-[]}}" \
    "$RPC_URL"
}

account_exists() {
  rpc getAccountInfo "[\"$1\",{\"encoding\":\"base64\"}]" 2>/dev/null \
    | grep -q '"value":{'
}

echo "==> Starting Surfpool ($("$SURFPOOL_BIN" --version 2>/dev/null | head -1))"
"$SURFPOOL_BIN" start --offline --no-tui --yes \
  --port "$RPC_PORT" --ws-port "$WS_PORT" >"$LOG" 2>&1 &
SURF_PID=$!

echo -n "==> Waiting for RPC"
for _ in $(seq 1 120); do
  if rpc getVersion >/dev/null 2>&1; then
    echo " — up"
    break
  fi
  if ! kill -0 "$SURF_PID" 2>/dev/null; then
    fail "surfpool exited during startup"
  fi
  echo -n "."
  sleep 1
done
rpc getVersion >/dev/null 2>&1 || fail "RPC did not become ready at $RPC_URL"

echo -n "==> Waiting for program deployment"
for _ in $(seq 1 120); do
  if account_exists "$REGISTRY_ID" && account_exists "$IDENTITY_ID"; then
    echo " — both programs live"
    break
  fi
  if ! kill -0 "$SURF_PID" 2>/dev/null; then
    fail "surfpool exited while deploying programs"
  fi
  echo -n "."
  sleep 1
done
account_exists "$REGISTRY_ID" || fail "terra_registry not deployed at $REGISTRY_ID"
account_exists "$IDENTITY_ID" || fail "terra_identity not deployed at $IDENTITY_ID"

echo "==> Running lifecycle smoke (tests/localnet_e2e.ts)"
if ! TERRA_RPC_URL="$RPC_URL" \
  ./node_modules/.bin/ts-mocha -p tsconfig.json tests/localnet_e2e.ts; then
  fail "localnet lifecycle smoke failed"
fi

echo "==> Localnet lifecycle smoke passed."
