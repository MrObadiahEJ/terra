use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Terra Demo Engine (advancement directive B5/B6).
//
// POST /api/v1/demo/scenarios/{name} returns a fully DETERMINISTIC scenario:
// the same name + seed always produces the same accounts, events and result —
// no chain state is written, every payload carries "demo": true so the
// frontend can label it honestly (never present simulated state as chain
// state). Event kinds mirror on-chain instruction/event names 1:1 so the
// UI animation maps 1:1 onto real protocol steps.
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/scenarios", get(index))
        .route("/scenarios/{name}", post(run))
}

#[derive(Debug, Deserialize, Default)]
pub struct ScenarioRequest {
    /// Deterministic seed; defaults to "default".
    #[serde(default)]
    pub seed: Option<String>,
}

/// Catalogue of available scenarios.
pub async fn index() -> Json<Value> {
    Json(json!({
        "demo": true,
        "deterministic": true,
        "seed_default": "default",
        "scenarios": SCENARIOS.iter().map(|(name, desc)| json!({
            "name": name,
            "description": desc,
            "method": "POST",
            "path": format!("/demo/scenarios/{name}"),
        })).collect::<Vec<_>>(),
    }))
}

/// Run one scenario. Unknown names → 404 (so typos never silently succeed).
pub async fn run(
    Path(name): Path<String>,
    State(_state): State<AppState>,
    Json(req): Json<ScenarioRequest>,
) -> Result<Json<Value>, AppError> {
    let seed = req
        .seed
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("default");
    let value = build_scenario(&name, seed).ok_or_else(|| {
        AppError::not_found(format!(
            "unknown scenario '{name}' — GET /demo/scenarios for the catalogue"
        ))
    })?;
    Ok(Json(value))
}

const SCENARIOS: &[(&str, &str)] = &[
    (
        "land-registration",
        "parcel → ownership right → spatial asset → geometry v0",
    ),
    (
        "verification",
        "claim → evidence → observation → task → routing → quorum → verified",
    ),
    (
        "cross-border",
        "jurisdictions → identity binding → membership → cross-border verification",
    ),
    (
        "dispute",
        "dispute → freeze → adjudication → judgment → execution",
    ),
    (
        "validator-routing",
        "requirement → eligibility gates → deterministic winner + random gate",
    ),
    (
        "spatial-update",
        "canonical v1 → new observation → candidate → quorum → v2 appended",
    ),
    (
        "fraud-review",
        "fraud report → committee → votes → finalize → restriction",
    ),
    (
        "recovery",
        "heartbeats → validator offline → quorum check → injection → restored",
    ),
];

