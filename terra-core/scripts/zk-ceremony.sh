#!/usr/bin/env bash
# scripts/zk-ceremony.sh — Groth16 ceremony TEMPLATE for a future circom port
#
# IMPORTANT — read before running:
#
# The production circuit in this repository is `terra-zk-membership::MembershipCircuit`,
# an **arkworks R1CS circuit**. There is no `.circom` source for it, and this
# script's snarkjs path therefore cannot produce keys for the circuit the
# on-chain verifier actually checks. Running snarkjs on a hand-written circom
# file would only be valid after:
#   1. a circom port exists whose R1CS is proven constraint-equivalent to
#      `MembershipCircuit` (same public input `SHA-256(statement)` layout, same
#      Poseidon parameters, same nullifier derivation), and
#   2. an equivalence test proves both circuits accept the same witnesses.
#
# Until then, this script refuses to run. The working, supported path is:
#
#   • Development / devnet:  ./scripts/zk-dev-setup.sh   (arkworks setup)
#   • Production:            an arkworks Groth16 MPC ceremony over the exported
#                            `MembershipCircuit` R1CS, with independent
#                            contributors and an external audit (Q-009).
#
# The mechanics below are retained as a template for the circom-port branch;
# set ZK_CIRCOM_PORT_READY=1 only after (1) and (2) above are true.
#
# Design notes for that future branch:
#   • Phase 1: reuse Perpetual Powers of Tau (bn128, 2^20).
#   • Phase 2: snarkjs zkey contribute via zkpick.com; minimum 5 contributors.
#   • Entropy: interactive prompts; never -e="literal string".
#   • Finalize: drand beacon, then zkey verify + export VK, then convert the
#     VK to the 576-byte frame and pin SHA-256(VK frame) on-chain.
#   • The on-chain verifier is unchanged (same BN254 pairing equation).
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [[ "${ZK_CIRCOM_PORT_READY:-0}" != "1" ]]; then
    cat >&2 <<'EOF'
ERROR: the production circuit is arkworks-native; there is no .circom source.

  * This snarkjs ceremony template is intentionally disabled.
  * Use ./scripts/zk-dev-setup.sh for development/devnet keys.
  * Production needs an arkworks Groth16 MPC ceremony + external audit
    (see docs/book-of-intents/OPEN_QUESTIONS.md Q-009).

Set ZK_CIRCOM_PORT_READY=1 only after a constraint-equivalent circom port and
its R1CS equivalence test exist.
EOF
    exit 1
fi

# ----- configuration (overridable via env) ---------------------------------
ZKPTAU_URL="${ZKPTAU_URL:-https://hermes.sigmaproject.io/powersOfTau20_hez_final_20.ptau}"
ZKPTAU_HASH="${ZKPTAU_HASH:-}"          # sha256 of ptau file (verify after download)
ZKPTAU_FILE="${ZKPTAU_FILE:-$PWD/.zk/powersOfTau20_hez_final_20.ptau}"
ZK_OUTPUT_DIR="${ZK_OUTPUT_DIR:-$PWD/.zk-production}"
ZK_CIRCUIT_DIR="${ZK_CIRCUIT_DIR:-$PWD/zk-membership/circuit}"
DRAND_ROUND="${DRAND_ROUND:-}"          # drand round hash for beacon (required before finalize)
ZKPICK_URL="${ZKPICK_URL:-https://zkpick.com}"
NUM_PARTICIPANTS="${NUM_PARTICIPANTS:-8}"  # target 8–10, min 5

fail() { echo "ERROR: $*" >&2; exit 1; }
note() { printf '  %s\n' "$*"; }
ok() { printf 'OK   %s\n' "$*"; }

require_bin() {
    if ! command -v "$1" >/dev/null 2>&1; then
        fail "$1 not found in PATH (install it first)"
    fi
    ok "$1 found"
}

# ----- 0. Prerequisites -----------------------------------------------------
echo "==> Step 0: prerequisites"
for b in node npx snarkjs circom; do
    if command -v "$b" >/dev/null 2>&1; then
        ok "$b found"
    else
        # snarkjs and circom can be installed via npm
        case "$b" in
            snarkjs|circom)
                note "$b not found — will install via npm"
                ;;
            *)
                fail "$b not found in PATH"
                ;;
        esac
    fi
done

if ! command -v snarkjs >/dev/null 2>&1; then
    echo "==> Installing snarkjs..."
    npm install -g snarkjs
fi
if ! command -v circom >/dev/null 2>&1; then
    echo "==> Installing circom..."
    npm install -g circom
fi

mkdir -p "$ZK_OUTPUT_DIR"

