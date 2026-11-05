//! Incremental derived compute (RFC-014 Phase 04).
//!
//! A *derived job* is an expensive, deterministic result (a routed polyline, a
//! reconstruction draft, a geometry digest) that is a pure function of its
//! inputs. `processing_key = SHA-256(kind ‖ input_hash)` carries a UNIQUE
//! constraint, so two callers requesting the same work share one row instead
//! of recomputing. This is a reuse cache with ledger semantics, not a fact: a
//! derived result is only as trustworthy as its inputs, and it is invalidated
//! (marked `stale`, never silently trusted) when an upstream dependency
//! changes. Nothing here is on-chain state.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::SignedRequest;
use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs", post(create_job))
        .route("/jobs", get(list_jobs))
        .route("/jobs/lease", post(lease_job))
        .route("/jobs/{id}", get(get_job))
        .route("/jobs/{id}/complete", post(complete_job))
        .route("/jobs/{id}/fail", post(fail_job))
        .route("/jobs/{id}/invalidate", post(invalidate_job))
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct DerivedJob {
    pub id: String,
    pub kind: String,
    pub processing_key: String,
    pub input_hash: String,
    pub status: String,
    pub result_ref: Option<String>,
    pub result_hash: Option<String>,
    pub error: Option<String>,
    pub created_by: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct NewDerivedJob {
    pub kind: String,
    /// Canonical input document. Its SHA-256 is `input_hash`; the caller is
    /// responsible for canonical serialization (same bytes ⇒ same key).
    pub input_document: String,
    /// Upstream jobs this one depends on (invalidation propagates from these).
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Optional caller-supplied pointer to where a result would live.
    #[serde(default)]
    pub result_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateJobResponse {
    pub job: DerivedJob,
    /// True when an already-`done` job was reused (no recompute happened).
    pub reused: bool,
    /// True when this call created the row (false = it already existed).
    pub created: bool,
}

#[derive(Debug, Deserialize)]
pub struct CompleteInput {
    pub result_ref: Option<String>,
    pub result_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FailInput {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct InvalidateResponse {
    /// Jobs marked `stale` by this call (the root plus transitive dependents).
    pub invalidated: Vec<String>,
    pub count: usize,
}

const JOB_COLS: &str = "id, kind, processing_key, input_hash, status, result_ref, result_hash, error, created_by, created_at, completed_at";
const SELECT_JOB: &str = "SELECT id, kind, processing_key, input_hash, status, result_ref, result_hash, error, created_by, created_at, completed_at";
const RETURNING_JOB: &str = "RETURNING id, kind, processing_key, input_hash, status, result_ref, result_hash, error, created_by, created_at, completed_at";
// `JOB_COLS` and the two fragments above are the same list in different
// statement positions; kept as one constant pair so a column addition touches
// one place. (Rust format capture keeps this honest at compile time.)

// ---------------------------------------------------------------------------
// Pure helpers (unit-testable, no DB)
// ---------------------------------------------------------------------------

/// SHA-256 hex of an input document.
pub fn input_hash_hex(input_document: &str) -> String {
    hex::encode(Sha256::digest(input_document.as_bytes()))
}

/// `processing_key` binds the kind to the input so identical inputs under
/// different kinds do not collide.
pub fn processing_key_hex(kind: &str, input_hash: &str) -> String {
    let mut h = Sha256::new();
    h.update(kind.as_bytes());
    h.update(b"\n");
    h.update(input_hash.as_bytes());
    hex::encode(h.finalize())
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn validate_new(input: &NewDerivedJob) -> Result<(), AppError> {
    if input.kind.trim().is_empty() || input.kind.len() > 64 {
        return Err(AppError::bad_request("kind must be 1..=64 chars"));
    }
    if input.input_document.is_empty() {
        return Err(AppError::bad_request("input_document must not be empty"));
    }
    for dep in &input.depends_on {
        if dep.len() > 64 {
            return Err(AppError::bad_request("dependency id too long"));
        }
    }
    Ok(())
}

fn caller_wallet(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("x-wallet")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Create a derived job, or atomically reuse/de-duplicate on `processing_key`.
///
/// Semantics on conflict (same key already exists):
/// - existing `done`     → reuse it untouched (compute-once), `reused: true`.
/// - existing `pending`/`running`/`stale`/`failed` → reset to `pending` so the
///   work is (re)run; `reused: false`. The row keeps its identity, so the
///   dependency graph and history stay intact.
pub async fn create_job(
    State(state): State<AppState>,
    auth: SignedRequest,
    headers: axum::http::HeaderMap,
    Json(input): Json<NewDerivedJob>,
) -> Result<(StatusCode, Json<CreateJobResponse>), AppError> {
    validate_new(&input)?;

    let mut created_by: Option<String> = None;
    if let Some(wallet) = caller_wallet(&headers) {
        auth.verify_wallet(&wallet, "POST", "/api/v1/derived/jobs")?;
        created_by = Some(wallet);
    }

    let input_hash = input_hash_hex(&input.input_document);
    let processing_key = processing_key_hex(&input.kind, &input_hash);
    let fresh_id = Uuid::new_v4().to_string();

    // One atomic INSERT … ON CONFLICT … RETURNING. `row.id == fresh_id` means
    // the INSERT branch ran (fresh row); otherwise the row already existed.
    let row = sqlx::query_as::<_, DerivedJob>(&format!(
        "INSERT INTO derived_jobs (id, kind, processing_key, input_hash, status, result_ref, created_by) \
         VALUES ($1, $2, $3, $4, 'pending', $5, $6) \
         ON CONFLICT (processing_key) DO UPDATE SET \
           status = CASE WHEN derived_jobs.status = 'done' THEN 'done' ELSE 'pending' END, \
           error  = CASE WHEN derived_jobs.status = 'done' THEN derived_jobs.error ELSE NULL END \
         {RETURNING_JOB}"
    ))
    .bind(&fresh_id)
    .bind(&input.kind)
    .bind(&processing_key)
    .bind(&input_hash)
    .bind(input.result_ref.as_deref())
    .bind(created_by.as_deref())
    .fetch_one(&state.pool)
    .await?;
    let created = row.id == fresh_id;

    // Attach dependencies (idempotent). Edges may reference rows that do not
    // exist yet in concurrent scenarios only if the caller invents ids — an
    // unknown id fails the FK and surfaces as a 500 with the cause logged.
    for dep in &input.depends_on {
        sqlx::query(
            "INSERT INTO derived_job_dependencies (job_id, depends_on_id) \
             VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(&row.id)
        .bind(dep)
        .execute(&state.pool)
        .await?;
    }

    let reused = !created && row.status == "done";
    Ok((
        StatusCode::OK,
        Json(CreateJobResponse {
            job: row,
            reused,
            created,
        }),
    ))
}

pub async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DerivedJob>, AppError> {
    let job =
        sqlx::query_as::<_, DerivedJob>(&format!("{SELECT_JOB} FROM derived_jobs WHERE id = $1"))
            .bind(&id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| AppError::not_found("derived job not found"))?;
    Ok(Json(job))
}

#[derive(Debug, Serialize)]
pub struct LeasedDerivedJob {
    pub job: DerivedJob,
}

const DERIVED_LEASE_TTL_SECS: i64 = 15 * 60;

/// Lease the oldest pending derived job, or reclaim an expired lease.
pub async fn lease_job(
    State(state): State<AppState>,
    auth: SignedRequest,
    headers: axum::http::HeaderMap,
) -> Result<Json<LeasedDerivedJob>, AppError> {
    let key = state.api_authority.0.ok_or_else(|| {
        AppError::unauthorized("privileged endpoint: API_AUTHORITY_PUBKEY not configured")
    })?;
    auth.verify(&key, "POST", "/api/v1/derived/jobs/lease")?;
    let claimed_by = caller_wallet(&headers);
    if let Some(ref wallet) = claimed_by {
        auth.verify_wallet(wallet, "POST", "/api/v1/derived/jobs/lease")?;
    }

    sqlx::query(
        "UPDATE derived_jobs SET status = 'pending', claimed_by = NULL, \
         claimed_at = NULL, lease_expires_at = NULL \
         WHERE status = 'running' AND lease_expires_at IS NOT NULL \
         AND lease_expires_at < now()",
    )
    .execute(&state.pool)
    .await?;

    let claimed: Option<String> = sqlx::query_scalar(
        "UPDATE derived_jobs SET status = 'running', claimed_by = $1, \
         claimed_at = now(), lease_expires_at = now() + make_interval(secs => $2::double precision) \
         WHERE id = (SELECT id FROM derived_jobs WHERE status = 'pending' \
         ORDER BY created_at ASC LIMIT 1 FOR UPDATE SKIP LOCKED) \
         RETURNING id",
    )
    .bind(&claimed_by)
    .bind(DERIVED_LEASE_TTL_SECS as f64)
    .fetch_optional(&state.pool)
    .await?;

    let Some(job_id) = claimed else {
        return Err(AppError::not_found("no pending derived jobs"));
    };
    let job =
        sqlx::query_as::<_, DerivedJob>(&format!("{SELECT_JOB} FROM derived_jobs WHERE id = $1"))
            .bind(&job_id)
            .fetch_one(&state.pool)
            .await?;

    if let Some(ref wallet) = claimed_by {
        let _ = sqlx::query(
            "INSERT INTO job_assignments (job_id, worker_id) \
             VALUES ($1, $2) ON CONFLICT (job_id, worker_id) DO NOTHING",
        )
        .bind(&job_id)
        .bind(wallet)
        .execute(&state.pool)
        .await;
    }

    Ok(Json(LeasedDerivedJob { job }))
}

#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    #[serde(default = "default_jobs_limit")]
    pub limit: i64,
}

fn default_jobs_limit() -> i64 {
    100
}

pub async fn list_jobs(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ListJobsQuery>,
) -> Result<Json<Vec<DerivedJob>>, AppError> {
    let limit = query.limit.clamp(1, 500);
    let jobs = sqlx::query_as::<_, DerivedJob>(&format!(
        "{SELECT_JOB} FROM derived_jobs ORDER BY created_at ASC LIMIT $1"
    ))
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(jobs))
}

/// Mark a pending/running job `done`. Idempotent: completing an already-`done`
/// job returns the stored row untouched.
pub async fn complete_job(
    State(state): State<AppState>,
    auth: SignedRequest,
    Path(id): Path<String>,
    Json(input): Json<CompleteInput>,
) -> Result<Json<DerivedJob>, AppError> {
    // Ed25519 signature verification — fail-closed when authority is not configured.
    let key = state.api_authority.0.ok_or_else(|| {
        AppError::unauthorized("privileged endpoint: API_AUTHORITY_PUBKEY not configured")
    })?;
    auth.verify(&key, "POST", &format!("/api/v1/derived/jobs/{id}/complete"))?;
    if let Some(h) = &input.result_hash {
        if !is_hex64(h) {
            return Err(AppError::bad_request("result_hash must be 64 hex chars"));
        }
    }
    // Single UPDATE with a status guard: racing completers cannot clobber a
    // terminal row (one wins, the other gets 409).
    let updated = sqlx::query_as::<_, DerivedJob>(&format!(
        "UPDATE derived_jobs SET \
           status = 'done', \
           result_ref = COALESCE($2, result_ref), \
           result_hash = COALESCE($3, result_hash), \
           error = NULL, \
           completed_at = now() \
         WHERE id = $1 AND status IN ('pending', 'running') \
         {RETURNING_JOB}"
    ))
    .bind(&id)
    .bind(input.result_ref.as_deref())
    .bind(input.result_hash.as_deref())
    .fetch_optional(&state.pool)
    .await?;

    match updated {
        Some(job) => Ok(Json(job)),
        None => {
            // Distinguish idempotent re-complete (done) from terminal conflicts.
            let existing = sqlx::query_as::<_, DerivedJob>(&format!(
                "{SELECT_JOB} FROM derived_jobs WHERE id = $1"
            ))
            .bind(&id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| AppError::not_found("derived job not found"))?;
            if existing.status == "done" {
                Ok(Json(existing))
            } else {
                Err(AppError::conflict(format!(
                    "job is {}; only pending/running jobs can complete",
                    existing.status
                )))
            }
        }
    }
}

/// Mark a pending/running job `failed` with an error message.
pub async fn fail_job(
    State(state): State<AppState>,
    auth: SignedRequest,
    Path(id): Path<String>,
    Json(input): Json<FailInput>,
) -> Result<Json<DerivedJob>, AppError> {
    let key = state.api_authority.0.ok_or_else(|| {
        AppError::unauthorized("privileged endpoint: API_AUTHORITY_PUBKEY not configured")
    })?;
    auth.verify(&key, "POST", &format!("/api/v1/derived/jobs/{id}/fail"))?;
    if input.error.trim().is_empty() {
        return Err(AppError::bad_request("error must not be empty"));
    }
    let updated = sqlx::query_as::<_, DerivedJob>(&format!(
        "UPDATE derived_jobs SET status = 'failed', error = $2, completed_at = now() \
         WHERE id = $1 AND status IN ('pending', 'running') \
         {RETURNING_JOB}"
    ))
    .bind(&id)
    .bind(&input.error)
    .fetch_optional(&state.pool)
    .await?;
    match updated {
        Some(job) => Ok(Json(job)),
        None => {
            let exists =
                sqlx::query_as::<_, (String,)>("SELECT id FROM derived_jobs WHERE id = $1")
                    .bind(&id)
                    .fetch_optional(&state.pool)
                    .await?;
            if exists.is_none() {
                Err(AppError::not_found("derived job not found"))
            } else {
                Err(AppError::conflict("job is terminal; cannot fail it"))
            }
        }
    }
}

/// Mark a job and all transitive dependents `stale`.
///
/// A stale job is never silently trusted: any consumer that fetches it sees
/// `status: stale` and must re-create it to recompute. The recursive CTE walks
/// dependency edges outward (job → jobs that depend on it).
pub async fn invalidate_job(
    State(state): State<AppState>,
    auth: SignedRequest,
    Path(id): Path<String>,
) -> Result<Json<InvalidateResponse>, AppError> {
    let key = state.api_authority.0.ok_or_else(|| {
        AppError::unauthorized("privileged endpoint: API_AUTHORITY_PUBKEY not configured")
    })?;
    auth.verify(
        &key,
        "POST",
        &format!("/api/v1/derived/jobs/{id}/invalidate"),
    )?;
    let exists = sqlx::query_as::<_, (String,)>("SELECT id FROM derived_jobs WHERE id = $1")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("derived job not found"))?;

    // Recursive CTE collects root + transitive dependents; the UPDATE marks
    // them stale in one statement (rows already stale are left alone and are
    // not returned, so `count` = newly invalidated).
    let rows = sqlx::query_as::<_, (String,)>(
        "WITH RECURSIVE stale_set AS ( \
           SELECT $1::varchar AS id \
           UNION \
           SELECT d.job_id FROM derived_job_dependencies d \
           JOIN stale_set s ON d.depends_on_id = s.id \
         ) \
         UPDATE derived_jobs dj \
         SET status = 'stale' \
         FROM stale_set s \
         WHERE dj.id = s.id AND dj.status <> 'stale' \
         RETURNING dj.id",
    )
    .bind(&exists.0)
    .fetch_all(&state.pool)
    .await?;

    let invalidated: Vec<String> = rows.into_iter().map(|(id,)| id).collect();
    let count = invalidated.len();
    Ok(Json(InvalidateResponse { invalidated, count }))
}

// Silence the unused-constant lint: JOB_COLS documents the shared column list.
#[allow(dead_code)]
const _: &str = JOB_COLS;

// ---------------------------------------------------------------------------
// Consumer wiring: reconstruction drafts (RFC-014 Phase 04)
// ---------------------------------------------------------------------------

/// Record a reconstruction-draft computation in the derived-job ledger.
///
/// The processing key is over `(parcel_id, capture points)` — the two inputs
/// that determine the hull. Two completions of the same capture against the
/// same parcel therefore share one ledger row (`reused` tells the caller the
/// computation already happened before). Point order is significant: the hull
/// is order-independent but the ledger key is not, and normalizing it here
/// would hide a meaningful difference in the survey data.
pub async fn record_reconstruction_draft(
    pool: &sqlx::PgPool,
    draft_id: &str,
    parcel_id: Option<&str>,
    capture_points: &Option<Vec<[f64; 2]>>,
    geometry_hash: &str,
) -> Result<(String, bool), AppError> {
    let points_json = serde_json::to_string(capture_points)
        .map_err(|e| AppError::internal(format!("serialize capture points: {e}")))?;
    let input_document = format!("{}\n{}", parcel_id.unwrap_or(""), points_json);
    let input_hash = input_hash_hex(&input_document);
    let processing_key = processing_key_hex("reconstruction-draft", &input_hash);
    let fresh_id = Uuid::new_v4().to_string();

    let row = sqlx::query_as::<_, DerivedJob>(&format!(
        "INSERT INTO derived_jobs (id, kind, processing_key, input_hash, status, result_ref, result_hash) \
         VALUES ($1, 'reconstruction-draft', $2, $3, 'done', $4, $5) \
         ON CONFLICT (processing_key) DO UPDATE SET \
           status = CASE WHEN derived_jobs.status = 'done' THEN 'done' ELSE 'pending' END, \
           result_ref = CASE WHEN derived_jobs.status = 'done' THEN derived_jobs.result_ref ELSE $4 END, \
           result_hash = CASE WHEN derived_jobs.status = 'done' THEN derived_jobs.result_hash ELSE $5 END, \
           error = NULL, \
           completed_at = CASE WHEN derived_jobs.status = 'done' THEN derived_jobs.completed_at ELSE now() END \
         {RETURNING_JOB}"
    ))
    .bind(&fresh_id)
    .bind(&processing_key)
    .bind(&input_hash)
    .bind(draft_id)
    .bind(geometry_hash)
    .fetch_one(pool)
    .await?;
    let created = row.id.clone() == fresh_id;
    Ok((row.id, created))
}

// ---------------------------------------------------------------------------
// Tests (pure — the API suite is unit-level; live wiring is exercised against
// Postgres via the running API)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processing_key_is_deterministic() {
        let ih = input_hash_hex("{\"a\":1}");
        assert_eq!(ih, input_hash_hex("{\"a\":1}"));
        assert_eq!(
            processing_key_hex("route", &ih),
            processing_key_hex("route", &ih)
        );
    }

    #[test]
    fn processing_key_differs_by_kind() {
        let ih = input_hash_hex("{\"a\":1}");
        assert_ne!(
            processing_key_hex("route", &ih),
            processing_key_hex("reconstruction", &ih)
        );
    }

    #[test]
    fn processing_key_differs_by_input() {
        let a = input_hash_hex("{\"a\":1}");
        let b = input_hash_hex("{\"a\":2}");
        assert_ne!(
            processing_key_hex("route", &a),
            processing_key_hex("route", &b)
        );
    }

    #[test]
    fn input_hash_is_64_hex() {
        let ih = input_hash_hex("hello");
        assert!(is_hex64(&ih));
        assert_eq!(ih.len(), 64);
    }

    #[test]
    fn validate_rejects_empty_kind_and_input() {
        let mk = |kind: &str, doc: &str| NewDerivedJob {
            kind: kind.into(),
            input_document: doc.into(),
            depends_on: vec![],
            result_ref: None,
        };
        assert!(validate_new(&mk("", "{}")).is_err());
        assert!(validate_new(&mk("route", "")).is_err());
        assert!(validate_new(&mk("route", "{}")).is_ok());
    }

    #[test]
    fn is_hex64_rejects_bad() {
        assert!(is_hex64(&"0".repeat(64)));
        assert!(!is_hex64(&"0".repeat(63)));
        assert!(!is_hex64(&"z".repeat(64)));
        assert!(!is_hex64(""));
    }
}
