# RFC-016 — Protocol Security, Decentralization, Privacy and Censorship Resilience

**Status:** `SPECIFIED ONLY` — specification, threat model and acceptance criteria; no implementation code yet.
**Depends on:** RFC-012 (architecture contract), RFC-013 (spatial intelligence pipeline), RFC-015 (spatial component package), Chapters 11–14 (Book of Intents).
**Implements:** Terra security directive (Parts I–XII) — a protocol-level security and decentralization layer.
**Related:** `docs/book-of-intents/SECURITY.md` (audit report), `docs/book-of-intents/RISK_REGISTER.md`, `docs/book-of-intents/OPEN_QUESTIONS.md`.

---

## 0. Core invariant

Terra must remain an **open, interoperable protocol**. No single company, government, administrator, validator group, cloud provider, storage provider, API, marketplace, identity provider or geographic authority may unilaterally:

- stop the entire protocol from functioning;
- prevent independent participants from joining or providing services;
- control all validator selection or verification outcomes;
- control all access to spatial information;
- rewrite, silently remove or manipulate protocol history;
- force every participant to use one application, API, marketplace, storage provider or infrastructure provider;
- decide unilaterally which jurisdictions, locations or people can participate.

At the same time, Terra must protect people whose personal, sensitive, private or security-related information has been exposed through its infrastructure.

**Decentralization must not mean that every piece of information is permanently public. Privacy protection must be a native protocol capability, not an administrative favour.**

---

## 1. Security and decentralization principles

### 1.1 No mandatory central operator

Normal protocol operation must not require permission from Terra's founder or development team, a central administrator, a single government, a single jurisdictional authority, a central validator committee, a single marketplace, a single API or RPC provider, a single storage provider, a single reconstruction engine, a single identity provider, or a single hosting company.

Independent implementations of these services are permitted, subject to published security, interoperability and verification requirements.

### 1.2 Separation of protocol and application control

**Protocol control** (enforced on-chain): identity and authorization rules, evidence and provenance requirements, validator eligibility and independence, consensus and quorum requirements, security and privacy request lifecycles, escrow and incentive conditions, jurisdictional restrictions and their scope, dispute and appeal procedures, state transitions and auditability.

**Application control** (off-chain, independent): user interfaces, search and discovery, provider marketplaces, reconstruction services, storage services, geographic indexes, routing and matching strategies, rendering and visualization, data delivery and caching, notifications and analytics.

No application or marketplace may acquire protocol-level authority merely because it has more users, more infrastructure or more economic activity. A user must be able to use a compatible alternative application or communicate directly with the protocol.

### 1.3 No universal shutdown mechanism

There must be no single global administrative instruction that can disable Terra's entire functionality. Emergency controls, where justified, must be narrowly scoped, explicitly authorized, time-limited, auditable, subject to review, and limited to the affected protocol function, data category, account or jurisdiction.

A security emergency affecting one jurisdiction must not automatically disable another jurisdiction. A privacy request affecting one person's sensitive information must not become a mechanism for deleting unrelated public information. A government restriction affecting a particular location must not automatically prevent the rest of the world from using Terra.

---

## 2. Decentralization must be measurable

Decentralization is not a slogan. The protocol defines a measurable decentralization model covering:

1. Smart-contract administration and upgrade authority.
2. Validator selection and verification.
3. Validator geographic distribution.
4. Validator ownership and organizational independence.
5. Provider discovery and marketplace access.
6. Storage replication and retrieval.
7. API and RPC availability.
8. Identity issuance and recovery.
9. Jurisdictional authority.
10. Governance and emergency intervention.
11. Reconstruction and spatial computation.
12. Frontend and application distribution.

For each category the protocol records: the current implementation, the controlling authority (if any), the number and independence of operators, the concentration risks, the failure and censorship scenarios, the recovery mechanism, and the tests that demonstrate resilience.

Multiple accounts or servers do **not** by themselves make a service decentralized. The protocol assesses whether operators are genuinely independent or controlled by the same organization, infrastructure provider, government or administrator.

Concentration-risk indicators are tracked where technically possible to inform risk management and validator selection — they inform, they do not automatically punish participants based on correlation alone.

---

## 3. On-chain security governance

### 3.1 Protocol-level security registry

The protocol maintains dedicated PDAs for:

- `SecurityRequest` — protection request with scope and desired remedy.
- `PrivacyRequest` — personal/sensitive-evidence/security-threat protection request.
- `JurisdictionalRestrictionRequest` — scoped legal/regulatory restriction request.
- `SecurityReviewCase` — independent review case.
- `SecurityReviewAssignment` — reviewer assignment.
- `SecurityDecision` — finalized decision.
- `PrivacyState` — versioned access-state for protected data.
- `RestrictionScope` — geographic and data scope of a restriction.
- `SecurityAppeal` — appeal of a decision.
- `SecurityAuditEvent` — auditable event log.

