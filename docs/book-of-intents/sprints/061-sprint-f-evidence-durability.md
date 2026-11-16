# Sprint F: Evidence durability and secure intake

**Date:** planned · **Status:** planned
**Exit gate:** evidence survives provider loss; hostile uploads are
quarantined; retention/purge is enforced; “verified” requires ≥2 replicas.

## Trigger

`StorageBackend::Local` is the only backend. Intake has no quarantine, MIME/
hash/size checks are incomplete, there is no retention/TTL/purge path, and
proof-of-possession exists but is not scheduled. RFC-017 already specifies the
required behavior; this sprint implements it.

## Scope

1. **S3-compatible + pinning backend** alongside local storage; content-
   addressed keys; credentials from environment.
2. **Quarantine intake** — hash, MIME sniffing, size/page/pixel and expansion-
   ratio limits; reject before anything is anchored.
3. **Retention / TTL / purge** per RFC-017: task-class TTL, byte purge + key
   destruction on reject/expire, keyed commitments instead of plain PII
   hashes, no anchor before acceptance.
4. **Scheduled proof-of-possession challenges** for replicas; record pass/fail
   with slot/signature.
5. **Replica rule** — an artifact is “verified” only with ≥2 independent
   replicas that passed PoP.

## Acceptance

- Delete one provider’s copy; retrieval still succeeds and the replica count
  is repaired.
- A zip bomb / wrong-MIME upload is quarantined and never anchored.
- A rejected or expired artifact is purged (bytes + key) within its TTL.
- `verified` is impossible with fewer than two live replicas.

## Blocked by

None; builds on Sprint C’s chain-verified indexer for the on-chain evidence
anchor.