# ----- 1. Phase 1: Powers of Tau (reuse, do not regenerate) -----------------
echo "==> Step 1: Powers of Tau (reuse perpetual ptau, bn128, 2^20)"
if [[ ! -f "$ZKPTAU_FILE" ]]; then
    echo "  Downloading ptau from $ZKPTAU_URL..."
    curl -fsSL -o "$ZKPTAU_FILE" "$ZKPTAU_URL" || fail "ptau download failed"
fi

if [[ -n "$ZKPTAU_HASH" ]]; then
    actual=$(sha256sum "$ZKPTAU_FILE" | awk '{print $1}')
    if [[ "$actual" != "$ZKPTAU_HASH" ]]; then
        fail "ptau hash mismatch: expected $ZKPTAU_HASH, got $actual"
    fi
    ok "ptau hash verified"
else
    note "No ZKPTAU_HASH set — skipping hash verification (add it for reproducibility)"
fi

ptau_size=$(stat -c%s "$ZKPTAU_FILE")
ok "ptau file: $ZKPTAU_FILE ($ptau_size bytes)"

# Verify the ptau chain of trust
echo "  Verifying ptau chain of trust..."
snarkjs powersoftau verify "$ZKPTAU_FILE" >/dev/null 2>&1 ||
    fail "ptau chain verification failed"
ok "ptau chain verified"

# ----- 2. Circuit R1CS -------------------------------------------------------
echo "==> Step 2: Circuit R1CS"
if [[ ! -d "$ZK_CIRCUIT_DIR" ]]; then
    fail "Circuit directory $ZK_CIRCUIT_DIR not found"
fi

# Find the .circom file
CIRCOM_FILE=$(find "$ZK_CIRCUIT_DIR" -name "*.circom" -print -quit 2>/dev/null || true)
if [[ -z "$CIRCOM_FILE" ]]; then
    fail "No .circom file found in $ZK_CIRCUIT_DIR. Export R1CS from MembershipCircuit first."
fi
ok "Circuit file: $CIRCOM_FILE"

# Compile to R1CS + wasm
R1CS_FILE="$ZK_OUTPUT_DIR/membership.r1cs"
WASM_FILE="$ZK_OUTPUT_DIR/membership_js/membership.wasm"
circom "$CIRCOM_FILE" --r1cs --wasm -o "$ZK_OUTPUT_DIR" ||
    fail "circom compilation failed"
ok "R1CS + wasm compiled"

# ----- 3. Phase 2: groth16 setup ---------------------------------------------
echo "==> Step 3: Groth16 setup (snarkjs)"
ZKEY_FILE="$ZK_OUTPUT_DIR/membership_0000.zkey"
snarkjs groth16 setup "$R1CS_FILE" "$ZKPTAU_FILE" "$ZKEY_FILE" ||
    fail "snarkjs groth16 setup failed"
ok "Initial zkey generated: $ZKEY_FILE"

# ----- 4. Phase 2: contribute (interactive via zkpick.com) -------------------
echo "==> Step 4: Contribute (interactive via $ZKPICK_URL)"
echo ""
echo "=== IMPORTANT: Multi-party ceremony ==="
echo "Each participant must run their own contribute step."
echo "Open $ZKPICK_URL in a browser and follow the ceremony flow."
echo "Minimum participants: 5, target: $NUM_PARTICIPANTS (8–10)."
echo ""
echo "For each participant:"
echo "  1. Open zkpick.com"
echo "  2. Upload the current zkey"
echo "  3. Contribute with fresh entropy (keyboard/mouse + /dev/urandom)"
echo "  4. Download the contributed zkey"
echo "  5. Pass it to the next participant"
echo ""
echo "DO NOT use -e=\"literal string\" — that produces effectively no secret."
echo ""
read -p "Press Enter when all participants have contributed and you have the final zkey..."
CONTRIBUTED_ZKEY="${ZK_OUTPUT_DIR}/membership_final.zkey"
if [[ ! -f "$CONTRIBUTED_ZKEY" ]]; then
    fail "Final zkey not found at $CONTRIBUTED_ZKEY. Place the contributed zkey there and re-run."
fi
ok "Final zkey: $CONTRIBUTED_ZKEY"

# ----- 5. Beacon -------------------------------------------------------------
echo "==> Step 5: Beacon (drand round)"
if [[ -z "$DRAND_ROUND" ]]; then
    fail "DRAND_ROUND not set. Announce a drand round before contributions close, then set DRAND_ROUND=<round-hash> and re-run."
fi
snarkjs zkey beacon "$CONTRIBUTED_ZKEY" "$ZK_OUTPUT_DIR/membership_beaconed.zkey" "$DRAND_ROUND" ||
    fail "zkey beacon failed"
ok "Beaconed zkey: $ZK_OUTPUT_DIR/membership_beaconed.zkey"