fn build_scenario(name: &str, seed: &str) -> Option<Value> {
    match name {
        "land-registration" => Some(build_land_registration(seed)),
        "verification" => Some(build_verification(seed)),
        "cross-border" => Some(build_cross_border(seed)),
        "dispute" => Some(build_dispute(seed)),
        "validator-routing" => Some(build_validator_routing(seed)),
        "spatial-update" => Some(build_spatial_update(seed)),
        "fraud-review" => Some(build_fraud_review(seed)),
        "recovery" => Some(build_recovery(seed)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Deterministic PRNG — xorshift64* seeded by FNV-1a of (scenario|seed).
// Same inputs → byte-identical output on every machine (B6 requirement).
// ---------------------------------------------------------------------------

struct Rng(u64);

const B58: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

impl Rng {
    fn new(key: &str) -> Self {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in key.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        Self(h | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// Deterministic base58-looking demo account (`demo…`, 44 chars).
    /// Clearly prefixed so it can never be mistaken for a funded pubkey.
    fn account(&mut self) -> String {
        let mut s = String::with_capacity(44);
        s.push_str("demo");
        for _ in 0..40 {
            s.push(char::from(B58[self.below(B58.len() as u64) as usize]));
        }
        s
    }

    /// Deterministic 32-byte hex id (sha256-shaped, but seeded — labelled demo).
    fn hash_id(&mut self) -> String {
        let mut s = String::with_capacity(64);
        for _ in 0..4 {
            s.push_str(&format!("{:016x}", self.next_u64()));
        }
        s
    }

    fn bps(&mut self) -> u16 {
        (self.below(10_000)) as u16
    }
}

// ---------------------------------------------------------------------------
// Scenario story builder — ordered events with deterministic timestamps
// ---------------------------------------------------------------------------

struct Story {
    events: Vec<Value>,
    t_ms: u64,
    rng: Rng,
}

impl Story {
    fn new(scenario: &str, seed: &str) -> Self {
        Self {
            events: Vec::new(),
            t_ms: 0,
            rng: Rng::new(&format!("{scenario}|{seed}")),
        }
    }

    fn push(&mut self, kind: &str, detail: &str, reference: &str) {
        self.t_ms += 180 + self.rng.below(900);
        let seq = self.events.len();
        self.events.push(json!({
            "seq": seq,
            "t_offset_ms": self.t_ms,
            "kind": kind,
            "detail": detail,
            "ref": reference,
        }));
    }

    fn attestors(&mut self, n: usize) -> Vec<Value> {
        (0..n)
            .map(|i| {
                let validator = self.rng.account();
                self.t_ms += 200 + self.rng.below(700);
                json!({ "validator": validator, "seq": i })
            })
            .collect()
    }
}

fn envelope(scenario: &str, seed: &str) -> Value {
    json!({
        "demo": true,
        "deterministic": true,
        "scenario": scenario,
        "seed": seed,
        "note": "deterministic demo state — no chain state written; accounts are demo-prefixed labels",
        "program": "terra_registry",
    })
}

/// Merge `fields` into the envelope (envelope keys win on conflict).
fn with(envelope: Value, fields: Value) -> Value {
    let mut obj = envelope
        .as_object()
        .cloned()
        .expect("envelope is an object");
    let extra = fields.as_object().expect("fields are an object").clone();
    obj.extend(extra);
    Value::Object(obj)
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

fn build_land_registration(seed: &str) -> Value {
    let mut s = Story::new("land-registration", seed);
    let parcel = s.rng.hash_id();
    let owner = s.rng.account();
    let spatial = s.rng.hash_id();
    let geometry = s.rng.hash_id();
    s.push(
        "PARCEL_REGISTERED",
        "register_parcel — status ACTIVE",
        &parcel,
    );
    s.push(
        "OWNERSHIP_RIGHT_GRANTED",
        "canonical OWNERSHIP Rights PDA minted",
        &owner,
    );
    s.push(
        "SPATIAL_ASSET_INITIALIZED",
        "init_spatial_asset — envelope 0..40 m",
        &spatial,
    );
    s.push(
        "GEOMETRY_VERSION_APPENDED",
        "append_geometry_version v0 — source SURVEY",
        &geometry,
    );
    s.push(
        "EVIDENCE_SUBMITTED",
        "evidence manifest linked to geometry claim",
        &geometry,
    );
    s.push(
        "REGISTRATION_COMPLETE",
        "parcel queryable, ownership derives from Rights PDA",
        &parcel,
    );
    with(
        envelope("land-registration", seed),
        json!({
            "result": "REGISTERED",
            "parcel": { "id": parcel, "status": "ACTIVE" },
            "ownership": { "holder": owner, "rights_kind": 0 },
            "spatial_asset": { "id": spatial, "geometry_version": 0 },
            "events": s.events,
        }),
    )
}

fn build_verification(seed: &str) -> Value {
    let mut s = Story::new("verification", seed);
    let parcel = s.rng.hash_id();
    let claim = s.rng.hash_id();
    let task = s.rng.hash_id();
    let manifest = s.rng.hash_id();
    let validators: Vec<String> = (0..4).map(|_| s.rng.account()).collect();
    let eligible_count = 3;
    let draw = s.rng.bps();
    let admit: u16 = 10_000 / eligible_count;

    s.push(
        "CLAIM_CREATED",
        "create_claim — geometry + elevation claim open",
        &claim,
    );
    s.push(
        "EVIDENCE_MANIFEST_SUBMITTED",
        "submit_evidence_manifest — 3 artifacts",
        &manifest,
    );
    s.push(
        "OBSERVATION_RECORDED",
        "submit_observation_v2 — device GNSS",
        &parcel,
    );
    s.push(
        "TASK_CREATED",
        "create_verification_task — quorum 3 of 8",
        &task,
    );
    s.push(
        "TASK_ROUTED",
        &format!(
            "{eligible_count} eligible via six gates; draw_bps {draw} < admit {admit} (0-based admission demo)"
        ),
        &task,
    );
    s.push(
        "VALIDATOR_ASSIGNED",
        "assignment created — status ASSIGNED",
        &validators[0],
    );
    let attestations = s.attestors(3);
    for (i, a) in attestations.iter().enumerate() {
        s.push(
            "ATTESTATION_SUBMITTED",
            &format!("submit_verification_attestation {i}/3"),
            a["validator"].as_str().unwrap_or(&task),
        );
    }
    s.push("QUORUM_REACHED", "3/3 attestors — quorum satisfied", &claim);
    s.push("CLAIM_VERIFIED", "verify_claim — status VERIFIED", &claim);
    s.push(
        "GEOMETRY_VERSION_APPENDED",
        "canonical geometry version appended",
        &claim,
    );

    with(
        envelope("verification", seed),
        json!({
            "result": "VERIFIED",
            "parcel": { "id": parcel },
            "claim": { "id": claim, "status": "VERIFIED" },
            "evidence": [manifest],
            "validators": validators,
            "routing": { "eligible": eligible_count, "winner": attestations[0]["validator"], "draw_bps": draw, "admit_bps": admit },
            "attestations": attestations,
            "quorum": { "required": 3, "received": 3 },
            "events": s.events,
        }),
    )
}

fn build_cross_border(seed: &str) -> Value {
    let mut s = Story::new("cross-border", seed);
    let jur_a = "ES";
    let jur_b = "PT";
    let identity = s.rng.account();
    let binding = s.rng.hash_id();
    s.push(
        "JURISDICTION_REGISTERED",
        "register_jurisdiction — ES",
        jur_a,
    );
    s.push(
        "JURISDICTION_REGISTERED",
        "register_jurisdiction — PT",
        jur_b,
    );
    s.push(
        "IDENTITY_BOUND",
        "bind_identity — holder wallet linked",
        &identity,
    );
    s.push(
        "MEMBERSHIP_VERIFIED",
        "verify_jurisdiction_membership — ES active",
        jur_a,
    );
    s.push(
        "CROSS_BORDER_BINDING_CREATED",
        "create_cross_border_binding — ES → PT",
        &binding,
    );
    s.push(
        "BINDING_STATUS_ACTIVE",
        "set_cross_border_binding_status — ACTIVE",
        &binding,
    );
    s.push(
        "CROSS_BORDER_VERIFIED",
        "verify_cross_border — session linked",
        &binding,
    );
    with(
        envelope("cross-border", seed),
        json!({
            "result": "VERIFIED",
            "jurisdictions": [jur_a, jur_b],
            "identity": identity,
            "binding": { "id": binding, "from": jur_a, "to": jur_b, "status": "ACTIVE" },
            "events": s.events,
        }),
    )
}

fn build_dispute(seed: &str) -> Value {
    let mut s = Story::new("dispute", seed);
    let parcel = s.rng.hash_id();
    let dispute = s.rng.hash_id();
    let judgment = s.rng.hash_id();
    s.push(
        "DISPUTE_FILED",
        "file_dispute — boundary challenge",
        &dispute,
    );
    s.push(
        "PARCEL_FROZEN",
        "freeze_parcel — transfers blocked",
        &parcel,
    );
    s.push(
        "ADJUDICATION_OPENED",
        "adjudicate_dispute — panel seated",
        &dispute,
    );
    let committee = (0..3).map(|_| s.rng.account()).collect::<Vec<_>>();
    s.push(
        "COMMITTEE_SELECTED",
        "3-validator adjudication committee",
        &dispute,
    );
    s.push(
        "JUDGMENT_RENDERED",
        "upheld — boundary corrected",
        &judgment,
    );
    s.push(
        "JUDGMENT_EXECUTED",
        "execute_judgment — parcel unfrozen, status updated",
        &parcel,
    );
    with(
        envelope("dispute", seed),
        json!({
            "result": "JUDGMENT_EXECUTED",
            "parcel": { "id": parcel, "status": "ACTIVE" },
            "dispute": { "id": dispute, "status": "EXECUTED" },
            "judgment": judgment,
            "committee": committee,
            "events": s.events,
        }),
    )
}

fn build_validator_routing(seed: &str) -> Value {
    let mut s = Story::new("validator-routing", seed);
    let task = s.rng.hash_id();
    let candidates: Vec<String> = (0..6).map(|_| s.rng.account()).collect();
    // Mirror the six gates + winner + random gate as explicit story beats.
    let eligible: u64 = 3;
    let winner_idx = s.rng.below(eligible) as usize;
    let draw = s.rng.bps();
    let admit: u16 = (10_000 / eligible) as u16;
    let gate_pass = draw < admit;
    s.push(
        "TASK_REQUIREMENTS_SET",
        "min_tier 2, min_rep 5000, cap SURVEY, radius 900 km",
        &task,
    );
    s.push(
        "CANDIDATES_GATHERED",
        "6 candidate profiles supplied as remaining accounts",
        &task,
    );
    s.push(
        "ELIGIBILITY_FILTERED",
        "availability → tier → reputation → capability → jurisdiction → geo",
        &task,
    );
    let winner = candidates[winner_idx].clone();
    s.push(
        "WINNER_SELECTED",
        "select_winner over sorted eligible set",
        &winner,
    );
    s.push(
        "RANDOM_GATE",
        &format!(
            "draw_bps {draw} vs admit {admit} — {}",
            if gate_pass { "PASS" } else { "REJECT" }
        ),
        &winner,
    );
    let result = if gate_pass {
        "ROUTED"
    } else {
        "RETRY_NEXT_SLOT"
    };
    s.push(
        if gate_pass {
            "ASSIGNMENT_CREATED"
        } else {
            "NOT_ROUTE_WINNER"
        },
        if gate_pass {
            "assignment status ASSIGNED"
        } else {
            "route rejected — caller retries at next slot"
        },
        &winner,
    );
    with(
        envelope("validator-routing", seed),
        json!({
            "result": result,
            "task": { "id": task },
            "candidates": candidates,
            "routing": { "eligible": eligible, "winner": winner, "draw_bps": draw, "admit_bps": admit, "gate_pass": gate_pass },
            "events": s.events,
        }),
    )
}

fn build_spatial_update(seed: &str) -> Value {
    let mut s = Story::new("spatial-update", seed);
    let spatial = s.rng.hash_id();
    let observation = s.rng.hash_id();
    let candidate = s.rng.hash_id();
    s.push(
        "GEOMETRY_V1_CANONICAL",
        "v1 verified under survey provenance",
        &spatial,
    );
    s.push(
        "OBSERVATION_RECORDED",
        "submit_observation_v2 — LiDAR strip, +0.4 m",
        &observation,
    );
    s.push(
        "CANDIDATE_GEOMETRY_DERIVED",
        "candidate v2 envelope 0..40.4 m",
        &candidate,
    );
    s.push(
        "TASK_CREATED",
        "geometry verification task — quorum 3",
        &candidate,
    );
    let attestations = s.attestors(3);
    s.push("QUORUM_REACHED", "3/3 geometry attestations", &candidate);
    s.push(
        "GEOMETRY_V2_APPENDED",
        "append_geometry_version v2 — DEM source",
        &spatial,
    );
    s.push(
        "GEOMETRY_V2_VERIFIED",
        "verify_geometry_version — canonical",
        &spatial,
    );
    with(
        envelope("spatial-update", seed),
        json!({
            "result": "GEOMETRY_V2_VERIFIED",
            "spatial_asset": { "id": spatial, "from_version": 1, "to_version": 2 },
            "observation": observation,
            "candidate": candidate,
            "attestations": attestations,
            "quorum": { "required": 3, "received": 3 },
            "events": s.events,
        }),
    )
}

fn build_fraud_review(seed: &str) -> Value {
    let mut s = Story::new("fraud-review", seed);
    let report = s.rng.hash_id();
    let offender = s.rng.account();
    let restriction = s.rng.hash_id();
    s.push(
        "FRAUD_REPORT_SUBMITTED",
        "submit_fraud_report — false attestation",
        &report,
    );
    s.push(
        "FRAUD_REVIEW_OPENED",
        "open_fraud_review — committee pool filtered",
        &report,
    );
    let committee = (0..5).map(|_| s.rng.account()).collect::<Vec<_>>();
    s.push(
        "COMMITTEE_SELECTED",
        "5-of-pool committee via seed + draw_bps",
        &report,
    );
    s.push("VOTES_CAST", "4 guilty / 1 abstain", &report);
    s.push(
        "REVIEW_FINALIZED",
        "finalize_fraud_review — threshold met",
        &report,
    );
    s.push(
        "RESTRICTION_APPLIED",
        &format!("capability restriction on {offender} — SURVEY blocked"),
        &restriction,
    );
    s.push(
        "APPEAL_WINDOW_OPEN",
        "file_fraud_appeal available until deadline",
        &report,
    );
    with(
        envelope("fraud-review", seed),
        json!({
            "result": "RESTRICTION_APPLIED",
            "report": { "id": report, "status": "FINALIZED" },
            "offender": offender,
            "restriction": { "id": restriction, "capability": "SURVEY", "status": "ACTIVE" },
            "committee": committee,
            "votes": { "guilty": 4, "abstain": 1, "lenient": 0 },
            "events": s.events,
        }),
    )
}

fn build_recovery(seed: &str) -> Value {
    let mut s = Story::new("recovery", seed);
    let registry = s.rng.hash_id();
    let offline = s.rng.account();
    let injected = s.rng.account();
    s.push(
        "HEARTBEAT_OK",
        "3/3 validators within heartbeat window",
        &registry,
    );
    s.push("VALIDATOR_OFFLINE", "heartbeat missed ×2", &offline);
    s.push(
        "SET_VALIDATOR_ACTIVE",
        "marked inactive — effective quorum 2/3",
        &offline,
    );
    s.push(
        "QUORUM_CHECK",
        "check_quorum_reachable — degraded but above threshold",
        &registry,
    );
    s.push(
        "EMERGENCY_INJECTION_QUEUED",
        "queue_emergency_injection — recovery window 60 s",
        &registry,
    );
    s.push(
        "EMERGENCY_INJECTION_EXECUTED",
        "recovery validator injected",
        &injected,
    );
    s.push("HEARTBEAT_OK", "3/3 healthy — quorum restored", &registry);
    with(
        envelope("recovery", seed),
        json!({
            "result": "QUORUM_RESTORED",
            "registry": registry,
            "failed_validator": offline,
            "recovery_validator": injected,
            "quorum": { "required": 3, "healthy": 3, "during_incident": 2 },
            "events": s.events,
        }),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario_names() -> Vec<&'static str> {
        SCENARIOS.iter().map(|(n, _)| *n).collect()
    }

    #[test]
    fn catalogue_lists_all_builders() {
        for name in scenario_names() {
            assert!(
                build_scenario(name, "default").is_some(),
                "catalogued scenario {name} has no builder"
            );
        }
        assert!(build_scenario("does-not-exist", "x").is_none());
    }

    #[test]
    fn same_seed_is_byte_identical() {
        for name in scenario_names() {
            let a = build_scenario(name, "demo-seed-42").expect("builder");
            let b = build_scenario(name, "demo-seed-42").expect("builder");
            assert_eq!(a, b, "{name} is not deterministic for a fixed seed");
        }
    }

    #[test]
    fn different_seed_changes_accounts() {
        for name in scenario_names() {
            let a = build_scenario(name, "seed-a").expect("builder");
            let b = build_scenario(name, "seed-b").expect("builder");
            assert_ne!(a, b, "{name} ignores the seed");
        }
    }

    #[test]
    fn every_scenario_is_labelled_demo_and_has_events() {
        for name in scenario_names() {
            let v = build_scenario(name, "default").expect("builder");
            assert_eq!(v["demo"], true, "{name} must carry demo: true");
            assert_eq!(v["deterministic"], true);
            assert_eq!(v["scenario"], name);
            assert!(!v["result"].as_str().unwrap_or_default().is_empty());
            let events = v["events"].as_array().expect("events array");
            assert!(events.len() >= 5, "{name}: too few events");
            for (i, e) in events.iter().enumerate() {
                assert_eq!(e["seq"], i, "{name}: event seq must be contiguous");
                assert!(!e["kind"].as_str().unwrap_or_default().is_empty());
            }
            let t_first = events[0]["t_offset_ms"].as_u64().unwrap();
            let t_last = events[events.len() - 1]["t_offset_ms"].as_u64().unwrap();
            assert!(t_last > t_first, "{name}: timestamps must advance");
        }
    }

    #[test]
    fn verification_scenario_reaches_quorum() {
        let v = build_verification("default");
        assert_eq!(v["result"], "VERIFIED");
        assert_eq!(v["quorum"]["required"], 3);
        assert_eq!(v["quorum"]["received"], 3);
        assert_eq!(v["claim"]["status"], "VERIFIED");
        assert_eq!(v["attestations"].as_array().expect("atts").len(), 3);
    }

    #[test]
    fn routing_scenario_gate_values_in_range() {
        for seed in ["default", "alt"] {
            let v = build_validator_routing(seed);
            let draw = v["routing"]["draw_bps"].as_u64().expect("draw");
            let admit = v["routing"]["admit_bps"].as_u64().expect("admit");
            assert!(draw < 10_000);
            assert_eq!(admit, 10_000 / 3);
            let expected = if draw < admit {
                "ROUTED"
            } else {
                "RETRY_NEXT_SLOT"
            };
            assert_eq!(v["result"], expected);
        }
    }

    #[test]
    fn demo_accounts_are_prefixed_and_b58_safe() {
        let v = build_verification("default");
        for acc in v["validators"].as_array().expect("validators") {
            let a = acc.as_str().expect("str");
            assert!(
                a.starts_with("demo") && a.len() == 44,
                "bad demo account: {a}"
            );
            assert!(
                a.chars().all(|c| B58.contains(&(c as u8))),
                "non-base58 char in {a}"
            );
        }
    }
}