These names are proposed vocabulary; existing account structures are reused or extended where safer and clearer (see Part IX audit rule). No duplicate account structures are created where an existing mechanism suffices.

Request types:

| Request | Purpose |
|---------|---------|
| Personal privacy request | Protect exposed personal or identifying information |
| Sensitive evidence request | Restrict access to private documents, images or evidence |
| Security threat request | Protect information that creates a credible risk to a person or property |
| Identity exposure request | Prevent unnecessary disclosure or linkage of identity information |
| Jurisdictional restriction request | Apply a properly scoped legal or regulatory restriction |
| Emergency protection request | Temporarily limit access while a serious threat is reviewed |
| Correction or reclassification request | Correct inaccurate visibility, sensitivity or classification metadata |

A request must identify its scope and desired remedy. It must **not** automatically authorize deletion of an entire asset, parcel, building, geographic region or historical record.

### 3.2 Distributed validator review

Privacy and security decisions must not be controlled by one administrator or one permanent committee.

**Validator selection** (when a request is submitted):
1. The protocol verifies the request is correctly formed.
2. It checks the requester's authorization or proof of standing.
3. It classifies the request and determines its review requirements.
4. It identifies the relevant geographic and jurisdictional context.
5. It filters eligible validators by capability, reputation, availability, conflicts of interest and independence.
6. It selects reviewers using an unpredictable, verifiable selection process.
7. It creates a review case with a defined deadline and quorum requirement.

Selection is not controlled by a single API or by a requester-provided list of preferred validators. Existing multi-factor routing and independence mechanisms (RFC-012 routing, `ValidatorRelationshipEdge`) are reused and audited before extension.

**Two complementary forms of review:**

- **Local or jurisdictionally relevant review** — validators with relevant geographic experience, jurisdictional knowledge or professional capabilities assess the local context.
- **Independent global review** — additional validators from outside the affected local group assess whether the proposed decision is justified, proportionate and consistent with Terra's global protocol rules.

Geographic relevance is one eligibility factor, not proof of neutrality. A government authority may submit an authenticated jurisdictional assertion, but it must not become the sole judge of whether the protocol should globally suppress information.

**Randomization and independence** — the review mechanism must prevent:
- Repeated selection of the same small group;
- One organization controlling most reviewers;
- A requester selecting only friendly validators;
- A government selecting all reviewers;
- Validators secretly coordinating decisions;
- A single validator deciding a high-impact request;
- Predictable selection enabling targeted bribery or coercion.

Review assignments use verifiable randomness or another auditable unpredictable selection method. Selection accounts for relationships, shared infrastructure, repeated co-validation, ownership concentration and other collusion indicators. Correlation triggers additional scrutiny, not automatic guilt.

The protocol defines what happens when the eligible validator pool is too small. It must **not** silently lower security requirements merely to complete a request.

---

## 4. Privacy request and information protection protocol

### 4.1 Privacy is a protocol-level right to request protection

Terra provides a standardized way for a person, authorized representative or affected organization to request that sensitive information be hidden, restricted, encrypted, redacted or reclassified. This remains possible even if the information was originally published through a decentralized application or by an independent storage provider.

Terra distinguishes:

1. Information stored directly by Terra.
2. Information stored by an independent Terra-compatible provider.
3. Information referenced by Terra through a hash or external identifier.
4. Information published by an unrelated third party outside Terra.
5. Information permanently recorded on a public blockchain.

The protocol cannot promise to erase information from the entire internet, remove copies held by unrelated third parties or reverse an immutable public-chain transaction. It defines how Terra-controlled access, references, replication, encryption keys, indexes, caches and future publication are handled.

### 4.2 Privacy request initiation

A request may be initiated by: the person whose information is exposed; a verified identity controller; a legally authorized representative; a guardian or other authorized person where applicable; an organization acting for an affected individual; a person reporting a credible safety risk (subject to additional verification).

The requester provides: a request identifier; the affected asset, document, evidence item or data reference; the type of information exposed; the reason for requesting protection; the requested remedy; a non-public proof of identity, control or authorization where necessary; evidence supporting the request; the relevant jurisdiction, if applicable.

**Never require a person to publish their identity documents, private address or sensitive evidence publicly in order to request privacy.**

### 4.3 Identity without public exposure

The protocol supports privacy-preserving proof of standing: wallet signatures, existing Terra identity credentials, selective disclosure, zero-knowledge proofs, authorized representative credentials, recovery and assisted identity procedures.

The public request must not reveal the requester's full identity unless that disclosure is strictly necessary and explicitly authorized. The review system establishes that the requester has a legitimate relationship to the affected information without unnecessarily exposing their identity to every validator. Where reviewers need additional information, it is provided through restricted, authenticated channels.

