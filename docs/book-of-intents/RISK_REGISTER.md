# RISK_REGISTER

Severity: H(igh) / M(edium) / L(ow). Status uses the 7-level vocabulary where
applicable. Owners: Founder / Agents. Review: per sprint close.

| ID | Risk | Sev | Mitigation (existing → planned) | Owner |
| --- | --- | --- | --- | --- |
| R-001 | **No public-cluster deployment.** All on-chain behavior proven only in local BPF tests; PDAs/errors/costs unverified on devnet. | H | `deploy.sh devnet` ready + keypair verification (ADR-012) → fund wallet (Q-001), deploy, run on-chain smoke | Founder (fund), Agents (deploy+verify) |
| R-002 | **Single-maintainer bus factor.** Protocol knowledge concentrated in one person + this Book. | H | Book of Intents (ADR-013) + AGENT_GUIDE; RFCs per subsystem → keep sprints/records current; consider review rotation later | Founder |
| R-003 | **Slashing economics unvalidated adversarially.** 10%/100% tiers and reporter bonds are untested against coordinated attackers. | M | Unit+BPF coverage of tiers → adversarial simulation (test-harness scenarios) before mainnet (Stage 5 gate) | Agents |
| R-004 | **Mirror drift** between chain and PostGIS (failed syncs, manual DB edits). | M | Migrations + API-owned writes → periodic audit tool comparing mirror vs on-chain hashes (planned; ties REQ-013) | Agents |
| R-005 | **Legal misinterpretation:** users treat protocol results as title/ownership proofs. | H | Honesty notes in UI/API/docs; TRADEMARKS.md; VISION §3 non-negotiable; Book disclaimers → keep disclaimers on every surface that shows state | Founder + Agents |
| R-006 | **Privacy exposure.** Public ledger + mirrors expose wallet↔parcel links; residency data would amplify this. | M | Pseudonymous wallets; ZK selective disclosure (REQ-012, status 2) → complete ZK paths before any residency feature (Q-011) | Agents |
| R-007 | **Dependency churn.** Anchor/Solana/Cesium major versions can break builds or PDAs. | M | Pinned versions (Anchor 1.1.2, Cesium 1.144); CI gates → upgrade sprints with BPF re-run | Agents |
| R-008 | **CI depth.** Frontend job runs tsc+vitest; Rust job coverage must be confirmed for full suites. | M | ci.yml exists (E1) → verify Rust jobs run BPF+lib+api suites on PRs (Stage 1 check) | Agents |
| R-009 | **Headless 3D verification limits.** SwiftShader never settles terrain; e2e globe checks are partial. | L | CDP smokes with documented workaround; real-GPU unaffected → keep workarounds documented in demo harness | Agents |
| R-010 | **Evidence storage availability.** IPFS mirror partial (B7); single-provider dependence would violate independence principle. | M | Migration 0018 → multi-provider content addressing study (Q-008) | Agents |
| R-011 | **Validator collusion / Sybil / geographic capture** residual despite randomization. | M | Nine-factor routing + reputation + slashing (A8–A11) → continued research; institutional scoping (REQ-020) | Agents |
| R-012 | **Frontend bundle growth.** Cesium widgets chunk ~4.1 MB dominates payload. | L | Lazy routes/chunks per page; StoryPlayer minimal mount (ADR-007) → keep route-level lazy loading discipline | Agents |
| R-013 | **Commit hygiene.** Large feature drops (e.g. 44-file merges) compress review. | M | dev→main sprint checkpoints (Animus structure); Book sprint records | Founder + Agents |
| R-014 | **AI provenance partial (REQ-024).** Worker labels drafts but model/version/confidence not fully recorded. | M | Draft `method` labels exist → extend geometry_drafts/workers with model provenance fields before AI outputs feed REQ-018 consumers | Agents |
| R-015 | **Faucet/deploy dependency.** Solana faucet rate-limits blocked devnet funding repeatedly. | M | Documented retry history; alternative: founder-funded wallet or partner RPC (Q-001) | Founder |