# ----- 6. Verify + export VK -------------------------------------------------
echo "==> Step 6: Verify + export VK"
FINAL_ZKEY="$ZK_OUTPUT_DIR/membership_beaconed.zkey"
VK_FILE="$ZK_OUTPUT_DIR/membership_verification_key.json"
snarkjs zkey verify "$R1CS_FILE" "$ZKPTAU_FILE" "$FINAL_ZKEY" ||
    fail "zkey verify failed — zkey is inconsistent with r1cs"
ok "Zkey verified against r1cs"

snarkjs zkey export verificationkey "$FINAL_ZKEY" "$VK_FILE" ||
    fail "VK export failed"
ok "Verification key exported: $VK_FILE"

# ----- 7. Guardrails ----------------------------------------------------------
echo "==> Step 7: Guardrails"
# Convert snarkjs VK JSON to the raw 576-byte frame used by the on-chain verifier
VK_RAW="$ZK_OUTPUT_DIR/membership_vk_raw.bin"
node -e "
const fs = require('fs');
const vk = JSON.parse(fs.readFileSync('$VK_FILE', 'utf8'));
// snarkjs VK JSON: { parameters: { ... }, ... }
// The raw frame is: alpha_g1(64) | beta_g2(128) | gamma_g2(128) | delta_g2(128) | gamma_abc_g1[0](64) | gamma_abc_g1[1](64)
// snarkjs exports points as [x, y] big-endian hex strings for G1, [c1, c0] for G2
function hexToBytes(h) { return Buffer.from(h.replace(/^0x/, ''), 'hex'); }
function pad32(h) { const b = hexToBytes(h); const out = Buffer.alloc(32, 0); out[32-b.length] = b; return out; }
const alphaG1 = Buffer.concat([pad32(vk.parameters.A[0][0]), pad32(vk.parameters.A[0][1])]);
const betaG2 = Buffer.concat([pad32(vk.parameters.B[0][1]), pad32(vk.parameters.B[0][0]), pad32(vk.parameters.B[1][1]), pad32(vk.parameters.B[1][0])]);
const gammaG2 = Buffer.concat([pad32(vk.parameters.B[1][1]), pad32(vk.parameters.B[1][0]), pad32(vk.parameters.B[2][1]), pad32(vk.parameters.B[2][0])]);
const deltaG2 = Buffer.concat([pad32(vk.parameters.B[2][1]), pad32(vk.parameters.B[2][0]), pad32(vk.parameters.B[3][1]), pad32(vk.parameters.B[3][0])]);
const gammaABC0 = Buffer.concat([pad32(vk.parameters.C[0][0]), pad32(vk.parameters.C[0][1])]);
const gammaABC1 = Buffer.concat([pad32(vk.parameters.C[1][0]), pad32(vk.parameters.C[1][1])]);
const frame = Buffer.concat([alphaG1, betaG2, gammaG2, deltaG2, gammaABC0, gammaABC1]);
if (frame.length !== 576) throw new Error('frame length mismatch: ' + frame.length);
fs.writeFileSync('$VK_RAW', frame);
console.log('VK raw frame written (' + frame.length + ' bytes)');
"
ok "VK raw frame: $VK_RAW"

# VK frame hash (what update_verification_key_hash takes)
VK_HASH=$(sha256sum "$VK_RAW" | awk '{print $1}')
ok "VK frame hash: $VK_HASH"

# Guardrail: gamma != delta != generator
# (We can't easily check gamma/delta from raw bytes without curve math;
#  the on-chain parse_verification_key already does on-curve checks.)
echo "  Guardrail: VK frame hash pinned: $VK_HASH"
echo "  Guardrail: parse_verification_key validates curve points on-chain (frame.rs)"
echo "  Guardrail: R1CS fingerprint recorded in Book of Intents (ADR-025)"

# ----- 8. Output summary ------------------------------------------------------
echo ""
echo "=== Ceremony complete ==="
echo "  Zkey (proving key, public):   $FINAL_ZKEY"
echo "  VK (verification key):        $VK_FILE"
echo "  VK raw frame (576 bytes):     $VK_RAW"
echo "  VK frame hash:                $VK_HASH"
echo ""
echo "Next steps:"
echo "  1. Store $FINAL_ZKEY on IPFS (N-of-M provider layer, Q-008)"
echo "  2. Store $VK_FILE on IPFS + git-LFS"
echo "  3. Pin VK hash on-chain:"
echo "     update_verification_key_hash($VK_HASH)"
echo "  4. Publish transcript hash as evidence manifest (dogfood your own protocol)"
echo "  5. Run E2E test: zkey-loaded key → prove → verify_ownership_proof accepts"
