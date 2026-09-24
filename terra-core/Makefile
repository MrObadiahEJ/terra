.PHONY: build test test-fast lint deploy deploy-devnet deploy-mainnet clean idl

# Build all programs
build:
	./build.sh

# Run Rust integration tests
test:
	cargo test --test integration -- --test-threads=1

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

# Auto-fix lint issues
fix:
	cargo fmt
	cargo clippy --workspace --all-targets --fix --allow-dirty

# Generate IDLs only (stdout JSON → target/idl; workspace `anchor build`
# pulls host-only deps via api/geo members and can fail on getrandom/SBF)
idl:
	mkdir -p target/idl
	anchor idl build --skip-lint -p terra_registry > target/idl/terra_registry.json
	anchor idl build --skip-lint -p terra_identity > target/idl/terra_identity.json
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
