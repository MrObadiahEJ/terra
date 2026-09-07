// Identity module — reserved for future identity-specific logic.
// Current identity handlers (bind_identity, attach_parcel, request_succession,
// endorse_succession, cancel_succession, claim_succession) remain inline in
// lib.rs due to deep integration with parcel, vault, cross-border, and
// guardian contexts. The separate terra_identity program under
// programs/terra_identity/ houses the standalone identity architecture
// for threshold credentials and ZK identity (Part 5).
