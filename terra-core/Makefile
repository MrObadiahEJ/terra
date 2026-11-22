.PHONY: build test test-ts test-localnet-e2e test-fast lint ci prune deploy deploy-devnet deploy-mainnet clean idl zk-dev-setup zk-info test-zk

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

# Sprint A lifecycle smoke on a local Surfnet (no AVX required).
# Needs: npm install, target/deploy/*.so (anchor build or CI artifacts),
# committed idl/*.json. Starts and stops Surfpool itself.
test-localnet-e2e:
	./scripts/run-localnet-e2e.sh

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
	cd ../terra-web && ./node_modules/.bin/tsc -b && ./node_modules/.bin/vitest run

# Auto-fix lint issues
fix:
	cargo fmt
	cargo clippy --workspace --all-targets --fix --allow-dirty

# Free disk on constrained machines: regenerable debug artifacts only.
# Each integration test binary is ~500MB and a new one is linked per
# source change; target/debug/incremental is fully regenerable.
# Keeps the two newest integration binaries (registry + identity) so the
# next test run only relinks what changed. Never touches target/deploy,
# target/idl, or cargo registry caches.
prune:
	rm -rf target/debug/incremental
	@cd target/debug/deps && \
	keep=$$(find . -maxdepth 1 -name 'integration-*' -type f -executable \
	  ! -name '*.tmp*' -printf '%T@ %f\n' 2>/dev/null | sort -rn | head -2 \
	  | cut -d' ' -f2- | tr '\n' ' '); \
	for f in integration-*; do \
		case " $$keep " in *" $$f "*) ;; *) rm -f "$$f";; esac; \
	done; \
	echo "pruned target/debug/incremental + stale integration binaries"
	@df -h /home | tail -1

# Generate IDLs only (-o JSON → target/idl; workspace `anchor build`
# pulls host-only deps via api/geo members and can fail on getrandom/SBF)
idl:
	mkdir -p target/idl idl
	anchor idl build --skip-lint -p terra_registry -o target/idl/terra_registry.json
	anchor idl build --skip-lint -p terra_identity -o target/idl/terra_identity.json
	cp target/idl/terra_registry.json idl/terra_registry.json
	cp target/idl/terra_identity.json idl/terra_identity.json
	@ls -lh idl/*.json

# --- Zero-knowledge membership proofs (RFC-011) -----------------------------

# Generate a DEVELOPMENT-ONLY Groth16 setup for the arkworks MembershipCircuit.
# Writes .zk/{membership.pk,membership.vk,env.sh} (gitignored) and prints the
# verification-key hash to pin with `update_verification_key_hash`.
# These keys are NOT ceremony artifacts: anyone holding the proving key can
# forge proofs. Never use them with real value (Q-009).
zk-dev-setup:
	./scripts/zk-dev-setup.sh

# Print the on-chain-visible identity (frame + SHA-256 hash) of a VK.
zk-info:
	cargo run --release -p terra-zk-membership --bin zk-info -- \
		--vk "$${TERRA_ZK_VERIFYING_KEY_PATH:?set TERRA_ZK_VERIFYING_KEY_PATH}" --json

# Authoritative real-circuit proof test: arkworks setup + prove for
# MembershipCircuit, verified on-chain through the alt_bn128 pairing syscalls.
# Slow (~5 min in debug) because it runs a real Groth16 setup and proof.
test-zk:
	cargo test -p terra-registry --test integration zk_membership_prover_verifier_e2e -- --test-threads=1

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
	solana program show HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p --url devnet 2>&1
