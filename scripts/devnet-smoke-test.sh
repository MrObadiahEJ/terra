#!/usr/bin/env bash
# Devnet/hosted smoke test for Terra. No secrets required.
# Usage: ./scripts/devnet-smoke-test.sh
# Optional env overrides: TERRA_API_BASE, TERRA_WEB_URL, TERRA_DEV_WALLET,
# TERRA_REGISTRY_PROGRAM, TERRA_IDENTITY_PROGRAM, TERRA_RPC_URL
set -uo pipefail

API="${TERRA_API_BASE:-https://terra-gscx.onrender.com}"
WEB="${TERRA_WEB_URL:-https://mrobadiahej.github.io/terra/}"
WALLET="${TERRA_DEV_WALLET:-AK3LmJmQijTJbsqj1PGt1hfXtdHirhNZybhPrYeRPvs3}"
REGISTRY="${TERRA_REGISTRY_PROGRAM:-GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage}"
IDENTITY="${TERRA_IDENTITY_PROGRAM:-HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p}"
RPC_URL="${TERRA_RPC_URL:-https://api.devnet.solana.com}"

pass=0; fail=0; skip=0
green(){ printf '\033[32mPASS\033[0m %s\n' "$1"; }
red(){ printf '\033[31mFAIL\033[0m %s\n' "$1"; }
yellow(){ printf '\033[33mSKIP\033[0m %s\n' "$1"; }

check_http() {
  local name="$1" url="$2" expect="${3:-200}"
  local code body
  body=$(mktemp)
  code=$(curl -sS -o "$body" -w "%{http_code}" --max-time 20 "$url" || true)
  if [ "$code" = "$expect" ]; then green "$name ($code)"; pass=$((pass+1))
  else red "$name (got $code, want $expect)"; head -c 200 "$body"; echo; fail=$((fail+1)); fi
  rm -f "$body"
}

check_http_json() {
  local name="$1" url="$2" pattern="$3"
  local code body
  body=$(mktemp)
  code=$(curl -sS -o "$body" -w "%{http_code}" --max-time 20 "$url" || true)
  if [ "$code" = "200" ] && grep -q "$pattern" "$body"; then green "$name"; pass=$((pass+1))
  else red "$name (code=$code)"; head -c 200 "$body"; echo; fail=$((fail+1)); fi
  rm -f "$body"
}

echo "== Terra devnet/hosted smoke test =="
echo "API=$API"
echo "WEB=$WEB"
echo "RPC=$RPC_URL"
echo

echo "-- HTTP/API --"
check_http_json "API health" "$API/health" '"status":"ok"'
check_http_json "API db up" "$API/health" '"database":"up"'
check_http "Mirror audit" "$API/api/v1/mirror/audit"
check_http "Parcels" "$API/api/v1/parcels"
check_http "Stories" "$API/api/v1/stories"
check_http "Providers" "$API/api/v1/providers"
check_http "Settlement assets" "$API/api/v1/providers/settlement/assets"
check_http "Pages root" "$WEB"

echo
echo "-- Frontend bundle config --"
if command -v curl >/dev/null 2>&1; then
  js_paths=$(curl -sS --max-time 20 "$WEB" | grep -oE '/terra/assets/[^\"]+\.js' || true)
  if [ -n "$js_paths" ]; then
    bundle=$(mktemp)
    : > "$bundle"
    while IFS= read -r js_path; do
      [ -n "$js_path" ] || continue
      curl -sS --max-time 30 "https://mrobadiahej.github.io${js_path}" >> "$bundle" || true
    done <<< "$js_paths"
    for needle in "terra-gscx.onrender.com" "devnet.helius-rpc.com" "$REGISTRY"; do
      if grep -q "$needle" "$bundle"; then green "Bundle contains $needle"; pass=$((pass+1))
      else red "Bundle missing $needle"; fail=$((fail+1)); fi
    done
    rm -f "$bundle"
  else
    yellow "Could not locate Pages JS bundle"; skip=$((skip+1))
  fi
fi

echo
echo "-- On-chain (solana CLI) --"
if command -v solana >/dev/null 2>&1; then
  bal=$(solana balance "$WALLET" --url "$RPC_URL" 2>/tmp/terra-smoke.err || true)
  if [ -n "$bal" ]; then green "Wallet balance: $bal"; pass=$((pass+1))
  else red "Wallet balance"; cat /tmp/terra-smoke.err || true; fail=$((fail+1)); fi

  if solana program show "$REGISTRY" --url "$RPC_URL" >/tmp/terra-smoke.out 2>&1; then
    green "Registry program deployed"; tail -2 /tmp/terra-smoke.out; pass=$((pass+1))
  else red "Registry program"; tail -5 /tmp/terra-smoke.out || true; fail=$((fail+1)); fi

  if solana program show "$IDENTITY" --url "$RPC_URL" >/tmp/terra-smoke.out 2>&1; then
    green "Identity program deployed"; tail -2 /tmp/terra-smoke.out; pass=$((pass+1))
  else red "Identity program"; tail -5 /tmp/terra-smoke.out || true; fail=$((fail+1)); fi
else
  yellow "solana CLI not found; skipping on-chain checks"; skip=$((skip+1))
fi

echo
echo "== Summary: PASS=$pass FAIL=$fail SKIP=$skip =="
[ "$fail" -eq 0 ]
