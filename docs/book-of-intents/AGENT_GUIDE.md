# AGENT_GUIDE — Operating Manual for AI Agents and Contributors

This Book is Terra's memory. You can continue the project from it without the
founder repeating anything. Follow this manual exactly.

## 0. Permanent agent rule

Before asking the founder anything: (1) consult this Book, (2) inspect the
relevant code, (3) determine whether the answer is reachable independently.
Ask only for decisions that genuinely require founder authority or personal
intent. Never ask the founder to repeat something documented here.

## 1. Read order (first session in this repo)

1. [BOOK_OF_INTENTS.md](BOOK_OF_INTENTS.md) — structure + status vocabulary.
2. [TERRA_VISION.md](TERRA_VISION.md) — non-negotiables.
3. [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — what is real.
4. [ROADMAP.md](ROADMAP.md) — what is next.
5. [work/CURRENT.md](work/CURRENT.md) — active execution state.
6. Relevant intent + chapter + the registers below.

## 2. Evidence rules (non-negotiable)

- Never claim a feature is implemented because docs describe it, a struct or
  instruction exists, or a previous summary said so. Verify code **and** tests.
- Every capability statement cites: file path + test (or an honest gap).
- Use only the 7 statuses. If unsure, use the *lower* status and note why.
- When repository access fails, report the limitation — never fabricate an
  audit. (This Book's audit basis is always recorded at the top of
  IMPLEMENTATION_STATUS.md.)
- Geometry hashes commit to representations, not real-world truth. Never write
  prose that blurs protocol results with legal rights (RISK R-005).

## 3. Working cycle (every substantial change)

```
RESEARCH → UNDERSTAND → COMPARE → IDENTIFY GAPS → FORM INTENT → DESIGN
  → DOCUMENT → TRACE TO CODE → DEFINE TESTS → REVIEW SECURITY
  → VALIDATE → UPDATE THE BOOK
```

Code-first discipline: specs live in the Book; code lives in
`terra-core`/`terra-web`; both move together in one change.

## 4. Update protocol (what to touch when code changes)

| Change | Update |
| --- | --- |
| Feature completed/tested | IMPLEMENTATION_STATUS row → status 1 with test name; REQUIREMENTS_TRACEABILITY row; CHANGELOG |
| New decision | ARCHITECTURAL_DECISIONS.md (append-only) |
| New risk | RISK_REGISTER.md |
| New unknown | OPEN_QUESTIONS.md |
| External source consulted | RESEARCH_REFERENCES.md (date + claims supported) |
| Sprint finished | `sprints/NNN-*.md` record + work/CURRENT.md |
| Stage exits | ROADMAP.md stage status |

Do not rewrite unaffected chapters. Keep changes modular; SUMMARY.md is
navigation only and never stores state.

## 5. Decision authority

**Decide autonomously (then record):** wording/formatting; chapter
reorganization; deduplication; resolving doc inconsistencies using stronger
code evidence; adding explanations/diagrams; proposing architecture
improvements; adding acceptance criteria; updating statuses when evidence
justifies; logging alternatives; small reversible technical choices that
preserve VISION principles.

**MUST stop and request founder authorization:** changing TERRA_VISION §1–§3;
irreversible protocol/economic commitments; mandatory centralized dependencies;
tokenomics; legal claims on the founder's behalf; production/mainnet deploys;
major breaking changes; exposing private information; anything with substantial
legal/financial/security consequence unresolved from evidence.

While a founder question is open: continue all independent work; keep the
question registered (OPEN_QUESTIONS.md).

## 6. Repository map & gates

```
terra-core/programs/terra_registry   on-chain program (instructions in lib.rs; modules per protocol area)
terra-core/programs/terra_identity   identity/succession/guardianship program
terra-core/api                       Axum REST mirror + stories/geo/reconstruction routes; migrations/
terra-core/geo-engine                road graph, sealed-reach Dijkstra, geometry digest
terra-core/zk-membership             membership proofs crate (consumers: see Q-009)
terra-web/                           React 19 + Vite + Cesium app (no Tailwind; hand-rolled CSS)
docs/                                RFCs + VISION (historical); docs/book-of-intents/ = this Book
```

Gates (run before claiming green; machine is resource-constrained — sequential
builds only, `CARGO_INCREMENTAL=0` for cargo):

```
cd terra-web && pnpm exec tsc -b --noEmit && pnpm exec eslint . && pnpm build && pnpm exec vitest run
CARGO_INCREMENTAL=0 cargo test -p terra-api
cargo test -p terra-registry --lib
anchor build --skip-lint            # BPF suites load target/deploy/*.so and
                                    # fail loudly on a stale/missing build
cargo test -p terra-registry --test integration -- --test-threads=1   # 307 baseline
cargo test -p terra-identity --test integration                       # 23 baseline
cargo test -p terra-geo             # 12 + 1 ignored (2026-11-02)
```

Local machine constraints (this dev box): ~1.6 GB free disk, 8 GB RAM, load
~3. Prune `terra-core/target/debug/incremental` when disk < 1 GB; never run
heavy builds in parallel. Vite :5173 and terra-api :8080 are the dev servers;
Postgres via `PGPASSWORD=terra psql -h localhost -U terra -d terra_dev`.

### Signed requests (privileged endpoints)

The API verifies Ed25519 signatures on privileged mutations (R-016 —
fail-closed when `API_AUTHORITY_PUBKEY` is unset). Signed payload:

```text
{METHOD}\n{PATH}\n{BODY_SHA256_HEX}\n{TIMESTAMP_EPOCH_SECS}
```

Headers: `Authorization: Signature <base64>`, `x-timestamp`, `x-body-hash`,
and optionally `x-wallet` (base58 public key, bound by `verify_wallet`).
Replay window: ±60 s.

Do not hand-roll crypto in smoke scripts — emit headers with the shipped
client (R-017) and pipe them to `curl`:

```bash
cd terra-core
SIGNING_KEY=<base58 32-byte seed> \
  cargo run -q -p terra-api --example sign_request -- \
  POST /api/v1/derived/jobs '{"kind":"route","input_document":"{}"}'
# -> x-body-hash / x-timestamp / Authorization / x-wallet lines
```

Local dev: launch the API with `API_AUTHORITY_PUBKEY=<base58 pubkey>` to
enable privileged endpoints; without it they return 401 by design.

## 7. Commit discipline

- Work on `dev`; sprints open one `dev → main` checkpoint (Animus model).
- **Do not commit unless the founder asks.** When asked: focused commits,
  no target/ or generated artifacts, message cites gates run.
- Never rewrite applied migrations (ADR-011). Forward-fix only.

## 8. Definition of done (any feature)

1. Code + tests exist and gates run green.
2. IMPLEMENTATION_STATUS row = status 1 with evidence links.
3. REQUIREMENTS_TRACEABILITY row updated (or new REQ ID minted).
4. CHANGELOG entry. 5. Sprint record updated. 6. Honest labels (method,
   provenance, demo-vs-saved) everywhere user- or integrator-facing (ADR-010).

## 9. Permanent security & privacy requirements (RFC-016)

These apply to every agent session, forever — they are not optional and they
are not waived by a sprint deadline.

- **Never expose private information.** Do not log, print, commit, or return in
  an API response any identity document, private photograph, personal address,
  sensitive evidence, or protected spatial component. Raw private evidence never
  goes on-chain (RFC-016 ADR-018 / design rule 3).
- **Never bypass a finalized privacy decision.** An API or storage
  implementation must not serve restricted/private content, must not invent a
  privacy decision the protocol has not authorized (except a clearly defined
  temporary local safety measure that is recorded and expires), and must respect
  finalized PrivacyState transitions (RFC-016 ADR-018 / design rule 13).
- **Never claim decentralization without evidence.** Do not describe a service
  as decentralized merely because multiple accounts/servers exist. Assess
  genuine independence (organization, infrastructure, government, administrator)
  and record concentration-risk indicators (RFC-016 §2 / design rules 1, 2).
- **No single point of control.** Do not introduce a global shutdown switch, a
  mandatory central marketplace/frontend/API/storage provider, or an authority
  that can unilaterally disable unrelated jurisdictions or rewrite protocol
  history (RFC-016 §1 / design rules 1, 6, 7).
- **No public identity disclosure merely to request privacy.** Privacy
  requests must support privacy-preserving proof of standing (wallet signature,
  existing Terra identity credential, selective disclosure, ZK proof) without
  publishing identity documents (RFC-016 §4.2 / design rule 4).
- **No single validator deciding a high-impact privacy request.** Privacy and
  security decisions use distributed reviewer selection with verifiable
  randomness, quorum, local+global review, and independent appeals
  (RFC-016 §3.2 / design rule 5).
- **Privacy protection is not a censorship tool.** Do not use privacy/restriction
  mechanisms to erase legitimate fraud evidence, hide ownership disputes, or
  suppress public records; apply proportionality (least restrictive remedy) and
  component-level protection (RFC-016 §9 / design rule 9).
- **Governance capture requires authorization.** Do not make irreversible
  economic or governance commitments (upgrade authority, emergency pause, global
  restriction) without founder authorization (RFC-016 design rule 16).
- **Audit before designing.** When adding a security or privacy mechanism,
  inspect the actual code and tests first (fraud_governance.rs, routing.rs,
  validator_registry.rs, staking.rs, evidence_manifest.rs, cross_border.rs,
  recovery.rs, SECURITY.md). Reuse or extend existing mechanisms where safer;
  do not duplicate unnecessarily (RFC-016 Part IX / design rule 17).
- **Document the on-chain/off-chain boundary.** On-chain enforces the decision
  process; off-chain services implement enforcement and follow protocol
  authorization and privacy-state rules. Off-chain services cannot create
  protocol decisions by themselves (RFC-016 ADR-017 / design rule 18).
