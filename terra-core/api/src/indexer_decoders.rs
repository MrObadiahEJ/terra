// GENERATED FILE — DO NOT EDIT.
// Regenerate: python3 terra-core/scripts/generate-indexer-decoders.py
// Source: terra-core/idl/terra_registry.json (all Anchor accounts + types).
// CI pin: `python3 scripts/generate-indexer-decoders.py --check`.
//
// Q-033 stage 1: every IDL account type is decodable and mirrored into the
// generic `chain_accounts` table. `RICH_ACCOUNT_NAMES` marks the types with an
// additional typed projection in `indexer.rs`.

//! IDL-derived Borsh decoders for every account in `terra_registry`.
//! String/vec/bytes length prefixes are u32 LE (Borsh), never u8/u16.
//! Trailing Anchor allocation padding is allowed; truncated buffers, invalid
//! UTF-8 and oversized untrusted lengths are rejected.

// Every IDL account gets a decoder; only the rich projections are called from
// non-test code. `decode_any` is the generic mirror path (Q-033 stage 1).
#![allow(dead_code, unused_mut)]

/// Upper bounds for untrusted length prefixes read from RPC account data.
pub const MAX_STRING_LEN: usize = 4096;
pub const MAX_BYTES_LEN: usize = 65536;
pub const MAX_VEC_LEN: usize = 256;

#[derive(Debug, Clone)]
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

#[allow(dead_code)] // Generated for every IDL primitive; not every account uses each one.
impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(len)?;
        let slice = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn i64(&mut self) -> Option<i64> {
        Some(i64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn boolean(&mut self) -> Option<bool> {
        match self.u8()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    fn bytes32(&mut self) -> Option<[u8; 32]> {
        self.take(32)?.try_into().ok()
    }

    fn string(&mut self) -> Option<String> {
        let len = self.u32()? as usize;
        if len > MAX_STRING_LEN {
            return None;
        }
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes).ok().map(str::to_string)
    }

    fn blob(&mut self) -> Option<Vec<u8>> {
        let len = self.u32()? as usize;
        if len > MAX_BYTES_LEN {
            return None;
        }
        Some(self.take(len)?.to_vec())
    }
}

fn discriminator(data: &[u8], expected: &[u8; 8]) -> bool {
    data.len() >= 8 && &data[..8] == expected
}

/// nested IDL struct `CountryAllocation` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCountryAllocation {
    pub country_code: [u8; 2],
    pub approved_admin: [u8; 32],
}

/// Read `CountryAllocation` fields (no discriminator) from a Borsh reader.
fn read_country_allocation(mut r: &mut Reader) -> Option<ParsedCountryAllocation> {
    Some(ParsedCountryAllocation {
        country_code: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        approved_admin: r.bytes32()?,
    })
}

/// nested IDL struct `GenesisConfirmation` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedGenesisConfirmation {
    pub validator: [u8; 32],
    pub country_code: [u8; 2],
}

/// Read `GenesisConfirmation` fields (no discriminator) from a Borsh reader.
fn read_genesis_confirmation(mut r: &mut Reader) -> Option<ParsedGenesisConfirmation> {
    Some(ParsedGenesisConfirmation {
        validator: r.bytes32()?,
        country_code: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
    })
}

/// Anchor discriminator from the IDL for `AmalgamationRecord`.
pub const AMALGAMATION_RECORD_DISCRIMINATOR: [u8; 8] = [4, 242, 238, 37, 228, 163, 250, 90];

/// IDL account `AmalgamationRecord` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedAmalgamationRecord {
    pub result_parcel: [u8; 32],
    pub source_parcel: [u8; 32],
    pub source_geometry_hash: [u8; 32],
    pub result_geometry_hash: [u8; 32],
    pub rights_merged: bool,
    pub initiated_by: [u8; 32],
    pub created_at: i64,
    pub completed_at: i64,
    pub status: u8,
}

/// Read `AmalgamationRecord` fields (no discriminator) from a Borsh reader.
fn read_amalgamation_record(mut r: &mut Reader) -> Option<ParsedAmalgamationRecord> {
    Some(ParsedAmalgamationRecord {
        result_parcel: r.bytes32()?,
        source_parcel: r.bytes32()?,
        source_geometry_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        result_geometry_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        rights_merged: r.boolean()?,
        initiated_by: r.bytes32()?,
        created_at: r.i64()?,
        completed_at: r.i64()?,
        status: r.u8()?,
    })
}

/// Decode an Anchor `AmalgamationRecord` account buffer.
pub fn parse_amalgamation_record(data: &[u8]) -> Option<ParsedAmalgamationRecord> {
    if !discriminator(data, &AMALGAMATION_RECORD_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_amalgamation_record(&mut r)
}

/// Anchor discriminator from the IDL for `Appeal`.
pub const APPEAL_DISCRIMINATOR: [u8; 8] = [155, 196, 80, 143, 64, 220, 198, 177];

/// IDL account `Appeal` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedAppeal {
    pub restriction: [u8; 32],
    pub appellant: [u8; 32],
    pub nonce: u16,
    pub evidence_hash: [u8; 32],
    pub status: u8,
    pub note: String,
    pub created_at: i64,
    pub decided_at: i64,
}

/// Read `Appeal` fields (no discriminator) from a Borsh reader.
fn read_appeal(mut r: &mut Reader) -> Option<ParsedAppeal> {
    Some(ParsedAppeal {
        restriction: r.bytes32()?,
        appellant: r.bytes32()?,
        nonce: r.u16()?,
        evidence_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        status: r.u8()?,
        note: r.string()?,
        created_at: r.i64()?,
        decided_at: r.i64()?,
    })
}

/// Decode an Anchor `Appeal` account buffer.
pub fn parse_appeal(data: &[u8]) -> Option<ParsedAppeal> {
    if !discriminator(data, &APPEAL_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_appeal(&mut r)
}

/// Anchor discriminator from the IDL for `AuditEntry`.
pub const AUDIT_ENTRY_DISCRIMINATOR: [u8; 8] = [254, 88, 234, 107, 205, 16, 148, 113];

/// IDL account `AuditEntry` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedAuditEntry {
    pub entity: [u8; 32],
    pub sequence: u32,
    pub action: u8,
    pub from_status: u8,
    pub to_status: u8,
    pub actor: [u8; 32],
    pub metadata_hash: [u8; 32],
    pub timestamp: i64,
}

/// Read `AuditEntry` fields (no discriminator) from a Borsh reader.
fn read_audit_entry(mut r: &mut Reader) -> Option<ParsedAuditEntry> {
    Some(ParsedAuditEntry {
        entity: r.bytes32()?,
        sequence: r.u32()?,
        action: r.u8()?,
        from_status: r.u8()?,
        to_status: r.u8()?,
        actor: r.bytes32()?,
        metadata_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        timestamp: r.i64()?,
    })
}

/// Decode an Anchor `AuditEntry` account buffer.
pub fn parse_audit_entry(data: &[u8]) -> Option<ParsedAuditEntry> {
    if !discriminator(data, &AUDIT_ENTRY_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_audit_entry(&mut r)
}

/// Anchor discriminator from the IDL for `CapabilityRestriction`.
pub const CAPABILITY_RESTRICTION_DISCRIMINATOR: [u8; 8] = [74, 254, 245, 177, 59, 188, 186, 56];

/// IDL account `CapabilityRestriction` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCapabilityRestriction {
    pub wallet: [u8; 32],
    pub capability_code: u8,
    pub report: [u8; 32],
    pub max_level: u8,
    pub status: u8,
    pub applied_at: i64,
    pub appeal_opens_at: i64,
    pub rehab_eligible_at: i64,
    pub lifted_at: i64,
    pub updated_at: i64,
}

