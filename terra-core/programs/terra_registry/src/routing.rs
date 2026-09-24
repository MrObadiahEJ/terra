use anchor_lang::prelude::*;
use solana_program::hash::hashv;

use crate::validator_profile::{
    availability_status, capability_level, presence_is_stale, ValidatorAvailability,
    ValidatorCapability, ValidatorPresence, ValidatorProfile,
};
use crate::verification::reputation::ValidatorReputation;
use crate::verification_task::{can_assign, task_is_past_deadline, TaskRequirement};
use crate::verification_task::{task_is_terminal, CAPABILITY_ANY};
use crate::TerraError;

// ---------------------------------------------------------------------------
// RFC-012 Phase 6 — dynamic routing
//
// Multi-factor selection per Design Rule 3:
//   Eligibility → Capability → Jurisdiction → Geographic relevance →
//   Availability → Reputation → Independence → Conflict → Randomized pick.
//
// On-chain: `route_task` takes an explicit candidate list (instruction arg),
// verifies each candidate's PDAs in `remaining_accounts`, filters, then
// picks a winner with a seed derived from (task_id, recent_blockhash, slot).
// Assignment PDA is created for the winner only. Single-candidate calls
// (n=1) always select that candidate if eligible — used by tests and by
// requesters who already short-listed off-chain.
// ---------------------------------------------------------------------------

/// Max candidates evaluated in one `route_task` call.
pub const MAX_ROUTE_CANDIDATES: usize = 8;
/// Max competitor_count for random admission math (sanity bound).
pub const MAX_COMPETITOR_COUNT: u16 = 64;
/// Earth mean radius (meters) for haversine distance.
pub const EARTH_RADIUS_M: f64 = 6_371_000.0;

// ---------------------------------------------------------------------------
// Pure helpers (unit-testable)
// ---------------------------------------------------------------------------

/// Haversine great-circle distance in meters between two e7 fixed-point points.
pub fn distance_m(lat1_e7: i32, lon1_e7: i32, lat2_e7: i32, lon2_e7: i32) -> u64 {
    let to_rad = |deg: f64| deg.to_radians();
    let lat1 = to_rad(lat1_e7 as f64 / 1e7);
    let lat2 = to_rad(lat2_e7 as f64 / 1e7);
    let dlat = to_rad((lat2_e7 - lat1_e7) as f64 / 1e7);
    let dlon = to_rad((lon2_e7 - lon1_e7) as f64 / 1e7);
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    (EARTH_RADIUS_M * c).round() as u64
}

/// Availability filter: AVAILABLE or ONLINE only (not BUSY/OFFLINE/SUSPENDED/…).
pub fn passes_availability(status: u8) -> bool {
    matches!(
        status,
        availability_status::AVAILABLE | availability_status::ONLINE
    )
}

/// Tier filter: `profile_tier >= min_tier` (min_tier=0 always passes).
pub fn passes_tier(profile_tier: u8, min_tier: u8) -> bool {
    profile_tier >= min_tier
}

/// Reputation filter: `score >= min_reputation` (min=0 always passes).
pub fn passes_reputation(score: u16, min_reputation: u16) -> bool {
    score >= min_reputation
}

/// Capability filter: CAPABILITY_ANY always passes; else candidate level must
/// be present and `>= capability_level::OBSERVED` (declared-only is too weak).
pub fn passes_capability(required_code: u8, candidate_level: Option<u8>) -> bool {
    if required_code == CAPABILITY_ANY {
        return true;
    }
    match candidate_level {
        Some(level) => (capability_level::OBSERVED..=capability_level::MAX).contains(&level),
        None => false,
    }
}

/// Geographic filter: `radius_m == 0` → no geo constraint (pass).
/// Else presence must exist, be fresh, and fall within radius of center.
pub fn passes_geo(
    radius_m: u32,
    center_lat_e7: i32,
    center_lon_e7: i32,
    presence: Option<&ValidatorPresence>,
    now: i64,
) -> bool {
    if radius_m == 0 {
        return true;
    }
    let Some(p) = presence else {
        return false;
    };
    if presence_is_stale(now, p.expires_at) {
        return false;
    }
    distance_m(p.latitude_e7, p.longitude_e7, center_lat_e7, center_lon_e7) <= u64::from(radius_m)
}

