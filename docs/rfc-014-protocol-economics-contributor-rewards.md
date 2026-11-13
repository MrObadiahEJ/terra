# RFC-014 — Protocol Economics & Contributor Rewards

- Status: PROPOSED (spec only; no implementation claimed)
- Depends on: RFC-005 (staking/slashing), RFC-007 (dispute), RFC-011 (nullifier first-use), RFC-012 Phase 8 (task_economics), RFC-016 (governance/privacy), RFC-017 (secure evidence intake)
- Hard gate: Q-005 (securities). No token and no passive yield until the Q-005 memo + counsel sign-off land in RESEARCH_REFERENCES.
- Module: optional `terra-economy` (directive §3). Chain-agnostic spec; Solana is one adapter (directive §10).

## 0. Core invariants (non-negotiable)
1. Free core, paid edges. Reading specs, running the SDK, verifying proofs,
   creating basic claims and basic validation are NEVER fee-bearing.
2. Pay for verified work, not passive holding. Holding a balance or token
   confers no yield.
3. Governance-capped and transparent. Parameters set by open governance with
   hard caps and timelock; all treasury flows auditable.
4. No self-declared trust (directive §6). A contribution earns only after
   independent attestation.
5. Optional and portable. Accounting rules are protocol schemas implementable
   on any chain or off-chain.
6. Never punish productive volume. Anti-spam measures must be cost-asymmetric:
   costly for spam, net-free for honest work.

## 1. Motivation
Sustain protocol/SDK/adapter development without donations and without taxing
the core. Reuse the fee and slashing inflows that ALREADY accumulate in the
treasury, and add a governed, Sybil-resistant outflow paying core maintainers,
adapter/tool authors and retroactive public-goods contributors.

## 2. Architecture (three layers)
INFLOW (exists, unchanged):
  fund_task_escrow --FeePolicy.fee_bps--> treasury
  staking slash ------------------------> treasury
POLICY (new):
  RewardPolicy (weights, caps, epoch length, reserve floors, authority)
  ContributionRegistry + ContributorRecord (work-verified payees)
  SubmissionPolicy (bonds, throughput budgets — node-local enforcement)
OUTFLOW (new):
  ContributionAttestation (quorum-signed link to a verified artifact)
  DistributionEpoch (allocatable snapshot + allocation Merkle root + status)
  RewardClaim (first-use nullifier claim, RFC-011 pattern)

Treasury outflow priority: (a) coverage-subsidy reserve, (b) staking-reward
reserve, (c) security/audit earmark, THEN (d) contributor distribution.
Contributor payouts can never drain reserves owed to validators/stakers.

## 3. Data model (chain-agnostic schemas)
RewardPolicy { authority, epoch_len_secs, core_guild_bps, adapter_royalty_bps,
  retro_grants_bps, security_bps, coverage_reserve_lamports,
  staking_reserve_lamports, per_epoch_contributor_cap, vesting_secs, updated_at }
  invariant: core+adapter+retro+security <= 10_000 bps.
ContributionRegistry { authority, contributor_count, min_reputation, version }
ContributorRecord { contributor_id, payout_address, category
  (core|sdk|adapter|retro|security), reputation_snapshot, status
  (active|suspended|slashed), vesting_start, registered_at }
ContributionAttestation { contribution_id, contributor_id, artifact_ref
  (EvidenceManifest | GeometryVersion | ThreeDModel | task_id | adapter_id),
  utility_weight, quality_class, attestors[], attest_count, required, created_at }
SubmissionPolicy { authority, staging_bond_lamports, base_rate, base_burst,
  reputation_multiplier, max_concurrent_jobs, bond_refund_on_reject, updated_at }
DistributionEpoch { epoch_id, prev_epoch, opened_at, closed_at,
  allocatable_lamports, allocation_root, total_allocated,
  status (open|finalized|challenged|settled), challenge_deadline }
RewardClaim { epoch_id, contributor_id, leaf_hash, amount, claimed_at }

