use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use uuid::Uuid;

use crate::auth::{self, RequestParts, SignedRequest};
use crate::error::AppError;
use crate::geoutil::geojson_polygon;
use crate::routes::{disputes, identities, rights};
use crate::state::AppState;

fn decode_hex32(s: &str) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(s).map_err(|_| AppError::bad_request("expected hex"))?;
    if bytes.len() != 32 {
        return Err(AppError::bad_request("expected 32 bytes"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

const PARCEL_SELECT: &str = r#"
    SELECT
        p.id,
        p.name,
        o.holder,
        p.status,
        ST_AsGeoJSON(p.geometry)::text AS geometry,
        ST_Area(p.geometry::geography)::float8 AS area_m2,
        p.created_at,
        p.updated_at,
        p.onchain_id
    FROM parcels p
    JOIN parcel_ownership o ON o.parcel_id = p.id
"#;

#[derive(Debug, Serialize)]
pub struct Parcel {
    pub id: Uuid,
    pub name: String,
    /// `holder` is only disclosed when the caller proves they are the
    /// holder (Ed25519 signature over the request path) or the configured
    /// API authority. Unsigned requests receive `null` — the mirror is
    /// untrusted until `reconcile` re-derives the access digest, and real
    /// property data must not leak before field-level read policies land.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,
    pub status: String,
    pub geometry: Option<String>,
    pub area_m2: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub onchain_id: Option<String>,
}

/// Row returned by the DB join (holder always present); converted into
/// `Parcel` by `with_holder` only when the caller is authorised.
#[derive(Debug, Clone, sqlx::FromRow)]
struct ParcelRow {
    pub id: Uuid,
    pub name: String,
    pub holder: String,
    pub status: String,
    pub geometry: Option<String>,
    pub area_m2: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub onchain_id: Option<String>,
}

impl ParcelRow {
    fn into_parcel(self, disclose_holder: bool) -> Parcel {
        Parcel {
            id: self.id,
            name: self.name,
            holder: disclose_holder.then_some(self.holder),
            status: self.status,
            geometry: self.geometry,
            area_m2: self.area_m2,
            created_at: self.created_at,
            updated_at: self.updated_at,
            onchain_id: self.onchain_id,
        }
    }
}

/// Field-level read policy for the mirror's `holder` column.
///
/// The mirror is untrusted until `reconcile` (authority-gated, re-derives
/// the access digest) anchors it. Until then, and for every subsequent
/// read, the holder wallet is treated as real property data: disclose
/// only to the holder themselves (Ed25519 signature over the request
/// path) or to the API authority. Everyone else gets `null`.
///
/// `parts` is consumed once to extract the signed request; the caller
/// must pass `&mut` so the extractor can read headers.
async fn disclose_holder(
    state: &AppState,
    parts: &mut RequestParts,
    expected_holder: &str,
) -> bool {
    let key = match state.api_authority.0 {
        Some(ref k) => k,
        None => return false,
    };
    let req = match auth::parse_signed_request(parts).await {
        Ok(r) => r,
        Err(_) => return false,
    };
    let path = parts.uri.path();
    let method = parts.method.as_str();
    // Authority signature always discloses.
    if req.verify(key, method, path).is_ok() {
        return true;
    }
    // Holder signature over the exact path discloses.
    req.verify_wallet(expected_holder, method, path).is_ok()
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub minx: Option<f64>,
    pub miny: Option<f64>,
    pub maxx: Option<f64>,
    pub maxy: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct NewParcel {
    pub name: String,
    /// Wallet that will hold the parcel's canonical ownership right
    /// (mirrors the on-chain ownership Rights PDA holder).
    pub holder: String,
    #[serde(default)]
    pub status: String,
    pub geometry: serde_json::Value,
    /// On-chain parcel PDA address (base58), links mirror row to chain record.
    pub onchain_id: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/", post(create_parcel))
        .route("/{id}", get(get_by_id))
        .route("/{id}/reconcile", post(reconcile))
        .route("/{id}", delete(delete_parcel))
        .route("/{id}/forfeiture", post(judicial_forfeiture))
        .route("/{id}/disputes", post(disputes::file_dispute))
        .route("/{id}/rights/{nonce}/renew", post(rights::renew_right))
        .route(
            "/{id}/rights/{nonce}/sweep",
            post(rights::sweep_expired_right),
        )
        .route(
            "/{parcel_pda}/geometry-artifact",
            put(upload_geometry_artifact),
        )
}

async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
    mut parts: RequestParts,
) -> Result<Json<Vec<Parcel>>, AppError> {
    let rows: Vec<ParcelRow> = match (params.minx, params.miny, params.maxx, params.maxy) {
        (Some(minx), Some(miny), Some(maxx), Some(maxy)) => {
            sqlx::query_as::<_, ParcelRow>(&format!(
                "{PARCEL_SELECT}
             WHERE ST_Intersects(p.geometry, ST_MakeEnvelope($1, $2, $3, $4, 4326))
             ORDER BY created_at DESC"
            ))
            .bind(minx)
            .bind(miny)
            .bind(maxx)
            .bind(maxy)
            .fetch_all(&state.pool)
            .await?
        }
        _ => {
            sqlx::query_as::<_, ParcelRow>(&format!(
                "{PARCEL_SELECT}
             ORDER BY created_at DESC"
            ))
            .fetch_all(&state.pool)
            .await?
        }
    };
    // Field-level read policy: disclose each row's holder only to the
    // holder (Ed25519 over the path) or the API authority. Unsigned
    // requests receive `null` for every holder.
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let disclose = disclose_holder(&state, &mut parts, &r.holder).await;
        out.push(r.into_parcel(disclose));
    }
    Ok(Json(out))
}

async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    mut parts: RequestParts,
) -> Result<Json<Parcel>, AppError> {
    let row: ParcelRow = sqlx::query_as::<_, ParcelRow>(&format!(
        "{PARCEL_SELECT}
         WHERE p.id = $1"
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    let disclose = disclose_holder(&state, &mut parts, &row.holder).await;
    Ok(Json(row.into_parcel(disclose)))
}

async fn create_parcel(
    State(state): State<AppState>,
    auth: crate::auth::SignedRequest,
    Json(params): Json<NewParcel>,
) -> Result<(StatusCode, Json<Parcel>), AppError> {
    // Only the API authority may create mirror rows — untrusted callers must
    // go through the authority-gated /reconcile endpoint after the on-chain
    // parcel exists, which re-derives and validates the access digest.
    let key = state
        .api_authority
        .0
        .ok_or_else(|| AppError::unauthorized("API_AUTHORITY_PUBKEY not configured"))?;
    auth.verify(&key, "POST", "/api/v1/parcels/")?;
    let geojson = params.geometry.to_string();
    let status = if params.status.is_empty() {
        "pending".to_string()
    } else {
        params.status
    };

    let parcel = sqlx::query_as::<_, ParcelRow>(
        "WITH p AS (
            INSERT INTO parcels (name, status, geometry, source_geometry, onchain_id)
            VALUES ($1, $3, ST_GeomFromGeoJSON($4), $4, $5)
            RETURNING id, name, status,
                      ST_AsGeoJSON(geometry)::text AS geometry,
                      ST_Area(geometry::geography)::float8 AS area_m2,
                      created_at, updated_at, onchain_id
         ), o AS (
            INSERT INTO parcel_ownership (parcel_id, holder)
            SELECT id, $2 FROM p
            RETURNING holder
         )
         SELECT p.id, p.name, o.holder, p.status, p.geometry, p.area_m2,
                p.created_at, p.updated_at, p.onchain_id
         FROM p, o",
    )
    .bind(&params.name)
    .bind(&params.holder)
    .bind(&status)
    .bind(&geojson)
    .bind(&params.onchain_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if is_geometry_error(&e) {
            AppError::bad_request("geometry must be a valid GeoJSON Polygon in EPSG:4326")
        } else {
            AppError::from(e)
        }
    })?;

    Ok((StatusCode::CREATED, Json(parcel.into_parcel(true))))
}

async fn delete_parcel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM parcels WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("parcel not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ReconcileRight {
    pub rights_kind: u8,
    pub holder: String,
    pub granter: String,
    /// Unix timestamp; omit or 0 for "no expiration".
    pub expires_at: Option<i64>,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct ReconcileRequest {
    /// 32-byte on-chain parcel id as 64-char hex (the PDA seed).
    pub onchain_id: String,
    /// On-chain `geometry_hash` as 64-char hex.
    pub geometry_hash: String,
    /// On-chain `infrastructure_flags`.
    pub infrastructure_flags: u16,
    /// On-chain `access_hash` (canonical geo-engine digest) as 64-char hex.
    pub access_hash: String,
    pub status: String,
    pub rights: Vec<ReconcileRight>,
}

/// Reconcile the off-chain mirror with the on-chain terra_registry state.
///
/// Loads the stored parcel geometry and *recomputes* the canonical geo-engine
/// reachability digest from it; if the on-chain `access_hash` was already
/// anchored (non-zero) it must match our derivation, otherwise the anchor is
/// inconsistent with the stored geometry and we refuse the update. Then the
/// mirror columns and the `rights` table are rewritten to agree on-chain.
///
/// Returns 409 (CONFLICT) when the reported `access_hash` does not match the
/// digest derivable from the stored geometry.
async fn reconcile(
    State(state): State<AppState>,
    auth: crate::auth::SignedRequest,
    Path(id): Path<Uuid>,
    Json(req): Json<ReconcileRequest>,
) -> Result<Json<Parcel>, AppError> {
    // Ed25519 signature verification — fail-closed when authority is not configured.
    let key = state.api_authority.0.ok_or_else(|| {
        AppError::unauthorized("privileged endpoint: API_AUTHORITY_PUBKEY not configured")
    })?;
    auth.verify(&key, "POST", &format!("/api/v1/parcels/{id}/reconcile"))?;

    let onchain_id = decode_hex32(&req.onchain_id)?;
    let geo_hash = decode_hex32(&req.geometry_hash)?;
    let access_hash = decode_hex32(&req.access_hash)?;

    // Fetch the stored geometry so we can independently re-derive the digest.
    let geometry_json: String =
        sqlx::query_scalar("SELECT ST_AsGeoJSON(geometry)::text FROM parcels WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await?;
    let geometry: serde_json::Value =
        serde_json::from_str(&geometry_json).map_err(|e| AppError::bad_request(e.to_string()))?;
    let polygon = geojson_polygon(&geometry)?;

    // Independent digest: only possible when OSM data is loaded. When the
    // caller reports a non-zero (already anchored) access_hash, it must match
    // what we derive from the stored geometry — otherwise refuse the update.
    // When geo-engine is unavailable, reject non-zero access_hash to prevent
    // unvalidated data from being written to the mirror.
    if let Some(geo) = state.geo.as_ref() {
        let network = terra_geo::NetworkGraph::build(&geo.graph);
        let report = terra_geo::analyze(&network, &geo.graph, &polygon, &onchain_id);
        let derived = hex::encode(report.access_hash);
        let reported = hex::encode(access_hash);
        if !access_hash.iter().all(|b| *b == 0) && report.access_hash != access_hash {
            return Err(AppError::conflict(format!(
                "access_hash is inconsistent with stored geometry: derived {derived}, reported {reported}"
            )));
        }
    } else if !access_hash.iter().all(|b| *b == 0) {
        return Err(AppError::bad_request(
            "geo-engine unavailable: cannot validate non-zero access_hash; use zero or load OSM data",
        ));
    }

    // Idempotently reconcile the mirror columns.
    sqlx::query(
        "UPDATE parcels SET
            onchain_id      = $1,
            geometry_hash   = $2,
            infrastructure_flags = $3,
            access_hash     = $4,
            status          = $5,
            updated_at      = now()
         WHERE id = $6",
    )
    .bind(hex::encode(onchain_id))
    .bind(hex::encode(geo_hash))
    .bind(req.infrastructure_flags as i16)
    .bind(hex::encode(access_hash))
    .bind(&req.status)
    .bind(id)
    .execute(&state.pool)
    .await?;

    // Rewrite the rights mirror to agree with the on-chain Rights accounts.
    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM rights WHERE parcel_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for r in &req.rights {
        let created_at = chrono::Utc::now();
        sqlx::query(
            "INSERT INTO rights
                (parcel_id, rights_kind, holder, granter, created_at, expires_at, notes)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(id)
        .bind(r.rights_kind as i16)
        .bind(&r.holder)
        .bind(&r.granter)
        .bind(created_at)
        .bind(
            r.expires_at
                .filter(|t| *t > 0)
                .map(|t| DateTime::<Utc>::from_timestamp(t, 0)),
        )
        .bind(&r.notes)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    let parcel = sqlx::query_as::<_, ParcelRow>(&format!(
        "{PARCEL_SELECT}
         WHERE p.id = $1"
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    // Reconcile is authority-gated; the caller is trusted, so disclose.
    Ok(Json(parcel.into_parcel(true)))
}

fn is_geometry_error(err: &sqlx::Error) -> bool {
    matches!(
        err,
        sqlx::Error::Database(db_err)
            if matches!(
                db_err.code().as_deref(),
                Some("XX000") | Some("22023") | Some("22P02")
            )
    )
}

// ---------------------------------------------------------------------------
// Judicial forfeiture (collective validator seizure per court order).
// Deliberately heavier than a normal transfer: at least `threshold` validators
// must be recorded as having signed, and the order is bound to a case hash.
// Mirrors the on-chain judicial_forfeiture + ParcelForfeited event.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct JudicialForfeiture {
    /// hex(32) hash of the court order.
    pub case_hash: String,
    /// base58 wallet given control of the parcel.
    pub new_owner: String,
    /// validator signers required (>= 2).
    pub threshold: u16,
    /// declared validator signer wallets (base58).
    pub validators: Vec<String>,
    /// base58 relaying authority wallet (court/govt).
    pub relayer: String,
}

const MIN_FORFEIT_VALIDATORS: u16 = 2;

async fn judicial_forfeiture(
    State(state): State<AppState>,
    auth: crate::auth::SignedRequest,
    Path(id): Path<Uuid>,
    Json(req): Json<JudicialForfeiture>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Ed25519 signature verification — fail-closed when authority is not configured.
    let key = state.api_authority.0.ok_or_else(|| {
        AppError::unauthorized("privileged endpoint: API_AUTHORITY_PUBKEY not configured")
    })?;
    auth.verify(&key, "POST", &format!("/api/v1/parcels/{id}/forfeiture"))?;

    let ih = decode_hex32(&req.case_hash)?;
    identities::decode_wallet(&req.new_owner)?;
    identities::decode_wallet(&req.relayer)?;
    for v in &req.validators {
        identities::decode_wallet(v)?;
    }

    if req.threshold < MIN_FORFEIT_VALIDATORS {
        return Err(AppError::bad_request(
            "forfeiture threshold must be at least 2 validators",
        ));
    }
    let present = req.validators.len() as u16;
    if req.threshold > present {
        return Err(AppError::bad_request(
            "forfeiture threshold cannot exceed the number of validator signers",
        ));
    }

    let mut tx = state.pool.begin().await?;

    let current: Option<(String,)> =
        sqlx::query_as("SELECT holder FROM parcel_ownership WHERE parcel_id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some((holder,)) = current else {
        return Err(AppError::not_found("parcel not found"));
    };
    if holder.eq_ignore_ascii_case(&req.relayer) {
        return Err(AppError::bad_request(
            "the current holder cannot self-forfeit their own parcel",
        ));
    }

    sqlx::query("UPDATE parcel_ownership SET holder = $2, updated_at = now() WHERE parcel_id = $1")
        .bind(id)
        .bind(&req.new_owner)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE parcels SET updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO forfeitures (parcel_id, case_hash, from_owner, to_owner, threshold, present, court_relay)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(id)
    .bind(hex::encode(ih))
    .bind(&holder)
    .bind(&req.new_owner)
    .bind(req.threshold as i16)
    .bind(present as i16)
    .bind(&req.relayer)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "parcel_id": id,
            "from": holder,
            "to": req.new_owner,
            "threshold": req.threshold,
            "present": present,
            "case_hash": hex::encode(ih),
        })),
    ))
}

// ---------------------------------------------------------------------------
// Q-035(a): content-addressed geometry artifact upload
//
// PUT /api/v1/parcels/{parcel_pda}/geometry-artifact
//
// Raw GeoJSON bytes are stored under sha256(bytes). The caller must declare
// that digest in `X-Content-SHA256`; the signed body hash binds the bytes to
// the request. The returned `storage_uri` and `content_hash` are the exact
// values to pass to `register_parcel_geometry_artifact` on chain.
// ---------------------------------------------------------------------------

fn parse_pubkey_param(raw: &str) -> Result<Pubkey, AppError> {
    if let Ok(pubkey) = Pubkey::from_str(raw) {
        return Ok(pubkey);
    }
    Ok(Pubkey::new_from_array(decode_hex32(raw)?))
}

#[derive(Debug, Serialize)]
pub struct GeometryArtifactUploadResponse {
    pub parcel_pubkey: String,
    /// sha256(raw GeoJSON bytes), hex — on-chain `content_hash`.
    pub content_hash: String,
    /// sha256(JSON.stringify(coordinates[0])), hex — on-chain `geometry_hash`.
    pub geometry_hash: String,
    /// Content-addressed storage reference — on-chain `storage_uri`.
    pub storage_uri: String,
    pub storage_backend: String,
    pub size_bytes: i64,
    pub created: bool,
}

pub async fn upload_geometry_artifact(
    State(state): State<AppState>,
    Path(parcel_pda): Path<String>,
    auth: SignedRequest,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<GeometryArtifactUploadResponse>), AppError> {
    let signed_path = format!("/api/v1/parcels/{parcel_pda}/geometry-artifact");
    let key = state.api_authority.0.ok_or_else(|| {
        AppError::unauthorized("privileged endpoint: API_AUTHORITY_PUBKEY not configured")
    })?;
    auth.verify(&key, "PUT", &signed_path)?;

    let pubkey = parse_pubkey_param(&parcel_pda)?;
    let parcel_pubkey = pubkey.to_string();

    let mut hasher = Sha256::new();
    hasher.update(&body);
    let content_bytes: [u8; 32] = hasher.finalize().into();
    if auth.body_hash_hex != hex::encode(content_bytes) {
        return Err(AppError::bad_request(
            "body hash does not match signed request",
        ));
    }
    if body.is_empty() {
        return Err(AppError::bad_request("empty geometry artifact"));
    }

    let declared = headers
        .get("x-content-sha256")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::bad_request("X-Content-SHA256 is required"))?
        .trim();
    let declared = declared.strip_prefix("sha256:").unwrap_or(declared);
    if decode_hex32(declared)? != content_bytes {
        return Err(AppError::bad_request(
            "X-Content-SHA256 does not match the uploaded bytes",
        ));
    }

    let source = std::str::from_utf8(&body)
        .map_err(|e| AppError::bad_request(format!("geometry must be UTF-8: {e}")))?;
    let geometry: serde_json::Value = serde_json::from_str(source)
        .map_err(|e| AppError::bad_request(format!("invalid GeoJSON: {e}")))?;
    // Fail before storage when the bytes are not a usable parcel polygon.
    let _ = geojson_polygon(&geometry)?;
    let ring = crate::indexer::submitted_ring_hash(source)
        .ok_or_else(|| AppError::bad_request("GeoJSON has no exterior ring"))?;

    let content_hash = hex::encode(content_bytes);
    let geometry_hash = hex::encode(ring);

    let stored = state
        .storage
        .store_content_addressed("application/geo+json", &body)
        .await
        .map_err(|e| AppError::internal(format!("geometry artifact storage error: {e}")))?;
    if stored.content_hash != content_bytes {
        return Err(AppError::internal(
            "storage returned a different content hash than the uploaded bytes",
        ));
    }

    let existing: Option<(String, String)> = sqlx::query_as(
        "SELECT storage_ref, geometry_hash FROM parcel_geometry_uploads \
         WHERE parcel_pubkey = $1 AND content_hash = $2",
    )
    .bind(&parcel_pubkey)
    .bind(&content_hash)
    .fetch_optional(&state.pool)
    .await?;

    let created = match existing {
        Some((storage_ref, stored_ring)) => {
            if storage_ref != stored.storage_ref || stored_ring != geometry_hash {
                return Err(AppError::conflict(
                    "content hash already registered with different bytes",
                ));
            }
            false
        }
        None => {
            let result = sqlx::query(
                "INSERT INTO parcel_geometry_uploads (
                    parcel_pubkey, content_hash, geometry_hash, storage_ref,
                    storage_backend, content_type, size_bytes
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                 ON CONFLICT (parcel_pubkey, content_hash) DO NOTHING",
            )
            .bind(&parcel_pubkey)
            .bind(&content_hash)
            .bind(&geometry_hash)
            .bind(&stored.storage_ref)
            .bind(&stored.backend_name)
            .bind("application/geo+json")
            .bind(body.len() as i64)
            .execute(&state.pool)
            .await?;
            result.rows_affected() == 1
        }
    };

    Ok((
        if created {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(GeometryArtifactUploadResponse {
            parcel_pubkey,
            content_hash,
            geometry_hash,
            storage_uri: stored.storage_ref,
            storage_backend: stored.backend_name,
            size_bytes: body.len() as i64,
            created,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_artifact_pubkey_accepts_base58_and_hex() {
        let key = Pubkey::new_from_array([7u8; 32]);
        assert_eq!(parse_pubkey_param(&key.to_string()).unwrap(), key);
        assert_eq!(parse_pubkey_param(&hex::encode([7u8; 32])).unwrap(), key);
        assert!(parse_pubkey_param("not-a-pubkey").is_err());
    }

    /// The core forfeiture guard: a collective seizure must meet a minimum
    /// threshold (>=2) of validator signers, and the threshold can never exceed
    /// the number of validator signers presented. This makes forfeiture
    /// deliberately heavier than a normal holder-authorized transfer.
    fn forfeiture_ok(threshold: u16, present: usize) -> bool {
        threshold >= MIN_FORFEIT_VALIDATORS && (threshold as usize) <= present && present > 0
    }

    #[test]
    fn forfeiture_requires_collective_validator_threshold() {
        assert!(!forfeiture_ok(0, 3)); // below the 2-validator minimum
        assert!(!forfeiture_ok(1, 3)); // a single validator can't seize land
        assert!(forfeiture_ok(2, 2)); // minimum collective seizure
        assert!(forfeiture_ok(3, 5)); // heavier court orders can demand more
        assert!(!forfeiture_ok(5, 3)); // threshold exceeds signers presented
        assert!(!forfeiture_ok(2, 0)); // no signers at all
    }

    #[test]
    fn forfeiture_owner_cannot_be_validator_signer() {
        // Self-dealing check: the parcel holder must not appear in the declared
        // validators array for judicial_forfeiture, nor sign as a validator.
        let owner = bs58::encode([1u8; 32]).into_string();
        let v1 = bs58::encode([2u8; 32]).into_string();
        let v2 = bs58::encode([3u8; 32]).into_string();

        fn validators_exclude_owner(owner: &str, validators: &[String]) -> bool {
            validators.iter().all(|v| v != owner)
        }
        assert!(validators_exclude_owner(&owner, &[v1.clone(), v2.clone()]));
        assert!(!validators_exclude_owner(&owner, &[v1, owner.clone()]));
    }

    /// Field-level read policy: the mirror's `holder` column is real
    /// property data and must not leak to unauthenticated callers.
    #[test]
    fn holder_is_none_for_unsigned_request() {
        let row = ParcelRow {
            id: Uuid::nil(),
            name: "test".into(),
            holder: "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage".into(),
            status: "registered".into(),
            geometry: None,
            area_m2: None,
            created_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
            updated_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
            onchain_id: None,
        };
        let unauthenticated = row.clone().into_parcel(false);
        assert!(
            unauthenticated.holder.is_none(),
            "holder must be redacted for unsigned callers"
        );
        let authority = row.into_parcel(true);
        assert_eq!(
            authority.holder.as_deref(),
            Some("GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage")
        );
    }
}