/// Jurisdiction filter: `[0,0]` = any; else candidate profile must be in
/// the required country. Profiles do not yet store jurisdiction — Phase 6
/// treats non-zero jurisdiction as a soft pass with a TODO for Phase 10
/// (cross-border bindings). Kept as a pure hook so Phase 10 can tighten it.
pub fn passes_jurisdiction(_required: [u8; 2]) -> bool {
    // Any: pass. Non-zero: pass until profile carries jurisdiction (Phase 10).
    true
}

/// Composite multi-factor eligibility (order matches Design Rule 3).
/// `candidate_level` is `Some` only when a capability PDA was supplied.
/// `reputation_score` is 0 when no reputation PDA was supplied.
pub fn is_eligible(
    requirement: &TaskRequirement,
    profile: &ValidatorProfile,
    availability: &ValidatorAvailability,
    reputation_score: u16,
    candidate_level: Option<u8>,
    presence: Option<&ValidatorPresence>,
    now: i64,
) -> bool {
    passes_availability(availability.status)
        && passes_tier(profile.tier, requirement.min_tier)
        && passes_reputation(reputation_score, requirement.min_reputation)
        && passes_capability(requirement.capability_code, candidate_level)
        && passes_jurisdiction(requirement.jurisdiction)
        && passes_geo(
            requirement.radius_m,
            requirement.center_lat_e7,
            requirement.center_lon_e7,
            presence,
            now,
        )
}

/// Deterministic route seed from task id + recent blockhash + slot.
pub fn route_seed(task_id: &[u8; 32], recent_blockhash: &[u8; 32], slot: u64) -> [u8; 32] {
    hashv(&[
        task_id.as_ref(),
        recent_blockhash.as_ref(),
        &slot.to_le_bytes(),
    ])
    .to_bytes()
}

/// Pick a winner among `candidates` using `seed`.
/// Candidates are sorted first so the pick is order-independent.
/// Empty input is a caller bug (use `select_winner_opt`).
pub fn select_winner(seed: &[u8; 32], candidates: &[Pubkey]) -> Pubkey {
    select_winner_opt(seed, candidates).expect("select_winner on empty candidate list")
}

/// Optional form for unit tests.
pub fn select_winner_opt(seed: &[u8; 32], candidates: &[Pubkey]) -> Option<Pubkey> {
    if candidates.is_empty() {
        return None;
    }
    let mut sorted = candidates.to_vec();
    sorted.sort();
    let h = hashv(&[seed.as_ref(), b"terra_route"]);
    let n = sorted.len() as u64;
    let idx = (u64::from_le_bytes(h.to_bytes()[..8].try_into().unwrap()) % n) as usize;
    Some(sorted[idx])
}

/// Filter `candidates` down to those whose wallet appears in `eligible_set`
/// (identity filter kept separate so pure tests can drive eligibility).
pub fn filter_eligible(candidates: &[Pubkey], is_ok: impl Fn(&Pubkey) -> bool) -> Vec<Pubkey> {
    candidates.iter().copied().filter(|c| is_ok(c)).collect()
}

/// Random admission: accept when `draw_bps < 10_000 / competitor_count`.
/// `competitor_count = 1` always accepts (deterministic tests / sole candidate).
pub fn passes_random_gate(draw_bps: u16, competitor_count: u16) -> bool {
    if competitor_count == 0 || competitor_count > MAX_COMPETITOR_COUNT {
        return false;
    }
    if competitor_count == 1 {
        return true;
    }
    let admit_bps = 10_000u32 / u32::from(competitor_count);
    u32::from(draw_bps) < admit_bps
}

