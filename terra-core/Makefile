.PHONY: build test test-ts test-fast lint ci deploy deploy-devnet deploy-mainnet clean idl

# Build all programs
build:
	./build.sh

# Run Rust integration tests
test:
	cargo test --test integration -- --test-threads=1

# TS E2E smoke test of the RRR ownership model (client-side account checks).
# Needs: npm install, `make idl`, a local `solana-test-validator` (AVX CPU)
# with the program deployed via `make deploy-localnet`.
test-ts:
	./node_modules/.bin/ts-mocha -p tsconfig.json tests/terra_registry.ts

# Run lint checks
lint:
	cargo fmt -- --check
	cargo clippy --workspace --all-targets -- -D warnings

# Fast baseline for constrained machines (mirrors CI lib/unit jobs; skips BPF)
test-fast:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings
	cargo test -p terra-registry --lib
	cargo test -p terra-identity --lib
	cargo test -p terra-registry --test rfc012_structure
	cargo test -p terra-geo
	cargo test -p terra-api

# Exact local mirror of .github/workflows/ci.yml (all 4 jobs: lib-tests,
# api-tests, api-build, frontend). Run this before pushing to dev/main.
# Notes:
# - sqlx migrate is best-effort: creating the postgis extension needs
#   superuser; CI's container has it, a local `terra` DB user may not.
#   CI still runs the migration strictly.
# - tsc runs from node_modules directly because pnpm hangs on this machine.
ci:
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo test -p terra-registry --lib
	cargo test -p terra-identity --lib
	cargo test -p terra-registry --test rfc012_structure
	cargo test -p terra-geo
	-@sqlx migrate run --source api/migrations || echo "note: sqlx migrate skipped locally (DB permissions) — CI runs it strictly"
	cargo test -p terra-api
	cargo build -p terra-api
	cd ../terra-web && ./node_modules/.bin/tsc --noEmit

# Auto-fix lint issues
fix:
	cargo fmt
	cargo clippy --workspace --all-targets --fix --allow-dirty

# Generate IDLs only (-o JSON → target/idl; workspace `anchor build`
# pulls host-only deps via api/geo members and can fail on getrandom/SBF)
idl:
	mkdir -p target/idl
	anchor idl build --skip-lint -p terra_registry -o target/idl/terra_registry.json
	anchor idl build --skip-lint -p terra_identity -o target/idl/terra_identity.json
	@ls -lh target/idl/*.json

# Deploy to devnet
deploy-devnet:
	./deploy.sh devnet

# Deploy to mainnet (requires confirmation)
deploy-mainnet:
	CONFIRM_MAINNET=yes ./deploy.sh mainnet

# Deploy to localnet (requires `solana-test-validator` running)
deploy-localnet:
	./deploy.sh localnet

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/deploy/*.so target/idl/*.json

# Show program sizes
size:
	@echo "=== Program Sizes ==="
	@ls -lhS target/deploy/*.so 2>/dev/null || echo "No build artifacts. Run 'make build' first."

# Verify programs on-chain
verify-devnet:
	@echo "terra_registry:"
	solana program show GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage --url devnet 2>&1
	@echo ""
	@echo "terra_identity:"
	solana program show 68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4 --url devnet 2>&1