### 4.4 Privacy request scope

A request must identify exactly what is affected: a document; a photograph or video; a person's name or identity reference; a phone number or residential address; a building's sensitive interior; an apartment layout; a private access point; a security-sensitive facility component; a spatial observation; a geometry or model component; a storage reference; an evidence artifact; a relationship linking a public record to an identifiable person.

The system supports **component-level protection**. For example, a public building model may remain available while an apartment's internal layout, resident identity and access information are restricted. The system does **not** hide an entire city or parcel simply because one component contains sensitive information.

### 4.5 Privacy remedies

The protocol supports multiple levels of protection rather than a single delete-or-keep decision:

| Remedy | Meaning |
|--------|---------|
| Restrict access | Only authorized parties can retrieve the information |
| Remove public indexing | The item is no longer discoverable through public Terra indexes |
| Redact | Sensitive fields are removed from public representations |
| Replace public representation | A safer representation is published while restricted data remains protected |
| Encrypt | The content is encrypted and access is limited by authorization |
| Revoke access | Existing access permissions or delivery capabilities are invalidated |
| Stop replication | Terra-compatible providers are instructed to stop serving or replicating the affected content where possible |
| Crypto-erase | Encryption keys are destroyed or revoked where the storage design permits it |
| Tombstone reference | A public record indicates that the referenced content is restricted, withdrawn or unavailable |
| Correct classification | Incorrect public/private/sensitive labels are corrected |
| Reject request | The evidence does not justify the requested restriction |

The protocol defines which remedies are technically enforceable, which require provider cooperation and which are only status declarations. A tombstone must **not** reveal the sensitive information it is intended to protect.

### 4.6 Privacy review lifecycle

```
REQUEST SUBMITTED
        |
        v
AUTHENTICATION AND SCOPE CHECK
        |
        v
REQUEST CLASSIFICATION
        |
        v
EMERGENCY RISK CHECK
        |
        +------ Credible immediate danger?
        |                  |
        |                 YES
        |                  |
        |                  v
        |        TEMPORARY PROTECTION
        |                  |
        |                  v
        |          EXPEDITED REVIEW
        |
        v
DISTRIBUTED REVIEW ASSIGNMENT
        |
        v
LOCAL / JURISDICTIONAL REVIEW
        |
        v
INDEPENDENT GLOBAL REVIEW
        |
        v
QUORUM DECISION
        |
        +------ Approved ------> PRIVACY STATE TRANSITION
        |
        +------ Rejected ------> REASONED DECISION
        |
        +------ Insufficient --> MORE EVIDENCE / REASSIGNMENT
        |
        v
APPEAL WINDOW
        |
        v
INDEPENDENT APPEAL REVIEW
        |
        v
FINALIZED DECISION AND AUDIT RECORD
```