/// Map a 32-byte draw to [0, 10000) bps.
pub fn draw_bps(seed: &[u8; 32], validator: &Pubkey) -> u16 {
    let h = hashv(&[seed.as_ref(), validator.as_ref(), b"terra_draw"]);
    let v = u64::from_le_bytes(h.to_bytes()[..8].try_into().unwrap());
    (v % 10_000) as u16
}

/// Deserialize an Anchor account from an `AccountInfo` (remaining_accounts).
fn deser<T: anchor_lang::AccountDeserialize>(ai: &AccountInfo) -> Result<T> {
    let data = ai.try_borrow_data()?;
    let mut slice: &[u8] = data.as_ref();
    T::try_deserialize(&mut slice)
}

// ---------------------------------------------------------------------------
// Instruction handler
// ---------------------------------------------------------------------------

/// Multi-factor route: verify every candidate in `candidates` against
/// `remaining_accounts`, filter, randomly pick a winner, create TaskAssignment
/// for `chosen` (which must equal the winner).
///
/// `remaining_accounts` layout (fixed stride per call, derived from req flags):
/// ```text
/// for cand in candidates:
///   profile, availability, reputation,
///   [presence]  if requirement.radius_m > 0
///   [capability] if requirement.capability_code != CAPABILITY_ANY
/// ```
#[allow(clippy::too_many_arguments)]
pub fn route_task(
    ctx: Context<crate::RouteTask>,
    task_id: [u8; 32],
    req_index: u8,
    candidates: Vec<Pubkey>,
    chosen: Pubkey,
    competitor_count: u16,
) -> Result<()> {
    require!(
        !candidates.is_empty() && candidates.len() <= MAX_ROUTE_CANDIDATES,
        TerraError::TooManyRouteCandidates
    );
    require!(candidates.contains(&chosen), TerraError::NotRouteWinner);
    require!(
        (1..=MAX_COMPETITOR_COUNT).contains(&competitor_count),
        TerraError::TooManyRouteCandidates
    );

    let now = Clock::get()?.unix_timestamp;
    let t = &ctx.accounts.task;
    require!(t.task_id == task_id, TerraError::InvalidTaskRequirement);
    require!(
        can_assign(now, t.status, t.deadline),
        if task_is_past_deadline(now, t.deadline) {
            TerraError::TaskDeadlinePassed
        } else if task_is_terminal(t.status) {
            TerraError::TaskAlreadyFinalized
        } else {
            TerraError::TaskNotOpen
        }
    );
    require!(
        req_index < t.requirement_count,
        TerraError::InvalidTaskRequirement
    );
    let req = &ctx.accounts.requirement;
    require!(
        req.task_id == task_id && req.req_index == req_index,
        TerraError::InvalidTaskRequirement
    );
    require!(
        t.assigned_count < t.required_validators,
        TerraError::TaskAlreadyAssigned
    );
    require!(chosen != t.requester, TerraError::SelfTaskAssignment);

    // Stride: reputation always; +presence if geo; +capability if specific code.
    let need_geo = req.radius_m > 0;
    let need_cap = req.capability_code != CAPABILITY_ANY;
    let stride = 3 + need_geo as usize + need_cap as usize;
    let expected_len = stride * candidates.len();
    require!(
        ctx.remaining_accounts.len() == expected_len,
        TerraError::RouteAccountMismatch
    );

    // Evaluate eligibility for every candidate.
    let mut eligible: Vec<Pubkey> = Vec::with_capacity(candidates.len());
    for (i, cand) in candidates.iter().enumerate() {
        let base = i * stride;
        let profile_ai = &ctx.remaining_accounts[base];
        let availability_ai = &ctx.remaining_accounts[base + 1];
        let reputation_ai = &ctx.remaining_accounts[base + 2];

        let profile: ValidatorProfile = deser(profile_ai)?;
        require!(profile.wallet == *cand, TerraError::RouteAccountMismatch);
        let availability: ValidatorAvailability = deser(availability_ai)?;
        require!(
            availability.wallet == *cand,
            TerraError::RouteAccountMismatch
        );
        let reputation_score: u16 = match deser::<ValidatorReputation>(reputation_ai) {
            Ok(r) => {
                require!(r.validator == *cand, TerraError::RouteAccountMismatch);
                r.reputation_score
            }
            Err(_) => 0,
        };

        let mut cursor = base + 3;
        let presence_ref: Option<ValidatorPresence> = if need_geo {
            let ai = &ctx.remaining_accounts[cursor];
            cursor += 1;
            let p: ValidatorPresence =
                deser(ai).map_err(|_| error!(TerraError::ValidatorNotEligible))?;
            require!(p.wallet == *cand, TerraError::RouteAccountMismatch);
            Some(p)
        } else {
            None
        };
        let candidate_level: Option<u8> = if need_cap {
            let ai = &ctx.remaining_accounts[cursor];
            let c: ValidatorCapability =
                deser(ai).map_err(|_| error!(TerraError::ValidatorNotEligible))?;
            require!(
                c.wallet == *cand && c.capability_code == req.capability_code,
                TerraError::RouteAccountMismatch
            );
            Some(c.level)
        } else {
            None
        };

        let ok = is_eligible(
            req,
            &profile,
            &availability,
            reputation_score,
            candidate_level,
            presence_ref.as_ref(),
            now,
        );
        if ok {
            eligible.push(*cand);
        }
    }

    require!(eligible.contains(&chosen), TerraError::ValidatorNotEligible);

    // Randomized selection among eligible candidates.
    // Entropy: task_id ⊕ task PDA ⊕ slot (Clock). Sufficient for non-extractive
    // fair pick; VRF remains an open question (RFC-012 §12).
    let slot = Clock::get()?.slot;
    let task_pda_bytes: [u8; 32] = ctx.accounts.task.key().to_bytes();
    let seed = route_seed(&task_id, &task_pda_bytes, slot);
    let winner = select_winner(&seed, &eligible);
    require!(winner == chosen, TerraError::NotRouteWinner);

    // Random admission gate (competitor_count=1 → always pass).
    let bps = draw_bps(&seed, &chosen);
    require!(
        passes_random_gate(bps, competitor_count),
        TerraError::NotRouteWinner
    );

    // Create assignment for winner (same bookkeeping as assign_task_validator).
    let validator = chosen;
    let now = Clock::get()?.unix_timestamp;
    let task = &mut ctx.accounts.task;
    task.assigned_count = task.assigned_count.saturating_add(1);
    if task.assigned_count == 1 {
        task.status = crate::verification_task::task_status::ASSIGNED;
    }
    task.updated_at = now;

    let a = &mut ctx.accounts.assignment;
    a.task_id = task.task_id;
    a.validator = validator;
    a.requester = task.requester;
    a.assigned_by = task.requester;
    a.status = crate::verification_task::assignment_status::ASSIGNED;
    a.assigned_at = now;
    a.submitted_at = 0;
    a.updated_at = now;

    emit!(crate::TaskRouted {
        task: task.key(),
        task_id: task.task_id,
        validator,
        req_index,
        eligible_count: eligible.len() as u16,
        competitor_count,
        draw_bps: bps,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator_profile::{
        availability_status, capability_level, profile_tier, PRESENCE_TTL_SECS,
    };

    fn dummy_presence(lat: i32, lon: i32, now: i64) -> ValidatorPresence {
        ValidatorPresence {
            wallet: Pubkey::new_unique(),
            latitude_e7: lat,
            longitude_e7: lon,
            accuracy_m: 10,
            provenance: 1,
            confidence_bps: 9_000,
            observed_at: now,
            expires_at: now + PRESENCE_TTL_SECS,
            updated_at: now,
        }
    }

    #[test]
    fn distance_zero_and_cross_town() {
        assert_eq!(distance_m(0, 0, 0, 0), 0);
        // ~0.01 deg lat ≈ 1.1 km
        let d = distance_m(387_500_000, 121_500_000, 387_600_000, 121_500_000);
        assert!(d > 900 && d < 1_300, "got {d}");
    }

    #[test]
    fn availability_tier_reputation_filters() {
        assert!(passes_availability(availability_status::AVAILABLE));
        assert!(passes_availability(availability_status::ONLINE));
        assert!(!passes_availability(availability_status::BUSY));
        assert!(!passes_availability(availability_status::OFFLINE));
        assert!(!passes_availability(availability_status::SUSPENDED));

        assert!(passes_tier(
            profile_tier::TRUSTED,
            profile_tier::ESTABLISHED
        ));
        assert!(!passes_tier(profile_tier::NEW, profile_tier::ESTABLISHED));
        assert!(passes_tier(profile_tier::NEW, 0));

        assert!(passes_reputation(5_000, 4_000));
        assert!(!passes_reputation(100, 4_000));
        assert!(passes_reputation(0, 0));
    }

    #[test]
    fn capability_and_geo_filters() {
        assert!(passes_capability(CAPABILITY_ANY, None));
        assert!(passes_capability(CAPABILITY_ANY, Some(0)));
        assert!(passes_capability(0, Some(capability_level::OBSERVED)));
        assert!(passes_capability(0, Some(capability_level::VERIFIED)));
        assert!(!passes_capability(0, Some(capability_level::DECLARED)));
        assert!(!passes_capability(0, None));

        let now = 1_000_000;
        let center = (387_500_000_i32, 121_500_000_i32);
        // radius 0 → always pass
        assert!(passes_geo(0, center.0, center.1, None, now));

        let near = dummy_presence(center.0, center.1, now);
        assert!(passes_geo(100, center.0, center.1, Some(&near), now));

        let far = dummy_presence(0, 0, now);
        assert!(!passes_geo(1_000, center.0, center.1, Some(&far), now));

        // Stale presence fails
        let mut stale = dummy_presence(center.0, center.1, now);
        stale.expires_at = now - 1;
        assert!(!passes_geo(10_000, center.0, center.1, Some(&stale), now));

        // Geo required but no presence
        assert!(!passes_geo(100, center.0, center.1, None, now));
    }

    #[test]
    fn select_winner_is_deterministic_and_order_independent() {
        let a = Pubkey::new_from_array([1u8; 32]);
        let b = Pubkey::new_from_array([2u8; 32]);
        let c = Pubkey::new_from_array([3u8; 32]);
        let seed = [9u8; 32];

        let w1 = select_winner_opt(&seed, &[a, b, c]).unwrap();
        let w2 = select_winner_opt(&seed, &[c, a, b]).unwrap();
        let w3 = select_winner_opt(&seed, &[b, c, a]).unwrap();
        assert_eq!(w1, w2);
        assert_eq!(w2, w3);
        assert!(w1 == a || w1 == b || w1 == c);

        assert!(select_winner_opt(&seed, &[]).is_none());
    }

    #[test]
    fn random_gate_and_draw_bounds() {
        assert!(passes_random_gate(0, 1));
        assert!(passes_random_gate(9_999, 1)); // sole candidate always passes
        assert!(!passes_random_gate(0, 0));
        assert!(!passes_random_gate(0, MAX_COMPETITOR_COUNT + 1));

        // competitor_count=2 → admit_bps = 5000
        assert!(passes_random_gate(4_999, 2));
        assert!(!passes_random_gate(5_000, 2));

        let seed = [7u8; 32];
        let v = Pubkey::new_unique();
        for _ in 0..32 {
            let d = draw_bps(&seed, &v);
            assert!(d < 10_000);
        }
    }

    #[test]
    fn filter_eligible_keeps_matching() {
        let a = Pubkey::new_from_array([1u8; 32]);
        let b = Pubkey::new_from_array([2u8; 32]);
        let out = filter_eligible(&[a, b], |p| *p == a);
        assert_eq!(out, vec![a]);
    }

    #[test]
    fn route_seed_changes_with_slot() {
        let task = [1u8; 32];
        let bh = [2u8; 32];
        let s1 = route_seed(&task, &bh, 1);
        let s2 = route_seed(&task, &bh, 2);
        assert_ne!(s1, s2);
        assert_eq!(route_seed(&task, &bh, 1), s1);
    }
}
