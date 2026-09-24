#!/usr/bin/env bash
# build.sh - Build all programs and generate IDLs
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> Building terra_registry..."
cargo build-sbf --manifest-path programs/terra_registry/Cargo.toml 2>&1

echo "==> Building terra_identity..."
cargo build-sbf --manifest-path programs/terra_identity/Cargo.toml 2>&1

echo "==> Generating IDLs..."
mkdir -p target/idl
anchor idl build --skip-lint -p terra_registry > target/idl/terra_registry.json
anchor idl build --skip-lint -p terra_identity > target/idl/terra_identity.json
echo "==> Syncing terra_registry IDL to terra-web..."
cp target/idl/terra_registry.json ../terra-web/src/idl/terra_registry.json
# Regenerate typed wrapper (JSON → Idl const)
python3 - <<'PY'
import json, pathlib
idl_path = pathlib.Path("../terra-web/src/idl/terra_registry.json")
ts_path = pathlib.Path("../terra-web/src/idl/terraRegistry.ts")
data = json.loads(idl_path.read_text())
body = json.dumps(data, indent=2)
ts_path.write_text(
    '// Auto-generated from terra_registry.json — DO NOT EDIT\n'
    'import type { Idl } from "@coral-xyz/oxn";\n\n'
    f'export const terraRegistry: Idl = {body};\n'
)
print("wrote", ts_path)
PY

echo "==> Build artifacts:"
ls -lh target/deploy/*.so 2>/dev/null || echo "  (no .so files found)"
ls -lh target/idl/*.json 2>/dev/null || echo "  (no IDL files found)"

echo "==> Done."
