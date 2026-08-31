use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::geoutil::geojson_polygon;
use crate::routes::attestations;
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
        id,
        name,
        owner,
        status,
        ST_AsGeoJSON(geometry)::text AS geometry,
        ST_Area(geometry::geography)::float8 AS area_m2,
        created_at,
        updated_at
    FROM parcels
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Parcel {
    pub id: Uuid,
    pub name: String,
    pub owner: String,
    pub status: String,
    pub geometry: Option<String>,
    pub area_m2: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub owner: String,
    #[serde(default)]
    pub status: String,
    pub geometry: serde_json::Value,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/", post(create_parcel))
        .route("/{id}", get(get_by_id))
        .route("/{id}/reconcile", post(reconcile))
        .route("/{id}", delete(delete_parcel))
        .route("/{id}/attestations", post(attestations::register_attestation))
        .route("/{id}/attestations/{specifier}", get(attestations::get_attestation))
        .route(
            "/{id}/attestations/{specifier}/validations",
            post(attestations::submit_validation),
        )
        .route("/{id}/documents", post(attestations::register_document))
        .route("/{id}/documents", get(attestations::list_documents))
}


async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Parcel>>, AppError> {
    let parcels = match (params.minx, params.miny, params.maxx, params.maxy) {
        (Some(minx), Some(miny), Some(maxx), Some(maxy)) => sqlx::query_as::<_, Parcel>(&format!(
            "{PARCEL_SELECT}
             WHERE ST_Intersects(geometry, ST_MakeEnvelope($1, $2, $3, $4, 4326))
             ORDER BY created_at DESC"
        ))
        .bind(minx)
        .bind(miny)
        .bind(maxx)
        .bind(maxy)
        .fetch_all(&state.pool)
        .await?,
        _ => sqlx::query_as::<_, Parcel>(&format!(
            "{PARCEL_SELECT}
             ORDER BY created_at DESC"
        ))
        .fetch_all(&state.pool)
        .await?,
    };
    Ok(Json(parcels))
}

async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Parcel>, AppError> {
    let parcel = sqlx::query_as::<_, Parcel>(&format!(
        "{PARCEL_SELECT}
         WHERE id = $1"
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(parcel))
}

async fn create_parcel(
    State(state): State<AppState>,
    Json(params): Json<NewParcel>,
) -> Result<(StatusCode, Json<Parcel>), AppError> {
    let geojson = params.geometry.to_string();
    let status = if params.status.is_empty() {
        "pending".to_string()
    } else {
        params.status
    };

    let parcel = sqlx::query_as::<_, Parcel>(&format!(
        "INSERT INTO parcels (name, owner, status, geometry)
         VALUES ($1, $2, $3, ST_GeomFromGeoJSON($4))
         RETURNING id, name, owner, status, ST_AsGeoJSON(geometry)::text AS geometry,
                   ST_Area(geometry::geography)::float8 AS area_m2, created_at, updated_at"
    ))
    .bind(&params.name)
    .bind(&params.owner)
    .bind(&status)
    .bind(&geojson)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if is_geometry_error(&e) {
            AppError::bad_request("geometry must be a valid GeoJSON Polygon in EPSG:4326")
        } else {
            AppError::from(e)
        }
    })?;

    Ok((StatusCode::CREATED, Json(parcel)))
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
    Path(id): Path<Uuid>,
    Json(req): Json<ReconcileRequest>,
) -> Result<Json<Parcel>, AppError> {
    let onchain_id = decode_hex32(&req.onchain_id)?;
    let geo_hash = decode_hex32(&req.geometry_hash)?;
    let access_hash = decode_hex32(&req.access_hash)?;

    // Fetch the stored geometry so we can independently re-derive the digest.
    let geometry_json: String = sqlx::query_scalar(
        "SELECT ST_AsGeoJSON(geometry)::text FROM parcels WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    let geometry: serde_json::Value =
        serde_json::from_str(&geometry_json).map_err(|e| AppError::bad_request(e.to_string()))?;
    let polygon = geojson_polygon(&geometry)?;

    // Independent digest: only possible when OSM data is loaded. When the
    // caller reports a non-zero (already anchored) access_hash, it must match
    // what we derive from the stored geometry — otherwise refuse the update.
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

    let parcel = sqlx::query_as::<_, Parcel>(&format!(
        "{PARCEL_SELECT}
         WHERE id = $1"
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(parcel))
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