/// Read `CapabilityRestriction` fields (no discriminator) from a Borsh reader.
fn read_capability_restriction(mut r: &mut Reader) -> Option<ParsedCapabilityRestriction> {
    Some(ParsedCapabilityRestriction {
        wallet: r.bytes32()?,
        capability_code: r.u8()?,
        report: r.bytes32()?,
        max_level: r.u8()?,
        status: r.u8()?,
        applied_at: r.i64()?,
        appeal_opens_at: r.i64()?,
        rehab_eligible_at: r.i64()?,
        lifted_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `CapabilityRestriction` account buffer.
pub fn parse_capability_restriction(data: &[u8]) -> Option<ParsedCapabilityRestriction> {
    if !discriminator(data, &CAPABILITY_RESTRICTION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_capability_restriction(&mut r)
}

/// Anchor discriminator from the IDL for `Challenge`.
pub const CHALLENGE_DISCRIMINATOR: [u8; 8] = [119, 250, 161, 121, 119, 81, 22, 208];

/// IDL account `Challenge` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedChallenge {
    pub claim: [u8; 32],
    pub challenger: [u8; 32],
    pub challenge_hash: [u8; 32],
    pub status: u8,
    pub uphold_votes: u8,
    pub overturn_votes: u8,
    pub required_votes: u8,
    pub voters: Vec<[u8; 32]>,
    pub created_at: i64,
    pub review_deadline: i64,
    pub resolved_at: i64,
}

/// Read `Challenge` fields (no discriminator) from a Borsh reader.
fn read_challenge(mut r: &mut Reader) -> Option<ParsedChallenge> {
    Some(ParsedChallenge {
        claim: r.bytes32()?,
        challenger: r.bytes32()?,
        challenge_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        status: r.u8()?,
        uphold_votes: r.u8()?,
        overturn_votes: r.u8()?,
        required_votes: r.u8()?,
        voters: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        created_at: r.i64()?,
        review_deadline: r.i64()?,
        resolved_at: r.i64()?,
    })
}

/// Decode an Anchor `Challenge` account buffer.
pub fn parse_challenge(data: &[u8]) -> Option<ParsedChallenge> {
    if !discriminator(data, &CHALLENGE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_challenge(&mut r)
}

/// Anchor discriminator from the IDL for `Claim`.
pub const CLAIM_DISCRIMINATOR: [u8; 8] = [155, 70, 22, 176, 123, 215, 246, 102];

/// IDL account `Claim` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedClaim {
    pub claim_id: [u8; 32],
    pub parcel: [u8; 32],
    pub claim_type: u8,
    pub submitted_by: [u8; 32],
    pub status: u8,
    pub statement_hash: [u8; 32],
    pub version: u8,
    pub evidence_count: u8,
    pub observation_count: u8,
    pub attestation_count: u8,
    pub required_attestations: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `Claim` fields (no discriminator) from a Borsh reader.
fn read_claim(mut r: &mut Reader) -> Option<ParsedClaim> {
    Some(ParsedClaim {
        claim_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        parcel: r.bytes32()?,
        claim_type: r.u8()?,
        submitted_by: r.bytes32()?,
        status: r.u8()?,
        statement_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        version: r.u8()?,
        evidence_count: r.u8()?,
        observation_count: r.u8()?,
        attestation_count: r.u8()?,
        required_attestations: r.u8()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `Claim` account buffer.
pub fn parse_claim(data: &[u8]) -> Option<ParsedClaim> {
    if !discriminator(data, &CLAIM_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_claim(&mut r)
}

/// Anchor discriminator from the IDL for `ClaimSessionTracker`.
pub const CLAIM_SESSION_TRACKER_DISCRIMINATOR: [u8; 8] = [210, 191, 84, 140, 190, 184, 31, 247];

/// IDL account `ClaimSessionTracker` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedClaimSessionTracker {
    pub claim: [u8; 32],
    pub active_session: [u8; 32],
    pub bump: u8,
}

/// Read `ClaimSessionTracker` fields (no discriminator) from a Borsh reader.
fn read_claim_session_tracker(mut r: &mut Reader) -> Option<ParsedClaimSessionTracker> {
    Some(ParsedClaimSessionTracker {
        claim: r.bytes32()?,
        active_session: r.bytes32()?,
        bump: r.u8()?,
    })
}

/// Decode an Anchor `ClaimSessionTracker` account buffer.
pub fn parse_claim_session_tracker(data: &[u8]) -> Option<ParsedClaimSessionTracker> {
    if !discriminator(data, &CLAIM_SESSION_TRACKER_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_claim_session_tracker(&mut r)
}

/// Anchor discriminator from the IDL for `CoverageIncentive`.
pub const COVERAGE_INCENTIVE_DISCRIMINATOR: [u8; 8] = [122, 104, 101, 154, 189, 178, 207, 110];

/// IDL account `CoverageIncentive` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCoverageIncentive {
    pub authority: [u8; 32],
    pub demand_bps: u16,
    pub deficit_bps: u16,
    pub difficulty_bps: u16,
    pub strategic_bps: u16,
    pub max_subsidy_bps: u16,
    pub updated_at: i64,
}

/// Read `CoverageIncentive` fields (no discriminator) from a Borsh reader.
fn read_coverage_incentive(mut r: &mut Reader) -> Option<ParsedCoverageIncentive> {
    Some(ParsedCoverageIncentive {
        authority: r.bytes32()?,
        demand_bps: r.u16()?,
        deficit_bps: r.u16()?,
        difficulty_bps: r.u16()?,
        strategic_bps: r.u16()?,
        max_subsidy_bps: r.u16()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `CoverageIncentive` account buffer.
pub fn parse_coverage_incentive(data: &[u8]) -> Option<ParsedCoverageIncentive> {
    if !discriminator(data, &COVERAGE_INCENTIVE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_coverage_incentive(&mut r)
}

/// Anchor discriminator from the IDL for `CredentialNullifier`.
pub const CREDENTIAL_NULLIFIER_DISCRIMINATOR: [u8; 8] = [242, 184, 146, 110, 112, 48, 79, 197];

/// IDL account `CredentialNullifier` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCredentialNullifier {
    pub nullifier_hash: [u8; 32],
    pub credential: [u8; 32],
    pub consumed_at: i64,
}

/// Read `CredentialNullifier` fields (no discriminator) from a Borsh reader.
fn read_credential_nullifier(mut r: &mut Reader) -> Option<ParsedCredentialNullifier> {
    Some(ParsedCredentialNullifier {
        nullifier_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        credential: r.bytes32()?,
        consumed_at: r.i64()?,
    })
}

/// Decode an Anchor `CredentialNullifier` account buffer.
pub fn parse_credential_nullifier(data: &[u8]) -> Option<ParsedCredentialNullifier> {
    if !discriminator(data, &CREDENTIAL_NULLIFIER_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_credential_nullifier(&mut r)
}

/// Anchor discriminator from the IDL for `CredentialRequest`.
pub const CREDENTIAL_REQUEST_DISCRIMINATOR: [u8; 8] = [13, 77, 28, 51, 251, 145, 231, 29];

/// IDL account `CredentialRequest` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCredentialRequest {
    pub request_hash: [u8; 32],
    pub prover: [u8; 32],
    pub purpose: String,
    pub disclosure_type: u8,
    pub region_registry: [u8; 32],
    pub signers: Vec<[u8; 32]>,
    pub signatures: Vec<u8>,
    pub finalized: bool,
    pub credential: Option<[u8; 32]>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `CredentialRequest` fields (no discriminator) from a Borsh reader.
fn read_credential_request(mut r: &mut Reader) -> Option<ParsedCredentialRequest> {
    Some(ParsedCredentialRequest {
        request_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        prover: r.bytes32()?,
        purpose: r.string()?,
        disclosure_type: r.u8()?,
        region_registry: r.bytes32()?,
        signers: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        signatures: r.blob()?,
        finalized: r.boolean()?,
        credential: match r.u8()? {
            0 => None,
            1 => Some(r.bytes32()?),
            _ => return None,
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `CredentialRequest` account buffer.
pub fn parse_credential_request(data: &[u8]) -> Option<ParsedCredentialRequest> {
    if !discriminator(data, &CREDENTIAL_REQUEST_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_credential_request(&mut r)
}

/// Anchor discriminator from the IDL for `CrossBorderBinding`.
pub const CROSS_BORDER_BINDING_DISCRIMINATOR: [u8; 8] = [90, 184, 2, 152, 183, 229, 63, 32];

/// IDL account `CrossBorderBinding` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCrossBorderBinding {
    pub country_min: [u8; 2],
    pub country_max: [u8; 2],
    pub jurisdiction_min: [u8; 32],
    pub jurisdiction_max: [u8; 32],
    pub status: u8,
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `CrossBorderBinding` fields (no discriminator) from a Borsh reader.
fn read_cross_border_binding(mut r: &mut Reader) -> Option<ParsedCrossBorderBinding> {
    Some(ParsedCrossBorderBinding {
        country_min: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        country_max: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        jurisdiction_min: r.bytes32()?,
        jurisdiction_max: r.bytes32()?,
        status: r.u8()?,
        expires_at: r.i64()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `CrossBorderBinding` account buffer.
pub fn parse_cross_border_binding(data: &[u8]) -> Option<ParsedCrossBorderBinding> {
    if !discriminator(data, &CROSS_BORDER_BINDING_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_cross_border_binding(&mut r)
}

/// Anchor discriminator from the IDL for `CrossBorderSpanRecord`.
pub const CROSS_BORDER_SPAN_RECORD_DISCRIMINATOR: [u8; 8] = [224, 66, 12, 25, 121, 238, 123, 191];

/// IDL account `CrossBorderSpanRecord` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCrossBorderSpanRecord {
    pub task_id: [u8; 32],
    pub validator: [u8; 32],
    pub req_index: u8,
    pub country_home: [u8; 2],
    pub country_required: [u8; 2],
    pub binding: [u8; 32],
    pub recorded_at: i64,
}

/// Read `CrossBorderSpanRecord` fields (no discriminator) from a Borsh reader.
fn read_cross_border_span_record(mut r: &mut Reader) -> Option<ParsedCrossBorderSpanRecord> {
    Some(ParsedCrossBorderSpanRecord {
        task_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        validator: r.bytes32()?,
        req_index: r.u8()?,
        country_home: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        country_required: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        binding: r.bytes32()?,
        recorded_at: r.i64()?,
    })
}

/// Decode an Anchor `CrossBorderSpanRecord` account buffer.
pub fn parse_cross_border_span_record(data: &[u8]) -> Option<ParsedCrossBorderSpanRecord> {
    if !discriminator(data, &CROSS_BORDER_SPAN_RECORD_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_cross_border_span_record(&mut r)
}

/// Anchor discriminator from the IDL for `CrossBorderVerification`.
pub const CROSS_BORDER_VERIFICATION_DISCRIMINATOR: [u8; 8] = [183, 6, 175, 22, 50, 174, 166, 63];

/// IDL account `CrossBorderVerification` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCrossBorderVerification {
    pub binding: [u8; 32],
    pub claim: [u8; 32],
    pub session: [u8; 32],
    pub jurisdiction: [u8; 32],
    pub status: u8,
    pub verified_by: [u8; 32],
    pub created_at: i64,
    pub verified_at: i64,
}

/// Read `CrossBorderVerification` fields (no discriminator) from a Borsh reader.
fn read_cross_border_verification(mut r: &mut Reader) -> Option<ParsedCrossBorderVerification> {
    Some(ParsedCrossBorderVerification {
        binding: r.bytes32()?,
        claim: r.bytes32()?,
        session: r.bytes32()?,
        jurisdiction: r.bytes32()?,
        status: r.u8()?,
        verified_by: r.bytes32()?,
        created_at: r.i64()?,
        verified_at: r.i64()?,
    })
}

/// Decode an Anchor `CrossBorderVerification` account buffer.
pub fn parse_cross_border_verification(data: &[u8]) -> Option<ParsedCrossBorderVerification> {
    if !discriminator(data, &CROSS_BORDER_VERIFICATION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_cross_border_verification(&mut r)
}

/// Anchor discriminator from the IDL for `DeviceIdentity`.
pub const DEVICE_IDENTITY_DISCRIMINATOR: [u8; 8] = [6, 98, 9, 166, 229, 128, 194, 58];

/// IDL account `DeviceIdentity` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedDeviceIdentity {
    pub owner: [u8; 32],
    pub device_nonce: u16,
    pub device_key: [u8; 32],
    pub source: u8,
    pub capabilities: u64,
    pub metadata_ref: String,
    pub calibration_hash: [u8; 32],
    pub calibrated_at: i64,
    pub status: u8,
    pub verified: bool,
    pub verified_by: [u8; 32],
    pub verify_version: u32,
    pub registered_at: i64,
    pub updated_at: i64,
}

/// Read `DeviceIdentity` fields (no discriminator) from a Borsh reader.
fn read_device_identity(mut r: &mut Reader) -> Option<ParsedDeviceIdentity> {
    Some(ParsedDeviceIdentity {
        owner: r.bytes32()?,
        device_nonce: r.u16()?,
        device_key: r.bytes32()?,
        source: r.u8()?,
        capabilities: r.u64()?,
        metadata_ref: r.string()?,
        calibration_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        calibrated_at: r.i64()?,
        status: r.u8()?,
        verified: r.boolean()?,
        verified_by: r.bytes32()?,
        verify_version: r.u32()?,
        registered_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `DeviceIdentity` account buffer.
pub fn parse_device_identity(data: &[u8]) -> Option<ParsedDeviceIdentity> {
    if !discriminator(data, &DEVICE_IDENTITY_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_device_identity(&mut r)
}

/// Anchor discriminator from the IDL for `Dispute`.
pub const DISPUTE_DISCRIMINATOR: [u8; 8] = [36, 49, 241, 67, 40, 36, 241, 74];

/// IDL account `Dispute` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedDispute {
    pub parcel: [u8; 32],
    pub filed_by: [u8; 32],
    pub case_hash: [u8; 32],
    pub status: u8,
    pub required: u8,
    pub declared_count: u8,
    pub validators: [[u8; 32]; 8],
    pub present_validators: [[u8; 32]; 8],
    pub present_count: u8,
    pub filed_at: i64,
    pub frozen_at: i64,
    pub adjudicated_at: i64,
    pub outcome: u8,
    pub new_owner: [u8; 32],
    pub bond_lamports: u64,
    pub bond_refund_to: [u8; 32],
    pub bond_resolved: bool,
    pub remedy_geometry_version: [u8; 32],
}

/// Read `Dispute` fields (no discriminator) from a Borsh reader.
fn read_dispute(mut r: &mut Reader) -> Option<ParsedDispute> {
    Some(ParsedDispute {
        parcel: r.bytes32()?,
        filed_by: r.bytes32()?,
        case_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        status: r.u8()?,
        required: r.u8()?,
        declared_count: r.u8()?,
        validators: {
            let mut out: [[u8; 32]; 8] = [[0u8; 32]; 8];
            for item in &mut out {
                *item = r.bytes32()?;
            }
            out
        },
        present_validators: {
            let mut out: [[u8; 32]; 8] = [[0u8; 32]; 8];
            for item in &mut out {
                *item = r.bytes32()?;
            }
            out
        },
        present_count: r.u8()?,
        filed_at: r.i64()?,
        frozen_at: r.i64()?,
        adjudicated_at: r.i64()?,
        outcome: r.u8()?,
        new_owner: r.bytes32()?,
        bond_lamports: r.u64()?,
        bond_refund_to: r.bytes32()?,
        bond_resolved: r.boolean()?,
        remedy_geometry_version: r.bytes32()?,
    })
}

/// Decode an Anchor `Dispute` account buffer.
pub fn parse_dispute(data: &[u8]) -> Option<ParsedDispute> {
    if !discriminator(data, &DISPUTE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_dispute(&mut r)
}

/// Anchor discriminator from the IDL for `EmergencyInjection`.
pub const EMERGENCY_INJECTION_DISCRIMINATOR: [u8; 8] = [101, 247, 85, 71, 70, 156, 253, 173];

/// IDL account `EmergencyInjection` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedEmergencyInjection {
    pub registry: [u8; 32],
    pub requested_by: [u8; 32],
    pub candidate: [u8; 32],
    pub execute_after: i64,
    pub executed: bool,
}

/// Read `EmergencyInjection` fields (no discriminator) from a Borsh reader.
fn read_emergency_injection(mut r: &mut Reader) -> Option<ParsedEmergencyInjection> {
    Some(ParsedEmergencyInjection {
        registry: r.bytes32()?,
        requested_by: r.bytes32()?,
        candidate: r.bytes32()?,
        execute_after: r.i64()?,
        executed: r.boolean()?,
    })
}

/// Decode an Anchor `EmergencyInjection` account buffer.
pub fn parse_emergency_injection(data: &[u8]) -> Option<ParsedEmergencyInjection> {
    if !discriminator(data, &EMERGENCY_INJECTION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_emergency_injection(&mut r)
}

/// Anchor discriminator from the IDL for `EmergencyProtection`.
pub const EMERGENCY_PROTECTION_DISCRIMINATOR: [u8; 8] = [172, 134, 210, 181, 140, 209, 219, 43];

/// IDL account `EmergencyProtection` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedEmergencyProtection {
    pub subject: [u8; 32],
    pub nonce: u16,
    pub requester: [u8; 32],
    pub expires_at: i64,
    pub created_at: i64,
    pub lifted: bool,
}

/// Read `EmergencyProtection` fields (no discriminator) from a Borsh reader.
fn read_emergency_protection(mut r: &mut Reader) -> Option<ParsedEmergencyProtection> {
    Some(ParsedEmergencyProtection {
        subject: r.bytes32()?,
        nonce: r.u16()?,
        requester: r.bytes32()?,
        expires_at: r.i64()?,
        created_at: r.i64()?,
        lifted: r.boolean()?,
    })
}

/// Decode an Anchor `EmergencyProtection` account buffer.
pub fn parse_emergency_protection(data: &[u8]) -> Option<ParsedEmergencyProtection> {
    if !discriminator(data, &EMERGENCY_PROTECTION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_emergency_protection(&mut r)
}

/// Anchor discriminator from the IDL for `EscrowRecord`.
pub const ESCROW_RECORD_DISCRIMINATOR: [u8; 8] = [174, 124, 167, 70, 91, 85, 212, 227];

/// IDL account `EscrowRecord` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedEscrowRecord {
    pub parcel: [u8; 32],
    pub seller: [u8; 32],
    pub buyer: [u8; 32],
    pub amount: u64,
    pub deposit_amount: u64,
    pub vault: [u8; 32],
    pub status: u8,
    pub created_at: i64,
    pub deposited_at: i64,
    pub accepted_at: i64,
    pub settle_deadline: i64,
    pub cancel_deadline: i64,
    pub dispute_case_hash: [u8; 32],
}

/// Read `EscrowRecord` fields (no discriminator) from a Borsh reader.
fn read_escrow_record(mut r: &mut Reader) -> Option<ParsedEscrowRecord> {
    Some(ParsedEscrowRecord {
        parcel: r.bytes32()?,
        seller: r.bytes32()?,
        buyer: r.bytes32()?,
        amount: r.u64()?,
        deposit_amount: r.u64()?,
        vault: r.bytes32()?,
        status: r.u8()?,
        created_at: r.i64()?,
        deposited_at: r.i64()?,
        accepted_at: r.i64()?,
        settle_deadline: r.i64()?,
        cancel_deadline: r.i64()?,
        dispute_case_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
    })
}

/// Decode an Anchor `EscrowRecord` account buffer.
pub fn parse_escrow_record(data: &[u8]) -> Option<ParsedEscrowRecord> {
    if !discriminator(data, &ESCROW_RECORD_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_escrow_record(&mut r)
}

/// Anchor discriminator from the IDL for `Evidence`.
pub const EVIDENCE_DISCRIMINATOR: [u8; 8] = [160, 73, 93, 206, 99, 242, 62, 92];

/// IDL account `Evidence` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedEvidence {
    pub claim: [u8; 32],
    pub nonce: u8,
    pub submitted_by: [u8; 32],
    pub evidence_type: u8,
    pub content_hash: [u8; 32],
    pub storage_reference: String,
    pub created_at: i64,
    pub observed_at: i64,
}

/// Read `Evidence` fields (no discriminator) from a Borsh reader.
fn read_evidence(mut r: &mut Reader) -> Option<ParsedEvidence> {
    Some(ParsedEvidence {
        claim: r.bytes32()?,
        nonce: r.u8()?,
        submitted_by: r.bytes32()?,
        evidence_type: r.u8()?,
        content_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        storage_reference: r.string()?,
        created_at: r.i64()?,
        observed_at: r.i64()?,
    })
}

/// Decode an Anchor `Evidence` account buffer.
pub fn parse_evidence(data: &[u8]) -> Option<ParsedEvidence> {
    if !discriminator(data, &EVIDENCE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_evidence(&mut r)
}

/// Anchor discriminator from the IDL for `EvidenceArtifact`.
pub const EVIDENCE_ARTIFACT_DISCRIMINATOR: [u8; 8] = [18, 40, 202, 16, 128, 245, 57, 135];

/// IDL account `EvidenceArtifact` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedEvidenceArtifact {
    pub manifest: [u8; 32],
    pub artifact_index: u16,
    pub kind: u8,
    pub source: u8,
    pub provenance: u8,
    pub content_hash: [u8; 32],
    pub storage_reference: String,
    pub created_at: i64,
}

/// Read `EvidenceArtifact` fields (no discriminator) from a Borsh reader.
fn read_evidence_artifact(mut r: &mut Reader) -> Option<ParsedEvidenceArtifact> {
    Some(ParsedEvidenceArtifact {
        manifest: r.bytes32()?,
        artifact_index: r.u16()?,
        kind: r.u8()?,
        source: r.u8()?,
        provenance: r.u8()?,
        content_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        storage_reference: r.string()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `EvidenceArtifact` account buffer.
pub fn parse_evidence_artifact(data: &[u8]) -> Option<ParsedEvidenceArtifact> {
    if !discriminator(data, &EVIDENCE_ARTIFACT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_evidence_artifact(&mut r)
}

/// Anchor discriminator from the IDL for `EvidenceManifest`.
pub const EVIDENCE_MANIFEST_DISCRIMINATOR: [u8; 8] = [9, 190, 128, 86, 64, 17, 31, 5];

/// IDL account `EvidenceManifest` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedEvidenceManifest {
    pub task_id: [u8; 32],
    pub submitter: [u8; 32],
    pub nonce: u16,
    pub observation: [u8; 32],
    pub artifact_count: u16,
    pub root_hash: [u8; 32],
    pub created_at: i64,
}

/// Read `EvidenceManifest` fields (no discriminator) from a Borsh reader.
fn read_evidence_manifest(mut r: &mut Reader) -> Option<ParsedEvidenceManifest> {
    Some(ParsedEvidenceManifest {
        task_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        submitter: r.bytes32()?,
        nonce: r.u16()?,
        observation: r.bytes32()?,
        artifact_count: r.u16()?,
        root_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `EvidenceManifest` account buffer.
pub fn parse_evidence_manifest(data: &[u8]) -> Option<ParsedEvidenceManifest> {
    if !discriminator(data, &EVIDENCE_MANIFEST_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_evidence_manifest(&mut r)
}

/// Anchor discriminator from the IDL for `FeePolicy`.
pub const FEE_POLICY_DISCRIMINATOR: [u8; 8] = [98, 183, 86, 160, 232, 222, 82, 45];

/// IDL account `FeePolicy` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedFeePolicy {
    pub authority: [u8; 32],
    pub fee_bps: u16,
    pub updated_at: i64,
}

/// Read `FeePolicy` fields (no discriminator) from a Borsh reader.
fn read_fee_policy(mut r: &mut Reader) -> Option<ParsedFeePolicy> {
    Some(ParsedFeePolicy {
        authority: r.bytes32()?,
        fee_bps: r.u16()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `FeePolicy` account buffer.
pub fn parse_fee_policy(data: &[u8]) -> Option<ParsedFeePolicy> {
    if !discriminator(data, &FEE_POLICY_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_fee_policy(&mut r)
}

/// Anchor discriminator from the IDL for `FraudReport`.
pub const FRAUD_REPORT_DISCRIMINATOR: [u8; 8] = [136, 69, 45, 181, 210, 24, 121, 177];

/// IDL account `FraudReport` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedFraudReport {
    pub accused: [u8; 32],
    pub reporter: [u8; 32],
    pub nonce: u16,
    pub evidence_hash: [u8; 32],
    pub reason_code: u8,
    pub note: String,
    pub status: u8,
    pub capability_code: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `FraudReport` fields (no discriminator) from a Borsh reader.
fn read_fraud_report(mut r: &mut Reader) -> Option<ParsedFraudReport> {
    Some(ParsedFraudReport {
        accused: r.bytes32()?,
        reporter: r.bytes32()?,
        nonce: r.u16()?,
        evidence_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        reason_code: r.u8()?,
        note: r.string()?,
        status: r.u8()?,
        capability_code: r.u8()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `FraudReport` account buffer.
pub fn parse_fraud_report(data: &[u8]) -> Option<ParsedFraudReport> {
    if !discriminator(data, &FRAUD_REPORT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_fraud_report(&mut r)
}

/// Anchor discriminator from the IDL for `GenesisRequest`.
pub const GENESIS_REQUEST_DISCRIMINATOR: [u8; 8] = [37, 134, 94, 228, 116, 67, 108, 235];

/// IDL account `GenesisRequest` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedGenesisRequest {
    pub country_code: [u8; 2],
    pub requested_by: [u8; 32],
    pub confirmations: Vec<ParsedGenesisConfirmation>,
    pub finalized: bool,
    pub created_at: i64,
}

/// Read `GenesisRequest` fields (no discriminator) from a Borsh reader.
fn read_genesis_request(mut r: &mut Reader) -> Option<ParsedGenesisRequest> {
    Some(ParsedGenesisRequest {
        country_code: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        requested_by: r.bytes32()?,
        confirmations: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(read_genesis_confirmation(&mut *r)?);
            }
            out
        },
        finalized: r.boolean()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `GenesisRequest` account buffer.
pub fn parse_genesis_request(data: &[u8]) -> Option<ParsedGenesisRequest> {
    if !discriminator(data, &GENESIS_REQUEST_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_genesis_request(&mut r)
}

/// Anchor discriminator from the IDL for `GeometryVersion`.
pub const GEOMETRY_VERSION_DISCRIMINATOR: [u8; 8] = [78, 143, 35, 194, 97, 213, 195, 216];

/// IDL account `GeometryVersion` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedGeometryVersion {
    pub asset: [u8; 32],
    pub parcel: [u8; 32],
    pub version: u32,
    pub geometry_hash: [u8; 32],
    pub source: u8,
    pub dimension: u8,
    pub storage_reference: String,
    pub submitted_by: [u8; 32],
    pub submitted_at: i64,
    pub verified: bool,
    pub verified_by: [u8; 32],
    pub verified_at: i64,
    pub evidence_manifest: [u8; 32],
    pub elevation_min_mm: i64,
    pub elevation_max_mm: i64,
    pub elevation_source: u8,
    pub required: u8,
    pub attestors: [[u8; 32]; 8],
    pub attest_count: u8,
}

/// Read `GeometryVersion` fields (no discriminator) from a Borsh reader.
fn read_geometry_version(mut r: &mut Reader) -> Option<ParsedGeometryVersion> {
    Some(ParsedGeometryVersion {
        asset: r.bytes32()?,
        parcel: r.bytes32()?,
        version: r.u32()?,
        geometry_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        source: r.u8()?,
        dimension: r.u8()?,
        storage_reference: r.string()?,
        submitted_by: r.bytes32()?,
        submitted_at: r.i64()?,
        verified: r.boolean()?,
        verified_by: r.bytes32()?,
        verified_at: r.i64()?,
        evidence_manifest: r.bytes32()?,
        elevation_min_mm: r.i64()?,
        elevation_max_mm: r.i64()?,
        elevation_source: r.u8()?,
        required: r.u8()?,
        attestors: {
            let mut out: [[u8; 32]; 8] = [[0u8; 32]; 8];
            for item in &mut out {
                *item = r.bytes32()?;
            }
            out
        },
        attest_count: r.u8()?,
    })
}

/// Decode an Anchor `GeometryVersion` account buffer.
pub fn parse_geometry_version(data: &[u8]) -> Option<ParsedGeometryVersion> {
    if !discriminator(data, &GEOMETRY_VERSION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_geometry_version(&mut r)
}

/// Anchor discriminator from the IDL for `GuardianClaim`.
pub const GUARDIAN_CLAIM_DISCRIMINATOR: [u8; 8] = [75, 164, 183, 31, 178, 231, 33, 133];

/// IDL account `GuardianClaim` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedGuardianClaim {
    pub claim: [u8; 32],
    pub identity: [u8; 32],
    pub triggered_by: [u8; 32],
    pub case_hash: [u8; 32],
    pub guardian_type: u8,
    pub status: u8,
    pub created_at: i64,
    pub resolved_at: i64,
}

/// Read `GuardianClaim` fields (no discriminator) from a Borsh reader.
fn read_guardian_claim(mut r: &mut Reader) -> Option<ParsedGuardianClaim> {
    Some(ParsedGuardianClaim {
        claim: r.bytes32()?,
        identity: r.bytes32()?,
        triggered_by: r.bytes32()?,
        case_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        guardian_type: r.u8()?,
        status: r.u8()?,
        created_at: r.i64()?,
        resolved_at: r.i64()?,
    })
}

/// Decode an Anchor `GuardianClaim` account buffer.
pub fn parse_guardian_claim(data: &[u8]) -> Option<ParsedGuardianClaim> {
    if !discriminator(data, &GUARDIAN_CLAIM_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_guardian_claim(&mut r)
}

/// Anchor discriminator from the IDL for `IdentityRights`.
pub const IDENTITY_RIGHTS_DISCRIMINATOR: [u8; 8] = [80, 133, 246, 210, 78, 97, 163, 46];

/// IDL account `IdentityRights` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedIdentityRights {
    pub identity: [u8; 32],
    pub parcel: [u8; 32],
    pub rights_kind: u8,
    pub granter: [u8; 32],
    pub created_at: i64,
    pub expires_at: i64,
    pub status: u8,
    pub notes: String,
}

/// Read `IdentityRights` fields (no discriminator) from a Borsh reader.
fn read_identity_rights(mut r: &mut Reader) -> Option<ParsedIdentityRights> {
    Some(ParsedIdentityRights {
        identity: r.bytes32()?,
        parcel: r.bytes32()?,
        rights_kind: r.u8()?,
        granter: r.bytes32()?,
        created_at: r.i64()?,
        expires_at: r.i64()?,
        status: r.u8()?,
        notes: r.string()?,
    })
}

/// Decode an Anchor `IdentityRights` account buffer.
pub fn parse_identity_rights(data: &[u8]) -> Option<ParsedIdentityRights> {
    if !discriminator(data, &IDENTITY_RIGHTS_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_identity_rights(&mut r)
}

/// Anchor discriminator from the IDL for `Jurisdiction`.
pub const JURISDICTION_DISCRIMINATOR: [u8; 8] = [151, 118, 212, 152, 149, 211, 87, 2];

/// IDL account `Jurisdiction` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedJurisdiction {
    pub country_code: [u8; 16],
    pub authority: [u8; 32],
    pub jurisdiction_name: String,
    pub credential_schema_cid: String,
    pub revocation_registry: [u8; 32],
    pub verification_key_hash: [u8; 32],
    pub algorithm_id: u8,
    pub status: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `Jurisdiction` fields (no discriminator) from a Borsh reader.
fn read_jurisdiction(mut r: &mut Reader) -> Option<ParsedJurisdiction> {
    Some(ParsedJurisdiction {
        country_code: {
            let mut out: [u8; 16] = [0u8; 16];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        authority: r.bytes32()?,
        jurisdiction_name: r.string()?,
        credential_schema_cid: r.string()?,
        revocation_registry: r.bytes32()?,
        verification_key_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        algorithm_id: r.u8()?,
        status: r.u8()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `Jurisdiction` account buffer.
pub fn parse_jurisdiction(data: &[u8]) -> Option<ParsedJurisdiction> {
    if !discriminator(data, &JURISDICTION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_jurisdiction(&mut r)
}

/// Anchor discriminator from the IDL for `JurisdictionBinding`.
pub const JURISDICTION_BINDING_DISCRIMINATOR: [u8; 8] = [35, 9, 193, 178, 225, 246, 46, 21];

/// IDL account `JurisdictionBinding` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedJurisdictionBinding {
    pub identity_hash: [u8; 32],
    pub jurisdiction_key: [u8; 32],
    pub credential_commitment: [u8; 32],
    pub nullifier: [u8; 32],
    pub proof_data: Vec<u8>,
    pub proof_version: u8,
    pub algorithm_id: u8,
    pub revoked: bool,
    pub revoked_at: i64,
    pub revoked_by: [u8; 32],
    pub bound_at: i64,
    pub expires_at: i64,
    pub verified: bool,
    pub verified_by: [u8; 32],
    pub version: u32,
}

/// Read `JurisdictionBinding` fields (no discriminator) from a Borsh reader.
fn read_jurisdiction_binding(mut r: &mut Reader) -> Option<ParsedJurisdictionBinding> {
    Some(ParsedJurisdictionBinding {
        identity_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        jurisdiction_key: r.bytes32()?,
        credential_commitment: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        nullifier: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        proof_data: r.blob()?,
        proof_version: r.u8()?,
        algorithm_id: r.u8()?,
        revoked: r.boolean()?,
        revoked_at: r.i64()?,
        revoked_by: r.bytes32()?,
        bound_at: r.i64()?,
        expires_at: r.i64()?,
        verified: r.boolean()?,
        verified_by: r.bytes32()?,
        version: r.u32()?,
    })
}

/// Decode an Anchor `JurisdictionBinding` account buffer.
pub fn parse_jurisdiction_binding(data: &[u8]) -> Option<ParsedJurisdictionBinding> {
    if !discriminator(data, &JURISDICTION_BINDING_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_jurisdiction_binding(&mut r)
}

/// Anchor discriminator from the IDL for `JurisdictionalRestrictionRequest`.
pub const JURISDICTIONAL_RESTRICTION_REQUEST_DISCRIMINATOR: [u8; 8] =
    [41, 159, 224, 142, 146, 123, 90, 20];

/// IDL account `JurisdictionalRestrictionRequest` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedJurisdictionalRestrictionRequest {
    pub authority: [u8; 32],
    pub subject: [u8; 32],
    pub nonce: u16,
    pub restriction_type: u8,
    pub description: String,
    pub requested_remedy: u8,
    pub jurisdiction: [u8; 2],
    pub status: u8,
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `JurisdictionalRestrictionRequest` fields (no discriminator) from a Borsh reader.
fn read_jurisdictional_restriction_request(
    mut r: &mut Reader,
) -> Option<ParsedJurisdictionalRestrictionRequest> {
    Some(ParsedJurisdictionalRestrictionRequest {
        authority: r.bytes32()?,
        subject: r.bytes32()?,
        nonce: r.u16()?,
        restriction_type: r.u8()?,
        description: r.string()?,
        requested_remedy: r.u8()?,
        jurisdiction: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        status: r.u8()?,
        expires_at: match r.u8()? {
            0 => None,
            1 => Some(r.i64()?),
            _ => return None,
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `JurisdictionalRestrictionRequest` account buffer.
pub fn parse_jurisdictional_restriction_request(
    data: &[u8],
) -> Option<ParsedJurisdictionalRestrictionRequest> {
    if !discriminator(data, &JURISDICTIONAL_RESTRICTION_REQUEST_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_jurisdictional_restriction_request(&mut r)
}

/// Anchor discriminator from the IDL for `NullifierRecord`.
pub const NULLIFIER_RECORD_DISCRIMINATOR: [u8; 8] = [56, 18, 57, 175, 69, 202, 189, 70];

/// IDL account `NullifierRecord` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedNullifierRecord {
    pub nullifier_hash: [u8; 32],
    pub zone_set: [u8; 32],
    pub root_version: u32,
    pub prover: [u8; 32],
    pub proof_purpose: String,
    pub disclosure_type: u8,
    pub block_time: i64,
    pub proof_hash: [u8; 32],
}

/// Read `NullifierRecord` fields (no discriminator) from a Borsh reader.
fn read_nullifier_record(mut r: &mut Reader) -> Option<ParsedNullifierRecord> {
    Some(ParsedNullifierRecord {
        nullifier_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        zone_set: r.bytes32()?,
        root_version: r.u32()?,
        prover: r.bytes32()?,
        proof_purpose: r.string()?,
        disclosure_type: r.u8()?,
        block_time: r.i64()?,
        proof_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
    })
}

/// Decode an Anchor `NullifierRecord` account buffer.
pub fn parse_nullifier_record(data: &[u8]) -> Option<ParsedNullifierRecord> {
    if !discriminator(data, &NULLIFIER_RECORD_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_nullifier_record(&mut r)
}

/// Anchor discriminator from the IDL for `Observation`.
pub const OBSERVATION_DISCRIMINATOR: [u8; 8] = [109, 190, 190, 95, 28, 172, 243, 74];

/// IDL account `Observation` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedObservation {
    pub claim: [u8; 32],
    pub validator: [u8; 32],
    pub observed_at: i64,
    pub location: [i64; 2],
    pub method: u8,
    pub findings_hash: [u8; 32],
    pub confidence: u8,
    pub signature_hash: [u8; 32],
    pub created_at: i64,
}

/// Read `Observation` fields (no discriminator) from a Borsh reader.
fn read_observation(mut r: &mut Reader) -> Option<ParsedObservation> {
    Some(ParsedObservation {
        claim: r.bytes32()?,
        validator: r.bytes32()?,
        observed_at: r.i64()?,
        location: {
            let mut out: [i64; 2] = [0i64; 2];
            for item in &mut out {
                *item = r.i64()?;
            }
            out
        },
        method: r.u8()?,
        findings_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        confidence: r.u8()?,
        signature_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `Observation` account buffer.
pub fn parse_observation(data: &[u8]) -> Option<ParsedObservation> {
    if !discriminator(data, &OBSERVATION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_observation(&mut r)
}

/// Anchor discriminator from the IDL for `ObservationV2`.
pub const OBSERVATION_V2_DISCRIMINATOR: [u8; 8] = [194, 12, 252, 52, 247, 150, 119, 195];

/// IDL account `ObservationV2` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedObservationV2 {
    pub task_id: [u8; 32],
    pub observer: [u8; 32],
    pub nonce: u16,
    pub subject: [u8; 32],
    pub capture_device: [u8; 32],
    pub source: u8,
    pub provenance: u8,
    pub location: [i64; 2],
    pub observed_at: i64,
    pub findings_hash: [u8; 32],
    pub evidence_hash: [u8; 32],
    pub confidence: u8,
    pub signature_hash: [u8; 32],
    pub created_at: i64,
}

/// Read `ObservationV2` fields (no discriminator) from a Borsh reader.
fn read_observation_v2(mut r: &mut Reader) -> Option<ParsedObservationV2> {
    Some(ParsedObservationV2 {
        task_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        observer: r.bytes32()?,
        nonce: r.u16()?,
        subject: r.bytes32()?,
        capture_device: r.bytes32()?,
        source: r.u8()?,
        provenance: r.u8()?,
        location: {
            let mut out: [i64; 2] = [0i64; 2];
            for item in &mut out {
                *item = r.i64()?;
            }
            out
        },
        observed_at: r.i64()?,
        findings_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        evidence_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        confidence: r.u8()?,
        signature_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `ObservationV2` account buffer.
pub fn parse_observation_v2(data: &[u8]) -> Option<ParsedObservationV2> {
    if !discriminator(data, &OBSERVATION_V2_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_observation_v2(&mut r)
}

/// Anchor discriminator from the IDL for `Observer`.
pub const OBSERVER_DISCRIMINATOR: [u8; 8] = [82, 255, 234, 217, 166, 201, 80, 72];

/// IDL account `Observer` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedObserver {
    pub wallet: [u8; 32],
    pub identity: [u8; 32],
    pub status: u8,
    pub total_observations: u64,
    pub confirmed_observations: u64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `Observer` fields (no discriminator) from a Borsh reader.
fn read_observer(mut r: &mut Reader) -> Option<ParsedObserver> {
    Some(ParsedObserver {
        wallet: r.bytes32()?,
        identity: r.bytes32()?,
        status: r.u8()?,
        total_observations: r.u64()?,
        confirmed_observations: r.u64()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `Observer` account buffer.
pub fn parse_observer(data: &[u8]) -> Option<ParsedObserver> {
    if !discriminator(data, &OBSERVER_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_observer(&mut r)
}

/// Anchor discriminator from the IDL for `OwnershipRoot`.
pub const OWNERSHIP_ROOT_DISCRIMINATOR: [u8; 8] = [130, 74, 117, 24, 60, 232, 70, 225];

/// IDL account `OwnershipRoot` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedOwnershipRoot {
    pub zone_set: [u8; 32],
    pub merkle_root: [u8; 32],
    pub version: u32,
    pub commitment_count: u32,
    pub algorithm_id: u8,
    pub snapshot_cid: String,
    pub snapshot_hash: [u8; 32],
    pub authority_signature: Vec<u8>,
    pub verification_key_hash: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `OwnershipRoot` fields (no discriminator) from a Borsh reader.
fn read_ownership_root(mut r: &mut Reader) -> Option<ParsedOwnershipRoot> {
    Some(ParsedOwnershipRoot {
        zone_set: r.bytes32()?,
        merkle_root: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        version: r.u32()?,
        commitment_count: r.u32()?,
        algorithm_id: r.u8()?,
        snapshot_cid: r.string()?,
        snapshot_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        authority_signature: {
            let mut out = Vec::with_capacity(64);
            for _ in 0..64 {
                out.push(r.u8()?);
            }
            out
        },
        verification_key_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `OwnershipRoot` account buffer.
pub fn parse_ownership_root(data: &[u8]) -> Option<ParsedOwnershipRoot> {
    if !discriminator(data, &OWNERSHIP_ROOT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_ownership_root(&mut r)
}

/// Anchor discriminator from the IDL for `Parcel`.
pub const PARCEL_DISCRIMINATOR: [u8; 8] = [149, 167, 245, 67, 209, 244, 214, 75];

/// IDL account `Parcel` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedParcel {
    pub id: [u8; 32],
    pub name: String,
    pub geometry_hash: [u8; 32],
    pub status: u8,
    pub rights_count: u8,
    pub infrastructure_flags: u16,
    pub access_hash: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `Parcel` fields (no discriminator) from a Borsh reader.
fn read_parcel(mut r: &mut Reader) -> Option<ParsedParcel> {
    Some(ParsedParcel {
        id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        name: r.string()?,
        geometry_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        status: r.u8()?,
        rights_count: r.u8()?,
        infrastructure_flags: r.u16()?,
        access_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `Parcel` account buffer.
pub fn parse_parcel(data: &[u8]) -> Option<ParsedParcel> {
    if !discriminator(data, &PARCEL_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_parcel(&mut r)
}

/// Anchor discriminator from the IDL for `ParcelGeometryArtifact`.
pub const PARCEL_GEOMETRY_ARTIFACT_DISCRIMINATOR: [u8; 8] = [8, 105, 58, 85, 110, 205, 206, 167];

/// IDL account `ParcelGeometryArtifact` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedParcelGeometryArtifact {
    pub parcel: [u8; 32],
    pub content_hash: [u8; 32],
    pub storage_uri: String,
    pub registrant: [u8; 32],
    pub created_at: i64,
}

/// Read `ParcelGeometryArtifact` fields (no discriminator) from a Borsh reader.
fn read_parcel_geometry_artifact(mut r: &mut Reader) -> Option<ParsedParcelGeometryArtifact> {
    Some(ParsedParcelGeometryArtifact {
        parcel: r.bytes32()?,
        content_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        storage_uri: r.string()?,
        registrant: r.bytes32()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `ParcelGeometryArtifact` account buffer.
pub fn parse_parcel_geometry_artifact(data: &[u8]) -> Option<ParsedParcelGeometryArtifact> {
    if !discriminator(data, &PARCEL_GEOMETRY_ARTIFACT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_parcel_geometry_artifact(&mut r)
}

/// Anchor discriminator from the IDL for `PrivacyRequest`.
pub const PRIVACY_REQUEST_DISCRIMINATOR: [u8; 8] = [108, 60, 110, 140, 63, 148, 161, 86];

/// IDL account `PrivacyRequest` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedPrivacyRequest {
    pub requester: [u8; 32],
    pub subject: [u8; 32],
    pub nonce: u16,
    pub request_type: u8,
    pub description: String,
    pub requested_remedy: u8,
    pub jurisdiction: [u8; 2],
    pub status: u8,
    pub emergency: bool,
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `PrivacyRequest` fields (no discriminator) from a Borsh reader.
fn read_privacy_request(mut r: &mut Reader) -> Option<ParsedPrivacyRequest> {
    Some(ParsedPrivacyRequest {
        requester: r.bytes32()?,
        subject: r.bytes32()?,
        nonce: r.u16()?,
        request_type: r.u8()?,
        description: r.string()?,
        requested_remedy: r.u8()?,
        jurisdiction: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        status: r.u8()?,
        emergency: r.boolean()?,
        expires_at: r.i64()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `PrivacyRequest` account buffer.
pub fn parse_privacy_request(data: &[u8]) -> Option<ParsedPrivacyRequest> {
    if !discriminator(data, &PRIVACY_REQUEST_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_privacy_request(&mut r)
}

/// Anchor discriminator from the IDL for `PrivacyState`.
pub const PRIVACY_STATE_DISCRIMINATOR: [u8; 8] = [57, 131, 134, 54, 197, 203, 111, 138];

/// IDL account `PrivacyState` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedPrivacyState {
    pub subject: [u8; 32],
    pub version: u16,
    pub state: u8,
    pub remedy: u8,
    pub restriction_request: Option<[u8; 32]>,
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `PrivacyState` fields (no discriminator) from a Borsh reader.
fn read_privacy_state(mut r: &mut Reader) -> Option<ParsedPrivacyState> {
    Some(ParsedPrivacyState {
        subject: r.bytes32()?,
        version: r.u16()?,
        state: r.u8()?,
        remedy: r.u8()?,
        restriction_request: match r.u8()? {
            0 => None,
            1 => Some(r.bytes32()?),
            _ => return None,
        },
        expires_at: match r.u8()? {
            0 => None,
            1 => Some(r.i64()?),
            _ => return None,
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `PrivacyState` account buffer.
pub fn parse_privacy_state(data: &[u8]) -> Option<ParsedPrivacyState> {
    if !discriminator(data, &PRIVACY_STATE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_privacy_state(&mut r)
}

/// Anchor discriminator from the IDL for `QuorumConfig`.
pub const QUORUM_CONFIG_DISCRIMINATOR: [u8; 8] = [20, 13, 136, 87, 137, 5, 77, 126];

/// IDL account `QuorumConfig` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedQuorumConfig {
    pub parcel_type: u8,
    pub region: [u8; 2],
    pub required_attestations: u8,
    pub required_confidence: u8,
    pub set_by: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `QuorumConfig` fields (no discriminator) from a Borsh reader.
fn read_quorum_config(mut r: &mut Reader) -> Option<ParsedQuorumConfig> {
    Some(ParsedQuorumConfig {
        parcel_type: r.u8()?,
        region: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        required_attestations: r.u8()?,
        required_confidence: r.u8()?,
        set_by: r.bytes32()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `QuorumConfig` account buffer.
pub fn parse_quorum_config(data: &[u8]) -> Option<ParsedQuorumConfig> {
    if !discriminator(data, &QUORUM_CONFIG_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_quorum_config(&mut r)
}

/// Anchor discriminator from the IDL for `QuorumTally`.
pub const QUORUM_TALLY_DISCRIMINATOR: [u8; 8] = [224, 79, 255, 67, 22, 107, 98, 231];

/// IDL account `QuorumTally` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedQuorumTally {
    pub claim: [u8; 32],
    pub total_weight: u16,
    pub confirm_weight: u16,
    pub dispute_weight: u16,
    pub abstain_weight: u16,
    pub total_votes: u8,
    pub quorum_threshold: u16,
    pub resolved: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `QuorumTally` fields (no discriminator) from a Borsh reader.
fn read_quorum_tally(mut r: &mut Reader) -> Option<ParsedQuorumTally> {
    Some(ParsedQuorumTally {
        claim: r.bytes32()?,
        total_weight: r.u16()?,
        confirm_weight: r.u16()?,
        dispute_weight: r.u16()?,
        abstain_weight: r.u16()?,
        total_votes: r.u8()?,
        quorum_threshold: r.u16()?,
        resolved: r.boolean()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `QuorumTally` account buffer.
pub fn parse_quorum_tally(data: &[u8]) -> Option<ParsedQuorumTally> {
    if !discriminator(data, &QUORUM_TALLY_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_quorum_tally(&mut r)
}

/// Anchor discriminator from the IDL for `QuorumVote`.
pub const QUORUM_VOTE_DISCRIMINATOR: [u8; 8] = [6, 0, 156, 42, 100, 253, 158, 169];

/// IDL account `QuorumVote` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedQuorumVote {
    pub claim: [u8; 32],
    pub voter: [u8; 32],
    pub vote: u8,
    pub weight: u16,
    pub stake_amount: u64,
    pub created_at: i64,
}

/// Read `QuorumVote` fields (no discriminator) from a Borsh reader.
fn read_quorum_vote(mut r: &mut Reader) -> Option<ParsedQuorumVote> {
    Some(ParsedQuorumVote {
        claim: r.bytes32()?,
        voter: r.bytes32()?,
        vote: r.u8()?,
        weight: r.u16()?,
        stake_amount: r.u64()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `QuorumVote` account buffer.
pub fn parse_quorum_vote(data: &[u8]) -> Option<ParsedQuorumVote> {
    if !discriminator(data, &QUORUM_VOTE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_quorum_vote(&mut r)
}

/// Anchor discriminator from the IDL for `ResourceQuote`.
pub const RESOURCE_QUOTE_DISCRIMINATOR: [u8; 8] = [12, 173, 242, 254, 102, 42, 31, 5];

/// IDL account `ResourceQuote` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedResourceQuote {
    pub task_id: [u8; 32],
    pub task: [u8; 32],
    pub quoter: [u8; 32],
    pub resource_cost: u64,
    pub created_at: i64,
}

/// Read `ResourceQuote` fields (no discriminator) from a Borsh reader.
fn read_resource_quote(mut r: &mut Reader) -> Option<ParsedResourceQuote> {
    Some(ParsedResourceQuote {
        task_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        task: r.bytes32()?,
        quoter: r.bytes32()?,
        resource_cost: r.u64()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `ResourceQuote` account buffer.
pub fn parse_resource_quote(data: &[u8]) -> Option<ParsedResourceQuote> {
    if !discriminator(data, &RESOURCE_QUOTE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_resource_quote(&mut r)
}

/// Anchor discriminator from the IDL for `ReviewCase`.
pub const REVIEW_CASE_DISCRIMINATOR: [u8; 8] = [91, 9, 100, 255, 116, 51, 53, 93];

/// IDL account `ReviewCase` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedReviewCase {
    pub report: [u8; 32],
    pub accused: [u8; 32],
    pub committee: Vec<[u8; 32]>,
    pub voted: Vec<[u8; 32]>,
    pub votes_cast: u8,
    pub votes_upheld: u8,
    pub votes_dismissed: u8,
    pub decision: u8,
    pub opened_at: i64,
    pub decided_at: i64,
}

/// Read `ReviewCase` fields (no discriminator) from a Borsh reader.
fn read_review_case(mut r: &mut Reader) -> Option<ParsedReviewCase> {
    Some(ParsedReviewCase {
        report: r.bytes32()?,
        accused: r.bytes32()?,
        committee: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        voted: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        votes_cast: r.u8()?,
        votes_upheld: r.u8()?,
        votes_dismissed: r.u8()?,
        decision: r.u8()?,
        opened_at: r.i64()?,
        decided_at: r.i64()?,
    })
}

/// Decode an Anchor `ReviewCase` account buffer.
pub fn parse_review_case(data: &[u8]) -> Option<ParsedReviewCase> {
    if !discriminator(data, &REVIEW_CASE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_review_case(&mut r)
}

/// Anchor discriminator from the IDL for `RewardAllocation`.
pub const REWARD_ALLOCATION_DISCRIMINATOR: [u8; 8] = [126, 230, 173, 145, 43, 165, 30, 112];

/// IDL account `RewardAllocation` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedRewardAllocation {
    pub task_id: [u8; 32],
    pub task: [u8; 32],
    pub validator: [u8; 32],
    pub base_paid: u64,
    pub subsidy_paid: u64,
    pub total_paid: u64,
    pub created_at: i64,
}

/// Read `RewardAllocation` fields (no discriminator) from a Borsh reader.
fn read_reward_allocation(mut r: &mut Reader) -> Option<ParsedRewardAllocation> {
    Some(ParsedRewardAllocation {
        task_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        task: r.bytes32()?,
        validator: r.bytes32()?,
        base_paid: r.u64()?,
        subsidy_paid: r.u64()?,
        total_paid: r.u64()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `RewardAllocation` account buffer.
pub fn parse_reward_allocation(data: &[u8]) -> Option<ParsedRewardAllocation> {
    if !discriminator(data, &REWARD_ALLOCATION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_reward_allocation(&mut r)
}

/// Anchor discriminator from the IDL for `Rights`.
pub const RIGHTS_DISCRIMINATOR: [u8; 8] = [79, 187, 86, 85, 243, 213, 103, 234];

/// IDL account `Rights` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedRights {
    pub parcel: [u8; 32],
    pub rights_kind: u8,
    pub holder: [u8; 32],
    pub granter: [u8; 32],
    pub created_at: i64,
    pub expires_at: i64,
    pub notes: String,
    pub status: u8,
    pub grace_period_secs: i64,
}

/// Read `Rights` fields (no discriminator) from a Borsh reader.
fn read_rights(mut r: &mut Reader) -> Option<ParsedRights> {
    Some(ParsedRights {
        parcel: r.bytes32()?,
        rights_kind: r.u8()?,
        holder: r.bytes32()?,
        granter: r.bytes32()?,
        created_at: r.i64()?,
        expires_at: r.i64()?,
        notes: r.string()?,
        status: r.u8()?,
        grace_period_secs: r.i64()?,
    })
}

/// Decode an Anchor `Rights` account buffer.
pub fn parse_rights(data: &[u8]) -> Option<ParsedRights> {
    if !discriminator(data, &RIGHTS_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_rights(&mut r)
}

/// Anchor discriminator from the IDL for `SecurityAppeal`.
pub const SECURITY_APPEAL_DISCRIMINATOR: [u8; 8] = [125, 1, 36, 0, 78, 64, 143, 200];

/// IDL account `SecurityAppeal` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSecurityAppeal {
    pub decision: [u8; 32],
    pub appellant: [u8; 32],
    pub nonce: u16,
    pub reason: String,
    pub status: u8,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}

/// Read `SecurityAppeal` fields (no discriminator) from a Borsh reader.
fn read_security_appeal(mut r: &mut Reader) -> Option<ParsedSecurityAppeal> {
    Some(ParsedSecurityAppeal {
        decision: r.bytes32()?,
        appellant: r.bytes32()?,
        nonce: r.u16()?,
        reason: r.string()?,
        status: r.u8()?,
        created_at: r.i64()?,
        resolved_at: match r.u8()? {
            0 => None,
            1 => Some(r.i64()?),
            _ => return None,
        },
    })
}

/// Decode an Anchor `SecurityAppeal` account buffer.
pub fn parse_security_appeal(data: &[u8]) -> Option<ParsedSecurityAppeal> {
    if !discriminator(data, &SECURITY_APPEAL_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_security_appeal(&mut r)
}

/// Anchor discriminator from the IDL for `SecurityAuditEvent`.
pub const SECURITY_AUDIT_EVENT_DISCRIMINATOR: [u8; 8] = [20, 158, 241, 227, 102, 108, 82, 110];

/// IDL account `SecurityAuditEvent` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSecurityAuditEvent {
    pub entity: [u8; 32],
    pub sequence: u32,
    pub action: u8,
    pub actor: [u8; 32],
    pub timestamp: i64,
    pub reason: String,
}

/// Read `SecurityAuditEvent` fields (no discriminator) from a Borsh reader.
fn read_security_audit_event(mut r: &mut Reader) -> Option<ParsedSecurityAuditEvent> {
    Some(ParsedSecurityAuditEvent {
        entity: r.bytes32()?,
        sequence: r.u32()?,
        action: r.u8()?,
        actor: r.bytes32()?,
        timestamp: r.i64()?,
        reason: r.string()?,
    })
}

/// Decode an Anchor `SecurityAuditEvent` account buffer.
pub fn parse_security_audit_event(data: &[u8]) -> Option<ParsedSecurityAuditEvent> {
    if !discriminator(data, &SECURITY_AUDIT_EVENT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_security_audit_event(&mut r)
}

/// Anchor discriminator from the IDL for `SecurityDecision`.
pub const SECURITY_DECISION_DISCRIMINATOR: [u8; 8] = [205, 222, 130, 33, 186, 126, 124, 228];

/// IDL account `SecurityDecision` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSecurityDecision {
    pub request: [u8; 32],
    pub case: [u8; 32],
    pub remedy: u8,
    pub state: u8,
    pub restriction_scope: [u8; 2],
    pub expires_at: Option<i64>,
    pub decided_at: i64,
    pub decided_by: [u8; 32],
}

/// Read `SecurityDecision` fields (no discriminator) from a Borsh reader.
fn read_security_decision(mut r: &mut Reader) -> Option<ParsedSecurityDecision> {
    Some(ParsedSecurityDecision {
        request: r.bytes32()?,
        case: r.bytes32()?,
        remedy: r.u8()?,
        state: r.u8()?,
        restriction_scope: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        expires_at: match r.u8()? {
            0 => None,
            1 => Some(r.i64()?),
            _ => return None,
        },
        decided_at: r.i64()?,
        decided_by: r.bytes32()?,
    })
}

/// Decode an Anchor `SecurityDecision` account buffer.
pub fn parse_security_decision(data: &[u8]) -> Option<ParsedSecurityDecision> {
    if !discriminator(data, &SECURITY_DECISION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_security_decision(&mut r)
}

/// Anchor discriminator from the IDL for `SecurityRequest`.
pub const SECURITY_REQUEST_DISCRIMINATOR: [u8; 8] = [85, 161, 107, 9, 224, 59, 84, 186];

/// IDL account `SecurityRequest` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSecurityRequest {
    pub requester: [u8; 32],
    pub subject: [u8; 32],
    pub nonce: u16,
    pub request_type: u8,
    pub description: String,
    pub requested_remedy: u8,
    pub jurisdiction: [u8; 2],
    pub status: u8,
    pub emergency: bool,
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `SecurityRequest` fields (no discriminator) from a Borsh reader.
fn read_security_request(mut r: &mut Reader) -> Option<ParsedSecurityRequest> {
    Some(ParsedSecurityRequest {
        requester: r.bytes32()?,
        subject: r.bytes32()?,
        nonce: r.u16()?,
        request_type: r.u8()?,
        description: r.string()?,
        requested_remedy: r.u8()?,
        jurisdiction: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        status: r.u8()?,
        emergency: r.boolean()?,
        expires_at: r.i64()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `SecurityRequest` account buffer.
pub fn parse_security_request(data: &[u8]) -> Option<ParsedSecurityRequest> {
    if !discriminator(data, &SECURITY_REQUEST_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_security_request(&mut r)
}

/// Anchor discriminator from the IDL for `SecurityReviewAssignment`.
pub const SECURITY_REVIEW_ASSIGNMENT_DISCRIMINATOR: [u8; 8] = [179, 187, 202, 26, 177, 130, 71, 17];

/// IDL account `SecurityReviewAssignment` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSecurityReviewAssignment {
    pub case: [u8; 32],
    pub reviewer: [u8; 32],
    pub voted: bool,
    pub vote: u8,
    pub voted_at: Option<i64>,
    pub assigned_at: i64,
}

/// Read `SecurityReviewAssignment` fields (no discriminator) from a Borsh reader.
fn read_security_review_assignment(mut r: &mut Reader) -> Option<ParsedSecurityReviewAssignment> {
    Some(ParsedSecurityReviewAssignment {
        case: r.bytes32()?,
        reviewer: r.bytes32()?,
        voted: r.boolean()?,
        vote: r.u8()?,
        voted_at: match r.u8()? {
            0 => None,
            1 => Some(r.i64()?),
            _ => return None,
        },
        assigned_at: r.i64()?,
    })
}

/// Decode an Anchor `SecurityReviewAssignment` account buffer.
pub fn parse_security_review_assignment(data: &[u8]) -> Option<ParsedSecurityReviewAssignment> {
    if !discriminator(data, &SECURITY_REVIEW_ASSIGNMENT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_security_review_assignment(&mut r)
}

/// Anchor discriminator from the IDL for `SecurityReviewCase`.
pub const SECURITY_REVIEW_CASE_DISCRIMINATOR: [u8; 8] = [65, 3, 209, 46, 164, 238, 0, 179];

/// IDL account `SecurityReviewCase` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSecurityReviewCase {
    pub request: [u8; 32],
    pub requester: [u8; 32],
    pub subject: [u8; 32],
    pub committee_size: u8,
    pub votes_upheld: u8,
    pub votes_dismissed: u8,
    pub decision: u8,
    pub finalized_at: Option<i64>,
    pub created_at: i64,
}

/// Read `SecurityReviewCase` fields (no discriminator) from a Borsh reader.
fn read_security_review_case(mut r: &mut Reader) -> Option<ParsedSecurityReviewCase> {
    Some(ParsedSecurityReviewCase {
        request: r.bytes32()?,
        requester: r.bytes32()?,
        subject: r.bytes32()?,
        committee_size: r.u8()?,
        votes_upheld: r.u8()?,
        votes_dismissed: r.u8()?,
        decision: r.u8()?,
        finalized_at: match r.u8()? {
            0 => None,
            1 => Some(r.i64()?),
            _ => return None,
        },
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `SecurityReviewCase` account buffer.
pub fn parse_security_review_case(data: &[u8]) -> Option<ParsedSecurityReviewCase> {
    if !discriminator(data, &SECURITY_REVIEW_CASE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_security_review_case(&mut r)
}

/// Anchor discriminator from the IDL for `SlashingReport`.
pub const SLASHING_REPORT_DISCRIMINATOR: [u8; 8] = [73, 176, 94, 36, 3, 159, 87, 218];

/// IDL account `SlashingReport` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSlashingReport {
    pub stake_pool: [u8; 32],
    pub reporter: [u8; 32],
    pub evidence_hash: [u8; 32],
    pub offender: [u8; 32],
    pub offense_type: u8,
    pub offense_details: Vec<u8>,
    pub reporter_bond: u64,
    pub status: u8,
    pub filed_at: i64,
    pub appeal_deadline: i64,
    pub resolved_at: i64,
}

/// Read `SlashingReport` fields (no discriminator) from a Borsh reader.
fn read_slashing_report(mut r: &mut Reader) -> Option<ParsedSlashingReport> {
    Some(ParsedSlashingReport {
        stake_pool: r.bytes32()?,
        reporter: r.bytes32()?,
        evidence_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        offender: r.bytes32()?,
        offense_type: r.u8()?,
        offense_details: {
            let mut out = Vec::with_capacity(64);
            for _ in 0..64 {
                out.push(r.u8()?);
            }
            out
        },
        reporter_bond: r.u64()?,
        status: r.u8()?,
        filed_at: r.i64()?,
        appeal_deadline: r.i64()?,
        resolved_at: r.i64()?,
    })
}

/// Decode an Anchor `SlashingReport` account buffer.
pub fn parse_slashing_report(data: &[u8]) -> Option<ParsedSlashingReport> {
    if !discriminator(data, &SLASHING_REPORT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_slashing_report(&mut r)
}

/// Anchor discriminator from the IDL for `SpatialAsset`.
pub const SPATIAL_ASSET_DISCRIMINATOR: [u8; 8] = [143, 133, 148, 4, 118, 50, 10, 170];

/// IDL account `SpatialAsset` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSpatialAsset {
    pub parcel: [u8; 32],
    pub authority: [u8; 32],
    pub dimensionality: u8,
    pub elevation_min_mm: i64,
    pub elevation_max_mm: i64,
    pub geometry_version_count: u32,
    pub latest_geometry: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `SpatialAsset` fields (no discriminator) from a Borsh reader.
fn read_spatial_asset(mut r: &mut Reader) -> Option<ParsedSpatialAsset> {
    Some(ParsedSpatialAsset {
        parcel: r.bytes32()?,
        authority: r.bytes32()?,
        dimensionality: r.u8()?,
        elevation_min_mm: r.i64()?,
        elevation_max_mm: r.i64()?,
        geometry_version_count: r.u32()?,
        latest_geometry: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `SpatialAsset` account buffer.
pub fn parse_spatial_asset(data: &[u8]) -> Option<ParsedSpatialAsset> {
    if !discriminator(data, &SPATIAL_ASSET_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_spatial_asset(&mut r)
}

/// Anchor discriminator from the IDL for `SpatialSnapshot`.
pub const SPATIAL_SNAPSHOT_DISCRIMINATOR: [u8; 8] = [156, 60, 119, 78, 61, 63, 161, 245];

/// IDL account `SpatialSnapshot` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSpatialSnapshot {
    pub asset: [u8; 32],
    pub parcel: [u8; 32],
    pub snapshot_index: u32,
    pub geometry_version_count: u32,
    pub latest_geometry: [u8; 32],
    pub dimensionality: u8,
    pub elevation_min_mm: i64,
    pub elevation_max_mm: i64,
    pub label: String,
    pub created_by: [u8; 32],
    pub created_at: i64,
}

/// Read `SpatialSnapshot` fields (no discriminator) from a Borsh reader.
fn read_spatial_snapshot(mut r: &mut Reader) -> Option<ParsedSpatialSnapshot> {
    Some(ParsedSpatialSnapshot {
        asset: r.bytes32()?,
        parcel: r.bytes32()?,
        snapshot_index: r.u32()?,
        geometry_version_count: r.u32()?,
        latest_geometry: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        dimensionality: r.u8()?,
        elevation_min_mm: r.i64()?,
        elevation_max_mm: r.i64()?,
        label: r.string()?,
        created_by: r.bytes32()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `SpatialSnapshot` account buffer.
pub fn parse_spatial_snapshot(data: &[u8]) -> Option<ParsedSpatialSnapshot> {
    if !discriminator(data, &SPATIAL_SNAPSHOT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_spatial_snapshot(&mut r)
}

/// Anchor discriminator from the IDL for `SpatialSnapshotMeta`.
pub const SPATIAL_SNAPSHOT_META_DISCRIMINATOR: [u8; 8] = [123, 54, 165, 171, 172, 35, 63, 23];

/// IDL account `SpatialSnapshotMeta` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSpatialSnapshotMeta {
    pub asset: [u8; 32],
    pub count: u32,
}

/// Read `SpatialSnapshotMeta` fields (no discriminator) from a Borsh reader.
fn read_spatial_snapshot_meta(mut r: &mut Reader) -> Option<ParsedSpatialSnapshotMeta> {
    Some(ParsedSpatialSnapshotMeta {
        asset: r.bytes32()?,
        count: r.u32()?,
    })
}

/// Decode an Anchor `SpatialSnapshotMeta` account buffer.
pub fn parse_spatial_snapshot_meta(data: &[u8]) -> Option<ParsedSpatialSnapshotMeta> {
    if !discriminator(data, &SPATIAL_SNAPSHOT_META_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_spatial_snapshot_meta(&mut r)
}

/// Anchor discriminator from the IDL for `StakePool`.
pub const STAKE_POOL_DISCRIMINATOR: [u8; 8] = [121, 34, 206, 21, 79, 127, 255, 28];

/// IDL account `StakePool` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedStakePool {
    pub region_registry: [u8; 32],
    pub total_staked: u64,
    pub reward_rate_bps: u16,
    pub accumulated_rewards: u64,
    pub reward_per_token_stored: u64,
    pub last_reward_distribution: i64,
    pub slash_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `StakePool` fields (no discriminator) from a Borsh reader.
fn read_stake_pool(mut r: &mut Reader) -> Option<ParsedStakePool> {
    Some(ParsedStakePool {
        region_registry: r.bytes32()?,
        total_staked: r.u64()?,
        reward_rate_bps: r.u16()?,
        accumulated_rewards: r.u64()?,
        reward_per_token_stored: r.u64()?,
        last_reward_distribution: r.i64()?,
        slash_count: r.u32()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `StakePool` account buffer.
pub fn parse_stake_pool(data: &[u8]) -> Option<ParsedStakePool> {
    if !discriminator(data, &STAKE_POOL_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_stake_pool(&mut r)
}

/// Anchor discriminator from the IDL for `SubdivisionRecord`.
pub const SUBDIVISION_RECORD_DISCRIMINATOR: [u8; 8] = [9, 214, 108, 50, 166, 44, 134, 32];

/// IDL account `SubdivisionRecord` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSubdivisionRecord {
    pub original_parcel: [u8; 32],
    pub sub_parcel: [u8; 32],
    pub original_geometry_hash: [u8; 32],
    pub new_geometry_hash: [u8; 32],
    pub survey_claim: [u8; 32],
    pub rights_migrated: bool,
    pub initiated_by: [u8; 32],
    pub created_at: i64,
    pub completed_at: i64,
    pub status: u8,
}

/// Read `SubdivisionRecord` fields (no discriminator) from a Borsh reader.
fn read_subdivision_record(mut r: &mut Reader) -> Option<ParsedSubdivisionRecord> {
    Some(ParsedSubdivisionRecord {
        original_parcel: r.bytes32()?,
        sub_parcel: r.bytes32()?,
        original_geometry_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        new_geometry_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        survey_claim: r.bytes32()?,
        rights_migrated: r.boolean()?,
        initiated_by: r.bytes32()?,
        created_at: r.i64()?,
        completed_at: r.i64()?,
        status: r.u8()?,
    })
}

/// Decode an Anchor `SubdivisionRecord` account buffer.
pub fn parse_subdivision_record(data: &[u8]) -> Option<ParsedSubdivisionRecord> {
    if !discriminator(data, &SUBDIVISION_RECORD_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_subdivision_record(&mut r)
}

/// Anchor discriminator from the IDL for `TaskAssignment`.
pub const TASK_ASSIGNMENT_DISCRIMINATOR: [u8; 8] = [89, 239, 95, 144, 182, 35, 47, 254];

/// IDL account `TaskAssignment` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedTaskAssignment {
    pub task_id: [u8; 32],
    pub validator: [u8; 32],
    pub requester: [u8; 32],
    pub assigned_by: [u8; 32],
    pub status: u8,
    pub assigned_at: i64,
    pub submitted_at: i64,
    pub updated_at: i64,
}

/// Read `TaskAssignment` fields (no discriminator) from a Borsh reader.
fn read_task_assignment(mut r: &mut Reader) -> Option<ParsedTaskAssignment> {
    Some(ParsedTaskAssignment {
        task_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        validator: r.bytes32()?,
        requester: r.bytes32()?,
        assigned_by: r.bytes32()?,
        status: r.u8()?,
        assigned_at: r.i64()?,
        submitted_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `TaskAssignment` account buffer.
pub fn parse_task_assignment(data: &[u8]) -> Option<ParsedTaskAssignment> {
    if !discriminator(data, &TASK_ASSIGNMENT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_task_assignment(&mut r)
}

/// Anchor discriminator from the IDL for `TaskEscrow`.
pub const TASK_ESCROW_DISCRIMINATOR: [u8; 8] = [209, 72, 197, 54, 17, 55, 3, 187];

/// IDL account `TaskEscrow` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedTaskEscrow {
    pub task_id: [u8; 32],
    pub task: [u8; 32],
    pub requester: [u8; 32],
    pub vault: [u8; 32],
    pub amount: u64,
    pub fee_paid: u64,
    pub released: u64,
    pub released_count: u8,
    pub status: u8,
    pub created_at: i64,
    pub funded_at: i64,
    pub updated_at: i64,
}

/// Read `TaskEscrow` fields (no discriminator) from a Borsh reader.
fn read_task_escrow(mut r: &mut Reader) -> Option<ParsedTaskEscrow> {
    Some(ParsedTaskEscrow {
        task_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        task: r.bytes32()?,
        requester: r.bytes32()?,
        vault: r.bytes32()?,
        amount: r.u64()?,
        fee_paid: r.u64()?,
        released: r.u64()?,
        released_count: r.u8()?,
        status: r.u8()?,
        created_at: r.i64()?,
        funded_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `TaskEscrow` account buffer.
pub fn parse_task_escrow(data: &[u8]) -> Option<ParsedTaskEscrow> {
    if !discriminator(data, &TASK_ESCROW_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_task_escrow(&mut r)
}

/// Anchor discriminator from the IDL for `TaskRequirement`.
pub const TASK_REQUIREMENT_DISCRIMINATOR: [u8; 8] = [70, 172, 186, 172, 160, 127, 79, 134];

/// IDL account `TaskRequirement` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedTaskRequirement {
    pub task_id: [u8; 32],
    pub req_index: u8,
    pub capability_code: u8,
    pub min_reputation: u16,
    pub min_tier: u8,
    pub jurisdiction: [u8; 2],
    pub radius_m: u32,
    pub center_lat_e7: i32,
    pub center_lon_e7: i32,
    pub independence_bps: u16,
    pub confidence_target_bps: u16,
    pub created_at: i64,
}

/// Read `TaskRequirement` fields (no discriminator) from a Borsh reader.
fn read_task_requirement(mut r: &mut Reader) -> Option<ParsedTaskRequirement> {
    Some(ParsedTaskRequirement {
        task_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        req_index: r.u8()?,
        capability_code: r.u8()?,
        min_reputation: r.u16()?,
        min_tier: r.u8()?,
        jurisdiction: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        radius_m: r.u32()?,
        center_lat_e7: r.i32()?,
        center_lon_e7: r.i32()?,
        independence_bps: r.u16()?,
        confidence_target_bps: r.u16()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `TaskRequirement` account buffer.
pub fn parse_task_requirement(data: &[u8]) -> Option<ParsedTaskRequirement> {
    if !discriminator(data, &TASK_REQUIREMENT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_task_requirement(&mut r)
}

/// Anchor discriminator from the IDL for `ThreeDModel`.
pub const THREE_D_MODEL_DISCRIMINATOR: [u8; 8] = [117, 174, 95, 32, 116, 194, 171, 170];

/// IDL account `ThreeDModel` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedThreeDModel {
    pub asset: [u8; 32],
    pub parcel: [u8; 32],
    pub model_version: u32,
    pub geometry_version: u32,
    pub model_hash: [u8; 32],
    pub storage_reference: String,
    pub model_format: u8,
    pub submitted_by: [u8; 32],
    pub submitted_at: i64,
    pub verified: bool,
    pub verified_by: [u8; 32],
    pub verified_at: i64,
}

/// Read `ThreeDModel` fields (no discriminator) from a Borsh reader.
fn read_three_d_model(mut r: &mut Reader) -> Option<ParsedThreeDModel> {
    Some(ParsedThreeDModel {
        asset: r.bytes32()?,
        parcel: r.bytes32()?,
        model_version: r.u32()?,
        geometry_version: r.u32()?,
        model_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        storage_reference: r.string()?,
        model_format: r.u8()?,
        submitted_by: r.bytes32()?,
        submitted_at: r.i64()?,
        verified: r.boolean()?,
        verified_by: r.bytes32()?,
        verified_at: r.i64()?,
    })
}

/// Decode an Anchor `ThreeDModel` account buffer.
pub fn parse_three_d_model(data: &[u8]) -> Option<ParsedThreeDModel> {
    if !discriminator(data, &THREE_D_MODEL_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_three_d_model(&mut r)
}

/// Anchor discriminator from the IDL for `ThreeDModelMeta`.
pub const THREE_D_MODEL_META_DISCRIMINATOR: [u8; 8] = [231, 80, 5, 178, 80, 198, 86, 7];

/// IDL account `ThreeDModelMeta` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedThreeDModelMeta {
    pub asset: [u8; 32],
    pub count: u32,
}

/// Read `ThreeDModelMeta` fields (no discriminator) from a Borsh reader.
fn read_three_d_model_meta(mut r: &mut Reader) -> Option<ParsedThreeDModelMeta> {
    Some(ParsedThreeDModelMeta {
        asset: r.bytes32()?,
        count: r.u32()?,
    })
}

/// Decode an Anchor `ThreeDModelMeta` account buffer.
pub fn parse_three_d_model_meta(data: &[u8]) -> Option<ParsedThreeDModelMeta> {
    if !discriminator(data, &THREE_D_MODEL_META_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_three_d_model_meta(&mut r)
}

/// Anchor discriminator from the IDL for `ThresholdCredential`.
pub const THRESHOLD_CREDENTIAL_DISCRIMINATOR: [u8; 8] = [217, 121, 146, 123, 84, 119, 65, 79];

/// IDL account `ThresholdCredential` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedThresholdCredential {
    pub credential_hash: [u8; 32],
    pub prover: [u8; 32],
    pub purpose: String,
    pub disclosure_type: u8,
    pub region_registry: [u8; 32],
    pub aggregate_signature: Vec<u8>,
    pub signers: Vec<[u8; 32]>,
    pub signer_count: u8,
    pub nullifier_hash: [u8; 32],
    pub consumed: bool,
    pub version: u32,
    pub issued_at: i64,
}

/// Read `ThresholdCredential` fields (no discriminator) from a Borsh reader.
fn read_threshold_credential(mut r: &mut Reader) -> Option<ParsedThresholdCredential> {
    Some(ParsedThresholdCredential {
        credential_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        prover: r.bytes32()?,
        purpose: r.string()?,
        disclosure_type: r.u8()?,
        region_registry: r.bytes32()?,
        aggregate_signature: r.blob()?,
        signers: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        signer_count: r.u8()?,
        nullifier_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        consumed: r.boolean()?,
        version: r.u32()?,
        issued_at: r.i64()?,
    })
}

/// Decode an Anchor `ThresholdCredential` account buffer.
pub fn parse_threshold_credential(data: &[u8]) -> Option<ParsedThresholdCredential> {
    if !discriminator(data, &THRESHOLD_CREDENTIAL_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_threshold_credential(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorActivityTracker`.
pub const VALIDATOR_ACTIVITY_TRACKER_DISCRIMINATOR: [u8; 8] = [45, 51, 146, 237, 228, 13, 159, 133];

/// IDL account `ValidatorActivityTracker` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorActivityTracker {
    pub registry: [u8; 32],
    pub validator: [u8; 32],
    pub last_active: i64,
    pub is_active: bool,
}

/// Read `ValidatorActivityTracker` fields (no discriminator) from a Borsh reader.
fn read_validator_activity_tracker(mut r: &mut Reader) -> Option<ParsedValidatorActivityTracker> {
    Some(ParsedValidatorActivityTracker {
        registry: r.bytes32()?,
        validator: r.bytes32()?,
        last_active: r.i64()?,
        is_active: r.boolean()?,
    })
}

/// Decode an Anchor `ValidatorActivityTracker` account buffer.
pub fn parse_validator_activity_tracker(data: &[u8]) -> Option<ParsedValidatorActivityTracker> {
    if !discriminator(data, &VALIDATOR_ACTIVITY_TRACKER_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_activity_tracker(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorAvailability`.
pub const VALIDATOR_AVAILABILITY_DISCRIMINATOR: [u8; 8] = [113, 127, 178, 236, 25, 220, 94, 89];

/// IDL account `ValidatorAvailability` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorAvailability {
    pub wallet: [u8; 32],
    pub status: u8,
    pub updated_at: i64,
}

/// Read `ValidatorAvailability` fields (no discriminator) from a Borsh reader.
fn read_validator_availability(mut r: &mut Reader) -> Option<ParsedValidatorAvailability> {
    Some(ParsedValidatorAvailability {
        wallet: r.bytes32()?,
        status: r.u8()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorAvailability` account buffer.
pub fn parse_validator_availability(data: &[u8]) -> Option<ParsedValidatorAvailability> {
    if !discriminator(data, &VALIDATOR_AVAILABILITY_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_availability(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorCapability`.
pub const VALIDATOR_CAPABILITY_DISCRIMINATOR: [u8; 8] = [119, 194, 253, 171, 190, 211, 118, 50];

/// IDL account `ValidatorCapability` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorCapability {
    pub wallet: [u8; 32],
    pub capability_code: u8,
    pub level: u8,
    pub evidence_hash: [u8; 32],
    pub declared_at: i64,
    pub verified_at: i64,
    pub updated_at: i64,
}

/// Read `ValidatorCapability` fields (no discriminator) from a Borsh reader.
fn read_validator_capability(mut r: &mut Reader) -> Option<ParsedValidatorCapability> {
    Some(ParsedValidatorCapability {
        wallet: r.bytes32()?,
        capability_code: r.u8()?,
        level: r.u8()?,
        evidence_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        declared_at: r.i64()?,
        verified_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorCapability` account buffer.
pub fn parse_validator_capability(data: &[u8]) -> Option<ParsedValidatorCapability> {
    if !discriminator(data, &VALIDATOR_CAPABILITY_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_capability(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorEndorsement`.
pub const VALIDATOR_ENDORSEMENT_DISCRIMINATOR: [u8; 8] = [140, 204, 234, 117, 105, 8, 203, 25];

/// IDL account `ValidatorEndorsement` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorEndorsement {
    pub registry: [u8; 32],
    pub proposed: [u8; 32],
    pub action: u8,
    pub endorsers: Vec<[u8; 32]>,
    pub required: u8,
    pub created_at: i64,
}

/// Read `ValidatorEndorsement` fields (no discriminator) from a Borsh reader.
fn read_validator_endorsement(mut r: &mut Reader) -> Option<ParsedValidatorEndorsement> {
    Some(ParsedValidatorEndorsement {
        registry: r.bytes32()?,
        proposed: r.bytes32()?,
        action: r.u8()?,
        endorsers: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        required: r.u8()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorEndorsement` account buffer.
pub fn parse_validator_endorsement(data: &[u8]) -> Option<ParsedValidatorEndorsement> {
    if !discriminator(data, &VALIDATOR_ENDORSEMENT_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_endorsement(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorNomination`.
pub const VALIDATOR_NOMINATION_DISCRIMINATOR: [u8; 8] = [210, 142, 12, 155, 104, 210, 45, 94];

/// IDL account `ValidatorNomination` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorNomination {
    pub registry: [u8; 32],
    pub country_code: [u8; 2],
    pub sponsor: [u8; 32],
    pub candidate: [u8; 32],
    pub documents_hash: [u8; 32],
    pub location_hash: [u8; 32],
    pub assigned_physical_confirmer: [u8; 32],
    pub confirmers: Vec<[u8; 32]>,
    pub finalized: bool,
    pub created_at: i64,
}

/// Read `ValidatorNomination` fields (no discriminator) from a Borsh reader.
fn read_validator_nomination(mut r: &mut Reader) -> Option<ParsedValidatorNomination> {
    Some(ParsedValidatorNomination {
        registry: r.bytes32()?,
        country_code: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        sponsor: r.bytes32()?,
        candidate: r.bytes32()?,
        documents_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        location_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        assigned_physical_confirmer: r.bytes32()?,
        confirmers: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        finalized: r.boolean()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorNomination` account buffer.
pub fn parse_validator_nomination(data: &[u8]) -> Option<ParsedValidatorNomination> {
    if !discriminator(data, &VALIDATOR_NOMINATION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_nomination(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorPresence`.
pub const VALIDATOR_PRESENCE_DISCRIMINATOR: [u8; 8] = [49, 114, 241, 63, 232, 95, 50, 90];

/// IDL account `ValidatorPresence` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorPresence {
    pub wallet: [u8; 32],
    pub latitude_e7: i32,
    pub longitude_e7: i32,
    pub accuracy_m: u16,
    pub provenance: u8,
    pub confidence_bps: u16,
    pub observed_at: i64,
    pub expires_at: i64,
    pub updated_at: i64,
}

/// Read `ValidatorPresence` fields (no discriminator) from a Borsh reader.
fn read_validator_presence(mut r: &mut Reader) -> Option<ParsedValidatorPresence> {
    Some(ParsedValidatorPresence {
        wallet: r.bytes32()?,
        latitude_e7: r.i32()?,
        longitude_e7: r.i32()?,
        accuracy_m: r.u16()?,
        provenance: r.u8()?,
        confidence_bps: r.u16()?,
        observed_at: r.i64()?,
        expires_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorPresence` account buffer.
pub fn parse_validator_presence(data: &[u8]) -> Option<ParsedValidatorPresence> {
    if !discriminator(data, &VALIDATOR_PRESENCE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_presence(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorProfile`.
pub const VALIDATOR_PROFILE_DISCRIMINATOR: [u8; 8] = [50, 223, 146, 114, 220, 208, 127, 86];

/// IDL account `ValidatorProfile` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorProfile {
    pub wallet: [u8; 32],
    pub tier: u8,
    pub identity_hash: [u8; 32],
    pub note: String,
    pub jurisdiction: [u8; 2],
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `ValidatorProfile` fields (no discriminator) from a Borsh reader.
fn read_validator_profile(mut r: &mut Reader) -> Option<ParsedValidatorProfile> {
    Some(ParsedValidatorProfile {
        wallet: r.bytes32()?,
        tier: r.u8()?,
        identity_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        note: r.string()?,
        jurisdiction: {
            let mut out: [u8; 2] = [0u8; 2];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorProfile` account buffer.
pub fn parse_validator_profile(data: &[u8]) -> Option<ParsedValidatorProfile> {
    if !discriminator(data, &VALIDATOR_PROFILE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_profile(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorRegistry`.
pub const VALIDATOR_REGISTRY_DISCRIMINATOR: [u8; 8] = [168, 113, 195, 186, 62, 121, 163, 230];

/// IDL account `ValidatorRegistry` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorRegistry {
    pub admin: [u8; 32],
    pub pending_admin: [u8; 32],
    pub validators: Vec<[u8; 32]>,
    pub required_endorsements: u8,
    pub paused: bool,
    pub version: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `ValidatorRegistry` fields (no discriminator) from a Borsh reader.
fn read_validator_registry(mut r: &mut Reader) -> Option<ParsedValidatorRegistry> {
    Some(ParsedValidatorRegistry {
        admin: r.bytes32()?,
        pending_admin: r.bytes32()?,
        validators: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        required_endorsements: r.u8()?,
        paused: r.boolean()?,
        version: r.u32()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorRegistry` account buffer.
pub fn parse_validator_registry(data: &[u8]) -> Option<ParsedValidatorRegistry> {
    if !discriminator(data, &VALIDATOR_REGISTRY_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_registry(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorRelationshipEdge`.
pub const VALIDATOR_RELATIONSHIP_EDGE_DISCRIMINATOR: [u8; 8] =
    [128, 240, 223, 172, 123, 148, 81, 48];

/// IDL account `ValidatorRelationshipEdge` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorRelationshipEdge {
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub edge_type: u8,
    pub weight_bps: u16,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `ValidatorRelationshipEdge` fields (no discriminator) from a Borsh reader.
fn read_validator_relationship_edge(mut r: &mut Reader) -> Option<ParsedValidatorRelationshipEdge> {
    Some(ParsedValidatorRelationshipEdge {
        from: r.bytes32()?,
        to: r.bytes32()?,
        edge_type: r.u8()?,
        weight_bps: r.u16()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorRelationshipEdge` account buffer.
pub fn parse_validator_relationship_edge(data: &[u8]) -> Option<ParsedValidatorRelationshipEdge> {
    if !discriminator(data, &VALIDATOR_RELATIONSHIP_EDGE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_relationship_edge(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorReputation`.
pub const VALIDATOR_REPUTATION_DISCRIMINATOR: [u8; 8] = [151, 57, 248, 51, 14, 124, 177, 47];

/// IDL account `ValidatorReputation` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorReputation {
    pub validator: [u8; 32],
    pub status: u8,
    pub total_attestations: u64,
    pub confirmed_attestations: u64,
    pub disputed_attestations: u64,
    pub reputation_score: u16,
    pub jailed_until: i64,
    pub challenges_received: u8,
    pub challenges_raised: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `ValidatorReputation` fields (no discriminator) from a Borsh reader.
fn read_validator_reputation(mut r: &mut Reader) -> Option<ParsedValidatorReputation> {
    Some(ParsedValidatorReputation {
        validator: r.bytes32()?,
        status: r.u8()?,
        total_attestations: r.u64()?,
        confirmed_attestations: r.u64()?,
        disputed_attestations: r.u64()?,
        reputation_score: r.u16()?,
        jailed_until: r.i64()?,
        challenges_received: r.u8()?,
        challenges_raised: r.u8()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorReputation` account buffer.
pub fn parse_validator_reputation(data: &[u8]) -> Option<ParsedValidatorReputation> {
    if !discriminator(data, &VALIDATOR_REPUTATION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_reputation(&mut r)
}

/// Anchor discriminator from the IDL for `ValidatorStake`.
pub const VALIDATOR_STAKE_DISCRIMINATOR: [u8; 8] = [95, 210, 178, 207, 78, 5, 247, 29];

/// IDL account `ValidatorStake` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedValidatorStake {
    pub stake_pool: [u8; 32],
    pub validator: [u8; 32],
    pub staked_amount: u64,
    pub unbonding_amount: u64,
    pub unbonding_starts_at: i64,
    pub reward_per_token_paid: u64,
    pub slash_history: u8,
    pub offenses: [u8; 4],
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `ValidatorStake` fields (no discriminator) from a Borsh reader.
fn read_validator_stake(mut r: &mut Reader) -> Option<ParsedValidatorStake> {
    Some(ParsedValidatorStake {
        stake_pool: r.bytes32()?,
        validator: r.bytes32()?,
        staked_amount: r.u64()?,
        unbonding_amount: r.u64()?,
        unbonding_starts_at: r.i64()?,
        reward_per_token_paid: r.u64()?,
        slash_history: r.u8()?,
        offenses: {
            let mut out: [u8; 4] = [0u8; 4];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `ValidatorStake` account buffer.
pub fn parse_validator_stake(data: &[u8]) -> Option<ParsedValidatorStake> {
    if !discriminator(data, &VALIDATOR_STAKE_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_validator_stake(&mut r)
}

/// Anchor discriminator from the IDL for `VaultRecord`.
pub const VAULT_RECORD_DISCRIMINATOR: [u8; 8] = [47, 1, 218, 116, 82, 70, 124, 119];

/// IDL account `VaultRecord` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedVaultRecord {
    pub subject: [u8; 32],
    pub ciphertext_cid: String,
    pub ciphertext_hash: [u8; 32],
    pub algorithm_id: u8,
    pub storage_uris: Vec<String>,
    pub shard_holders: Vec<[u8; 32]>,
    pub threshold: u8,
    pub version: u32,
    pub last_ping_at: i64,
    pub created_at: i64,
}

/// Read `VaultRecord` fields (no discriminator) from a Borsh reader.
fn read_vault_record(mut r: &mut Reader) -> Option<ParsedVaultRecord> {
    Some(ParsedVaultRecord {
        subject: r.bytes32()?,
        ciphertext_cid: r.string()?,
        ciphertext_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        algorithm_id: r.u8()?,
        storage_uris: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.string()?);
            }
            out
        },
        shard_holders: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        threshold: r.u8()?,
        version: r.u32()?,
        last_ping_at: r.i64()?,
        created_at: r.i64()?,
    })
}

/// Decode an Anchor `VaultRecord` account buffer.
pub fn parse_vault_record(data: &[u8]) -> Option<ParsedVaultRecord> {
    if !discriminator(data, &VAULT_RECORD_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_vault_record(&mut r)
}

/// Anchor discriminator from the IDL for `VaultShardRotation`.
pub const VAULT_SHARD_ROTATION_DISCRIMINATOR: [u8; 8] = [33, 187, 116, 141, 215, 66, 44, 139];

/// IDL account `VaultShardRotation` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedVaultShardRotation {
    pub vault: [u8; 32],
    pub old_ciphertext_hash: [u8; 32],
    pub new_ciphertext_hash: [u8; 32],
    pub new_shard_holders: Vec<[u8; 32]>,
    pub new_threshold: u8,
    pub initiated_by: [u8; 32],
    pub endorsements: Vec<[u8; 32]>,
    pub required_endorsements: u8,
    pub initiated_at: i64,
    pub effective_at: i64,
    pub status: u8,
}

/// Read `VaultShardRotation` fields (no discriminator) from a Borsh reader.
fn read_vault_shard_rotation(mut r: &mut Reader) -> Option<ParsedVaultShardRotation> {
    Some(ParsedVaultShardRotation {
        vault: r.bytes32()?,
        old_ciphertext_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        new_ciphertext_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        new_shard_holders: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        new_threshold: r.u8()?,
        initiated_by: r.bytes32()?,
        endorsements: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(r.bytes32()?);
            }
            out
        },
        required_endorsements: r.u8()?,
        initiated_at: r.i64()?,
        effective_at: r.i64()?,
        status: r.u8()?,
    })
}

/// Decode an Anchor `VaultShardRotation` account buffer.
pub fn parse_vault_shard_rotation(data: &[u8]) -> Option<ParsedVaultShardRotation> {
    if !discriminator(data, &VAULT_SHARD_ROTATION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_vault_shard_rotation(&mut r)
}

/// Anchor discriminator from the IDL for `VerificationAttestation`.
pub const VERIFICATION_ATTESTATION_DISCRIMINATOR: [u8; 8] = [231, 126, 92, 51, 84, 178, 81, 242];

/// IDL account `VerificationAttestation` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedVerificationAttestation {
    pub claim: [u8; 32],
    pub validator: [u8; 32],
    pub observation: [u8; 32],
    pub credential_subject: [u8; 32],
    pub vc_type: String,
    pub issuance_date: i64,
    pub expiration_date: i64,
    pub sd_jwt_digest: [u8; 32],
    pub selective_disclosure: bool,
    pub created_at: i64,
    pub protocol_version: u8,
}

/// Read `VerificationAttestation` fields (no discriminator) from a Borsh reader.
fn read_verification_attestation(mut r: &mut Reader) -> Option<ParsedVerificationAttestation> {
    Some(ParsedVerificationAttestation {
        claim: r.bytes32()?,
        validator: r.bytes32()?,
        observation: r.bytes32()?,
        credential_subject: r.bytes32()?,
        vc_type: r.string()?,
        issuance_date: r.i64()?,
        expiration_date: r.i64()?,
        sd_jwt_digest: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        selective_disclosure: r.boolean()?,
        created_at: r.i64()?,
        protocol_version: r.u8()?,
    })
}

/// Decode an Anchor `VerificationAttestation` account buffer.
pub fn parse_verification_attestation(data: &[u8]) -> Option<ParsedVerificationAttestation> {
    if !discriminator(data, &VERIFICATION_ATTESTATION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_verification_attestation(&mut r)
}

/// Anchor discriminator from the IDL for `VerificationSession`.
pub const VERIFICATION_SESSION_DISCRIMINATOR: [u8; 8] = [247, 69, 199, 26, 112, 195, 243, 254];

/// IDL account `VerificationSession` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedVerificationSession {
    pub claim: [u8; 32],
    pub session_id: [u8; 32],
    pub status: u8,
    pub opened_by: [u8; 32],
    pub evidence_count: u8,
    pub observation_count: u8,
    pub attestation_count: u8,
    pub dispute_count: u8,
    pub required_attestations: u8,
    pub created_at: i64,
    pub expires_at: i64,
    pub closed_at: i64,
    pub updated_at: i64,
}

/// Read `VerificationSession` fields (no discriminator) from a Borsh reader.
fn read_verification_session(mut r: &mut Reader) -> Option<ParsedVerificationSession> {
    Some(ParsedVerificationSession {
        claim: r.bytes32()?,
        session_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        status: r.u8()?,
        opened_by: r.bytes32()?,
        evidence_count: r.u8()?,
        observation_count: r.u8()?,
        attestation_count: r.u8()?,
        dispute_count: r.u8()?,
        required_attestations: r.u8()?,
        created_at: r.i64()?,
        expires_at: r.i64()?,
        closed_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `VerificationSession` account buffer.
pub fn parse_verification_session(data: &[u8]) -> Option<ParsedVerificationSession> {
    if !discriminator(data, &VERIFICATION_SESSION_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_verification_session(&mut r)
}

/// Anchor discriminator from the IDL for `VerificationTask`.
pub const VERIFICATION_TASK_DISCRIMINATOR: [u8; 8] = [96, 182, 137, 216, 62, 108, 74, 190];

/// IDL account `VerificationTask` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedVerificationTask {
    pub task_id: [u8; 32],
    pub requester: [u8; 32],
    pub subject: [u8; 32],
    pub task_class: u8,
    pub status: u8,
    pub reward_lamports: u64,
    pub deadline: i64,
    pub description_hash: [u8; 32],
    pub requirement_count: u8,
    pub assigned_count: u8,
    pub required_validators: u8,
    pub result_count: u8,
    pub outcome: u8,
    pub result_hash: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: i64,
}

/// Read `VerificationTask` fields (no discriminator) from a Borsh reader.
fn read_verification_task(mut r: &mut Reader) -> Option<ParsedVerificationTask> {
    Some(ParsedVerificationTask {
        task_id: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        requester: r.bytes32()?,
        subject: r.bytes32()?,
        task_class: r.u8()?,
        status: r.u8()?,
        reward_lamports: r.u64()?,
        deadline: r.i64()?,
        description_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        requirement_count: r.u8()?,
        assigned_count: r.u8()?,
        required_validators: r.u8()?,
        result_count: r.u8()?,
        outcome: r.u8()?,
        result_hash: {
            let mut out: [u8; 32] = [0u8; 32];
            for item in &mut out {
                *item = r.u8()?;
            }
            out
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
        completed_at: r.i64()?,
    })
}

/// Decode an Anchor `VerificationTask` account buffer.
pub fn parse_verification_task(data: &[u8]) -> Option<ParsedVerificationTask> {
    if !discriminator(data, &VERIFICATION_TASK_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_verification_task(&mut r)
}

/// Anchor discriminator from the IDL for `VrfAuthority`.
pub const VRF_AUTHORITY_DISCRIMINATOR: [u8; 8] = [38, 133, 126, 18, 55, 188, 204, 215];

/// IDL account `VrfAuthority` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedVrfAuthority {
    pub authority: [u8; 32],
    pub admin: [u8; 32],
    pub updated_at: i64,
    pub bump: u8,
}

/// Read `VrfAuthority` fields (no discriminator) from a Borsh reader.
fn read_vrf_authority(mut r: &mut Reader) -> Option<ParsedVrfAuthority> {
    Some(ParsedVrfAuthority {
        authority: r.bytes32()?,
        admin: r.bytes32()?,
        updated_at: r.i64()?,
        bump: r.u8()?,
    })
}

/// Decode an Anchor `VrfAuthority` account buffer.
pub fn parse_vrf_authority(data: &[u8]) -> Option<ParsedVrfAuthority> {
    if !discriminator(data, &VRF_AUTHORITY_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_vrf_authority(&mut r)
}

/// Anchor discriminator from the IDL for `WorldRegistry`.
pub const WORLD_REGISTRY_DISCRIMINATOR: [u8; 8] = [222, 41, 153, 93, 64, 60, 109, 27];

/// IDL account `WorldRegistry` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedWorldRegistry {
    pub admin: [u8; 32],
    pub allocations: Vec<ParsedCountryAllocation>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `WorldRegistry` fields (no discriminator) from a Borsh reader.
fn read_world_registry(mut r: &mut Reader) -> Option<ParsedWorldRegistry> {
    Some(ParsedWorldRegistry {
        admin: r.bytes32()?,
        allocations: {
            let len = r.u32()? as usize;
            if len > MAX_VEC_LEN {
                return None;
            }
            let mut out = Vec::with_capacity(len);
            for _ in 0..len {
                out.push(read_country_allocation(&mut *r)?);
            }
            out
        },
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `WorldRegistry` account buffer.
pub fn parse_world_registry(data: &[u8]) -> Option<ParsedWorldRegistry> {
    if !discriminator(data, &WORLD_REGISTRY_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_world_registry(&mut r)
}

/// Anchor discriminator from the IDL for `ZoneSet`.
pub const ZONE_SET_DISCRIMINATOR: [u8; 8] = [57, 110, 98, 208, 4, 53, 98, 77];

/// IDL account `ZoneSet` decoded from Borsh bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedZoneSet {
    pub zone_id: [u8; 32],
    pub authority: [u8; 32],
    pub parcel_count: u32,
    pub current_root_version: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read `ZoneSet` fields (no discriminator) from a Borsh reader.
fn read_zone_set(mut r: &mut Reader) -> Option<ParsedZoneSet> {
    Some(ParsedZoneSet {
        zone_id: r.bytes32()?,
        authority: r.bytes32()?,
        parcel_count: r.u32()?,
        current_root_version: r.u32()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

/// Decode an Anchor `ZoneSet` account buffer.
pub fn parse_zone_set(data: &[u8]) -> Option<ParsedZoneSet> {
    if !discriminator(data, &ZONE_SET_DISCRIMINATOR) {
        return None;
    }
    let mut r = Reader::new(data);
    r.take(8)?;
    read_zone_set(&mut r)
}

/// Every account type in the checked-in IDL. Used by the staged-rollout
/// acceptance tests and by `/mirror/audit` coverage reporting.
pub const ALL_ACCOUNT_NAMES: &[&str] = &[
    "AmalgamationRecord",
    "Appeal",
    "AuditEntry",
    "CapabilityRestriction",
    "Challenge",
    "Claim",
    "ClaimSessionTracker",
    "CoverageIncentive",
    "CredentialNullifier",
    "CredentialRequest",
    "CrossBorderBinding",
    "CrossBorderSpanRecord",
    "CrossBorderVerification",
    "DeviceIdentity",
    "Dispute",
    "EmergencyInjection",
    "EmergencyProtection",
    "EscrowRecord",
    "Evidence",
    "EvidenceArtifact",
    "EvidenceManifest",
    "FeePolicy",
    "FraudReport",
    "GenesisRequest",
    "GeometryVersion",
    "GuardianClaim",
    "IdentityRights",
    "Jurisdiction",
    "JurisdictionBinding",
    "JurisdictionalRestrictionRequest",
    "NullifierRecord",
    "Observation",
    "ObservationV2",
    "Observer",
    "OwnershipRoot",
    "Parcel",
    "ParcelGeometryArtifact",
    "PrivacyRequest",
    "PrivacyState",
    "QuorumConfig",
    "QuorumTally",
    "QuorumVote",
    "ResourceQuote",
    "ReviewCase",
    "RewardAllocation",
    "Rights",
    "SecurityAppeal",
    "SecurityAuditEvent",
    "SecurityDecision",
    "SecurityRequest",
    "SecurityReviewAssignment",
    "SecurityReviewCase",
    "SlashingReport",
    "SpatialAsset",
    "SpatialSnapshot",
    "SpatialSnapshotMeta",
    "StakePool",
    "SubdivisionRecord",
    "TaskAssignment",
    "TaskEscrow",
    "TaskRequirement",
    "ThreeDModel",
    "ThreeDModelMeta",
    "ThresholdCredential",
    "ValidatorActivityTracker",
    "ValidatorAvailability",
    "ValidatorCapability",
    "ValidatorEndorsement",
    "ValidatorNomination",
    "ValidatorPresence",
    "ValidatorProfile",
    "ValidatorRegistry",
    "ValidatorRelationshipEdge",
    "ValidatorReputation",
    "ValidatorStake",
    "VaultRecord",
    "VaultShardRotation",
    "VerificationAttestation",
    "VerificationSession",
    "VerificationTask",
    "VrfAuthority",
    "WorldRegistry",
    "ZoneSet",
];

/// Account types with a typed projection in `indexer.rs` (stage 0).
/// The rest are mirrored generically into `chain_accounts` (stage 1).
pub const RICH_ACCOUNT_NAMES: &[&str] = &[
    "Parcel",
    "VerificationAttestation",
    "Rights",
    "Claim",
    "SpatialAsset",
    "GeometryVersion",
    "ThreeDModel",
    "SpatialSnapshot",
];

/// All discriminators the indexer scans for. Used by the two-phase RPC read
/// (header-only program scan followed by a batched full-account fetch).
pub const INDEXER_DISCRIMINATORS: &[[u8; 8]] = &[
    AMALGAMATION_RECORD_DISCRIMINATOR,
    APPEAL_DISCRIMINATOR,
    AUDIT_ENTRY_DISCRIMINATOR,
    CAPABILITY_RESTRICTION_DISCRIMINATOR,
    CHALLENGE_DISCRIMINATOR,
    CLAIM_DISCRIMINATOR,
    CLAIM_SESSION_TRACKER_DISCRIMINATOR,
    COVERAGE_INCENTIVE_DISCRIMINATOR,
    CREDENTIAL_NULLIFIER_DISCRIMINATOR,
    CREDENTIAL_REQUEST_DISCRIMINATOR,
    CROSS_BORDER_BINDING_DISCRIMINATOR,
    CROSS_BORDER_SPAN_RECORD_DISCRIMINATOR,
    CROSS_BORDER_VERIFICATION_DISCRIMINATOR,
    DEVICE_IDENTITY_DISCRIMINATOR,
    DISPUTE_DISCRIMINATOR,
    EMERGENCY_INJECTION_DISCRIMINATOR,
    EMERGENCY_PROTECTION_DISCRIMINATOR,
    ESCROW_RECORD_DISCRIMINATOR,
    EVIDENCE_DISCRIMINATOR,
    EVIDENCE_ARTIFACT_DISCRIMINATOR,
    EVIDENCE_MANIFEST_DISCRIMINATOR,
    FEE_POLICY_DISCRIMINATOR,
    FRAUD_REPORT_DISCRIMINATOR,
    GENESIS_REQUEST_DISCRIMINATOR,
    GEOMETRY_VERSION_DISCRIMINATOR,
    GUARDIAN_CLAIM_DISCRIMINATOR,
    IDENTITY_RIGHTS_DISCRIMINATOR,
    JURISDICTION_DISCRIMINATOR,
    JURISDICTION_BINDING_DISCRIMINATOR,
    JURISDICTIONAL_RESTRICTION_REQUEST_DISCRIMINATOR,
    NULLIFIER_RECORD_DISCRIMINATOR,
    OBSERVATION_DISCRIMINATOR,
    OBSERVATION_V2_DISCRIMINATOR,
    OBSERVER_DISCRIMINATOR,
    OWNERSHIP_ROOT_DISCRIMINATOR,
    PARCEL_DISCRIMINATOR,
    PARCEL_GEOMETRY_ARTIFACT_DISCRIMINATOR,
    PRIVACY_REQUEST_DISCRIMINATOR,
    PRIVACY_STATE_DISCRIMINATOR,
    QUORUM_CONFIG_DISCRIMINATOR,
    QUORUM_TALLY_DISCRIMINATOR,
    QUORUM_VOTE_DISCRIMINATOR,
    RESOURCE_QUOTE_DISCRIMINATOR,
    REVIEW_CASE_DISCRIMINATOR,
    REWARD_ALLOCATION_DISCRIMINATOR,
    RIGHTS_DISCRIMINATOR,
    SECURITY_APPEAL_DISCRIMINATOR,
    SECURITY_AUDIT_EVENT_DISCRIMINATOR,
    SECURITY_DECISION_DISCRIMINATOR,
    SECURITY_REQUEST_DISCRIMINATOR,
    SECURITY_REVIEW_ASSIGNMENT_DISCRIMINATOR,
    SECURITY_REVIEW_CASE_DISCRIMINATOR,
    SLASHING_REPORT_DISCRIMINATOR,
    SPATIAL_ASSET_DISCRIMINATOR,
    SPATIAL_SNAPSHOT_DISCRIMINATOR,
    SPATIAL_SNAPSHOT_META_DISCRIMINATOR,
    STAKE_POOL_DISCRIMINATOR,
    SUBDIVISION_RECORD_DISCRIMINATOR,
    TASK_ASSIGNMENT_DISCRIMINATOR,
    TASK_ESCROW_DISCRIMINATOR,
    TASK_REQUIREMENT_DISCRIMINATOR,
    THREE_D_MODEL_DISCRIMINATOR,
    THREE_D_MODEL_META_DISCRIMINATOR,
    THRESHOLD_CREDENTIAL_DISCRIMINATOR,
    VALIDATOR_ACTIVITY_TRACKER_DISCRIMINATOR,
    VALIDATOR_AVAILABILITY_DISCRIMINATOR,
    VALIDATOR_CAPABILITY_DISCRIMINATOR,
    VALIDATOR_ENDORSEMENT_DISCRIMINATOR,
    VALIDATOR_NOMINATION_DISCRIMINATOR,
    VALIDATOR_PRESENCE_DISCRIMINATOR,
    VALIDATOR_PROFILE_DISCRIMINATOR,
    VALIDATOR_REGISTRY_DISCRIMINATOR,
    VALIDATOR_RELATIONSHIP_EDGE_DISCRIMINATOR,
    VALIDATOR_REPUTATION_DISCRIMINATOR,
    VALIDATOR_STAKE_DISCRIMINATOR,
    VAULT_RECORD_DISCRIMINATOR,
    VAULT_SHARD_ROTATION_DISCRIMINATOR,
    VERIFICATION_ATTESTATION_DISCRIMINATOR,
    VERIFICATION_SESSION_DISCRIMINATOR,
    VERIFICATION_TASK_DISCRIMINATOR,
    VRF_AUTHORITY_DISCRIMINATOR,
    WORLD_REGISTRY_DISCRIMINATOR,
    ZONE_SET_DISCRIMINATOR,
];

/// Human-readable name for an account discriminator.
pub fn mirrored_account_name(data: &[u8]) -> Option<&'static str> {
    if discriminator(data, &AMALGAMATION_RECORD_DISCRIMINATOR) {
        return Some("AmalgamationRecord");
    }
    if discriminator(data, &APPEAL_DISCRIMINATOR) {
        return Some("Appeal");
    }
    if discriminator(data, &AUDIT_ENTRY_DISCRIMINATOR) {
        return Some("AuditEntry");
    }
    if discriminator(data, &CAPABILITY_RESTRICTION_DISCRIMINATOR) {
        return Some("CapabilityRestriction");
    }
    if discriminator(data, &CHALLENGE_DISCRIMINATOR) {
        return Some("Challenge");
    }
    if discriminator(data, &CLAIM_DISCRIMINATOR) {
        return Some("Claim");
    }
    if discriminator(data, &CLAIM_SESSION_TRACKER_DISCRIMINATOR) {
        return Some("ClaimSessionTracker");
    }
    if discriminator(data, &COVERAGE_INCENTIVE_DISCRIMINATOR) {
        return Some("CoverageIncentive");
    }
    if discriminator(data, &CREDENTIAL_NULLIFIER_DISCRIMINATOR) {
        return Some("CredentialNullifier");
    }
    if discriminator(data, &CREDENTIAL_REQUEST_DISCRIMINATOR) {
        return Some("CredentialRequest");
    }
    if discriminator(data, &CROSS_BORDER_BINDING_DISCRIMINATOR) {
        return Some("CrossBorderBinding");
    }
    if discriminator(data, &CROSS_BORDER_SPAN_RECORD_DISCRIMINATOR) {
        return Some("CrossBorderSpanRecord");
    }
    if discriminator(data, &CROSS_BORDER_VERIFICATION_DISCRIMINATOR) {
        return Some("CrossBorderVerification");
    }
    if discriminator(data, &DEVICE_IDENTITY_DISCRIMINATOR) {
        return Some("DeviceIdentity");
    }
    if discriminator(data, &DISPUTE_DISCRIMINATOR) {
        return Some("Dispute");
    }
    if discriminator(data, &EMERGENCY_INJECTION_DISCRIMINATOR) {
        return Some("EmergencyInjection");
    }
    if discriminator(data, &EMERGENCY_PROTECTION_DISCRIMINATOR) {
        return Some("EmergencyProtection");
    }
    if discriminator(data, &ESCROW_RECORD_DISCRIMINATOR) {
        return Some("EscrowRecord");
    }
    if discriminator(data, &EVIDENCE_DISCRIMINATOR) {
        return Some("Evidence");
    }
    if discriminator(data, &EVIDENCE_ARTIFACT_DISCRIMINATOR) {
        return Some("EvidenceArtifact");
    }
    if discriminator(data, &EVIDENCE_MANIFEST_DISCRIMINATOR) {
        return Some("EvidenceManifest");
    }
    if discriminator(data, &FEE_POLICY_DISCRIMINATOR) {
        return Some("FeePolicy");
    }
    if discriminator(data, &FRAUD_REPORT_DISCRIMINATOR) {
        return Some("FraudReport");
    }
    if discriminator(data, &GENESIS_REQUEST_DISCRIMINATOR) {
        return Some("GenesisRequest");
    }
    if discriminator(data, &GEOMETRY_VERSION_DISCRIMINATOR) {
        return Some("GeometryVersion");
    }
    if discriminator(data, &GUARDIAN_CLAIM_DISCRIMINATOR) {
        return Some("GuardianClaim");
    }
    if discriminator(data, &IDENTITY_RIGHTS_DISCRIMINATOR) {
        return Some("IdentityRights");
    }
    if discriminator(data, &JURISDICTION_DISCRIMINATOR) {
        return Some("Jurisdiction");
    }
    if discriminator(data, &JURISDICTION_BINDING_DISCRIMINATOR) {
        return Some("JurisdictionBinding");
    }
    if discriminator(data, &JURISDICTIONAL_RESTRICTION_REQUEST_DISCRIMINATOR) {
        return Some("JurisdictionalRestrictionRequest");
    }
    if discriminator(data, &NULLIFIER_RECORD_DISCRIMINATOR) {
        return Some("NullifierRecord");
    }
    if discriminator(data, &OBSERVATION_DISCRIMINATOR) {
        return Some("Observation");
    }
    if discriminator(data, &OBSERVATION_V2_DISCRIMINATOR) {
        return Some("ObservationV2");
    }
    if discriminator(data, &OBSERVER_DISCRIMINATOR) {
        return Some("Observer");
    }
    if discriminator(data, &OWNERSHIP_ROOT_DISCRIMINATOR) {
        return Some("OwnershipRoot");
    }
    if discriminator(data, &PARCEL_DISCRIMINATOR) {
        return Some("Parcel");
    }
    if discriminator(data, &PARCEL_GEOMETRY_ARTIFACT_DISCRIMINATOR) {
        return Some("ParcelGeometryArtifact");
    }
    if discriminator(data, &PRIVACY_REQUEST_DISCRIMINATOR) {
        return Some("PrivacyRequest");
    }
    if discriminator(data, &PRIVACY_STATE_DISCRIMINATOR) {
        return Some("PrivacyState");
    }
    if discriminator(data, &QUORUM_CONFIG_DISCRIMINATOR) {
        return Some("QuorumConfig");
    }
    if discriminator(data, &QUORUM_TALLY_DISCRIMINATOR) {
        return Some("QuorumTally");
    }
    if discriminator(data, &QUORUM_VOTE_DISCRIMINATOR) {
        return Some("QuorumVote");
    }
    if discriminator(data, &RESOURCE_QUOTE_DISCRIMINATOR) {
        return Some("ResourceQuote");
    }
    if discriminator(data, &REVIEW_CASE_DISCRIMINATOR) {
        return Some("ReviewCase");
    }
    if discriminator(data, &REWARD_ALLOCATION_DISCRIMINATOR) {
        return Some("RewardAllocation");
    }
    if discriminator(data, &RIGHTS_DISCRIMINATOR) {
        return Some("Rights");
    }
    if discriminator(data, &SECURITY_APPEAL_DISCRIMINATOR) {
        return Some("SecurityAppeal");
    }
    if discriminator(data, &SECURITY_AUDIT_EVENT_DISCRIMINATOR) {
        return Some("SecurityAuditEvent");
    }
    if discriminator(data, &SECURITY_DECISION_DISCRIMINATOR) {
        return Some("SecurityDecision");
    }
    if discriminator(data, &SECURITY_REQUEST_DISCRIMINATOR) {
        return Some("SecurityRequest");
    }
    if discriminator(data, &SECURITY_REVIEW_ASSIGNMENT_DISCRIMINATOR) {
        return Some("SecurityReviewAssignment");
    }
    if discriminator(data, &SECURITY_REVIEW_CASE_DISCRIMINATOR) {
        return Some("SecurityReviewCase");
    }
    if discriminator(data, &SLASHING_REPORT_DISCRIMINATOR) {
        return Some("SlashingReport");
    }
    if discriminator(data, &SPATIAL_ASSET_DISCRIMINATOR) {
        return Some("SpatialAsset");
    }
    if discriminator(data, &SPATIAL_SNAPSHOT_DISCRIMINATOR) {
        return Some("SpatialSnapshot");
    }
    if discriminator(data, &SPATIAL_SNAPSHOT_META_DISCRIMINATOR) {
        return Some("SpatialSnapshotMeta");
    }
    if discriminator(data, &STAKE_POOL_DISCRIMINATOR) {
        return Some("StakePool");
    }
    if discriminator(data, &SUBDIVISION_RECORD_DISCRIMINATOR) {
        return Some("SubdivisionRecord");
    }
    if discriminator(data, &TASK_ASSIGNMENT_DISCRIMINATOR) {
        return Some("TaskAssignment");
    }
    if discriminator(data, &TASK_ESCROW_DISCRIMINATOR) {
        return Some("TaskEscrow");
    }
    if discriminator(data, &TASK_REQUIREMENT_DISCRIMINATOR) {
        return Some("TaskRequirement");
    }
    if discriminator(data, &THREE_D_MODEL_DISCRIMINATOR) {
        return Some("ThreeDModel");
    }
    if discriminator(data, &THREE_D_MODEL_META_DISCRIMINATOR) {
        return Some("ThreeDModelMeta");
    }
    if discriminator(data, &THRESHOLD_CREDENTIAL_DISCRIMINATOR) {
        return Some("ThresholdCredential");
    }
    if discriminator(data, &VALIDATOR_ACTIVITY_TRACKER_DISCRIMINATOR) {
        return Some("ValidatorActivityTracker");
    }
    if discriminator(data, &VALIDATOR_AVAILABILITY_DISCRIMINATOR) {
        return Some("ValidatorAvailability");
    }
    if discriminator(data, &VALIDATOR_CAPABILITY_DISCRIMINATOR) {
        return Some("ValidatorCapability");
    }
    if discriminator(data, &VALIDATOR_ENDORSEMENT_DISCRIMINATOR) {
        return Some("ValidatorEndorsement");
    }
    if discriminator(data, &VALIDATOR_NOMINATION_DISCRIMINATOR) {
        return Some("ValidatorNomination");
    }
    if discriminator(data, &VALIDATOR_PRESENCE_DISCRIMINATOR) {
        return Some("ValidatorPresence");
    }
    if discriminator(data, &VALIDATOR_PROFILE_DISCRIMINATOR) {
        return Some("ValidatorProfile");
    }
    if discriminator(data, &VALIDATOR_REGISTRY_DISCRIMINATOR) {
        return Some("ValidatorRegistry");
    }
    if discriminator(data, &VALIDATOR_RELATIONSHIP_EDGE_DISCRIMINATOR) {
        return Some("ValidatorRelationshipEdge");
    }
    if discriminator(data, &VALIDATOR_REPUTATION_DISCRIMINATOR) {
        return Some("ValidatorReputation");
    }
    if discriminator(data, &VALIDATOR_STAKE_DISCRIMINATOR) {
        return Some("ValidatorStake");
    }
    if discriminator(data, &VAULT_RECORD_DISCRIMINATOR) {
        return Some("VaultRecord");
    }
    if discriminator(data, &VAULT_SHARD_ROTATION_DISCRIMINATOR) {
        return Some("VaultShardRotation");
    }
    if discriminator(data, &VERIFICATION_ATTESTATION_DISCRIMINATOR) {
        return Some("VerificationAttestation");
    }
    if discriminator(data, &VERIFICATION_SESSION_DISCRIMINATOR) {
        return Some("VerificationSession");
    }
    if discriminator(data, &VERIFICATION_TASK_DISCRIMINATOR) {
        return Some("VerificationTask");
    }
    if discriminator(data, &VRF_AUTHORITY_DISCRIMINATOR) {
        return Some("VrfAuthority");
    }
    if discriminator(data, &WORLD_REGISTRY_DISCRIMINATOR) {
        return Some("WorldRegistry");
    }
    if discriminator(data, &ZONE_SET_DISCRIMINATOR) {
        return Some("ZoneSet");
    }
    None
}

/// Decode any known account into its name and canonical JSON view.
/// The JSON is stored in `chain_accounts.fields` with transaction provenance.
pub fn decode_any(data: &[u8]) -> Option<(&'static str, serde_json::Value)> {
    if discriminator(data, &AMALGAMATION_RECORD_DISCRIMINATOR) {
        return parse_amalgamation_record(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("AmalgamationRecord", json));
    }
    if discriminator(data, &APPEAL_DISCRIMINATOR) {
        return parse_appeal(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Appeal", json));
    }
    if discriminator(data, &AUDIT_ENTRY_DISCRIMINATOR) {
        return parse_audit_entry(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("AuditEntry", json));
    }
    if discriminator(data, &CAPABILITY_RESTRICTION_DISCRIMINATOR) {
        return parse_capability_restriction(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("CapabilityRestriction", json));
    }
    if discriminator(data, &CHALLENGE_DISCRIMINATOR) {
        return parse_challenge(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Challenge", json));
    }
    if discriminator(data, &CLAIM_DISCRIMINATOR) {
        return parse_claim(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Claim", json));
    }
    if discriminator(data, &CLAIM_SESSION_TRACKER_DISCRIMINATOR) {
        return parse_claim_session_tracker(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ClaimSessionTracker", json));
    }
    if discriminator(data, &COVERAGE_INCENTIVE_DISCRIMINATOR) {
        return parse_coverage_incentive(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("CoverageIncentive", json));
    }
    if discriminator(data, &CREDENTIAL_NULLIFIER_DISCRIMINATOR) {
        return parse_credential_nullifier(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("CredentialNullifier", json));
    }
    if discriminator(data, &CREDENTIAL_REQUEST_DISCRIMINATOR) {
        return parse_credential_request(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("CredentialRequest", json));
    }
    if discriminator(data, &CROSS_BORDER_BINDING_DISCRIMINATOR) {
        return parse_cross_border_binding(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("CrossBorderBinding", json));
    }
    if discriminator(data, &CROSS_BORDER_SPAN_RECORD_DISCRIMINATOR) {
        return parse_cross_border_span_record(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("CrossBorderSpanRecord", json));
    }
    if discriminator(data, &CROSS_BORDER_VERIFICATION_DISCRIMINATOR) {
        return parse_cross_border_verification(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("CrossBorderVerification", json));
    }
    if discriminator(data, &DEVICE_IDENTITY_DISCRIMINATOR) {
        return parse_device_identity(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("DeviceIdentity", json));
    }
    if discriminator(data, &DISPUTE_DISCRIMINATOR) {
        return parse_dispute(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Dispute", json));
    }
    if discriminator(data, &EMERGENCY_INJECTION_DISCRIMINATOR) {
        return parse_emergency_injection(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("EmergencyInjection", json));
    }
    if discriminator(data, &EMERGENCY_PROTECTION_DISCRIMINATOR) {
        return parse_emergency_protection(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("EmergencyProtection", json));
    }
    if discriminator(data, &ESCROW_RECORD_DISCRIMINATOR) {
        return parse_escrow_record(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("EscrowRecord", json));
    }
    if discriminator(data, &EVIDENCE_DISCRIMINATOR) {
        return parse_evidence(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Evidence", json));
    }
    if discriminator(data, &EVIDENCE_ARTIFACT_DISCRIMINATOR) {
        return parse_evidence_artifact(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("EvidenceArtifact", json));
    }
    if discriminator(data, &EVIDENCE_MANIFEST_DISCRIMINATOR) {
        return parse_evidence_manifest(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("EvidenceManifest", json));
    }
    if discriminator(data, &FEE_POLICY_DISCRIMINATOR) {
        return parse_fee_policy(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("FeePolicy", json));
    }
    if discriminator(data, &FRAUD_REPORT_DISCRIMINATOR) {
        return parse_fraud_report(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("FraudReport", json));
    }
    if discriminator(data, &GENESIS_REQUEST_DISCRIMINATOR) {
        return parse_genesis_request(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("GenesisRequest", json));
    }
    if discriminator(data, &GEOMETRY_VERSION_DISCRIMINATOR) {
        return parse_geometry_version(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("GeometryVersion", json));
    }
    if discriminator(data, &GUARDIAN_CLAIM_DISCRIMINATOR) {
        return parse_guardian_claim(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("GuardianClaim", json));
    }
    if discriminator(data, &IDENTITY_RIGHTS_DISCRIMINATOR) {
        return parse_identity_rights(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("IdentityRights", json));
    }
    if discriminator(data, &JURISDICTION_DISCRIMINATOR) {
        return parse_jurisdiction(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Jurisdiction", json));
    }
    if discriminator(data, &JURISDICTION_BINDING_DISCRIMINATOR) {
        return parse_jurisdiction_binding(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("JurisdictionBinding", json));
    }
    if discriminator(data, &JURISDICTIONAL_RESTRICTION_REQUEST_DISCRIMINATOR) {
        return parse_jurisdictional_restriction_request(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("JurisdictionalRestrictionRequest", json));
    }
    if discriminator(data, &NULLIFIER_RECORD_DISCRIMINATOR) {
        return parse_nullifier_record(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("NullifierRecord", json));
    }
    if discriminator(data, &OBSERVATION_DISCRIMINATOR) {
        return parse_observation(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Observation", json));
    }
    if discriminator(data, &OBSERVATION_V2_DISCRIMINATOR) {
        return parse_observation_v2(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ObservationV2", json));
    }
    if discriminator(data, &OBSERVER_DISCRIMINATOR) {
        return parse_observer(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Observer", json));
    }
    if discriminator(data, &OWNERSHIP_ROOT_DISCRIMINATOR) {
        return parse_ownership_root(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("OwnershipRoot", json));
    }
    if discriminator(data, &PARCEL_DISCRIMINATOR) {
        return parse_parcel(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Parcel", json));
    }
    if discriminator(data, &PARCEL_GEOMETRY_ARTIFACT_DISCRIMINATOR) {
        return parse_parcel_geometry_artifact(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ParcelGeometryArtifact", json));
    }
    if discriminator(data, &PRIVACY_REQUEST_DISCRIMINATOR) {
        return parse_privacy_request(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("PrivacyRequest", json));
    }
    if discriminator(data, &PRIVACY_STATE_DISCRIMINATOR) {
        return parse_privacy_state(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("PrivacyState", json));
    }
    if discriminator(data, &QUORUM_CONFIG_DISCRIMINATOR) {
        return parse_quorum_config(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("QuorumConfig", json));
    }
    if discriminator(data, &QUORUM_TALLY_DISCRIMINATOR) {
        return parse_quorum_tally(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("QuorumTally", json));
    }
    if discriminator(data, &QUORUM_VOTE_DISCRIMINATOR) {
        return parse_quorum_vote(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("QuorumVote", json));
    }
    if discriminator(data, &RESOURCE_QUOTE_DISCRIMINATOR) {
        return parse_resource_quote(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ResourceQuote", json));
    }
    if discriminator(data, &REVIEW_CASE_DISCRIMINATOR) {
        return parse_review_case(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ReviewCase", json));
    }
    if discriminator(data, &REWARD_ALLOCATION_DISCRIMINATOR) {
        return parse_reward_allocation(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("RewardAllocation", json));
    }
    if discriminator(data, &RIGHTS_DISCRIMINATOR) {
        return parse_rights(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("Rights", json));
    }
    if discriminator(data, &SECURITY_APPEAL_DISCRIMINATOR) {
        return parse_security_appeal(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SecurityAppeal", json));
    }
    if discriminator(data, &SECURITY_AUDIT_EVENT_DISCRIMINATOR) {
        return parse_security_audit_event(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SecurityAuditEvent", json));
    }
    if discriminator(data, &SECURITY_DECISION_DISCRIMINATOR) {
        return parse_security_decision(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SecurityDecision", json));
    }
    if discriminator(data, &SECURITY_REQUEST_DISCRIMINATOR) {
        return parse_security_request(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SecurityRequest", json));
    }
    if discriminator(data, &SECURITY_REVIEW_ASSIGNMENT_DISCRIMINATOR) {
        return parse_security_review_assignment(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SecurityReviewAssignment", json));
    }
    if discriminator(data, &SECURITY_REVIEW_CASE_DISCRIMINATOR) {
        return parse_security_review_case(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SecurityReviewCase", json));
    }
    if discriminator(data, &SLASHING_REPORT_DISCRIMINATOR) {
        return parse_slashing_report(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SlashingReport", json));
    }
    if discriminator(data, &SPATIAL_ASSET_DISCRIMINATOR) {
        return parse_spatial_asset(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SpatialAsset", json));
    }
    if discriminator(data, &SPATIAL_SNAPSHOT_DISCRIMINATOR) {
        return parse_spatial_snapshot(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SpatialSnapshot", json));
    }
    if discriminator(data, &SPATIAL_SNAPSHOT_META_DISCRIMINATOR) {
        return parse_spatial_snapshot_meta(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SpatialSnapshotMeta", json));
    }
    if discriminator(data, &STAKE_POOL_DISCRIMINATOR) {
        return parse_stake_pool(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("StakePool", json));
    }
    if discriminator(data, &SUBDIVISION_RECORD_DISCRIMINATOR) {
        return parse_subdivision_record(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("SubdivisionRecord", json));
    }
    if discriminator(data, &TASK_ASSIGNMENT_DISCRIMINATOR) {
        return parse_task_assignment(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("TaskAssignment", json));
    }
    if discriminator(data, &TASK_ESCROW_DISCRIMINATOR) {
        return parse_task_escrow(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("TaskEscrow", json));
    }
    if discriminator(data, &TASK_REQUIREMENT_DISCRIMINATOR) {
        return parse_task_requirement(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("TaskRequirement", json));
    }
    if discriminator(data, &THREE_D_MODEL_DISCRIMINATOR) {
        return parse_three_d_model(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ThreeDModel", json));
    }
    if discriminator(data, &THREE_D_MODEL_META_DISCRIMINATOR) {
        return parse_three_d_model_meta(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ThreeDModelMeta", json));
    }
    if discriminator(data, &THRESHOLD_CREDENTIAL_DISCRIMINATOR) {
        return parse_threshold_credential(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ThresholdCredential", json));
    }
    if discriminator(data, &VALIDATOR_ACTIVITY_TRACKER_DISCRIMINATOR) {
        return parse_validator_activity_tracker(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorActivityTracker", json));
    }
    if discriminator(data, &VALIDATOR_AVAILABILITY_DISCRIMINATOR) {
        return parse_validator_availability(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorAvailability", json));
    }
    if discriminator(data, &VALIDATOR_CAPABILITY_DISCRIMINATOR) {
        return parse_validator_capability(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorCapability", json));
    }
    if discriminator(data, &VALIDATOR_ENDORSEMENT_DISCRIMINATOR) {
        return parse_validator_endorsement(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorEndorsement", json));
    }
    if discriminator(data, &VALIDATOR_NOMINATION_DISCRIMINATOR) {
        return parse_validator_nomination(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorNomination", json));
    }
    if discriminator(data, &VALIDATOR_PRESENCE_DISCRIMINATOR) {
        return parse_validator_presence(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorPresence", json));
    }
    if discriminator(data, &VALIDATOR_PROFILE_DISCRIMINATOR) {
        return parse_validator_profile(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorProfile", json));
    }
    if discriminator(data, &VALIDATOR_REGISTRY_DISCRIMINATOR) {
        return parse_validator_registry(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorRegistry", json));
    }
    if discriminator(data, &VALIDATOR_RELATIONSHIP_EDGE_DISCRIMINATOR) {
        return parse_validator_relationship_edge(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorRelationshipEdge", json));
    }
    if discriminator(data, &VALIDATOR_REPUTATION_DISCRIMINATOR) {
        return parse_validator_reputation(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorReputation", json));
    }
    if discriminator(data, &VALIDATOR_STAKE_DISCRIMINATOR) {
        return parse_validator_stake(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ValidatorStake", json));
    }
    if discriminator(data, &VAULT_RECORD_DISCRIMINATOR) {
        return parse_vault_record(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("VaultRecord", json));
    }
    if discriminator(data, &VAULT_SHARD_ROTATION_DISCRIMINATOR) {
        return parse_vault_shard_rotation(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("VaultShardRotation", json));
    }
    if discriminator(data, &VERIFICATION_ATTESTATION_DISCRIMINATOR) {
        return parse_verification_attestation(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("VerificationAttestation", json));
    }
    if discriminator(data, &VERIFICATION_SESSION_DISCRIMINATOR) {
        return parse_verification_session(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("VerificationSession", json));
    }
    if discriminator(data, &VERIFICATION_TASK_DISCRIMINATOR) {
        return parse_verification_task(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("VerificationTask", json));
    }
    if discriminator(data, &VRF_AUTHORITY_DISCRIMINATOR) {
        return parse_vrf_authority(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("VrfAuthority", json));
    }
    if discriminator(data, &WORLD_REGISTRY_DISCRIMINATOR) {
        return parse_world_registry(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("WorldRegistry", json));
    }
    if discriminator(data, &ZONE_SET_DISCRIMINATOR) {
        return parse_zone_set(data)
            .and_then(|value| serde_json::to_value(value).ok())
            .map(|json| ("ZoneSet", json));
    }
    None
}

#[cfg(test)]
/// Per-type acceptance tests (Q-035/Q-033): each IDL account has a fixture
/// built from its own field spec. A type is only accepted if its decoder,
/// `decode_any` classification and JSON projection all agree.
mod idl_acceptance_tests {
    use super::*;

    fn byte_list(values: &[u8]) -> String {
        values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
    #[test]
    fn every_idl_account_has_a_unique_discriminator() {
        assert_eq!(ALL_ACCOUNT_NAMES.len(), 83);
        let mut seen = std::collections::HashSet::new();
        for name in ALL_ACCOUNT_NAMES {
            assert!(seen.insert(*name), "duplicate account name {name}");
        }
        let mut discs = std::collections::HashSet::new();
        for disc in INDEXER_DISCRIMINATORS {
            assert!(discs.insert(*disc), "duplicate discriminator {disc:?}");
        }
    }

    #[test]
    fn accepts_amalgamation_record() {
        let mut data = vec![4, 242, 238, 37, 228, 163, 250, 90];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0,
            0, 0, 0, 7,
        ]);
        let parsed =
            parse_amalgamation_record(&data).expect("AmalgamationRecord IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify AmalgamationRecord");
        assert_eq!(decoded_name, "AmalgamationRecord");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 9);
        for field in [
            "result_parcel",
            "source_parcel",
            "source_geometry_hash",
            "result_geometry_hash",
            "rights_merged",
            "initiated_by",
            "created_at",
            "completed_at",
            "status",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_appeal() {
        let mut data = vec![155, 196, 80, 143, 64, 220, 198, 177];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0,
            0, 0, 0, 0, 0,
        ]);
        let parsed = parse_appeal(&data).expect("Appeal IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify Appeal");
        assert_eq!(decoded_name, "Appeal");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "restriction",
            "appellant",
            "nonce",
            "evidence_hash",
            "status",
            "note",
            "created_at",
            "decided_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_audit_entry() {
        let mut data = vec![254, 88, 234, 107, 205, 16, 148, 113];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_audit_entry(&data).expect("AuditEntry IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify AuditEntry");
        assert_eq!(decoded_name, "AuditEntry");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "entity",
            "sequence",
            "action",
            "from_status",
            "to_status",
            "actor",
            "metadata_hash",
            "timestamp",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_capability_restriction() {
        let mut data = vec![74, 254, 245, 177, 59, 188, 186, 56];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_capability_restriction(&data)
            .expect("CapabilityRestriction IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify CapabilityRestriction");
        assert_eq!(decoded_name, "CapabilityRestriction");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 10);
        for field in [
            "wallet",
            "capability_code",
            "report",
            "max_level",
            "status",
            "applied_at",
            "appeal_opens_at",
            "rehab_eligible_at",
            "lifted_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_challenge() {
        let mut data = vec![119, 250, 161, 121, 119, 81, 22, 208];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7,
            0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_challenge(&data).expect("Challenge IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify Challenge");
        assert_eq!(decoded_name, "Challenge");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 11);
        for field in [
            "claim",
            "challenger",
            "challenge_hash",
            "status",
            "uphold_votes",
            "overturn_votes",
            "required_votes",
            "voters",
            "created_at",
            "review_deadline",
            "resolved_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_claim() {
        let mut data = vec![155, 70, 22, 176, 123, 215, 246, 102];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0,
            0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_claim(&data).expect("Claim IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify Claim");
        assert_eq!(decoded_name, "Claim");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 13);
        for field in [
            "claim_id",
            "parcel",
            "claim_type",
            "submitted_by",
            "status",
            "statement_hash",
            "version",
            "evidence_count",
            "observation_count",
            "attestation_count",
            "required_attestations",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_claim_session_tracker() {
        let mut data = vec![210, 191, 84, 140, 190, 184, 31, 247];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7,
        ]);
        let parsed = parse_claim_session_tracker(&data)
            .expect("ClaimSessionTracker IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ClaimSessionTracker");
        assert_eq!(decoded_name, "ClaimSessionTracker");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 3);
        for field in ["claim", "active_session", "bump"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_coverage_incentive() {
        let mut data = vec![122, 104, 101, 154, 189, 178, 207, 110];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 7, 0, 7, 0, 7, 0, 7, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_coverage_incentive(&data).expect("CoverageIncentive IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify CoverageIncentive");
        assert_eq!(decoded_name, "CoverageIncentive");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 7);
        for field in [
            "authority",
            "demand_bps",
            "deficit_bps",
            "difficulty_bps",
            "strategic_bps",
            "max_subsidy_bps",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_credential_nullifier() {
        let mut data = vec![242, 184, 146, 110, 112, 48, 79, 197];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_credential_nullifier(&data).expect("CredentialNullifier IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify CredentialNullifier");
        assert_eq!(decoded_name, "CredentialNullifier");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 3);
        for field in ["nullifier_hash", "credential", "consumed_at"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_credential_request() {
        let mut data = vec![13, 77, 28, 51, 251, 145, 231, 29];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 170,
            1, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_credential_request(&data).expect("CredentialRequest IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify CredentialRequest");
        assert_eq!(decoded_name, "CredentialRequest");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 11);
        for field in [
            "request_hash",
            "prover",
            "purpose",
            "disclosure_type",
            "region_registry",
            "signers",
            "signatures",
            "finalized",
            "credential",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_cross_border_binding() {
        let mut data = vec![90, 184, 2, 152, 183, 229, 63, 32];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0,
            0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_cross_border_binding(&data).expect("CrossBorderBinding IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify CrossBorderBinding");
        assert_eq!(decoded_name, "CrossBorderBinding");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "country_min",
            "country_max",
            "jurisdiction_min",
            "jurisdiction_max",
            "status",
            "expires_at",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_cross_border_span_record() {
        let mut data = vec![224, 66, 12, 25, 121, 238, 123, 191];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_cross_border_span_record(&data)
            .expect("CrossBorderSpanRecord IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify CrossBorderSpanRecord");
        assert_eq!(decoded_name, "CrossBorderSpanRecord");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 7);
        for field in [
            "task_id",
            "validator",
            "req_index",
            "country_home",
            "country_required",
            "binding",
            "recorded_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_cross_border_verification() {
        let mut data = vec![183, 6, 175, 22, 50, 174, 166, 63];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        let parsed = parse_cross_border_verification(&data)
            .expect("CrossBorderVerification IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify CrossBorderVerification");
        assert_eq!(decoded_name, "CrossBorderVerification");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "binding",
            "claim",
            "session",
            "jurisdiction",
            "status",
            "verified_by",
            "created_at",
            "verified_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_device_identity() {
        let mut data = vec![6, 98, 9, 166, 229, 128, 194, 58];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0,
            0, 0, 0, 0, 0, 7, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0,
            0,
        ]);
        let parsed = parse_device_identity(&data).expect("DeviceIdentity IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify DeviceIdentity");
        assert_eq!(decoded_name, "DeviceIdentity");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 14);
        for field in [
            "owner",
            "device_nonce",
            "device_key",
            "source",
            "capabilities",
            "metadata_ref",
            "calibration_hash",
            "calibrated_at",
            "status",
            "verified",
            "verified_by",
            "verify_version",
            "registered_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_dispute() {
        let mut data = vec![36, 49, 241, 67, 40, 36, 241, 74];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        ]);
        let parsed = parse_dispute(&data).expect("Dispute IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify Dispute");
        assert_eq!(decoded_name, "Dispute");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 18);
        for field in [
            "parcel",
            "filed_by",
            "case_hash",
            "status",
            "required",
            "declared_count",
            "validators",
            "present_validators",
            "present_count",
            "filed_at",
            "frozen_at",
            "adjudicated_at",
            "outcome",
            "new_owner",
            "bond_lamports",
            "bond_refund_to",
            "bond_resolved",
            "remedy_geometry_version",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_emergency_injection() {
        let mut data = vec![101, 247, 85, 71, 70, 156, 253, 173];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        let parsed =
            parse_emergency_injection(&data).expect("EmergencyInjection IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify EmergencyInjection");
        assert_eq!(decoded_name, "EmergencyInjection");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 5);
        for field in [
            "registry",
            "requested_by",
            "candidate",
            "execute_after",
            "executed",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_emergency_protection() {
        let mut data = vec![172, 134, 210, 181, 140, 209, 219, 43];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        let parsed =
            parse_emergency_protection(&data).expect("EmergencyProtection IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify EmergencyProtection");
        assert_eq!(decoded_name, "EmergencyProtection");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 6);
        for field in [
            "subject",
            "nonce",
            "requester",
            "expires_at",
            "created_at",
            "lifted",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_escrow_record() {
        let mut data = vec![174, 124, 167, 70, 91, 85, 212, 227];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0,
            0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        ]);
        let parsed = parse_escrow_record(&data).expect("EscrowRecord IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify EscrowRecord");
        assert_eq!(decoded_name, "EscrowRecord");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 13);
        for field in [
            "parcel",
            "seller",
            "buyer",
            "amount",
            "deposit_amount",
            "vault",
            "status",
            "created_at",
            "deposited_at",
            "accepted_at",
            "settle_deadline",
            "cancel_deadline",
            "dispute_case_hash",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_evidence() {
        let mut data = vec![160, 73, 93, 206, 99, 242, 62, 92];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        let parsed = parse_evidence(&data).expect("Evidence IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify Evidence");
        assert_eq!(decoded_name, "Evidence");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "claim",
            "nonce",
            "submitted_by",
            "evidence_type",
            "content_hash",
            "storage_reference",
            "created_at",
            "observed_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_evidence_artifact() {
        let mut data = vec![18, 40, 202, 16, 128, 245, 57, 135];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_evidence_artifact(&data).expect("EvidenceArtifact IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify EvidenceArtifact");
        assert_eq!(decoded_name, "EvidenceArtifact");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "manifest",
            "artifact_index",
            "kind",
            "source",
            "provenance",
            "content_hash",
            "storage_reference",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_evidence_manifest() {
        let mut data = vec![9, 190, 128, 86, 64, 17, 31, 5];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_evidence_manifest(&data).expect("EvidenceManifest IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify EvidenceManifest");
        assert_eq!(decoded_name, "EvidenceManifest");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 7);
        for field in [
            "task_id",
            "submitter",
            "nonce",
            "observation",
            "artifact_count",
            "root_hash",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_fee_policy() {
        let mut data = vec![98, 183, 86, 160, 232, 222, 82, 45];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_fee_policy(&data).expect("FeePolicy IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify FeePolicy");
        assert_eq!(decoded_name, "FeePolicy");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 3);
        for field in ["authority", "fee_bps", "updated_at"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_fraud_report() {
        let mut data = vec![136, 69, 45, 181, 210, 24, 121, 177];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7,
            0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_fraud_report(&data).expect("FraudReport IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify FraudReport");
        assert_eq!(decoded_name, "FraudReport");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 10);
        for field in [
            "accused",
            "reporter",
            "nonce",
            "evidence_hash",
            "reason_code",
            "note",
            "status",
            "capability_code",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_genesis_request() {
        let mut data = vec![37, 134, 94, 228, 116, 67, 108, 235];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_genesis_request(&data).expect("GenesisRequest IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify GenesisRequest");
        assert_eq!(decoded_name, "GenesisRequest");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 5);
        for field in [
            "country_code",
            "requested_by",
            "confirmations",
            "finalized",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_geometry_version() {
        let mut data = vec![78, 143, 35, 194, 97, 213, 195, 216];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0,
            0, 0, 0, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0,
            0, 0, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7,
        ]);
        let parsed =
            parse_geometry_version(&data).expect("GeometryVersion IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify GeometryVersion");
        assert_eq!(decoded_name, "GeometryVersion");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 19);
        for field in [
            "asset",
            "parcel",
            "version",
            "geometry_hash",
            "source",
            "dimension",
            "storage_reference",
            "submitted_by",
            "submitted_at",
            "verified",
            "verified_by",
            "verified_at",
            "evidence_manifest",
            "elevation_min_mm",
            "elevation_max_mm",
            "elevation_source",
            "required",
            "attestors",
            "attest_count",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_guardian_claim() {
        let mut data = vec![75, 164, 183, 31, 178, 231, 33, 133];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0,
            0,
        ]);
        let parsed = parse_guardian_claim(&data).expect("GuardianClaim IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify GuardianClaim");
        assert_eq!(decoded_name, "GuardianClaim");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "claim",
            "identity",
            "triggered_by",
            "case_hash",
            "guardian_type",
            "status",
            "created_at",
            "resolved_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_identity_rights() {
        let mut data = vec![80, 133, 246, 210, 78, 97, 163, 46];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 1, 0,
            0, 0, 120,
        ]);
        let parsed = parse_identity_rights(&data).expect("IdentityRights IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify IdentityRights");
        assert_eq!(decoded_name, "IdentityRights");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "identity",
            "parcel",
            "rights_kind",
            "granter",
            "created_at",
            "expires_at",
            "status",
            "notes",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_jurisdiction() {
        let mut data = vec![151, 118, 212, 152, 149, 211, 87, 2];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 1, 0, 0, 0,
            120, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_jurisdiction(&data).expect("Jurisdiction IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify Jurisdiction");
        assert_eq!(decoded_name, "Jurisdiction");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 10);
        for field in [
            "country_code",
            "authority",
            "jurisdiction_name",
            "credential_schema_cid",
            "revocation_registry",
            "verification_key_hash",
            "algorithm_id",
            "status",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_jurisdiction_binding() {
        let mut data = vec![35, 9, 193, 178, 225, 246, 46, 21];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 170, 7, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0,
        ]);
        let parsed =
            parse_jurisdiction_binding(&data).expect("JurisdictionBinding IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify JurisdictionBinding");
        assert_eq!(decoded_name, "JurisdictionBinding");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 15);
        for field in [
            "identity_hash",
            "jurisdiction_key",
            "credential_commitment",
            "nullifier",
            "proof_data",
            "proof_version",
            "algorithm_id",
            "revoked",
            "revoked_at",
            "revoked_by",
            "bound_at",
            "expires_at",
            "verified",
            "verified_by",
            "version",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_jurisdictional_restriction_request() {
        let mut data = vec![41, 159, 224, 142, 146, 123, 90, 20];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0, 7,
            0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_jurisdictional_restriction_request(&data)
            .expect("JurisdictionalRestrictionRequest IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify JurisdictionalRestrictionRequest");
        assert_eq!(decoded_name, "JurisdictionalRestrictionRequest");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 11);
        for field in [
            "authority",
            "subject",
            "nonce",
            "restriction_type",
            "description",
            "requested_remedy",
            "jurisdiction",
            "status",
            "expires_at",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_nullifier_record() {
        let mut data = vec![56, 18, 57, 175, 69, 202, 189, 70];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7,
        ]);
        let parsed =
            parse_nullifier_record(&data).expect("NullifierRecord IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify NullifierRecord");
        assert_eq!(decoded_name, "NullifierRecord");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "nullifier_hash",
            "zone_set",
            "root_version",
            "prover",
            "proof_purpose",
            "disclosure_type",
            "block_time",
            "proof_hash",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_observation() {
        let mut data = vec![109, 190, 190, 95, 28, 172, 243, 74];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0,
            0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_observation(&data).expect("Observation IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify Observation");
        assert_eq!(decoded_name, "Observation");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 9);
        for field in [
            "claim",
            "validator",
            "observed_at",
            "location",
            "method",
            "findings_hash",
            "confidence",
            "signature_hash",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_observation_v2() {
        let mut data = vec![194, 12, 252, 52, 247, 150, 119, 195];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0,
            0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_observation_v2(&data).expect("ObservationV2 IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ObservationV2");
        assert_eq!(decoded_name, "ObservationV2");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 14);
        for field in [
            "task_id",
            "observer",
            "nonce",
            "subject",
            "capture_device",
            "source",
            "provenance",
            "location",
            "observed_at",
            "findings_hash",
            "evidence_hash",
            "confidence",
            "signature_hash",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_observer() {
        let mut data = vec![82, 255, 234, 217, 166, 201, 80, 72];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0,
            0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_observer(&data).expect("Observer IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify Observer");
        assert_eq!(decoded_name, "Observer");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 7);
        for field in [
            "wallet",
            "identity",
            "status",
            "total_observations",
            "confirmed_observations",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_ownership_root() {
        let mut data = vec![130, 74, 117, 24, 60, 232, 70, 225];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 7, 0, 0, 0, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_ownership_root(&data).expect("OwnershipRoot IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify OwnershipRoot");
        assert_eq!(decoded_name, "OwnershipRoot");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 11);
        for field in [
            "zone_set",
            "merkle_root",
            "version",
            "commitment_count",
            "algorithm_id",
            "snapshot_cid",
            "snapshot_hash",
            "authority_signature",
            "verification_key_hash",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_parcel() {
        let mut data = vec![149, 167, 245, 67, 209, 244, 214, 75];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0,
            0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_parcel(&data).expect("Parcel IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify Parcel");
        assert_eq!(decoded_name, "Parcel");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 9);
        for field in [
            "id",
            "name",
            "geometry_hash",
            "status",
            "rights_count",
            "infrastructure_flags",
            "access_hash",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_parcel_geometry_artifact() {
        let mut data = vec![8, 105, 58, 85, 110, 205, 206, 167];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_parcel_geometry_artifact(&data)
            .expect("ParcelGeometryArtifact IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ParcelGeometryArtifact");
        assert_eq!(decoded_name, "ParcelGeometryArtifact");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 5);
        for field in [
            "parcel",
            "content_hash",
            "storage_uri",
            "registrant",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_privacy_request() {
        let mut data = vec![108, 60, 110, 140, 63, 148, 161, 86];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0, 7,
            0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_privacy_request(&data).expect("PrivacyRequest IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify PrivacyRequest");
        assert_eq!(decoded_name, "PrivacyRequest");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 12);
        for field in [
            "requester",
            "subject",
            "nonce",
            "request_type",
            "description",
            "requested_remedy",
            "jurisdiction",
            "status",
            "emergency",
            "expires_at",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_privacy_state() {
        let mut data = vec![57, 131, 134, 54, 197, 203, 111, 138];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 7, 7, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7,
            0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_privacy_state(&data).expect("PrivacyState IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify PrivacyState");
        assert_eq!(decoded_name, "PrivacyState");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "subject",
            "version",
            "state",
            "remedy",
            "restriction_request",
            "expires_at",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_quorum_config() {
        let mut data = vec![20, 13, 136, 87, 137, 5, 77, 126];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_quorum_config(&data).expect("QuorumConfig IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify QuorumConfig");
        assert_eq!(decoded_name, "QuorumConfig");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 7);
        for field in [
            "parcel_type",
            "region",
            "required_attestations",
            "required_confidence",
            "set_by",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_quorum_tally() {
        let mut data = vec![224, 79, 255, 67, 22, 107, 98, 231];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 7, 0, 7, 0, 7, 0, 7, 7, 0, 1, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0,
            0, 0,
        ]);
        let parsed = parse_quorum_tally(&data).expect("QuorumTally IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify QuorumTally");
        assert_eq!(decoded_name, "QuorumTally");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 10);
        for field in [
            "claim",
            "total_weight",
            "confirm_weight",
            "dispute_weight",
            "abstain_weight",
            "total_votes",
            "quorum_threshold",
            "resolved",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_quorum_vote() {
        let mut data = vec![6, 0, 156, 42, 100, 253, 158, 169];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_quorum_vote(&data).expect("QuorumVote IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify QuorumVote");
        assert_eq!(decoded_name, "QuorumVote");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 6);
        for field in [
            "claim",
            "voter",
            "vote",
            "weight",
            "stake_amount",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_resource_quote() {
        let mut data = vec![12, 173, 242, 254, 102, 42, 31, 5];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_resource_quote(&data).expect("ResourceQuote IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ResourceQuote");
        assert_eq!(decoded_name, "ResourceQuote");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 5);
        for field in ["task_id", "task", "quoter", "resource_cost", "created_at"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_review_case() {
        let mut data = vec![91, 9, 100, 255, 116, 51, 53, 93];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0,
            0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_review_case(&data).expect("ReviewCase IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify ReviewCase");
        assert_eq!(decoded_name, "ReviewCase");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 10);
        for field in [
            "report",
            "accused",
            "committee",
            "voted",
            "votes_cast",
            "votes_upheld",
            "votes_dismissed",
            "decision",
            "opened_at",
            "decided_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_reward_allocation() {
        let mut data = vec![126, 230, 173, 145, 43, 165, 30, 112];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_reward_allocation(&data).expect("RewardAllocation IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify RewardAllocation");
        assert_eq!(decoded_name, "RewardAllocation");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 7);
        for field in [
            "task_id",
            "task",
            "validator",
            "base_paid",
            "subsidy_paid",
            "total_paid",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_rights() {
        let mut data = vec![79, 187, 86, 85, 243, 213, 103, 234];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
            0, 120, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_rights(&data).expect("Rights IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify Rights");
        assert_eq!(decoded_name, "Rights");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 9);
        for field in [
            "parcel",
            "rights_kind",
            "holder",
            "granter",
            "created_at",
            "expires_at",
            "notes",
            "status",
            "grace_period_secs",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_security_appeal() {
        let mut data = vec![125, 1, 36, 0, 78, 64, 143, 200];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 1, 0, 0, 0, 120, 7, 7, 0, 0, 0, 0, 0, 0, 0, 1, 7, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        let parsed = parse_security_appeal(&data).expect("SecurityAppeal IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SecurityAppeal");
        assert_eq!(decoded_name, "SecurityAppeal");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 7);
        for field in [
            "decision",
            "appellant",
            "nonce",
            "reason",
            "status",
            "created_at",
            "resolved_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_security_audit_event() {
        let mut data = vec![20, 158, 241, 227, 102, 108, 82, 110];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 120,
        ]);
        let parsed =
            parse_security_audit_event(&data).expect("SecurityAuditEvent IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SecurityAuditEvent");
        assert_eq!(decoded_name, "SecurityAuditEvent");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 6);
        for field in [
            "entity",
            "sequence",
            "action",
            "actor",
            "timestamp",
            "reason",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_security_decision() {
        let mut data = vec![205, 222, 130, 33, 186, 126, 124, 228];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7,
        ]);
        let parsed =
            parse_security_decision(&data).expect("SecurityDecision IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SecurityDecision");
        assert_eq!(decoded_name, "SecurityDecision");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "request",
            "case",
            "remedy",
            "state",
            "restriction_scope",
            "expires_at",
            "decided_at",
            "decided_by",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_security_request() {
        let mut data = vec![85, 161, 107, 9, 224, 59, 84, 186];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0, 7,
            0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_security_request(&data).expect("SecurityRequest IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SecurityRequest");
        assert_eq!(decoded_name, "SecurityRequest");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 12);
        for field in [
            "requester",
            "subject",
            "nonce",
            "request_type",
            "description",
            "requested_remedy",
            "jurisdiction",
            "status",
            "emergency",
            "expires_at",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_security_review_assignment() {
        let mut data = vec![179, 187, 202, 26, 177, 130, 71, 17];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 1, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_security_review_assignment(&data)
            .expect("SecurityReviewAssignment IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SecurityReviewAssignment");
        assert_eq!(decoded_name, "SecurityReviewAssignment");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 6);
        for field in [
            "case",
            "reviewer",
            "voted",
            "vote",
            "voted_at",
            "assigned_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_security_review_case() {
        let mut data = vec![65, 3, 209, 46, 164, 238, 0, 179];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0,
            0,
        ]);
        let parsed =
            parse_security_review_case(&data).expect("SecurityReviewCase IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SecurityReviewCase");
        assert_eq!(decoded_name, "SecurityReviewCase");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 9);
        for field in [
            "request",
            "requester",
            "subject",
            "committee_size",
            "votes_upheld",
            "votes_dismissed",
            "decision",
            "finalized_at",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_slashing_report() {
        let mut data = vec![73, 176, 94, 36, 3, 159, 87, 218];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7,
            0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_slashing_report(&data).expect("SlashingReport IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SlashingReport");
        assert_eq!(decoded_name, "SlashingReport");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 11);
        for field in [
            "stake_pool",
            "reporter",
            "evidence_hash",
            "offender",
            "offense_type",
            "offense_details",
            "reporter_bond",
            "status",
            "filed_at",
            "appeal_deadline",
            "resolved_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_spatial_asset() {
        let mut data = vec![143, 133, 148, 4, 118, 50, 10, 170];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_spatial_asset(&data).expect("SpatialAsset IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SpatialAsset");
        assert_eq!(decoded_name, "SpatialAsset");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 9);
        for field in [
            "parcel",
            "authority",
            "dimensionality",
            "elevation_min_mm",
            "elevation_max_mm",
            "geometry_version_count",
            "latest_geometry",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_spatial_snapshot() {
        let mut data = vec![156, 60, 119, 78, 61, 63, 161, 245];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 7, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_spatial_snapshot(&data).expect("SpatialSnapshot IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SpatialSnapshot");
        assert_eq!(decoded_name, "SpatialSnapshot");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 11);
        for field in [
            "asset",
            "parcel",
            "snapshot_index",
            "geometry_version_count",
            "latest_geometry",
            "dimensionality",
            "elevation_min_mm",
            "elevation_max_mm",
            "label",
            "created_by",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_spatial_snapshot_meta() {
        let mut data = vec![123, 54, 165, 171, 172, 35, 63, 23];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 0, 0,
        ]);
        let parsed = parse_spatial_snapshot_meta(&data)
            .expect("SpatialSnapshotMeta IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SpatialSnapshotMeta");
        assert_eq!(decoded_name, "SpatialSnapshotMeta");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 2);
        for field in ["asset", "count"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_stake_pool() {
        let mut data = vec![121, 34, 206, 21, 79, 127, 255, 28];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
            7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_stake_pool(&data).expect("StakePool IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify StakePool");
        assert_eq!(decoded_name, "StakePool");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 9);
        for field in [
            "region_registry",
            "total_staked",
            "reward_rate_bps",
            "accumulated_rewards",
            "reward_per_token_stored",
            "last_reward_distribution",
            "slash_count",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_subdivision_record() {
        let mut data = vec![9, 214, 108, 50, 166, 44, 134, 32];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0,
            0, 0, 0, 0, 0, 0, 7,
        ]);
        let parsed =
            parse_subdivision_record(&data).expect("SubdivisionRecord IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify SubdivisionRecord");
        assert_eq!(decoded_name, "SubdivisionRecord");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 10);
        for field in [
            "original_parcel",
            "sub_parcel",
            "original_geometry_hash",
            "new_geometry_hash",
            "survey_claim",
            "rights_migrated",
            "initiated_by",
            "created_at",
            "completed_at",
            "status",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_task_assignment() {
        let mut data = vec![89, 239, 95, 144, 182, 35, 47, 254];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
            7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_task_assignment(&data).expect("TaskAssignment IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify TaskAssignment");
        assert_eq!(decoded_name, "TaskAssignment");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "task_id",
            "validator",
            "requester",
            "assigned_by",
            "status",
            "assigned_at",
            "submitted_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_task_escrow() {
        let mut data = vec![209, 72, 197, 54, 17, 55, 3, 187];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7,
            0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        let parsed = parse_task_escrow(&data).expect("TaskEscrow IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify TaskEscrow");
        assert_eq!(decoded_name, "TaskEscrow");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 12);
        for field in [
            "task_id",
            "task",
            "requester",
            "vault",
            "amount",
            "fee_paid",
            "released",
            "released_count",
            "status",
            "created_at",
            "funded_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_task_requirement() {
        let mut data = vec![70, 172, 186, 172, 160, 127, 79, 134];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 0, 7, 7, 7, 7, 0, 0, 0, 7, 0, 0, 0, 7, 0, 0, 0, 7, 0, 7, 0, 7, 0, 0,
            0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_task_requirement(&data).expect("TaskRequirement IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify TaskRequirement");
        assert_eq!(decoded_name, "TaskRequirement");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 12);
        for field in [
            "task_id",
            "req_index",
            "capability_code",
            "min_reputation",
            "min_tier",
            "jurisdiction",
            "radius_m",
            "center_lat_e7",
            "center_lon_e7",
            "independence_bps",
            "confidence_target_bps",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_three_d_model() {
        let mut data = vec![117, 174, 95, 32, 116, 194, 171, 170];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 7, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0,
            0, 0, 0, 0, 0, 0, 1, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_three_d_model(&data).expect("ThreeDModel IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify ThreeDModel");
        assert_eq!(decoded_name, "ThreeDModel");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 12);
        for field in [
            "asset",
            "parcel",
            "model_version",
            "geometry_version",
            "model_hash",
            "storage_reference",
            "model_format",
            "submitted_by",
            "submitted_at",
            "verified",
            "verified_by",
            "verified_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_three_d_model_meta() {
        let mut data = vec![231, 80, 5, 178, 80, 198, 86, 7];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 0, 0,
        ]);
        let parsed =
            parse_three_d_model_meta(&data).expect("ThreeDModelMeta IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ThreeDModelMeta");
        assert_eq!(decoded_name, "ThreeDModelMeta");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 2);
        for field in ["asset", "count"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_threshold_credential() {
        let mut data = vec![217, 121, 146, 123, 84, 119, 65, 79];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 170, 1, 0, 0, 0, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 1, 7, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_threshold_credential(&data).expect("ThresholdCredential IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ThresholdCredential");
        assert_eq!(decoded_name, "ThresholdCredential");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 12);
        for field in [
            "credential_hash",
            "prover",
            "purpose",
            "disclosure_type",
            "region_registry",
            "aggregate_signature",
            "signers",
            "signer_count",
            "nullifier_hash",
            "consumed",
            "version",
            "issued_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_activity_tracker() {
        let mut data = vec![45, 51, 146, 237, 228, 13, 159, 133];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        let parsed = parse_validator_activity_tracker(&data)
            .expect("ValidatorActivityTracker IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorActivityTracker");
        assert_eq!(decoded_name, "ValidatorActivityTracker");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 4);
        for field in ["registry", "validator", "last_active", "is_active"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_availability() {
        let mut data = vec![113, 127, 178, 236, 25, 220, 94, 89];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_validator_availability(&data)
            .expect("ValidatorAvailability IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorAvailability");
        assert_eq!(decoded_name, "ValidatorAvailability");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 3);
        for field in ["wallet", "status", "updated_at"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_capability() {
        let mut data = vec![119, 194, 253, 171, 190, 211, 118, 50];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        let parsed =
            parse_validator_capability(&data).expect("ValidatorCapability IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorCapability");
        assert_eq!(decoded_name, "ValidatorCapability");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 7);
        for field in [
            "wallet",
            "capability_code",
            "level",
            "evidence_hash",
            "declared_at",
            "verified_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_endorsement() {
        let mut data = vec![140, 204, 234, 117, 105, 8, 203, 25];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_validator_endorsement(&data)
            .expect("ValidatorEndorsement IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorEndorsement");
        assert_eq!(decoded_name, "ValidatorEndorsement");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 6);
        for field in [
            "registry",
            "proposed",
            "action",
            "endorsers",
            "required",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_nomination() {
        let mut data = vec![210, 142, 12, 155, 104, 210, 45, 94];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 7,
            0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_validator_nomination(&data).expect("ValidatorNomination IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorNomination");
        assert_eq!(decoded_name, "ValidatorNomination");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 10);
        for field in [
            "registry",
            "country_code",
            "sponsor",
            "candidate",
            "documents_hash",
            "location_hash",
            "assigned_physical_confirmer",
            "confirmers",
            "finalized",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_presence() {
        let mut data = vec![49, 114, 241, 63, 232, 95, 50, 90];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 0, 0, 0, 7, 0, 0, 0, 7, 0, 7, 7, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0,
            0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_validator_presence(&data).expect("ValidatorPresence IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorPresence");
        assert_eq!(decoded_name, "ValidatorPresence");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 9);
        for field in [
            "wallet",
            "latitude_e7",
            "longitude_e7",
            "accuracy_m",
            "provenance",
            "confidence_bps",
            "observed_at",
            "expires_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_profile() {
        let mut data = vec![50, 223, 146, 114, 220, 208, 127, 86];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0,
            0, 0,
        ]);
        let parsed =
            parse_validator_profile(&data).expect("ValidatorProfile IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorProfile");
        assert_eq!(decoded_name, "ValidatorProfile");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 7);
        for field in [
            "wallet",
            "tier",
            "identity_hash",
            "note",
            "jurisdiction",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_registry() {
        let mut data = vec![168, 113, 195, 186, 62, 121, 163, 230];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 7, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0,
            0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_validator_registry(&data).expect("ValidatorRegistry IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorRegistry");
        assert_eq!(decoded_name, "ValidatorRegistry");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 8);
        for field in [
            "admin",
            "pending_admin",
            "validators",
            "required_endorsements",
            "paused",
            "version",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_relationship_edge() {
        let mut data = vec![128, 240, 223, 172, 123, 148, 81, 48];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_validator_relationship_edge(&data)
            .expect("ValidatorRelationshipEdge IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorRelationshipEdge");
        assert_eq!(decoded_name, "ValidatorRelationshipEdge");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 6);
        for field in [
            "from",
            "to",
            "edge_type",
            "weight_bps",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_reputation() {
        let mut data = vec![151, 57, 248, 51, 14, 124, 177, 47];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7,
            0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_validator_reputation(&data).expect("ValidatorReputation IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorReputation");
        assert_eq!(decoded_name, "ValidatorReputation");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 11);
        for field in [
            "validator",
            "status",
            "total_attestations",
            "confirmed_attestations",
            "disputed_attestations",
            "reputation_score",
            "jailed_until",
            "challenges_received",
            "challenges_raised",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_validator_stake() {
        let mut data = vec![95, 210, 178, 207, 78, 5, 247, 29];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0,
            0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0,
            0,
        ]);
        let parsed = parse_validator_stake(&data).expect("ValidatorStake IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify ValidatorStake");
        assert_eq!(decoded_name, "ValidatorStake");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 10);
        for field in [
            "stake_pool",
            "validator",
            "staked_amount",
            "unbonding_amount",
            "unbonding_starts_at",
            "reward_per_token_paid",
            "slash_history",
            "offenses",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_vault_record() {
        let mut data = vec![47, 1, 218, 116, 82, 70, 124, 119];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 1, 0, 0, 0, 120, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 1, 0, 0, 0, 120, 1, 0, 0, 0, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_vault_record(&data).expect("VaultRecord IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify VaultRecord");
        assert_eq!(decoded_name, "VaultRecord");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 10);
        for field in [
            "subject",
            "ciphertext_cid",
            "ciphertext_hash",
            "algorithm_id",
            "storage_uris",
            "shard_holders",
            "threshold",
            "version",
            "last_ping_at",
            "created_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_vault_shard_rotation() {
        let mut data = vec![33, 187, 116, 141, 215, 66, 44, 139];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7,
        ]);
        let parsed =
            parse_vault_shard_rotation(&data).expect("VaultShardRotation IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify VaultShardRotation");
        assert_eq!(decoded_name, "VaultShardRotation");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 11);
        for field in [
            "vault",
            "old_ciphertext_hash",
            "new_ciphertext_hash",
            "new_shard_holders",
            "new_threshold",
            "initiated_by",
            "endorsements",
            "required_endorsements",
            "initiated_at",
            "effective_at",
            "status",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_verification_attestation() {
        let mut data = vec![231, 126, 92, 51, 84, 178, 81, 242];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1, 0, 0, 0, 120, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0,
            0, 0, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 1, 7, 0, 0, 0, 0, 0, 0, 0, 7,
        ]);
        let parsed = parse_verification_attestation(&data)
            .expect("VerificationAttestation IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify VerificationAttestation");
        assert_eq!(decoded_name, "VerificationAttestation");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 11);
        for field in [
            "claim",
            "validator",
            "observation",
            "credential_subject",
            "vc_type",
            "issuance_date",
            "expiration_date",
            "sd_jwt_digest",
            "selective_disclosure",
            "created_at",
            "protocol_version",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_verification_session() {
        let mut data = vec![247, 69, 199, 26, 112, 195, 243, 254];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0,
            0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed =
            parse_verification_session(&data).expect("VerificationSession IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify VerificationSession");
        assert_eq!(decoded_name, "VerificationSession");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 13);
        for field in [
            "claim",
            "session_id",
            "status",
            "opened_by",
            "evidence_count",
            "observation_count",
            "attestation_count",
            "dispute_count",
            "required_attestations",
            "created_at",
            "expires_at",
            "closed_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_verification_task() {
        let mut data = vec![96, 182, 137, 216, 62, 108, 74, 190];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        let parsed =
            parse_verification_task(&data).expect("VerificationTask IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify VerificationTask");
        assert_eq!(decoded_name, "VerificationTask");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 17);
        for field in [
            "task_id",
            "requester",
            "subject",
            "task_class",
            "status",
            "reward_lamports",
            "deadline",
            "description_hash",
            "requirement_count",
            "assigned_count",
            "required_validators",
            "result_count",
            "outcome",
            "result_hash",
            "created_at",
            "updated_at",
            "completed_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_vrf_authority() {
        let mut data = vec![38, 133, 126, 18, 55, 188, 204, 215];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7,
        ]);
        let parsed = parse_vrf_authority(&data).expect("VrfAuthority IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify VrfAuthority");
        assert_eq!(decoded_name, "VrfAuthority");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 4);
        for field in ["authority", "admin", "updated_at", "bump"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_world_registry() {
        let mut data = vec![222, 41, 153, 93, 64, 60, 109, 27];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let parsed = parse_world_registry(&data).expect("WorldRegistry IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) =
            decode_any(&data).expect("decode_any must classify WorldRegistry");
        assert_eq!(decoded_name, "WorldRegistry");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 4);
        for field in ["admin", "allocations", "created_at", "updated_at"] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }

    #[test]
    fn accepts_zone_set() {
        let mut data = vec![57, 110, 98, 208, 4, 53, 98, 77];
        data.extend_from_slice(&[
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 0, 0, 0, 7, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0,
            0,
        ]);
        let parsed = parse_zone_set(&data).expect("ZoneSet IDL fixture must decode");
        let _ = parsed;
        let (decoded_name, json) = decode_any(&data).expect("decode_any must classify ZoneSet");
        assert_eq!(decoded_name, "ZoneSet");
        let object = json.as_object().expect("JSON object");
        assert_eq!(object.len(), 6);
        for field in [
            "zone_id",
            "authority",
            "parcel_count",
            "current_root_version",
            "created_at",
            "updated_at",
        ] {
            assert!(object.contains_key(field), "missing field {field}");
        }
    }
}
