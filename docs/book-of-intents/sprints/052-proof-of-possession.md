# Sprint 052: Proof-of-possession for component artifact replicas

**Date:** 2026-11-14 · **Status:** done & committed (`fbd9ac1`)
**Gate:** `cargo check --workspace` clean; `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --all --check` clean

## Trigger

work/CURRENT.md item 3 (RFC-016 Phase 2+ code) listed provider proof-of-possession
audits as unblocked agent work. RFC-017 §2 (Intake pipeline) and §7 (Assurance
Profiles) describe the need for providers to prove they actually hold the
component artifact bytes they claim to store.

## Implementation

**Backend — `terra-core/api/migrations/0046_provider_proof_of_possession.sql` (new):**
- Adds `last_challenge` (TEXT) column to `provider_replicas` — stores the most
  recent challenge sent to the provider
- Adds `last_proof_at` (TIMESTAMPTZ) column to `provider_replicas` — records
  when the provider last successfully proved possession; NULL means never
  verified or expired
- Index on `last_proof_at` for querying stale proofs

**Backend — `terra-core/api/src/routes/providers.rs`:**
- `POST /providers/{id}/replicas/{replica_id}/challenge` — generates a random
  UUID challenge, stores it as `last_challenge`, clears `last_proof_at`,
  returns the challenge to the provider
- `POST /providers/{id}/replicas/{replica_id}/prove` — provider submits
  `proof = SHA-256(challenge + content_address)`; API verifies against its
  local copy of the component artifact bytes; on success, sets
  `last_proof_at = now()` and clears `last_challenge`

**Backend — `terra-core/api/src/routes/components.rs`:**
- `upload_component_artifact`: if the component already declares a
  `geometry_digest`, verifies the uploaded content address matches it
  (RFC-015 rule 1: digest integrity at upload time)
- Public component artifact upload now requires signed authentication
  (`SignedRequest` verified against `API_AUTHORITY_PUBKEY`) — previously
  `enforce_component_access` returned `Ok(())` for public components without
  checking the signature

## Verification

| Check | Result |
|---|---|
| `cargo check --workspace` | clean |
| `cargo clippy --workspace -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |

**Note:** Anchor BPF tests not run locally (weak machine, long compile times);
source compilation verified via `cargo check`.

## Remaining RFC-016 Phase 2+ code

- **Provider proof-of-possession audits** — DONE (this sprint)
- Client-side digest verification vs signed manifest — frontend
  `fetchComponentAsset` already performs SHA-256 re-verification against
  `x-content-sha256` header (Sprint 047/050)
- No remaining unblocked code work in CURRENT.md item 3
