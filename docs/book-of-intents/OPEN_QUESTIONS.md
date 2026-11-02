# OPEN_QUESTIONS

Unresolved questions requiring research, evidence, or founder authority.
Agents must check this register before asking the founder anything
(PERMANENT AGENT RULE). Statuses use the 7-level vocabulary.

| ID | Question | Type | Status | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| Q-001 | Fund deployer wallet `CCng7wFcSK…Drsy4m` (0 SOL) for devnet deploy? | Founder decision | 6 – RESEARCH REQUIRED (blocked on funds) | Founder | `./deploy.sh devnet` ready; ~46 faucet attempts rate-limited. Unblocks live verification of A1–A21 (B6). |
| Q-002 | Tokenomics: does Terra ever have a token? Emissions? | Founder decision | 5 – PROPOSED (nothing designed) | Founder | Directive: token changes need authorization. Staking rewards currently program-internal constants. |
| Q-003 | Institutional bootstrap: who composes the trust council / oracle that verifies off-chain registries (DNS TXT, chambers)? | Research | 5 – PROPOSED (research done; founder decision pending) | Agents (research) + Founder (approve) | Gate for REQ-020 / INT-005. **Research recorded 2026-11-02** (RESEARCH_REFERENCES): DNS-TXT is a *discovery* layer, not a claims database; it is repudiable without DNSSEC. Trust-minimising designs converge on: DNS as discovery + external attestation service that re-verifies live + N-of-M consensus + reputation gating (Groundmark, Froglet, HAVID, MetaMail DON). A pure DNS-TXT root is weaker than a council and must not be the sole gate. Founder still must name the bootstrap council. |
| Q-004 | Residency credential: which jurisdictions recognize dual-signed residency attestations; tenant-protection requirements? | Research | 5 – PROPOSED (research done; no jurisdiction confirmed) | Agents | Gate for REQ-021. **Research recorded 2026-11-02** (RESEARCH_REFERENCES): surveyed Portugal, Queensland (AU), DC (US), New York (US), Monaco, Luxembourg. **Every** jurisdiction requires two independent proofs of address from distinct sources and a recency window; **none** issues a selective-disclosure credential — the resident always hands over the underlying document. A dual-signed attestation is therefore a novel instrument whose recognition cannot be assumed. REQ-021 stays `PROPOSED`. |
| Q-005 | Fractional investment: securities law per jurisdiction (collective investment, custody, consumer protection)? | Research | 6 | Agents | **Hard gate:** no implementation of REQ-023 before written research in RESEARCH_REFERENCES. |
| Q-006 | Mainnet gating criteria: what must be true (devnet soak duration, audits, slashing dry-runs)? | Founder decision | 5 | Founder | Recorded as criteria list in ROADMAP Stage 5; not yet ratified. |
| Q-007 | Server-side `GET /geo/route`: needed for external consumers (robotics/API), or is client-side routing (D4) sufficient? | **1 – IMPLEMENTED AND VERIFIED** | 5 | Agents | Client-side routing (D4) in `terra-web/src/story/roads.ts` is sufficient: `buildRoadRoute`, `snapToRoadNetwork`, `buildRoadGraph` are functional and vitest-tested (roads suite). No server-side endpoint added; external consumers can use nearest-roads API or client library. RFC-014 Ph.05 resolved. **Closed 2026-11-02.** |
| Q-008 | Evidence storage strategy: IPFS mirror (0018) vs multiple providers; availability guarantees? | Research | 6 | Agents | RISK R-010; independence principle (VISION §5.3). |
| Q-009 | ZK roadmap: Groth16 trusted-setup ceremony tooling; migration path to newer proof systems? `zk-membership` crate consumers? | Research | 6 | Agents | REQ-012 status 2. |
| Q-010 | RFC-014 Phases 04–05 (incremental compute, external replay) — proceed next after book? | Planning | 5 | Agents + Founder confirm | In ROADMAP Stage 1. |
| Q-011 | Privacy vs public ledger: erasure-request handling (GDPR-style) for mirrored personal data? | Research | 6 | Agents | RISK R-006; VISION §5.4. |
| Q-012 | Book maintenance cadence: who triggers re-audits; per-sprint or calendar? | Process | 5 | Founder (lightweight) | Default proposed: per sprint close (AGENT_GUIDE). |

Rules:
- Agents resolve `5/6` questions autonomously where evidence allows, recording
  findings in RESEARCH_REFERENCES and updating this register.
- Questions needing founder authority never block independent work: continue
  other tasks and keep the question open (AGENT_GUIDE §Decision Authority).