## 4. Lifecycle
register_contributor -> attest_contribution (quorum) -> open_epoch (crank) ->
allocate OFF-CHAIN (anyone can recompute from public records) ->
finalize_epoch (council + timelock + challenge window) -> claim_reward
(Merkle proof + first-use nullifier, paid from treasury).

## 5. Allocation policy
- Core/SDK guild: registered maintainers, linear vesting.
- Adapter/tool royalties: weighted by VERIFIED usage. Provenance already records
  reconstruction_engine_id/version (migration 0034, components.rs); each verified
  component/task using adapter X accrues utility_weight to X's maintainer.
- Retro grants: governance-approved public goods, paid for proven impact.
- Security earmark: audits and bounties.
Caps: per-epoch per-contributor cap; weights sum <= 10_000 bps;
allocatable = treasury_balance - coverage_reserve - staking_reserve - security_earmark.

## 5b. Utility weighting and staging bonds
utility_weight = 0 until an artifact is ACCEPTED and USED (consumed by a verified
geometry version, an accepted manifest, or a quality-passing reconstruction).
Irrelevant != fraudulent: a valid-but-unneeded submission earns nothing and
incurs NO penalty. Only proven spam/fraud is slashed (RFC-005/007 due process).
A refundable staging bond (RFC-017 §2) is returned on honest rejection and
forfeited only on proven spam. Rewards remain payment for verified work —
never passive yield (Q-005 hard gate).

## 6. Spam resistance without restricting productive work
Goal: make junk costly and honest volume cheap. No fixed global caps on output.

6.1 Deduplication. Artifacts are content-addressed; identical bytes are
    idempotent and consume no new storage (already true in components.rs).
6.2 Cost asymmetry. Paid edges (storage, reconstruction, anchoring, large
    exports) carry real cost; the free core stays free. Spam of free operations
    is bounded by bonds and reputation, never by fees on existence.
6.3 Refundable staging bond. Required to submit; returned in full on honest
    rejection; forfeited only after adjudicated spam. Net-zero for honest work.
6.4 Reputation-weighted throughput. Admission budget =
    base_rate * f(reputation), with burst allowance and max_concurrent_jobs.
    New identities start small; proven contributors scale to high volume.
    Throughput is EARNED, not purchased, and never capped by a fixed global rule.
6.5 Version-spam collapse. Many versions of one object are preserved as
    evidence (directive §6) but supersede semantics apply: only the accepted
    version anchors, and only utility is rewarded. Conflicting evidence is
    retained, not paid for repeatedly.
6.6 Backpressure and fair queueing. Job leasing (SKIP LOCKED) plus demand/
    coverage priority prevents junk from starving legitimate jobs.
6.7 Node-local admission, protocol-level economics. Rate limits and quotas are
    each provider's LOCAL policy (SubmissionPolicy defaults are advisory).
    The protocol supplies economic Sybil resistance (bonds, fees, reputation,
    slashing). No mandatory central rate limiter (directive §10).
6.8 Punish fraud, never volume. Slashing requires provable violation and due
    process; high output alone is never evidence of abuse.

## 7. Anti-Sybil & fraud
- register_contributor gated by reputation >= min_reputation OR council
  endorsement (bootstrap); payout_address must sign.
- attest_contribution requires validator quorum (QuorumConfig) bound to a real
  verified artifact; self-attestation rejected.
- Random audits (routing/fraud_governance) re-check attestations.
- finalize_epoch opens a challenge window; disputes via RFC-007; proven fraud
  -> demotion + clawback of UNCLAIMED allocations.
- Prefer retroactive payment over pay-per-PR (avoids Goodhart spam).

## 8. Governance
RewardPolicy.authority and finalize_epoch authority = council multisig +
timelock. Bootstrap admin authority is a founder-gated migration (Q-003/Q-019).
All parameter changes emit events; caps enforced on-chain, not by document.

