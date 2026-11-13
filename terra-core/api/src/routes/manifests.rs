//! Chapter 12 — World Representation Manifest API (REQ-027).
//!
//! Endpoints:
//!   POST   /manifests                          create a manifest version
//!   GET    /manifests/{id}                     get a manifest with its objects
//!   GET    /manifests                          list manifests (filter by region)
//!   PUT    /manifests/{id}/objects             update/replace objects in a manifest
//!   POST   /manifests/verify                   verify a signed manifest envelope
//!   POST   /manifests/signed                   store a verified signed envelope
//!   GET    /manifests/signed                   list stored envelopes by region/cell
//!   GET    /manifests/signed/{address}         fetch a signed envelope by SHA-256
//!
//! The manifest stores references and relationships, not geometry.

use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use terra_geo::{ContentAddress, QuadCell, SignedManifest};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_manifest).get(list_manifests))
        .route("/verify", post(verify_manifest))
        .route(
            "/signed",
            post(store_signed_manifest).get(list_signed_manifests),
        )
        .route("/signed/{address}", get(get_signed_manifest))
        .route("/{id}", get(get_manifest))
        .route("/{id}/objects", put(update_manifest_objects))
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateManifestRequest {
    pub region_id: String,
    pub region_name: String,
    pub parent_manifest_id: Option<Uuid>,
    pub neighbor_manifest_ids: Option<Vec<Uuid>>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ManifestObjectInput {
    pub object_id: String,
    pub component_version_id: String,
    pub assembly_version_id: Option<String>,
    pub spatial_extent: serde_json::Value,
    pub storage_reference: String,
    pub available_lods: Option<Vec<i32>>,
    pub validation_state: Option<String>,
    pub access_requirements: Option<serde_json::Value>,
    pub update_history: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateManifestObjectsRequest {
    pub objects: Vec<ManifestObjectInput>,
}

#[derive(Debug, Deserialize)]
pub struct ListManifestsParams {
    pub region_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

// ---------------------------------------------------------------------------
// Row types (sqlx::FromRow + Serialize)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ManifestRow {
    pub id: Uuid,
    pub region_id: String,
    pub region_name: String,
    pub version: i32,
    pub parent_manifest_id: Option<Uuid>,
    pub neighbor_manifest_ids: Vec<Uuid>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ManifestObjectRow {
    pub id: Uuid,
    pub manifest_id: Uuid,
    pub object_id: String,
    pub component_version_id: String,
    pub assembly_version_id: Option<String>,
    pub spatial_extent: serde_json::Value,
    pub storage_reference: String,
    pub available_lods: Vec<i32>,
    pub validation_state: String,
    pub access_requirements: serde_json::Value,
    pub update_history: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// POST /manifests
// ---------------------------------------------------------------------------

pub async fn create_manifest(
    State(state): State<AppState>,
    Json(req): Json<CreateManifestRequest>,
) -> Result<(axum::http::StatusCode, Json<ManifestRow>), AppError> {
    // Determine next version for the region.
    let next_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM world_manifests WHERE region_id = $1",
    )
    .bind(&req.region_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("version query failed: {e}")))?;

    let neighbor_ids = req.neighbor_manifest_ids.unwrap_or_default();

    let row = sqlx::query_as::<_, ManifestRow>(
        r#"
        INSERT INTO world_manifests (
            region_id, region_name, version, parent_manifest_id,
            neighbor_manifest_ids, description
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, region_id, region_name, version, parent_manifest_id,
                  neighbor_manifest_ids, description, created_at, updated_at
        "#,
    )
    .bind(&req.region_id)
    .bind(&req.region_name)
    .bind(next_version)
    .bind(req.parent_manifest_id)
    .bind(&neighbor_ids)
    .bind(&req.description)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// POST /manifests/verify — content-addressed, signed manifest verification
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct VerifyManifestResponse {
    pub valid: bool,
    pub content_address: String,
    pub region_id: String,
    pub version: u32,
    pub object_count: usize,
}

/// Verify a `terra.manifest/v1` envelope: payload invariants, Ed25519
/// signature, publisher key, and content address. This is the control-plane
/// counterpart to data-plane digest checks; it does not persist anything.
pub async fn verify_manifest(
    Json(manifest): Json<SignedManifest>,
) -> Result<Json<VerifyManifestResponse>, AppError> {
    let address = manifest
        .verify()
        .map_err(|e| AppError::bad_request(format!("invalid signed manifest: {e}")))?;
    Ok(Json(VerifyManifestResponse {
        valid: true,
        content_address: address.to_string(),
        region_id: manifest.payload.region_id.clone(),
        version: manifest.payload.version,
        object_count: manifest.payload.objects.len(),
    }))
}

// ---------------------------------------------------------------------------
// POST /manifests/signed — store a verified envelope (content-addressed)
// GET  /manifests/signed/{address} — retrieve and re-verify by SHA-256
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct StoreSignedManifestResponse {
    pub content_address: String,
    pub publisher: String,
    pub created: bool,
    pub region_id: String,
    pub version: u32,
}

/// Persist a pre-signed `terra.manifest/v1` envelope. The API does not hold a
/// publisher key: callers sign externally, and this endpoint only verifies and
/// stores the artifact under its canonical payload digest. Re-publishing the
/// same payload with the same publisher is idempotent.
pub async fn store_signed_manifest(
    State(state): State<AppState>,
    Json(manifest): Json<SignedManifest>,
) -> Result<(axum::http::StatusCode, Json<StoreSignedManifestResponse>), AppError> {
    let address = manifest
        .verify()
        .map_err(|e| AppError::bad_request(format!("invalid signed manifest: {e}")))?;
    let envelope = serde_json::to_value(&manifest)
        .map_err(|e| AppError::internal(format!("serialize envelope failed: {e}")))?;

    let result = sqlx::query(
        "INSERT INTO signed_manifests (content_address, publisher, region_id, \
         region_name, version, region_cell, signature, envelope) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (content_address, publisher) DO NOTHING",
    )
    .bind(address.to_string())
    .bind(&manifest.publisher)
    .bind(&manifest.payload.region_id)
    .bind(&manifest.payload.region_name)
    .bind(i64::from(manifest.payload.version))
    .bind(manifest.payload.region_cell.to_string())
    .bind(&manifest.signature)
    .bind(envelope)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert signed manifest failed: {e}")))?;

    let created = result.rows_affected() == 1;
    Ok((
        if created {
            axum::http::StatusCode::CREATED
        } else {
            axum::http::StatusCode::OK
        },
        Json(StoreSignedManifestResponse {
            content_address: address.to_string(),
            publisher: manifest.publisher.clone(),
            created,
            region_id: manifest.payload.region_id.clone(),
            version: manifest.payload.version,
        }),
    ))
}

/// List stored envelopes, optionally filtered by region ID and exact quadtree
/// cell. Only metadata is returned; fetch an envelope by its content address.
#[derive(Debug, Deserialize)]
pub struct ListSignedManifestsParams {
    pub region_id: Option<String>,
    pub cell: Option<String>,
    #[serde(default = "default_signed_limit")]
    pub limit: i64,
}

fn default_signed_limit() -> i64 {
    20
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SignedManifestSummary {
    pub content_address: String,
    pub publisher: String,
    pub region_id: String,
    pub region_name: String,
    pub version: i64,
    pub region_cell: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// `GET /manifests/signed?region_id=&cell=&limit=` — discovery over persisted
/// signed envelopes. `cell` is an exact `q/level/x/y` match; invalid cells are
/// rejected before querying.
pub async fn list_signed_manifests(
    State(state): State<AppState>,
    Query(params): Query<ListSignedManifestsParams>,
) -> Result<Json<Vec<SignedManifestSummary>>, AppError> {
    let cell = match params.cell {
        Some(raw) => Some(
            raw.parse::<QuadCell>()
                .map_err(|e| AppError::bad_request(format!("invalid cell: {e}")))?
                .to_string(),
        ),
        None => None,
    };
    let limit = params.limit.clamp(1, 100);

    let rows = sqlx::query_as::<_, SignedManifestSummary>(
        "SELECT content_address, publisher, region_id, region_name, version, \
         region_cell, created_at FROM signed_manifests \
         WHERE ($1::text IS NULL OR region_id = $1) \
         AND ($2::text IS NULL OR region_cell = $2) \
         ORDER BY created_at DESC, content_address ASC LIMIT $3",
    )
    .bind(params.region_id)
    .bind(cell)
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("query signed manifests failed: {e}")))?;

    Ok(Json(rows))
}

/// Retrieve a stored envelope by content address and re-verify it before
/// returning. A stored artifact that no longer verifies is an internal error,
/// not a client error.
pub async fn get_signed_manifest(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<SignedManifest>, AppError> {
    let requested: ContentAddress = address
        .parse()
        .map_err(|e| AppError::bad_request(format!("invalid content address: {e}")))?;

    let envelope = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT envelope FROM signed_manifests WHERE content_address = $1 \
         ORDER BY publisher LIMIT 1",
    )
    .bind(requested.to_string())
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("query signed manifest failed: {e}")))?
    .ok_or_else(|| AppError::not_found(format!("signed manifest {address} not found")))?;

    let manifest: SignedManifest = serde_json::from_value(envelope)
        .map_err(|e| AppError::internal(format!("stored envelope is malformed: {e}")))?;
    let actual = manifest
        .verify()
        .map_err(|e| AppError::internal(format!("stored envelope failed verification: {e}")))?;
    if actual != requested {
        return Err(AppError::internal(
            "stored envelope content address does not match its key",
        ));
    }
    Ok(Json(manifest))
}

// ---------------------------------------------------------------------------
// GET /manifests/{id}
// ---------------------------------------------------------------------------

const MANIFEST_SELECT: &str = "SELECT id, region_id, region_name, version, \
    parent_manifest_id, neighbor_manifest_ids, description, \
    created_at, updated_at FROM world_manifests WHERE id = $1";

const OBJECTS_SELECT: &str = "SELECT id, manifest_id, object_id, \
    component_version_id, assembly_version_id, spatial_extent, \
    storage_reference, available_lods, validation_state, \
    access_requirements, update_history, created_at \
    FROM manifest_objects WHERE manifest_id = $1 ORDER BY object_id";

pub async fn get_manifest(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ManifestWithObjects>, AppError> {
    let manifest = sqlx::query_as::<_, ManifestRow>(MANIFEST_SELECT)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("manifest {id} not found")))?;

    let objects = sqlx::query_as::<_, ManifestObjectRow>(OBJECTS_SELECT)
        .bind(id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?;

    Ok(Json(ManifestWithObjects { manifest, objects }))
}

#[derive(Debug, Serialize)]
pub struct ManifestWithObjects {
    pub manifest: ManifestRow,
    pub objects: Vec<ManifestObjectRow>,
}

// ---------------------------------------------------------------------------
// GET /manifests?region_id=...
// ---------------------------------------------------------------------------

const LIST_MANIFESTS: &str = "SELECT id, region_id, region_name, version, \
    parent_manifest_id, neighbor_manifest_ids, description, \
    created_at, updated_at FROM world_manifests";

pub async fn list_manifests(
    State(state): State<AppState>,
    Query(params): Query<ListManifestsParams>,
) -> Result<Json<Vec<ManifestRow>>, AppError> {
    let limit = params.limit.min(500);
    let rows: Vec<ManifestRow> = match &params.region_id {
        Some(rid) => sqlx::query_as::<_, ManifestRow>(
            format!("{LIST_MANIFESTS} WHERE region_id = $1 ORDER BY version DESC LIMIT $2")
                .as_str(),
        )
        .bind(rid)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        None => sqlx::query_as::<_, ManifestRow>(
            format!("{LIST_MANIFESTS} ORDER BY region_id, version DESC LIMIT $1").as_str(),
        )
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
    };

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// PUT /manifests/{id}/objects
// ---------------------------------------------------------------------------

pub async fn update_manifest_objects(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateManifestObjectsRequest>,
) -> Result<Json<Vec<ManifestObjectRow>>, AppError> {
    // Verify manifest exists.
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM world_manifests WHERE id = $1)")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::internal(format!("exists check failed: {e}")))?;
    if !exists {
        return Err(AppError::not_found(format!("manifest {id} not found")));
    }

    // Replace all objects for this manifest (idempotent upsert per object).
    let mut updated = Vec::with_capacity(req.objects.len());
    for obj in &req.objects {
        let lods = obj.available_lods.clone().unwrap_or_default();
        let validation = obj
            .validation_state
            .clone()
            .unwrap_or_else(|| "pending".into());
        let access = obj
            .access_requirements
            .clone()
            .unwrap_or(serde_json::json!({}));
        let history = obj.update_history.clone().unwrap_or(serde_json::json!([]));

        let row = sqlx::query_as::<_, ManifestObjectRow>(
            r#"
            INSERT INTO manifest_objects (
                manifest_id, object_id, component_version_id,
                assembly_version_id, spatial_extent, storage_reference,
                available_lods, validation_state, access_requirements,
                update_history
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (manifest_id, object_id) DO UPDATE SET
                component_version_id = EXCLUDED.component_version_id,
                assembly_version_id = EXCLUDED.assembly_version_id,
                spatial_extent = EXCLUDED.spatial_extent,
                storage_reference = EXCLUDED.storage_reference,
                available_lods = EXCLUDED.available_lods,
                validation_state = EXCLUDED.validation_state,
                access_requirements = EXCLUDED.access_requirements,
                update_history = EXCLUDED.update_history
            RETURNING id, manifest_id, object_id, component_version_id,
                      assembly_version_id, spatial_extent, storage_reference,
                      available_lods, validation_state, access_requirements,
                      update_history, created_at
            "#,
        )
        .bind(id)
        .bind(&obj.object_id)
        .bind(&obj.component_version_id)
        .bind(&obj.assembly_version_id)
        .bind(serde_json::to_value(&obj.spatial_extent).unwrap_or(serde_json::Value::Null))
        .bind(&obj.storage_reference)
        .bind(&lods)
        .bind(validation)
        .bind(&access)
        .bind(&history)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("upsert failed: {e}")))?;
        updated.push(row);
    }

    Ok(Json(updated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ApiAuthority;
    use crate::storage::StorageBackend;
    use ed25519_dalek::SigningKey;
    use std::sync::Arc;
    use terra_geo::{
        ContentAddress, GeoBounds, ManifestObject, ManifestPayload, QuadCell, ValidationState,
    };

    fn test_state() -> AppState {
        AppState {
            pool: sqlx::PgPool::connect_lazy("postgres://x:x@localhost/x").unwrap(),
            geo: None,
            api_authority: ApiAuthority(None),
            storage: Arc::new(StorageBackend::Local {
                root: std::path::PathBuf::from("/tmp/terra-test-signed-manifests"),
            }),
            zk: None,
        }
    }

    fn signed_manifest() -> SignedManifest {
        let cell = QuadCell::from_point(11.5, 3.85, 8).unwrap();
        let object = ManifestObject {
            object_id: "parcel-1".to_string(),
            component_version_id: "parcel-1-v1".to_string(),
            assembly_version_id: None,
            content: ContentAddress::of(b"component-bytes"),
            storage_reference: "ipfs://bafy-parcel-1".to_string(),
            bounds: GeoBounds::new(11.49, 3.84, 11.51, 3.86).unwrap(),
            available_lods: vec![0, 1, 2],
            validation_state: ValidationState::TechnicallyValid,
        };
        let payload = ManifestPayload::new(
            "cm-centre",
            "Centre, Cameroon",
            cell,
            1,
            vec![object],
            1_700_000_000,
        );
        SignedManifest::sign(payload, &SigningKey::from_bytes(&[7u8; 32])).unwrap()
    }

    #[tokio::test]
    async fn verify_endpoint_accepts_signed_manifest() {
        let manifest = signed_manifest();
        let expected = manifest.content_address().unwrap().to_string();
        let Json(resp) = verify_manifest(Json(manifest)).await.unwrap();
        assert!(resp.valid);
        assert_eq!(resp.region_id, "cm-centre");
        assert_eq!(resp.version, 1);
        assert_eq!(resp.object_count, 1);
        assert_eq!(resp.content_address, expected);
    }

    #[tokio::test]
    async fn verify_endpoint_rejects_tampered_manifest() {
        let mut manifest = signed_manifest();
        manifest.payload.version = 2;
        let err = verify_manifest(Json(manifest)).await.unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[tokio::test]
    async fn store_endpoint_rejects_tampered_manifest_before_persisting() {
        let mut manifest = signed_manifest();
        manifest.payload.version = 2;
        let err = store_signed_manifest(State(test_state()), Json(manifest))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[tokio::test]
    async fn get_endpoint_rejects_malformed_content_address() {
        let err = get_signed_manifest(State(test_state()), Path("sha256:nothex".to_string()))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));

        let err = get_signed_manifest(State(test_state()), Path("not-an-address".to_string()))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[tokio::test]
    async fn list_endpoint_rejects_malformed_cell_before_querying() {
        let err = list_signed_manifests(
            State(test_state()),
            Query(ListSignedManifestsParams {
                region_id: None,
                cell: Some("q/8/not-a-number/1".to_string()),
                limit: 20,
            }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    /// End-to-end persistence/retrieval. CI sets `DATABASE_URL` and applies
    /// migrations; local runs without a database skip the test.
    #[tokio::test]
    async fn signed_manifest_store_and_retrieve_roundtrip() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed signed manifest test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        let state = AppState {
            pool: pool.clone(),
            geo: None,
            api_authority: ApiAuthority(None),
            storage: Arc::new(StorageBackend::Local {
                root: std::path::PathBuf::from("/tmp/terra-test-signed-manifests"),
            }),
            zk: None,
        };

        let manifest = signed_manifest();
        let address = manifest.content_address().unwrap().to_string();

        let (status, Json(stored)) =
            store_signed_manifest(State(state.clone()), Json(manifest.clone()))
                .await
                .unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(stored.content_address, address);
        assert!(stored.created);

        let (status, Json(repeated)) =
            store_signed_manifest(State(state.clone()), Json(manifest.clone()))
                .await
                .unwrap();
        assert_eq!(status, axum::http::StatusCode::OK);
        assert!(!repeated.created);

        let Json(fetched) = get_signed_manifest(State(state.clone()), Path(address.clone()))
            .await
            .unwrap();
        assert_eq!(fetched, manifest);

        let Json(listed) = list_signed_manifests(
            State(state.clone()),
            Query(ListSignedManifestsParams {
                region_id: Some("cm-centre".to_string()),
                cell: Some(manifest.payload.region_cell.to_string()),
                limit: 10,
            }),
        )
        .await
        .unwrap();
        assert!(listed.iter().any(|row| row.content_address == address));

        sqlx::query("DELETE FROM signed_manifests WHERE content_address = $1")
            .bind(&address)
            .execute(&pool)
            .await
            .expect("clean up test manifest");
    }
}