**Emergency temporary protection** — a credible and immediate risk may justify temporary protection before the full review is complete (e.g. a person's home address exposed alongside a credible threat; identity documents published; private access or security information exposed; information creating an immediate risk of physical harm).

Temporary protection must: have a short, explicit expiry period; be narrowly scoped; require a defined threshold of authorization; trigger an expedited independent review; be recorded in an auditable event; be reversible if the request is rejected; avoid permanently deleting evidence or changing legal rights. One person or one validator must not be able to impose indefinite restrictions through an emergency request.

**Standard review** — reviewers assess: whether the requester has a legitimate connection to the information; whether the information is genuinely personal, sensitive or security-relevant; whether the information is already public and whether that changes the risk; whether the publication creates a credible harm; whether the requested restriction is proportionate; whether a less restrictive remedy is sufficient; whether the information has a legitimate public-interest or verification purpose; whether the request is being used to conceal fraud, corruption, ownership disputes or evidence of misconduct.

A person must not be allowed to erase an adverse but legitimate verification result merely because it is inconvenient. Privacy protection must not become a tool for falsifying the history of a parcel, hiding fraud, concealing a property dispute or suppressing evidence of wrongdoing.

**Quorum and decisions** — separate thresholds for: temporary emergency protection; standard privacy requests; high-impact requests affecting public records; jurisdictional restrictions; appeals; requests affecting multiple jurisdictions. Thresholds are selected through documented threat modelling and tested against collusion, bribery, validator scarcity and malicious requests. A high-impact decision requires independent reviewers and a clearly defined quorum.

A decision records: the request identifier; the affected scope; the review requirements; the selected reviewers or privacy-preserving reviewer references; the decision outcome; the decision reason code; the remedy authorized; the relevant jurisdictional scope; the review and appeal deadlines; the resulting privacy-state version. Confidential reviewer deliberations or private evidence are **not** published in the public audit record.

### 4.7 Privacy state and public verification

A versioned privacy state is introduced for protected data. States include:

- `PUBLIC`
- `RESTRICTED`
- `PRIVATE`
- `REDACTED`
- `TEMPORARILY_PROTECTED`
- `UNDER_REVIEW`
- `WITHDRAWN_FROM_PUBLIC_ACCESS`
- `TOMBSTONED`
- `RESTORED`

A public consumer can determine whether an item is available, restricted or superseded without receiving the protected content. The system preserves a minimal, non-sensitive audit trail.

Where a privacy request is approved, the protocol does **not** require the private content itself to remain publicly retrievable merely to prove that a request occurred.

**Immutable history versus private content** — Terra distinguishes between immutable protocol history, public metadata, private evidence, external storage references, encryption keys, search indexes, cached representations and derived spatial models.

A public blockchain record cannot be assumed erasable. Therefore:

- Never place raw identity documents, private photographs, personal addresses or sensitive evidence directly on-chain.
- Store only the minimum commitments, state references, authorization data and non-sensitive metadata required for protocol verification.
- Avoid unsalted hashes of predictable personal documents where they can be matched through dictionary attacks. Evaluate salted commitments, keyed commitments and privacy-preserving references.
- Keep sensitive content encrypted off-chain.
- Make storage and access providers implement privacy-state changes.
- Require compatible indexes and frontends to respect current access-state information.
- Define how cached copies and previously issued access capabilities are invalidated.

If the content has already been copied outside Terra, the system clearly records that Terra cannot guarantee external deletion.

---

## 5. Jurisdictional restriction and global interoperability

### 5.1 Jurisdictional authority must be scoped

Terra supports local legal requirements without allowing one jurisdiction to control the global protocol.

A jurisdiction may have authority over: its own official cadastral records; government-recognized credentials; local land administration procedures; regulated professional activities; legally restricted datasets; information subject to valid local legal orders; officially recognized property and spatial records.

Jurisdictional authority must **not** automatically extend to: unrelated jurisdictions; independent validators outside its legal scope; the entire global spatial database; unrelated public information; the global protocol's continued operation; the ability to rewrite protocol history.

### 5.2 Jurisdictional restriction object

Each restriction identifies: issuing authority; authenticated authority credential; jurisdiction; legal or policy basis; affected information or asset; geographic and data scope; requested action; effective date; expiry or review date; appeal or challenge mechanism; whether the restriction is local, cross-border or globally applicable under a separately established process.

A government assertion is recorded as an authenticated external authority claim. It must **not** automatically be treated as a globally valid protocol decision.

### 5.3 Local restrictions and global access

The protocol supports different access states in different jurisdictions where lawful and technically appropriate. For example, a government-recognized land record may be restricted from public display within a jurisdiction while the underlying global protocol continues operating, independent validators continue validating unrelated information, and other jurisdictions continue accessing data they are legally permitted to access. Public and private information remain separated.

A local restriction must not silently rewrite the global asset history. Where a restriction is legally binding on Terra-compatible providers, the system distinguishes the local legal obligation from the protocol's global verification state.

### 5.4 Disputed jurisdictional orders

A formal process exists for challenging: an unauthenticated government request; a request issued by an authority outside its jurisdiction; an overbroad restriction; a restriction without a defined scope; an expired order; a restriction that conflicts with a higher-priority legal requirement; an attempt to suppress unrelated information. The protocol preserves the record of the request and its status without publishing protected details.

The protocol does not assume validators can override a valid court order or that a decentralized protocol can prevent governments from enforcing laws against infrastructure operators in their territory. Terra's objective is to prevent one authority from acquiring universal protocol control, not to claim immunity from all legal enforcement.

---

## 6. Security incentives and validator economics

### 6.1 Security must be economically incentivized

Security work is treated as a measurable service, not an unpaid assumption. Security roles include: privacy reviewer; independent appeal reviewer; jurisdictional reviewer; evidence integrity validator; security incident reporter; storage availability provider; data recovery provider; encryption and access-control service provider; protocol security auditor; anti-collusion analyst; independent infrastructure operator.

The protocol does **not** automatically create a new token or assume that all security roles need separate economic assets. Existing staking, task escrow, resource quote, reward allocation and coverage mechanisms (RFC-012 Phase 8) are reused where appropriate.

### 6.2 Security task rewards

A valid security review may generate a reward when the reviewer was correctly selected; met the required capabilities; followed the review procedure; submitted the decision within the deadline; and disclosed conflicts of interest. Reward must **not** depend on approving or rejecting the request — otherwise the incentive may encourage validators to produce whichever outcome pays more. The reward compensates correct process, evidence handling and timely independent assessment.

### 6.3 Validator collateral

Evaluate collateral or stake requirements for high-impact security responsibilities (sensitive data, high-impact privacy decisions, emergency restrictions, jurisdictional decisions, appeal review, critical infrastructure verification). Collateral must be proportionate to the risk and must not make participation economically impossible for independent validators. Wealth alone must not determine trustworthiness or eligibility.

### 6.4 Slashing and penalties

Slashing must require a defined, provable violation: deliberate disclosure of protected information; fabricating review evidence; accepting bribes; concealing a material conflict of interest; signing contradictory decisions in coordinated bad faith; repeatedly ignoring access-control obligations; deliberately submitting false attestations; unauthorized use of private evidence; collusion to suppress legitimate requests; collusion to expose protected information.

Slashing must **not** be triggered merely because a validator's decision differs from the majority. A minority opinion is not fraud. A mistaken decision made in good faith is not automatically misconduct.

Evidence, notice, response, independent review and appeal must precede severe penalties, except for narrowly defined temporary capability protections.

### 6.5 Security reporting incentives

Participants may report: exposed personal information; vulnerable access-control mechanisms; malicious storage providers; compromised validator accounts; centralization risks; Sybil attacks; collusion; unauthorized data replication; protocol-level vulnerabilities. Rewards depend on verified impact and responsible disclosure. The protocol avoids incentives that encourage mass false reports or public disclosure of vulnerabilities before affected users can be protected.

### 6.6 Anti-collusion incentives

The protocol discourages validator groups from becoming permanent gatekeepers: randomized reviewer selection; selection diversity; organization and infrastructure concentration analysis; relationship graph analysis (`ValidatorRelationshipEdge`, RFC-012 Phase 2); geographic diversity; repeated co-review patterns; stake concentration; reviewer rotation; independent appeal pools; conflict-of-interest declarations. Geographic diversity alone is not relied upon — validators from different countries may still share ownership, funding, infrastructure or coordinated incentives.

---

## 7. Storage, data availability and censorship resilience

### 7.1 No mandatory storage provider

Terra must not depend on one storage provider to preserve the world's spatial information. The protocol defines interoperable storage interfaces and content verification requirements. Compatible providers may use: IPFS; independent object storage; community-operated nodes; distributed storage networks; institutional storage; encrypted private storage; other compatible implementations.

The protocol supports multiple independent copies where availability requirements justify replication. Content addressing alone does **not** guarantee availability.

### 7.2 Storage-provider independence

The system supports: multiple providers for the same content; independent retrieval; replication status; availability proofs or attestations; content integrity verification; provider replacement; data export; recovery from provider failure; privacy-aware replication; revocation of access to restricted content.

A storage provider must not be able to modify verified content without detection. A storage provider must not become the sole authority capable of retrieving a record.

### 7.3 Censorship and failure recovery

The system defines behavior when: the official Terra API is offline; the official frontend is blocked; a storage provider refuses to serve data; a cloud provider terminates hosting; an RPC provider blocks requests; a government restricts a specific domain or application; a validator group becomes unavailable; Solana becomes temporarily unavailable.

The protocol remains compatible with independent frontends, APIs, storage providers, validators and data retrieval implementations. Where the blockchain is unavailable, the system defines which operations can continue offline and which must wait for consensus settlement. Offline operations are never claimed to have been globally finalized when they have not.

---

## 8. On-chain and off-chain responsibility

### 8.1 What must be enforced on-chain

The on-chain protocol enforces rules that require shared, independently verifiable state transitions:

1. Security and privacy request creation.
2. Request scope and lifecycle state.
3. Requester authorization or proof of standing.
4. Reviewer eligibility requirements.
5. Reviewer assignment or verifiable assignment commitment.
6. Conflict-of-interest declarations.
7. Quorum thresholds.
8. Reviewer attestations.
9. Decision finalization.
10. Emergency protection expiry.
11. Appeal creation and assignment.
12. Privacy-state transitions.
13. Jurisdictional restriction scope and expiry.
14. Security collateral and escrow conditions.
15. Reward eligibility and distribution.
16. Slashing conditions after due process.
17. Audit events and decision commitments.
18. Versioning and supersession of security decisions.
19. Authorization to restore or reclassify information.
20. Protection against duplicate, replayed or unauthorized state transitions.

On-chain data is minimal and privacy-preserving. Private evidence, identity documents, raw personal information, private photographs or sensitive spatial data are **not** stored directly on-chain. The chain enforces the decision process; it does not become a public repository of the information being protected.

### 8.2 What must remain off-chain

The following responsibilities generally remain outside the blockchain: raw documents and photographs; sensitive identity evidence; encrypted evidence storage; private review material; large spatial models; detailed personal data; search indexes; data replication; encryption and key management; content redaction; rendering and visualization; storage-provider communication; notifications; user interfaces; jurisdiction-specific legal interpretation; advanced fraud analysis; AI-assisted sensitivity detection; external legal-document verification.

Off-chain services follow the protocol's authorization and privacy-state rules. They must not be able to create protocol decisions by themselves. The protocol defines interoperable interfaces so independent services can implement the same rules.

### 8.3 AI must not decide privacy disputes

AI may help: detect personal information; identify exposed identity documents; detect residential addresses; identify sensitive building interiors; classify potentially sensitive spatial components; detect duplicate privacy requests; identify suspicious request patterns; recommend a possible remedy; help reviewers understand evidence.

AI must **not** independently approve a permanent restriction, reject a legitimate request or determine legal authority. The final decision follows the required human or validator review process.

---

## 9. Security against abuse

### 9.1 Prevent privacy requests from becoming a censorship tool

A privacy mechanism can itself become a centralization or censorship mechanism if poorly designed. The protocol defends against:

- Governments submitting mass requests to suppress public information.
- Property owners hiding legitimate disputes.
- Fraudsters hiding evidence of fraud.
- Competitors attempting to suppress another provider's data.
- Attackers submitting false identity claims.
- Validators colluding to remove legitimate information.
- Wealthy participants buying favourable privacy decisions.
- Repeated emergency requests to keep information hidden indefinitely.
- Organizations using privacy requests to manipulate public spatial records.

**Proportionality** — every decision selects the least restrictive remedy that adequately addresses the risk. If an apartment's interior is sensitive, restrict that component rather than suppressing the building's public exterior. If a person's name is exposed unnecessarily, redact or restrict the identifying link rather than deleting the entire verification record.

**Public-interest and evidence preservation** — privacy protection does not erase the existence of legitimate protocol evidence. Where legally and technically appropriate: preserve a non-sensitive commitment to the historical record; preserve the fact that a privacy decision occurred; preserve a record of the decision's scope and version; restrict access to sensitive content; maintain lawful evidence-preservation procedures; avoid revealing protected details through audit events.

The protocol distinguishes privacy protection from falsification of historical facts.

**Appeals and restoration** — a rejected requester can appeal. An approved restriction is also challengeable by an affected party with legitimate standing. Appeals are reviewed by an independent group that does not simply repeat the original reviewer set. If a restriction is overturned: the decision is versioned; access may be restored where legally and technically permissible; a new public representation may be published; old sensitive copies are not automatically republished; the protocol preserves the history of the restriction and appeal without disclosing protected information.

---

## 9. Required repository audit and implementation

### 9.1 Inspect existing code before designing new modules

Before implementing anything, audit the actual code and tests:

- `terra-core/programs/terra_registry/src/fraud_governance.rs`
- `terra-core/programs/terra_registry/src/routing.rs`
- `terra-core/programs/terra_registry/src/validator_registry.rs`
- `terra-core/programs/terra_registry/src/validator_profile.rs`
- `terra-core/programs/terra_registry/src/world_registry.rs`
- `terra-core/programs/terra_registry/src/recovery.rs`
- `terra-core/programs/terra_registry/src/staking.rs`
- `terra-core/programs/terra_registry/src/task_economics.rs`
- `terra-core/programs/terra_registry/src/evidence_manifest.rs`
- `terra-core/programs/terra_registry/src/cross_border.rs`
- `terra-core/programs/terra_registry/src/verification/`
- `terra-core/programs/terra_identity/`
- `terra-core/api/`
- `terra-core/api/migrations/`
- `terra-core/SECURITY.md`
- `docs/rfc-012-global-physical-digital-trust-architecture.md`
- `docs/book-of-intents/IMPLEMENTATION_STATUS.md`, `OPEN_QUESTIONS.md`, `ARCHITECTURAL_DECISIONS.md`, `RISK_REGISTER.md`, `REQUIREMENTS_TRACEABILITY.md`, `ROADMAP.md`

Inspect instruction contexts, account ownership checks, PDA seeds, signer constraints, authorization paths, state transitions, event definitions, randomization mechanism, validator independence filters, escrow logic and tests.

### 9.2 Specific existing-code concerns to investigate

- The fraud-governance mechanism (five-member random review committee, minimum reputation threshold, appeal/restriction lifecycle) — investigate whether it can be generalized safely or whether privacy decisions need a separate review domain.
- The routing design (supplied candidates and randomized selection after eligibility checks) — investigate whether candidate discovery and candidate-list construction can be manipulated by one service.
- The global `WorldRegistry` PDA and jurisdiction authority records — inspect whether these authorities can acquire excessive global control.
- The IPFS registry mirror (partially implemented) — treat independent storage availability as an unresolved security requirement.
- The programs have not yet been deployed to a public cluster — do not describe any new security mechanism as live, decentralized in production or externally validated until deployment and independent testing prove it.

### 9.3 Do not infer

Do not infer that an existing fraud review system is suitable for privacy review without testing the differences. Do not assume that existing jurisdictional records implement jurisdiction-specific access control. Do not assume that existing staking and slashing mechanisms adequately protect sensitive information. Do not assume that the API mirror, IPFS mirror or frontend currently enforce privacy-state transitions.

---

## 10. Implementation phases

### Phase 0 — Threat model and architecture (before coding)

- Inventory all current centralization points.
- Identify all privileged authorities.
- Map every sensitive data path.
- Map every public and private storage reference.
- Identify how validator selection is currently performed.
- Identify all current privacy-related access controls.
- Identify the limits of current blockchain immutability.
- Model government, corporate, infrastructure and validator-group censorship.
- Model malicious privacy requests.
- Model unauthorized exposure and data replication.
- Define the protocol/off-chain boundary.

Deliver a documented threat model and a gap analysis based on source code.

### Phase 1 — Protocol security specification

Define: security request data structures; privacy request data structures; jurisdictional restriction data structures; reviewer assignment and selection rules; quorum requirements; emergency protection rules; appeal rules; privacy-state transitions; security incentives; slashing conditions; audit event semantics; privacy-preserving identity and authorization requirements.

Write tests for all state transitions and invalid transitions before treating the design as complete.

### Phase 2 — On-chain core

Implement the minimum secure on-chain mechanisms. Prioritize:

1. Privacy and security request registration.
2. Request scope validation.
3. Reviewer assignment.
4. Independent quorum attestations.
5. Decision finalization.
6. Emergency protection expiry.
7. Appeal registration and review.
8. Privacy-state transitions.
9. Jurisdictional scope and expiry.
10. Security reward and escrow integration.
11. Due-process slashing integration.
12. Replay protection and authorization checks.

Do not add unnecessary on-chain data or duplicate existing accounts. Do not make irreversible global governance decisions without founder authorization.

### Phase 3 — Off-chain privacy enforcement

Implement the API and storage interfaces that consume protocol decisions:

- Authorized retrieval only.
- Public/private representation separation.
- Component-level access control.
- Redaction and replacement.
- Search-index removal.
- Cache invalidation.
- Storage-provider notifications.
- Replication controls.
- Encryption-key revocation where applicable.
- Privacy-state version checking.
- Independent provider compatibility.
- Audit logs without sensitive data leakage.

An API must not be able to override a finalized protocol privacy decision. An API must also not be able to invent a privacy decision that the protocol has not authorized, except for a clearly defined temporary local safety measure that is recorded and expires.

### Phase 4 — Incentives and independent participation

Implement and test: security task rewards; reviewer compensation; security reporting rewards; proportionate collateral requirements; due-process slashing; independent validator discovery; reviewer rotation; alternative service-provider participation; multiple storage providers; data export and recovery. Keep the economic model open to multiple marketplaces and service providers. Do not make a single marketplace the mandatory gateway to security work.

### Phase 5 — Censorship and recovery testing

Test the protocol under simulated failures:

1. Disable the official frontend.
2. Disable the primary API.
3. Disable the primary storage provider.
4. Disable one RPC provider.
5. Remove a validator group.
6. Make the original privacy-review committee unavailable.
7. Submit a malicious privacy request.
8. Submit a false jurisdictional restriction.
9. Attempt to expose a requester's identity through public events.
10. Attempt to retrieve restricted evidence through an alternative API.
11. Attempt to serve stale cached information after restriction.
12. Attempt to replay an expired emergency restriction.
13. Attempt to use one administrator to restrict unrelated jurisdictions.
14. Attempt to manipulate reviewer selection.
15. Attempt to bribe or coordinate reviewers.
16. Attempt to suppress a legitimate privacy request through collusion.
17. Attempt to use privacy protection to conceal legitimate fraud evidence.

For each test, document expected behavior, actual behavior, failures and remediation.

---

## 11. Acceptance criteria

The security layer is **not** complete until these conditions are proven:

**Decentralization**
- No single application is required for protocol access.
- No single marketplace controls all task participation.
- No single storage provider controls all data availability.
- Independent validators can discover and participate in eligible work.
- Reviewer selection cannot be controlled by a requester or one central service.
- One jurisdiction cannot unilaterally disable unrelated jurisdictions.
- Emergency mechanisms cannot become permanent global shutdown controls.
- Upgrade and administrative authorities are explicitly documented and appropriately distributed.
- The protocol can recover from individual infrastructure failures.

**Privacy**
- A person can submit a privacy request without publicly exposing their identity documents.
- A request can identify a specific document, field, component or asset.
- A legitimate request can trigger temporary protection when a credible immediate danger exists.
- Standard decisions require independent review.
- High-impact decisions cannot be made by one validator.
- A privacy decision can restrict access without deleting unrelated public information.
- Sensitive data is not published directly on-chain.
- Public records do not expose the protected content through metadata or predictable hashes.
- API and storage implementations respect finalized privacy-state changes.
- Appeals and restoration are supported.
- The system accurately distinguishes Terra-controlled copies from external copies it cannot erase.

**Jurisdiction**
- Jurisdictional requests are authenticated.
- Restrictions have explicit scope and expiry or review requirements.
- A local restriction does not automatically become a global restriction.
- Legal recognition remains distinct from Terra's technical verification.
- Conflicting or overbroad requests can be challenged.

**Security incentives**
- Reviewers are compensated for valid independent work, not for choosing a predetermined outcome.
- Security reporting has responsible-disclosure protections.
- Slashing requires defined evidence and due process.
- Minority opinions are not automatically punished.
- Collusion and concentration risks are monitored.
- Economic requirements do not make independent participation impossible.

**Evidence** — Every capability marked `IMPLEMENTED AND VERIFIED` must have: source-code references; unit tests; integration or BPF tests where applicable; negative authorization tests; replay and expiry tests; collusion and adversarial tests where relevant; API and storage integration tests where applicable; a documented execution result.

---

## 11. Evidence Retention & Erasure

### 11.1 Retention principles
Retention is a deliberate decision, never an accident of implementation. Terra
keeps only what the protocol needs to verify, challenge, or appeal, for as long
as that verification may matter, and destroys the rest.

- **No document bytes on-chain or in the relational database.** Only
  commitments + references (extends DR10).
- **Rejected or expired evidence is purged.** Bytes and any derived working
  copies are destroyed; the decryption key is destroyed (cryptographic erasure).
  Only a non-identifying audit line may remain: no hash, no filename, no PII.
- **Accepted evidence keeps minimal provenance:** a keyed commitment (HMAC with
  a destroyable key) rather than a plain PII hash, plus metadata sufficient to
  challenge or appeal. The keyed commitment lets the custodian cryptographically
  erase the linkage when the retention period ends.
- **No anchor until accepted.** A document reference or commitment is not
  anchored on-chain until the evidence has passed intake validation and been
  accepted.
- **Keyed commitments instead of plain PII hashes.** A raw hash of PII is still
  PII (it can be attacked by precomputation/dictionary over known identities).
  Use keyed commitments so linkage can be destroyed by key destruction.
- **Explicit TTLs per task class and reputation tier.** Retention periods are
  declared, not inherited. When a TTL expires, bytes and keys are destroyed.
- **Purge is observable.** Destruction of bytes and keys is auditable without
  revealing what was destroyed.
- **Crypto-erasure is not guaranteed anonymization.** Destroying a key breaks
  linkage, but residual copies or side channels may remain; assess case-by-case
  (see EDPB Guidelines 02/2025 nuance).
- **Mirrors implement tombstone + suppression.** Off-chain mirrors that cache
  accepted evidence must honor retention: suppressed items are not re-displayed,
  and corrected records replace superseded ones through append-only correction
  (never silent rewrite).

### 11.2 Disposal path
accepted → (retention period elapses OR request granted) → purge bytes →
destroy decryption key → record non-identifying audit line → complete.

rejected/expired → purge bytes → destroy key → record non-identifying audit
line → complete.

### 11.3 Cross-reference
Sealed-session attribution opening (RFC-017 §6) governs when, how, and by whom
a closed session may be re-opened; retention does not extend the opening
window.

---

A written RFC is not implementation. A passing unit test is not production deployment. A deployed smart contract is not proof of global decentralization. A cryptographic hash is not proof that information is true or that it is safe to publish.

---

## 12. Non-negotiable design rules

1. No single global shutdown authority.
2. No mandatory central marketplace, frontend, API or storage provider.
3. No raw private evidence on-chain.
4. No public identity disclosure merely to request privacy.
5. No single validator deciding a high-impact privacy request.
6. No government authority automatically controlling global protocol state.
7. No local restriction automatically disabling unrelated jurisdictions.
8. No permanent emergency restriction without independent review.
9. No privacy mechanism that allows legitimate fraud evidence to be silently erased.
10. No automatic slashing for a good-faith minority decision.
11. No assumption that geographic proximity proves neutrality.
12. No assumption that IPFS or content addressing guarantees availability.
13. No assumption that blockchain immutability guarantees confidentiality.
14. No assumption that decentralization makes Terra immune to government enforcement.
15. No claim of implementation without code and test evidence.
16. No irreversible economic or governance commitment without the required authorization.
17. No unnecessary duplication of existing protocol mechanisms.
18. No central service may become the only way to discover, verify, retrieve or challenge a protocol record.