## 9. Securities gate (Q-005) — HARD
Phases 1–3 pay in SOL/USDC from treasury as payment for verified services.
NO protocol token, NO fee-yield to passive holders, NO profit-sharing instrument
until the Q-005 per-jurisdiction memo + counsel sign-off. Any token is Phase 4,
optional, adapter-level, never required to use the protocol.

## 10. Decentralization & failure analysis
- Independent recomputation: allocation is a pure function of public
  ContributorRecords + Attestations + usage events; any node can rebuild the
  Merkle root and challenge a wrong finalize. On-chain stores root + nullifiers.
- Provider replacement / partition: claims verify against the root with locally
  held proofs; no central API required.
- Key compromise: payout rotation via council + timelock; nullifiers prevent
  double-claim.
- Treasury drain: reserve floors enforced on-chain.
- Governance capture: caps + timelock + challenge window + open recomputation.
- Spam/partition abuse: local admission policies diverge safely; economic
  guarantees remain globally verifiable.

## 11. Backward compatibility & migration
- NO change to FeePolicy, TaskEscrow, RewardAllocation, CoverageIncentive layouts.
- Append-only: new PDAs, instructions, events, TerraError variants after the
  current highest code (regenerate from source; never reuse numbers).
- Regenerate IDL (make idl); sync terra-web/src/idl.
- Existing fee/escrow/staking flows untouched; economy module optional.

## 12. Solana adapter mapping (proposed — confirm seeds in code)
Reuse the existing treasury account referenced by task_economics/staking
(confirm its PDA seeds before wiring). Proposed PDAs:
  ["reward_policy"]; ["contribution_registry"]; ["contributor", contributor_id];
  ["contribution", contribution_id]; ["submission_policy"];
  ["epoch", epoch_id_le]; ["reward_claim", epoch_id_le, contributor_id]
Instructions (delta +9): set_reward_policy, register_contributor,
  attest_contribution, set_submission_policy, open_epoch, finalize_epoch,
  claim_reward, rotate_payout_address, challenge_epoch.
claim_reward reuses the RFC-011 nullifier first-use pattern; Merkle proof
verified on-chain (sha256 sorted-pair leaves).

## 13. Acceptance criteria & tests
Unit: weight/cap math; reserve-floor allocatable; epoch sequencing; Merkle
  verify; nullifier first-use; vesting; throughput budget f(reputation).
BPF: register->attest->open->finalize->claim happy path; guards: self-attest
  rejected, double-claim rejected, over-cap rejected, non-council finalize
  rejected, claim before challenge deadline rejected, reserve drain rejected.
Anti-spam: duplicate artifact is idempotent (no new storage); honest rejection
  refunds bond in full; adjudicated spam forfeits bond; high-volume reputable
  contributor is NOT throttled below base_rate; version spam yields no extra
  reward; junk cannot starve a funded job.
Independent recomputation: off-chain tool reproduces the root from public records.
Evidence: named passing tests + CHANGELOG entry + sprint note.

## 14. Phased roadmap
P0: this RFC + Book registers (no code).
P1: ContributionRegistry/Record + register/attest (quorum) + tests. No money.
P2: RewardPolicy + epochs + Merkle claims + treasury outflow + governance.
P3: adapter-usage royalties + retro-grant round + independent recomputation tool.
P3b: SubmissionPolicy + staging bonds + reputation-weighted throughput.
P4 (founder-gated, post Q-005): optional token/vesting only if counsel clears.

## 15. Open questions / founder gates
Q-005 securities hard gate; Q-019 reward-governance council (links Q-003);
Q-022 bond sizing and reputation curve (must not exclude low-resource
contributors); bootstrap-admin -> council migration is founder-authorized.

## Handoff — status & next steps
Spec only. Next: Phase 1 in terra-core/programs/terra_registry (new economy.rs),
append-only IDL, tests above. Confirm treasury PDA seeds and current source
counts from code. Do not assert counts; regenerate.
