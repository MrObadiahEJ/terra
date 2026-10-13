//! 3D reconstruction scaffolding (RFC-013 Phase C).
//!
//! Durable records for the photogrammetry pipeline: capture sessions (photo /
//! video surveys of a parcel), reconstruction jobs (queued work that turns a
//! session into a mesh), and the resulting 3D models. The heavy worker that
//! actually runs photogrammetry lives outside the API; here we only persist
//! the queue and expose status. Model payloads stay off-chain (Design Rule 10)
//! — only URI + content hash are stored, the hash can be anchored on-chain
//! through EvidenceManifest artifacts.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::SignedRequest;
use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(list_capture_sessions).post(create_capture_session))
        .route("/jobs", post(create_reconstruction_job))
        .route("/jobs/lease", post(lease_reconstruction_job))
        .route("/jobs/{id}", get(get_reconstruction_job))
        .route("/jobs/{id}/progress", post(job_progress))
        .route("/jobs/{id}/complete", post(job_complete))
        .route("/jobs/{id}/draft", get(get_job_draft))
        .route("/drafts/{id}/anchor", post(draft_anchor_confirmed))
        .route("/models", get(list_three_d_models))
        .route("/models/{id}", get(get_three_d_model))
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CaptureSession {
    pub id: String,
    pub parcel_id: Option<String>,
    pub title: Option<String>,
    pub captured_by: Option<String>,
    pub captured_at: Option<chrono::DateTime<chrono::Utc>>,
    pub media_count: i32,
    pub evidence_refs: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ReconstructionJob {
    pub id: String,
    pub capture_session_id: String,
    pub status: String,
    pub progress: f32,
    pub error: Option<String>,
    pub artifact_uri: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ThreeDModel {
    pub id: String,
    pub parcel_id: Option<String>,
    pub reconstruction_job_id: Option<String>,
    pub uri: String,
    pub format: Option<String>,
    pub content_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ParcelQuery {
    pub parcel_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewCaptureSession {
    pub parcel_id: Option<String>,
    pub title: Option<String>,
    pub evidence_refs: Option<Vec<String>>,
    pub captured_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct NewReconstructionJob {
    pub capture_session_id: String,
}

const SELECT_SESSION: &str = "SELECT id, parcel_id, title, captured_by, captured_at, media_count, evidence_refs, created_at";
const SELECT_JOB: &str = "SELECT id, capture_session_id, status, progress, error, artifact_uri, created_at, updated_at";
const SELECT_MODEL: &str = "SELECT id, parcel_id, reconstruction_job_id, uri, format, content_hash, created_at";

// ---------------------------------------------------------------------------
// Capture sessions
// ---------------------------------------------------------------------------

fn caller_wallet(headers: axum::http::HeaderMap) -> Option<String> {
    headers
        .get("x-wallet")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

pub async fn create_capture_session(
    State(state): State<AppState>,
    auth: SignedRequest,
    headers: axum::http::HeaderMap,
    Json(input): Json<NewCaptureSession>,
) -> Result<(StatusCode, Json<CaptureSession>), AppError> {
    // The signed request proves *a* key signed the body; if the caller also
    // sent x-wallet we verify it owns that key and record it as the surveyor.
    let mut captured_by: Option<String> = None;
    if let Some(wallet) = caller_wallet(headers) {
        auth.verify_wallet(&wallet, "POST", "/api/v1/reconstruction/sessions")?;
        captured_by = Some(wallet);
    }

    let id = Uuid::new_v4().to_string();
    let media_count = input.evidence_refs.as_ref().map(|r| r.len() as i32).unwrap_or(0);

    let row = sqlx::query_as::<_, CaptureSession>(&format!(
        "INSERT INTO capture_sessions (id, parcel_id, title, captured_by, captured_at, media_count, evidence_refs)
         VALUES ($1, $2, $3, $4, COALESCE($5, now()), $6, $7)
         RETURNING id, parcel_id, title, captured_by, captured_at, media_count, evidence_refs, created_at"
    ))
    .bind(&id)
    .bind(&input.parcel_id)
    .bind(&input.title)
    .bind(&captured_by)
    .bind(input.captured_at)
    .bind(media_count)
    .bind(input.evidence_refs.unwrap_or_default())
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn list_capture_sessions(
    State(state): State<AppState>,
    Query(params): Query<ParcelQuery>,
) -> Result<Json<Vec<CaptureSession>>, AppError> {
    let rows = match params.parcel_id {
        Some(parcel) => {
            sqlx::query_as::<_, CaptureSession>(&format!(
                "{SELECT_SESSION} FROM capture_sessions WHERE parcel_id = $1 ORDER BY captured_at DESC LIMIT 100"
            ))
            .bind(parcel)
            .fetch_all(&state.pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, CaptureSession>(&format!(
                "{SELECT_SESSION} FROM capture_sessions ORDER BY created_at DESC LIMIT 100"
            ))
            .fetch_all(&state.pool)
            .await?
        }
    };
    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// Reconstruction jobs
// ---------------------------------------------------------------------------

pub async fn create_reconstruction_job(
    State(state): State<AppState>,
    _auth: SignedRequest,
    Json(input): Json<NewReconstructionJob>,
) -> Result<(StatusCode, Json<ReconstructionJob>), AppError> {
    // Session must exist (foreign key would also catch this, but a clear 404
    // beats a database error surface).
    let exists: Option<String> =
        sqlx::query_scalar("SELECT id FROM capture_sessions WHERE id = $1")
            .bind(&input.capture_session_id)
            .fetch_optional(&state.pool)
            .await?;
    if exists.is_none() {
        return Err(AppError::not_found("capture session not found"));
    }

    let id = Uuid::new_v4().to_string();
    let row = sqlx::query_as::<_, ReconstructionJob>(
        "INSERT INTO reconstruction_jobs (id, capture_session_id, status, progress)
         VALUES ($1, $2, 'queued', 0)
         RETURNING id, capture_session_id, status, progress, error, artifact_uri, created_at, updated_at",
    )
    .bind(&id)
    .bind(&input.capture_session_id)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn get_reconstruction_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ReconstructionJob>, AppError> {
    let row = sqlx::query_as::<_, ReconstructionJob>(&format!(
        "{SELECT_JOB} FROM reconstruction_jobs WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("reconstruction job not found"))?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// 3D models
// ---------------------------------------------------------------------------

pub async fn list_three_d_models(
    State(state): State<AppState>,
    Query(params): Query<ParcelQuery>,
) -> Result<Json<Vec<ThreeDModel>>, AppError> {
    let rows = match params.parcel_id {
        Some(parcel) => {
            sqlx::query_as::<_, ThreeDModel>(&format!(
                "{SELECT_MODEL} FROM three_d_models WHERE parcel_id = $1 ORDER BY created_at DESC LIMIT 100"
            ))
            .bind(parcel)
            .fetch_all(&state.pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, ThreeDModel>(&format!(
                "{SELECT_MODEL} FROM three_d_models ORDER BY created_at DESC LIMIT 100"
            ))
            .fetch_all(&state.pool)
            .await?
        }
    };
    Ok(Json(rows))
}

pub async fn get_three_d_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ThreeDModel>, AppError> {
    let row = sqlx::query_as::<_, ThreeDModel>(&format!("{SELECT_MODEL} FROM three_d_models WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("3D model not found"))?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Worker pipeline (RFC-013 Phase D): lease → progress → complete (draft +
// anchor prep) → anchor-confirmed
// ---------------------------------------------------------------------------

/// Canonical registry program on-chain (devnet/localnet share this ID).
const REGISTRY_PROGRAM_ID: &str = "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage";
/// How long a worker keeps its lease on a job before others may reclaim it.
const LEASE_TTL_SECS: i64 = 15 * 60;

const SELECT_DRAFT: &str = "SELECT id, reconstruction_job_id, parcel_id, method, geometry_geojson, geometry_hash, storage_reference, source, dimension, elevation_min_mm, elevation_max_mm, elevation_source, anchor_status, anchor_signature, created_at";

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct GeometryDraft {
    pub id: String,
    pub reconstruction_job_id: String,
    pub parcel_id: Option<String>,
    pub method: String,
    pub geometry_geojson: String,
    pub geometry_hash: String,
    pub storage_reference: String,
    pub source: i16,
    pub dimension: i16,
    pub elevation_min_mm: i32,
    pub elevation_max_mm: i32,
    pub elevation_source: i16,
    pub anchor_status: String,
    pub anchor_signature: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct LeasedJob {
    pub job: ReconstructionJob,
    pub session: CaptureSession,
}

#[derive(Debug, Deserialize)]
pub struct ProgressInput {
    /// 0.0 .. 1.0
    pub progress: f32,
}

#[derive(Debug, Deserialize)]
pub struct CompleteInput {
    /// Where the finished artifact (tileset / mesh) lives, if the pipeline
    /// produced one. Draft-only pipelines may omit it.
    pub artifact_uri: Option<String>,
    /// Content hash of the artifact (hex), recorded with the model row.
    pub content_hash: Option<String>,
    /// Artifact format label, e.g. "3dtiles" / "glb".
    pub format: Option<String>,
    /// [lon, lat] capture points extracted from the survey (EXIF etc.).
    /// With ≥3 points the worker drafts the convex hull (clipped to the
    /// parcel when its geometry exists); with fewer it falls back to the
    /// parcel footprint so the draft is still honest about its method.
    pub capture_points: Option<Vec<[f64; 2]>>,
    pub elevation_min_mm: Option<i32>,
    pub elevation_max_mm: Option<i32>,
    pub elevation_source: Option<i16>,
    /// geometry_source code for the draft (default PHOTOGRAMMETRY=5).
    pub source: Option<i16>,
    /// spatial_dimension code for the draft (default D2_5=1).
    pub dimension: Option<i16>,
}

#[derive(Debug, Serialize)]
pub struct CompleteResponse {
    pub job: ReconstructionJob,
    pub model: Option<ThreeDModel>,
    pub draft: GeometryDraft,
    /// Ready-to-submit `append_geometry_version` payload; null when the
    /// session's parcel has no on-chain identity yet.
    pub anchor_prep: Option<AnchorPrep>,
}

#[derive(Debug, Serialize)]
pub struct AnchorPrep {
    pub program_id: String,
    pub instruction: String,
    pub accounts: AnchorAccounts,
    pub next_version: u32,
    pub args: AnchorArgs,
}

#[derive(Debug, Serialize)]
pub struct AnchorAccounts {
    pub asset: String,
    pub geometry_version: String,
    pub payer: String,
    pub system_program: String,
}

#[derive(Debug, Serialize)]
pub struct AnchorArgs {
    /// Hex SHA-256 of the draft's canonical geometry document (the same
    /// bytes that were hashed into `geometry_hash`).
    pub geometry_hash: String,
    pub source: u8,
    pub dimension: u8,
    pub storage_reference: String,
    pub elevation_min_mm: i32,
    pub elevation_max_mm: i32,
    pub elevation_source: u8,
}

#[derive(Debug, Deserialize)]
pub struct AnchorConfirmedInput {
    pub signature: String,
}

fn registry_program_id() -> solana_sdk::pubkey::Pubkey {
    REGISTRY_PROGRAM_ID
        .parse()
        .expect("registry program id is valid base58")
}

/// Resolve a session's free-form `parcel_id` against the parcels table: it
/// may be the UUID primary key, the on-chain id (hex32), or the base58
/// address of the on-chain id bytes.
///
/// A dashed UUID is unambiguous; a bare 64-char hex string is treated as an
/// on-chain id (the UUID simple form is never stored — `gen_random_uuid()`
/// text output always carries dashes).
fn parse_parcel_lookup(parcel_id: &str) -> (Option<uuid::Uuid>, Option<String>) {
    if parcel_id.contains('-') {
        if let Ok(u) = uuid::Uuid::parse_str(parcel_id) {
            return (Some(u), None);
        }
    }
    if parcel_id.len() == 64 && parcel_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return (None, Some(parcel_id.to_lowercase()));
    }
    if let Ok(bytes) = bs58::decode(parcel_id).into_vec() {
        if bytes.len() == 32 {
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            return (None, Some(hex));
        }
    }
    (None, Some(parcel_id.to_string()))
}

async fn find_parcel_geometry(
    pool: &sqlx::PgPool,
    parcel_id: &str,
) -> Result<Option<(Option<String>, Option<String>)>, AppError> {
    let (as_uuid, onchain) = parse_parcel_lookup(parcel_id);
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT onchain_id, ST_AsGeoJSON(geometry)::text FROM parcels
         WHERE ($1::uuid IS NOT NULL AND id = $1)
            OR ($2::text IS NOT NULL AND onchain_id = $2)
         LIMIT 1",
    )
    .bind(as_uuid)
    .bind(onchain)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Build the draft geometry: convex hull of ≥3 capture points, clipped to the
/// parcel footprint when available; otherwise the parcel footprint itself.
/// The `method` label records exactly which of the two ran — the draft is an
/// honest intermediate, never presented as photogrammetry output.
async fn build_draft_geometry(
    pool: &sqlx::PgPool,
    capture_points: &Option<Vec<[f64; 2]>>,
    parcel_geom_json: Option<&str>,
) -> Result<(String, String), AppError> {
    let points = capture_points.as_deref().unwrap_or(&[]);
    let has_parcel = parcel_geom_json.is_some();

    if points.len() >= 3 {
        let lons: Vec<f64> = points.iter().map(|p| p[0]).collect();
        let lats: Vec<f64> = points.iter().map(|p| p[1]).collect();
        let geo: Option<String> = match parcel_geom_json {
            Some(pg) => {
                sqlx::query_scalar(
                    "WITH pts AS (
                         SELECT ST_SetSRID(ST_MakePoint(x, y), 4326) AS geom
                         FROM unnest($1::float8[], $2::float8[]) AS t(x, y)
                     ),
                     hull AS (SELECT ST_ConvexHull(ST_Collect(geom)) AS g FROM pts),
                     parcel AS (SELECT ST_SetSRID(ST_GeomFromGeoJSON($3), 4326) AS g)
                     SELECT ST_AsGeoJSON(ST_CollectionExtract(ST_Intersection(hull.g, parcel.g), 3))::text
                     FROM hull, parcel",
                )
                .bind(&lons)
                .bind(&lats)
                .bind(pg)
                .fetch_optional(pool)
                .await?
            }
            None => {
                sqlx::query_scalar(
                    "WITH pts AS (
                         SELECT ST_SetSRID(ST_MakePoint(x, y), 4326) AS geom
                         FROM unnest($1::float8[], $2::float8[]) AS t(x, y)
                     )
                     SELECT ST_AsGeoJSON(ST_ConvexHull(ST_Collect(geom)))::text FROM pts",
                )
                .bind(&lons)
                .bind(&lats)
                .fetch_optional(pool)
                .await?
            }
        };
        let geo = geo.filter(|g| !g.is_empty() && g != "null");
        return match geo {
            Some(g) => Ok((g, "capture-hull".to_string())),
            None if has_parcel => Err(AppError::bad_request(
                "capture hull does not intersect the parcel footprint",
            )),
            None => Err(AppError::bad_request(
                "capture points do not form a valid convex hull",
            )),
        };
    }

    // Fewer than 3 points: fall back to the parcel footprint (honest label).
    match parcel_geom_json {
        Some(g) => Ok((g.to_string(), "parcel-footprint-fallback".to_string())),
        None => Err(AppError::bad_request(
            "need ≥3 capture points or a parcel geometry to draft from",
        )),
    }
}

fn canonical_geometry_hash(geojson: &str) -> String {
    use sha2::Digest;
    format!("{:x}", sha2::Sha256::digest(geojson.as_bytes()))
}

fn hex32_to_pubkey(hex: &str) -> Option<solana_sdk::pubkey::Pubkey> {
    let bytes = hex::decode(hex).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let arr: [u8; 32] = bytes.try_into().ok()?;
    Some(solana_sdk::pubkey::Pubkey::new_from_array(arr))
}

/// Derive the `append_geometry_version` account set + args for a draft.
/// `geometry_version_count` comes from the mirrored on-chain asset row.
fn build_anchor_prep(
    onchain_asset_hex: Option<&str>,
    geometry_version_count: i32,
    draft: &GeometryDraft,
    payer: &str,
) -> Option<AnchorPrep> {
    let asset_bytes = hex32_to_pubkey(onchain_asset_hex?)?;
    let program = registry_program_id();
    let (asset_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[b"spatial_asset", asset_bytes.as_ref()],
        &program,
    );
    let next_version = geometry_version_count.max(0) as u32;
    let (gv_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[
            b"geometry_version",
            asset_pda.as_ref(),
            &next_version.to_le_bytes(),
        ],
        &program,
    );
    Some(AnchorPrep {
        program_id: REGISTRY_PROGRAM_ID.to_string(),
        instruction: "append_geometry_version".to_string(),
        accounts: AnchorAccounts {
            asset: asset_pda.to_string(),
            geometry_version: gv_pda.to_string(),
            payer: payer.to_string(),
            system_program: "11111111111111111111111111111111".to_string(),
        },
        next_version,
        args: AnchorArgs {
            geometry_hash: draft.geometry_hash.clone(),
            source: draft.source as u8,
            dimension: draft.dimension as u8,
            storage_reference: draft.storage_reference.clone(),
            elevation_min_mm: draft.elevation_min_mm,
            elevation_max_mm: draft.elevation_max_mm,
            elevation_source: draft.elevation_source as u8,
        },
    })
}

/// Atomically claim the oldest queued job (or reclaim an expired lease).
pub async fn lease_reconstruction_job(
    State(state): State<AppState>,
    _auth: SignedRequest,
    headers: axum::http::HeaderMap,
) -> Result<Json<LeasedJob>, AppError> {
    let claimed_by = caller_wallet(headers);

    // Reclaim expired leases first so a crashed worker does not strand jobs.
    sqlx::query(
        "UPDATE reconstruction_jobs SET status = 'queued', claimed_by = NULL,
             claimed_at = NULL, lease_expires_at = NULL
         WHERE status = 'running' AND lease_expires_at IS NOT NULL AND lease_expires_at < now()",
    )
    .execute(&state.pool)
    .await?;

    let claimed: Option<String> = sqlx::query_scalar(
        "UPDATE reconstruction_jobs
         SET status = 'running', claimed_by = $1, claimed_at = now(),
             lease_expires_at = now() + make_interval(secs => $2::double precision)
         WHERE id = (
             SELECT id FROM reconstruction_jobs WHERE status = 'queued'
             ORDER BY created_at ASC LIMIT 1
             FOR UPDATE SKIP LOCKED
         )
         RETURNING id",
    )
    .bind(&claimed_by)
    .bind(LEASE_TTL_SECS as f64)
    .fetch_optional(&state.pool)
    .await?;

    let Some(job_id) = claimed else {
        return Err(AppError::not_found("no queued reconstruction jobs"));
    };

    let job = sqlx::query_as::<_, ReconstructionJob>(&format!(
        "{SELECT_JOB} FROM reconstruction_jobs WHERE id = $1"
    ))
    .bind(&job_id)
    .fetch_one(&state.pool)
    .await?;

    let session = sqlx::query_as::<_, CaptureSession>(&format!(
        "{SELECT_SESSION} FROM capture_sessions WHERE id = $1"
    ))
    .bind(&job.capture_session_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(LeasedJob { job, session }))
}

pub async fn job_progress(
    State(state): State<AppState>,
    _auth: SignedRequest,
    Path(id): Path<String>,
    Json(input): Json<ProgressInput>,
) -> Result<Json<ReconstructionJob>, AppError> {
    if !(0.0..=1.0).contains(&input.progress) {
        return Err(AppError::bad_request("progress must be within 0.0..=1.0"));
    }
    let running = sqlx::query_scalar::<_, String>(
        "UPDATE reconstruction_jobs SET progress = $2
         WHERE id = $1 AND status = 'running'
         RETURNING id",
    )
    .bind(&id)
    .bind(input.progress)
    .fetch_optional(&state.pool)
    .await?;
    if running.is_none() {
        return Err(AppError::conflict("job is not running (lease it first)"));
    }
    let job = sqlx::query_as::<_, ReconstructionJob>(&format!(
        "{SELECT_JOB} FROM reconstruction_jobs WHERE id = $1"
    ))
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(job))
}

pub async fn job_complete(
    State(state): State<AppState>,
    _auth: SignedRequest,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<CompleteInput>,
) -> Result<(StatusCode, Json<CompleteResponse>), AppError> {
    let wallet = caller_wallet(headers);

    // Must hold the lease (running); queued → conflict, missing → 404.
    let job = sqlx::query_as::<_, ReconstructionJob>(&format!(
        "{SELECT_JOB} FROM reconstruction_jobs WHERE id = $1"
    ))
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("reconstruction job not found"))?;
    if job.status != "running" {
        return Err(AppError::conflict("job is not running (lease it first)"));
    }

    let session = sqlx::query_as::<_, CaptureSession>(&format!(
        "{SELECT_SESSION} FROM capture_sessions WHERE id = $1"
    ))
    .bind(&job.capture_session_id)
    .fetch_one(&state.pool)
    .await?;

    // --- draft geometry (the honest intermediate) ------------------------
    let parcel = match &session.parcel_id {
        Some(pid) => find_parcel_geometry(&state.pool, pid).await?,
        None => None,
    };
    let (parcel_geom, onchain_hex) = parcel.map(|(o, g)| (g, o)).unwrap_or_default();
    let (geojson, method) = build_draft_geometry(
        &state.pool,
        &input.capture_points,
        parcel_geom.as_deref(),
    )
    .await?;
    let geometry_hash = canonical_geometry_hash(&geojson);
    let storage_reference = format!("recon://{id}/draft.geojson");
    let draft_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO geometry_drafts (
             id, reconstruction_job_id, parcel_id, method, geometry_geojson,
             geometry_hash, storage_reference, source, dimension,
             elevation_min_mm, elevation_max_mm, elevation_source
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(&draft_id)
    .bind(&id)
    .bind(&session.parcel_id)
    .bind(&method)
    .bind(&geojson)
    .bind(&geometry_hash)
    .bind(&storage_reference)
    .bind(input.source.unwrap_or(5))
    .bind(input.dimension.unwrap_or(1))
    .bind(input.elevation_min_mm.unwrap_or(0))
    .bind(input.elevation_max_mm.unwrap_or(0))
    .bind(input.elevation_source.unwrap_or(0))
    .execute(&state.pool)
    .await?;

    let draft = sqlx::query_as::<_, GeometryDraft>(&format!(
        "{SELECT_DRAFT} FROM geometry_drafts WHERE id = $1"
    ))
    .bind(&draft_id)
    .fetch_one(&state.pool)
    .await?;

    // --- pipeline product (model row), when the worker produced one ------
    let model = match &input.artifact_uri {
        Some(uri) => {
            let model_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO three_d_models (id, parcel_id, reconstruction_job_id, uri, format, content_hash)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&model_id)
            .bind(&session.parcel_id)
            .bind(&id)
            .bind(uri)
            .bind(&input.format)
            .bind(&input.content_hash)
            .execute(&state.pool)
            .await?;
            Some(
                sqlx::query_as::<_, ThreeDModel>(&format!(
                    "{SELECT_MODEL} FROM three_d_models WHERE id = $1"
                ))
                .bind(&model_id)
                .fetch_one(&state.pool)
                .await?,
            )
        }
        None => None,
    };

    // --- finish the job ---------------------------------------------------
    sqlx::query(
        "UPDATE reconstruction_jobs
         SET status = 'done', progress = 1.0, artifact_uri = $2,
             lease_expires_at = NULL
         WHERE id = $1",
    )
    .bind(&id)
    .bind(&input.artifact_uri)
    .execute(&state.pool)
    .await?;

    let job = sqlx::query_as::<_, ReconstructionJob>(&format!(
        "{SELECT_JOB} FROM reconstruction_jobs WHERE id = $1"
    ))
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;

    // --- anchor prep (what the wallet must sign next) --------------------
    // The mirror keys the asset by its PDA; look it up via the parcel's
    // on-chain pubkey (base58) so `next_version` is the real live cursor.
    let next_cursor = match &onchain_hex {
        Some(hex) => match hex32_to_pubkey(hex) {
            Some(parcel_pk) => {
                let cursor: Option<i32> = sqlx::query_scalar(
                    "SELECT geometry_version_count FROM spatial_assets_onchain WHERE parcel_pubkey = $1",
                )
                .bind(parcel_pk.to_string())
                .fetch_optional(&state.pool)
                .await?;
                Some(cursor.unwrap_or(0))
            }
            None => None,
        },
        None => None,
    };
    let anchor_prep = match (onchain_hex.as_deref(), next_cursor) {
        (Some(hex), Some(cursor)) => {
            let payer = wallet
                .as_deref()
                .unwrap_or("11111111111111111111111111111111");
            build_anchor_prep(Some(hex), cursor, &draft, payer)
        }
        _ => None,
    };

    Ok((
        StatusCode::OK,
        Json(CompleteResponse {
            job,
            model,
            draft,
            anchor_prep,
        }),
    ))
}

pub async fn get_job_draft(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GeometryDraft>, AppError> {
    let row = sqlx::query_as::<_, GeometryDraft>(&format!(
        "{SELECT_DRAFT} FROM geometry_drafts
         WHERE reconstruction_job_id = $1 ORDER BY created_at DESC LIMIT 1"
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("no geometry draft for this job"))?;
    Ok(Json(row))
}

/// Record that the draft's `append_geometry_version` landed on-chain — the
/// draft becomes an anchored claim (fact once verified by validators).
pub async fn draft_anchor_confirmed(
    State(state): State<AppState>,
    _auth: SignedRequest,
    Path(id): Path<String>,
    Json(input): Json<AnchorConfirmedInput>,
) -> Result<Json<GeometryDraft>, AppError> {
    if input.signature.trim().is_empty() {
        return Err(AppError::bad_request("signature must not be empty"));
    }
    let updated = sqlx::query_scalar::<_, String>(
        "UPDATE geometry_drafts SET anchor_status = 'anchored', anchor_signature = $2
         WHERE id = $1 AND anchor_status = 'pending_anchor'
         RETURNING id",
    )
    .bind(&id)
    .bind(&input.signature)
    .fetch_optional(&state.pool)
    .await?;
    if updated.is_none() {
        return Err(AppError::conflict(
            "draft not found or already confirmed/rejected",
        ));
    }
    let draft = sqlx::query_as::<_, GeometryDraft>(&format!(
        "{SELECT_DRAFT} FROM geometry_drafts WHERE id = $1"
    ))
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(draft))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_row_serializes() {
        let row = CaptureSession {
            id: "s-1".into(),
            parcel_id: Some("P-1".into()),
            title: Some("Courtyard survey".into()),
            captured_by: Some("wallet".into()),
            captured_at: Some(chrono::Utc::now()),
            media_count: 3,
            evidence_refs: vec!["e-1".into(), "e-2".into(), "e-3".into()],
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("capture_sessions") || json.contains("s-1"));
        assert!(json.contains("evidence_refs"));
        assert!(json.contains("media_count"));
    }

    #[test]
    fn job_row_serializes() {
        let row = ReconstructionJob {
            id: "j-1".into(),
            capture_session_id: "s-1".into(),
            status: "queued".into(),
            progress: 0.0,
            error: None,
            artifact_uri: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"status\":\"queued\""));
        assert!(json.contains("capture_session_id"));
    }

    #[test]
    fn model_row_serializes() {
        let row = ThreeDModel {
            id: "m-1".into(),
            parcel_id: Some("P-1".into()),
            reconstruction_job_id: Some("j-1".into()),
            uri: "s3://terra-models/P-1/tileset.json".into(),
            format: Some("3dtiles".into()),
            content_hash: Some("aabb".into()),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("tileset.json"));
        assert!(json.contains("content_hash"));
    }

    #[test]
    fn canonical_geometry_hash_is_sha256_of_exact_bytes() {
        use sha2::Digest;
        // The hash that will be anchored on-chain must be reproducible from
        // the stored GeoJSON document alone — no re-serialization allowed.
        let geo = r#"{"type":"Polygon","coordinates":[[[8.5,47.4],[8.6,47.4],[8.6,47.5],[8.5,47.4]]]}"#;
        let expected = format!("{:x}", sha2::Sha256::digest(geo.as_bytes()));
        assert_eq!(canonical_geometry_hash(geo), expected);
        assert_ne!(canonical_geometry_hash(geo), canonical_geometry_hash(&geo.to_uppercase()));
    }

    #[test]
    fn parse_parcel_lookup_handles_uuid_hex_and_base58() {
        let (u, h) = parse_parcel_lookup("550e8400-e29b-41d4-a716-446655440000");
        assert!(u.is_some() && h.is_none());

        let hex32 = "ab".repeat(32);
        let (u, h) = parse_parcel_lookup(&hex32);
        assert!(u.is_none());
        assert_eq!(h.as_deref(), Some(hex32.as_str()));

        // base58 of 32 zero bytes → hex of the same bytes.
        let (u, h) = parse_parcel_lookup("11111111111111111111111111111111");
        assert!(u.is_none());
        assert_eq!(h.as_deref(), Some(&"00".repeat(32)[..]));

        // Anything else passes through as an opaque onchain_id text match.
        let (u, h) = parse_parcel_lookup("SpAtDemo1111111111111111111111111111111111");
        assert!(u.is_none());
        assert_eq!(h.as_deref(), Some("SpAtDemo1111111111111111111111111111111111"));
    }

    #[test]
    fn hex32_to_pubkey_rejects_bad_lengths() {
        assert!(hex32_to_pubkey("abcd").is_none());
        assert!(hex32_to_pubkey(&"zz".repeat(32)).is_none());
        let pk = hex32_to_pubkey(&"11".repeat(32)).expect("valid hex32");
        assert_eq!(pk.to_string().len() >= 32, true);
    }

    #[test]
    fn anchor_prep_derives_expected_account_seeds() {
        use solana_sdk::pubkey::Pubkey;

        // Parcel pubkey = all 0x11; asset PDA must derive from
        // ["spatial_asset", parcel] against the registry program.
        let parcel_hex = "11".repeat(32);
        let draft = GeometryDraft {
            id: "d-1".into(),
            reconstruction_job_id: "j-1".into(),
            parcel_id: None,
            method: "capture-hull".into(),
            geometry_geojson: "{}".into(),
            geometry_hash: "aa".repeat(32),
            storage_reference: "recon://j-1/draft.geojson".into(),
            source: 5,
            dimension: 1,
            elevation_min_mm: 0,
            elevation_max_mm: 0,
            elevation_source: 0,
            anchor_status: "pending_anchor".into(),
            anchor_signature: None,
            created_at: chrono::Utc::now(),
        };
        let prep = build_anchor_prep(Some(&parcel_hex), 3, &draft, "Wallet111")
            .expect("hex32 parcel resolves");
        assert_eq!(prep.instruction, "append_geometry_version");
        assert_eq!(prep.next_version, 3);
        assert_eq!(prep.args.geometry_hash, draft.geometry_hash);
        assert_eq!(prep.args.storage_reference, draft.storage_reference);

        let parcel_pk = Pubkey::new_from_array([0x11; 32]);
        let program: Pubkey = REGISTRY_PROGRAM_ID.parse().unwrap();
        let (asset, _) = Pubkey::find_program_address(&[b"spatial_asset", parcel_pk.as_ref()], &program);
        assert_eq!(prep.accounts.asset, asset.to_string());
        let (gv, _) = Pubkey::find_program_address(
            &[b"geometry_version", asset.as_ref(), &3u32.to_le_bytes()],
            &program,
        );
        assert_eq!(prep.accounts.geometry_version, gv.to_string());

        // Non-hex32 parcel → no prep (honest null, never a guessed account set).
        assert!(build_anchor_prep(Some("not-hex32"), 0, &draft, "w").is_none());
        assert!(build_anchor_prep(None, 0, &draft, "w").is_none());
    }
}
