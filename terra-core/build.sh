#!/usr/bin/env bash
# build.sh - Build all programs and generate IDLs
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> Building workspace..."
cargo build-sbf 2>&1

echo "==> Generating IDLs..."
anchor build --skip-lint 2>&1

echo "==> Build artifacts:"
ls -lh target/deploy/*.so 2>/dev/null || echo "  (no .so files found)"
ls -lh target/idl/*.json 2>/dev/null || echo "  (no IDL files found)"

echo "==> Done."
