# RFC-017 — Secure Evidence Intake, Rendition & Accountable Validation

- Status: PROPOSED (spec only; no implementation claimed)
- Depends on: RFC-005, RFC-007, RFC-010, RFC-011, RFC-012 (Phases 4–6), RFC-016
- Module: optional `terra-evidence` + `terra-cryptography` capabilities (directive §3)
- Chain-agnostic; Solana is one adapter (directive §10)

## 0. Invariants
1. Document bytes NEVER on-chain and NEVER in the relational database. Only
   commitments + references (extends DR10).
2. The signed original is NEVER modified. Watermarks/sanitization apply only to
   derived renditions, recorded as derived artifacts in provenance.
3. Untrusted content is NEVER parsed inside the API process.
4. No self-declared trust: assurance and attribution are independently verified.
5. No OS/vendor lock-in: requirements are Assurance Profiles, not brands.
6. Append-only: nothing here rewrites history.

## 1. Threat model
DoS via oversized/decompression-bomb files; malware in PDF active content or
embedded files; encrypted payloads evading scanners; parser RCE; document leaks
by validators; collusion to strip marks; coercion/pressure on contributors;
compromised validator devices; Sybil farming of evidence rewards.

## 2. Intake pipeline (mandatory order)
UPLOAD (authenticated, quota-checked)
  → size + page + pixel + expansion-ratio limits (reject early, stream to disk)
  → magic-byte sniffing + format allowlist (never trust MIME/extension)
  → QUARANTINE object store (encrypted, private, no public URL, TTL)
  → SANDBOX worker (unprivileged, read-only FS, no network, CPU/mem/time caps)
      → malware scan; encrypted files: decrypt only with escrowed key inside sandbox
      → active-content disarm (strip JS/Launch/embedded/RichMedia/XFA)
      → CDR: reconstruct safe rendition (raster or sanitized PDF)
  → ACCEPT: minimal provenance + keyed commitment anchored
  → REJECT/EXPIRE: purge bytes + destroy key (cryptographic erasure);
     keep only a non-identifying audit line (no hash, no filename, no PII)

Limits are declared per task class and per contributor reputation tier.
A refundable staging bond (RFC-014 §5b) deters spam; honest rejection refunds it.

## 3. Format policy
- Accept many formats (format-agnostic, directive §11); store native original.
- Archival of ACCEPTED documents: PDF/A + PAdES-LTA (long-term validation).
- PDF/A-3 permitted only post-sanitization (it allows embedded files).
- OpenPGP/age are encryption layers, never document formats.
- Encrypted originals allowed only with threshold-escrowed keys; never bypass scanning.

## 4. Disclosure tiers (privacy first)
Tier 0 attribute proof (VC/ZK) — no document circulates (DEFAULT for tenancy,
  ownership links, identity attributes)
Tier 1 minimized/redacted rendition — only required fields
Tier 2 full watermarked rendition — controlled viewer, time-limited, audited
Tier 3 original in escrow — threshold-opened only in a dispute

## 5. Rendition & forensic marking
Per recipient, per session, generated on demand (never N stored copies):
- visible subtle mark: session pseudonym + timestamp + short QR, in margins or
  tiled at low opacity; never over text; OCR must remain usable
- invisible robust forensic mark: survives screenshot/print-scan/re-encode/crop
- anti-collusion fingerprint coding (e.g. Tardos) so coalitions cannot erase marks
- signed provenance manifest with soft binding (C2PA model) so the manifest is
  recoverable even if metadata is stripped
Renditions are ephemeral, TTL-bound, and hash-recorded as derived artifacts.

## 6. Sealed session attribution
Each validation session records:
  session_id (per-session nullifier; publicly unlinkable)
  Enc_threshold(plaintiff_identity_hash, validator_id, session_id, mark_id)
  + ZK proof of correct encryption (verifiable encryption)
Opening requires: RFC-007 dispute → t-of-n independent key-holders → timelock →
appeal window → minimal disclosure of THAT session only.
No single party (founder, plaintiff, validator, company) can open alone.

## 7. Assurance Profiles (device/environment) — vendor-neutral
AP-0 Web session: TLS + wallet signature. Public/low-sensitivity tasks only.
AP-1 Hardware-bound key: WebAuthn/FIDO2 platform or roaming authenticator.
AP-2 Attested native client: app attestation + hardware-backed key attestation
     (Apple App Attest; Android Key Attestation TEE/StrongBox; Play Integrity
     optional, NOT required — alternatives must be accepted for global access).
AP-3 Controlled environment: AP-2 + capture-resistant viewer + server-side
     rendering (pixels only, file never lands on device) + short-lived capability.
Task classes declare a minimum profile (e.g. identity-document review >= AP-2;
Tier-3 original viewing = AP-3). Enforcement happens at capability-token issuance
and is re-checked per session; every check is audited.
Attestation is risk reduction, NOT proof of honesty. Store only the assurance
level + attestation digest — never device serials/IMEI (data minimization).
An exception/accessibility path must exist so low-resource regions are not excluded.

## 8. Capture-resistant viewer requirements
Android FLAG_SECURE; Windows SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE);
macOS NSWindow.sharingType = .none; iOS capture detection + blanking.
Web cannot block OS screenshots — therefore Tier 2/3 identity documents are
NOT viewable in a plain browser; they require AP-2/AP-3 clients.
The analog hole (photographing the screen) is unavoidable: marking provides
traceability, not prevention. This limit must be documented, not hidden.

## 9. Leak-tracing procedure
leaked copy → extract forensic mark → mark_id → session registry →
threshold opening (§6) → identified validator + session →
RFC-007 dispute → RFC-005 slashing + RFC-016 remedy.
Without the mark→session→validator registry, a watermark has no accountability value.

## 10. Anti-coercion guards
Random/blind routing (validator cannot be pre-selected or bribed for a known
plaintiff); least privilege + purpose limitation; time-limited, revocable
capability tokens; never public by default; every view audited; minimization first.

## 11. Backward compatibility
Additive only: new quarantine/rendition/mark tables + new PDAs for sealed
attribution; no change to existing evidence or component account layouts.
Append events/errors after the current highest code; regenerate IDL from source.

## 12. Acceptance tests
- oversized / zip-bomb / pixel-bomb / million-page uploads rejected pre-parse
- encrypted PDF cannot bypass scanning; decrypt happens only in sandbox
- PDF with /JavaScript, /Launch, /EmbeddedFile is disarmed in rendition
- signed original byte-identical after rendition generation (signature still valid)
- rejected/expired evidence purged; key destroyed; no PII hash anchored
- AP-0 client denied a Tier-2 identity task; AP-2 admitted; attestation replay rejected
- mark extraction → session lookup → threshold opening succeeds; unilateral opening fails
- double-claim/double-session nullifier reuse rejected

## 13. Phasing
P0 spec + Book registers. P1 intake limits + quarantine + sandbox scan/CDR.
P2 rendition + marking + viewer. P3 sealed attribution + threshold opening.
P4 Assurance Profiles + task-class policy. P5 leak-tracing drill + audit.

## Handoff
Spec only. Confirm existing body-limit/timeout values and whether any
retention/purge path exists before implementing. Do not invent counts.
