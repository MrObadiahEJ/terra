//! RFC-015 — Spatial Component Package API (REQ-026).
//!
//! Endpoints:
//!   POST   /components          submit + conformance-validate a package
//!   GET    /components/{id}     retrieve a package by id
//!   GET    /components          list packages (filter by physical_entity_id)
//!   PUT    /components/{id}/artifact  store the package artifact bytes
//!   GET    /components/{id}/artifact  retrieve and verify the artifact bytes

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::auth::{OptionalSignedRequest, SignedRequest};
use crate::error::AppError;
use crate::state::AppState;

use terra_geo::component_package::{validate, ComponentPackage};
use terra_geo::ContentAddress;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(submit_component).get(list_components))
        .route(
            "/{id}/artifact",
            put(upload_component_artifact).get(get_component_artifact),
        )
        .route("/{id}", get(get_component))
}

// ---------------------------------------------------------------------------
// POST /api/v1/components
//
// Submit a Spatial Component Package. The package is conformance-validated
// (RFC-015 §4) before insert; invalid packages are rejected with 400.
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ComponentRow {
    pub id: Uuid,
    pub component_id: String,
    pub physical_entity_id: String,
    pub component_type: String,
    pub validation_state: String,
    pub geometry_digest: String,
    pub digest_algorithm_version: String,
    pub crs: String,
    pub elevation_ref: String,
    pub vertical_units: String,
    pub bounding_volume: serde_json::Value,
    pub spatial_extent: serde_json::Value,
    pub resolution: f64,
    pub lod: i32,
    pub reconstruction_engine_id: String,
    pub reconstruction_engine_version: String,
    pub creation_timestamp: chrono::DateTime<chrono::Utc>,
    pub provenance_classification: String,
    pub privacy_classification: String,
    pub access_policy: serde_json::Value,
    pub version_lineage: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitComponent {
    #[serde(flatten)]
    pub package: ComponentPackage,
}

pub async fn submit_component(
    State(state): State<AppState>,
    Json(input): Json<SubmitComponent>,
) -> Result<(axum::http::StatusCode, Json<ComponentRow>), AppError> {
    let pkg = &input.package;

    // RFC-015 §4 conformance (rules 2–8). Rule 1 (digest integrity) needs the
    // artifact bytes; those are checked when the artifact is uploaded, where
    // the raw SHA-256 content address is computed and recorded.
    validate(pkg).map_err(|e| AppError::bad_request(e.to_string()))?;

    let row = sqlx::query_as::<_, ComponentRow>(
        r#"
        INSERT INTO component_packages (
            component_id, physical_entity_id, parent_id, component_type,
            geometry_artifact, format, format_version, geometry_digest,
            digest_algorithm_version, crs, elevation_ref, vertical_units,
            local_frame, global_transform, bounding_volume, spatial_extent,
            resolution, lod, source_evidence_refs,
            reconstruction_engine_id, reconstruction_engine_version,
            processing_configuration, creation_timestamp,
            resource_usage_receipt, accuracy, uncertainty, coverage,
            confidence, provenance_classification, dependencies, overlap_ids,
            privacy_classification, access_policy, validation_state,
            version_lineage
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,
                $18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,
                $33,$34,$35)
        RETURNING id, component_id, physical_entity_id, component_type,
                  validation_state, geometry_digest, digest_algorithm_version,
                  crs, elevation_ref, vertical_units, bounding_volume,
                  spatial_extent, resolution, lod,
                  reconstruction_engine_id, reconstruction_engine_version,
                  creation_timestamp, provenance_classification,
                  privacy_classification, access_policy, version_lineage,
                  created_at
        "#,
    )
    .bind(&pkg.component_id)
    .bind(&pkg.physical_entity_id)
    .bind(&pkg.parent_id)
    .bind(&pkg.component_type)
    .bind(&pkg.geometry_artifact)
    .bind(&pkg.format)
    .bind(&pkg.format_version)
    .bind(&pkg.geometry_digest)
    .bind(&pkg.digest_algorithm_version)
    .bind(&pkg.crs)
    .bind(&pkg.elevation_ref)
    .bind(&pkg.vertical_units)
    .bind(serde_json::to_value(&pkg.local_frame).unwrap_or(Value::Null))
    .bind(serde_json::to_value(&pkg.global_transform).unwrap_or(Value::Null))
    .bind(serde_json::to_value(&pkg.bounding_volume).unwrap_or(Value::Null))
    .bind(serde_json::to_value(&pkg.spatial_extent).unwrap_or(Value::Null))
    .bind(pkg.resolution)
    .bind(pkg.lod)
    .bind(&pkg.source_evidence_refs)
    .bind(&pkg.reconstruction_engine_id)
    .bind(&pkg.reconstruction_engine_version)
    .bind(serde_json::to_value(&pkg.processing_configuration).unwrap_or(Value::Null))
    .bind(
        pkg.creation_timestamp
            .parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(|e| AppError::bad_request(format!("invalid creation_timestamp: {e}")))?,
    )
    .bind(serde_json::to_value(&pkg.resource_usage_receipt).unwrap_or(Value::Null))
    .bind(pkg.accuracy)
    .bind(pkg.uncertainty)
    .bind(serde_json::to_value(&pkg.coverage).unwrap_or(Value::Null))
    .bind(pkg.confidence)
    .bind(&pkg.provenance_classification)
    .bind(pkg.dependencies.as_deref().unwrap_or(&[]))
    .bind(pkg.overlap_ids.as_deref().unwrap_or(&[]))
    .bind(&pkg.privacy_classification)
    .bind(serde_json::to_value(&pkg.access_policy).unwrap_or(Value::Null))
    .bind(&pkg.validation_state)
    .bind(serde_json::to_value(&pkg.version_lineage).unwrap_or(Value::Null))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// GET /api/v1/components/{id}
// ---------------------------------------------------------------------------

const SELECT_COMPONENT: &str = "SELECT id, component_id, physical_entity_id, \
    component_type, validation_state, geometry_digest, \
    digest_algorithm_version, crs, elevation_ref, vertical_units, \
    bounding_volume, spatial_extent, resolution, lod, \
    reconstruction_engine_id, reconstruction_engine_version, \
    creation_timestamp, provenance_classification, privacy_classification, \
    access_policy, version_lineage, created_at \
    FROM component_packages WHERE id = $1";

pub async fn get_component(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: OptionalSignedRequest,
    headers: axum::http::HeaderMap,
) -> Result<Json<ComponentRow>, AppError> {
    let row = fetch_component(&state, id).await?;

    enforce_component_access(
        &row.privacy_classification,
        &row.access_policy,
        auth.0.as_ref(),
        &headers,
        "GET",
        &format!("/api/v1/components/{id}"),
    )?;

    Ok(Json(row))
}

async fn fetch_component(state: &AppState, id: Uuid) -> Result<ComponentRow, AppError> {
    sqlx::query_as::<_, ComponentRow>(SELECT_COMPONENT)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("component {id} not found")))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/components/{id}/artifact
//
// Store the package's bytes (mesh, point cloud, BIM footprint, …) under the
// SHA-256 of the raw content. A package's artifact is immutable: uploading the
// same bytes again is idempotent, a different artifact is a conflict.
// ---------------------------------------------------------------------------

const SELECT_ARTIFACT: &str = "SELECT component_package_id, content_address, \
    storage_ref, content_type, size_bytes \
    FROM component_artifacts WHERE component_package_id = $1";

#[derive(Debug, sqlx::FromRow)]
struct ComponentArtifactRecord {
    component_package_id: Uuid,
    content_address: String,
    storage_ref: String,
    content_type: String,
    size_bytes: i64,
}

#[derive(Debug, Serialize)]
pub struct ArtifactUploadResponse {
    pub component_package_id: Uuid,
    pub content_address: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created: bool,
}

fn artifact_response(record: ComponentArtifactRecord, created: bool) -> ArtifactUploadResponse {
    ArtifactUploadResponse {
        component_package_id: record.component_package_id,
        content_address: record.content_address,
        content_type: record.content_type,
        size_bytes: record.size_bytes,
        created,
    }
}

async fn fetch_artifact(
    state: &AppState,
    id: Uuid,
) -> Result<Option<ComponentArtifactRecord>, AppError> {
    sqlx::query_as::<_, ComponentArtifactRecord>(SELECT_ARTIFACT)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))
}

/// `PUT /api/v1/components/{id}/artifact` — raw body upload. The optional
/// `X-Content-SHA256` header lets the client declare the expected digest; a
/// mismatch is rejected before storage.
pub async fn upload_component_artifact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: SignedRequest,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<ArtifactUploadResponse>), AppError> {
    let component = fetch_component(&state, id).await?;
    enforce_component_access(
        &component.privacy_classification,
        &component.access_policy,
        Some(&auth),
        &headers,
        "PUT",
        &format!("/api/v1/components/{id}/artifact"),
    )?;

    if body.is_empty() {
        return Err(AppError::bad_request("empty artifact"));
    }

    let content_address = ContentAddress::of(&body).to_string();

    if let Some(declared) = headers
        .get("x-content-sha256")
        .and_then(|value| value.to_str().ok())
    {
        let declared = declared.trim();
        let normalized = if declared.starts_with("sha256:") {
            declared.to_string()
        } else {
            format!("sha256:{declared}")
        };
        let expected: ContentAddress = normalized
            .parse()
            .map_err(|e| AppError::bad_request(format!("invalid x-content-sha256: {e}")))?;
        if expected.to_string() != content_address {
            return Err(AppError::bad_request(
                "x-content-sha256 does not match the uploaded bytes",
            ));
        }
    }

    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let stored = state
        .storage
        .store_content_addressed(&content_type, &body)
        .await
        .map_err(|e| AppError::internal(format!("artifact storage error: {e}")))?;

    let inserted = sqlx::query_as::<_, ComponentArtifactRecord>(
        "INSERT INTO component_artifacts (component_package_id, content_address, \
         storage_ref, storage_backend, content_type, size_bytes) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (component_package_id) DO NOTHING \
         RETURNING component_package_id, content_address, storage_ref, \
         content_type, size_bytes",
    )
    .bind(id)
    .bind(&content_address)
    .bind(&stored.storage_ref)
    .bind(&stored.backend_name)
    .bind(&content_type)
    .bind(stored.size_bytes as i64)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("artifact insert failed: {e}")))?;

    match inserted {
        Some(record) => Ok((StatusCode::CREATED, Json(artifact_response(record, true)))),
        None => {
            let existing = fetch_artifact(&state, id).await?.ok_or_else(|| {
                AppError::internal("artifact insert conflicted but no row was found")
            })?;
            if existing.content_address != content_address {
                return Err(AppError::conflict(
                    "component already has a different artifact; packages are immutable",
                ));
            }
            Ok((StatusCode::OK, Json(artifact_response(existing, false))))
        }
    }
}

/// `GET /api/v1/components/{id}/artifact` — serve the bytes only after the
/// stored file re-hashes to its recorded content address.
pub async fn get_component_artifact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: OptionalSignedRequest,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let component = fetch_component(&state, id).await?;
    enforce_component_access(
        &component.privacy_classification,
        &component.access_policy,
        auth.0.as_ref(),
        &headers,
        "GET",
        &format!("/api/v1/components/{id}/artifact"),
    )?;

    let artifact = fetch_artifact(&state, id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("component {id} has no stored artifact")))?;

    let bytes = state
        .storage
        .load(&artifact.storage_ref)
        .await
        .map_err(|e| AppError::internal(format!("artifact storage read error: {e}")))?
        .ok_or_else(|| AppError::not_found("artifact bytes are no longer stored"))?;

    let actual = ContentAddress::of(&bytes).to_string();
    if actual != artifact.content_address {
        return Err(AppError::internal(
            "stored artifact bytes do not match their content address",
        ));
    }

    let mut response = Response::new(axum::body::Body::from(bytes));
    let headers_mut = response.headers_mut();
    let content_type = artifact
        .content_type
        .parse()
        .unwrap_or_else(|_| header::HeaderValue::from_static("application/octet-stream"));
    headers_mut.insert(header::CONTENT_TYPE, content_type);
    headers_mut.insert(
        header::ETAG,
        format!("\"{}\"", artifact.content_address)
            .parse()
            .expect("content address is a valid header value"),
    );
    headers_mut.insert(
        "x-content-sha256",
        artifact
            .content_address
            .parse()
            .expect("content address is a valid header value"),
    );
    if component
        .privacy_classification
        .eq_ignore_ascii_case("public")
    {
        headers_mut.insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }
    Ok(response)
}

pub fn enforce_component_access(
    privacy_classification: &str,
    access_policy: &serde_json::Value,
    auth: Option<&SignedRequest>,
    headers: &axum::http::HeaderMap,
    method: &str,
    path: &str,
) -> Result<(), AppError> {
    if privacy_classification.eq_ignore_ascii_case("public") {
        return Ok(());
    }

    let wallet = headers
        .get("x-wallet")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("x-wallet required for private component"))?;
    let auth = auth
        .ok_or_else(|| AppError::unauthorized("signed request required for private component"))?;
    auth.verify_wallet(wallet, method, path)?;

    if access_policy
        .get("owner_wallet")
        .and_then(|value| value.as_str())
        == Some(wallet)
    {
        return Ok(());
    }
    if access_policy
        .get("allowed_wallets")
        .and_then(|value| value.as_array())
        .is_some_and(|allowed| allowed.iter().any(|value| value.as_str() == Some(wallet)))
    {
        return Ok(());
    }

    Err(AppError::Forbidden(
        "wallet is not authorized to access private component".into(),
    ))
}

// ---------------------------------------------------------------------------
// GET /api/v1/components?physical_entity_id=...
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListComponentsParams {
    pub physical_entity_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

const LIST_COMPONENTS: &str = "SELECT id, component_id, physical_entity_id, \
    component_type, validation_state, geometry_digest, \
    digest_algorithm_version, crs, elevation_ref, vertical_units, \
    bounding_volume, spatial_extent, resolution, lod, \
    reconstruction_engine_id, reconstruction_engine_version, \
    creation_timestamp, provenance_classification, privacy_classification, \
    access_policy, version_lineage, created_at \
    FROM component_packages";

pub async fn list_components(
    State(state): State<AppState>,
    Query(params): Query<ListComponentsParams>,
    auth: OptionalSignedRequest,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<ComponentRow>>, AppError> {
    let limit = params.limit.min(500);
    let rows: Vec<ComponentRow> = match &params.physical_entity_id {
        Some(peid) => sqlx::query_as::<_, ComponentRow>(
            format!(
                "{LIST_COMPONENTS} WHERE physical_entity_id = $1 ORDER BY created_at DESC LIMIT $2"
            )
            .as_str(),
        )
        .bind(peid)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        None => sqlx::query_as::<_, ComponentRow>(
            format!("{LIST_COMPONENTS} ORDER BY created_at DESC LIMIT $1").as_str(),
        )
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
    };

    let wallet = headers
        .get("x-wallet")
        .and_then(|value| value.to_str().ok());
    let signed = auth.0.as_ref();

    let rows = rows
        .into_iter()
        .filter(|row| {
            if row.privacy_classification.eq_ignore_ascii_case("public") {
                return true;
            }
            let Some(wallet) = wallet else {
                return false;
            };
            let Some(signed) = signed else {
                return false;
            };
            signed
                .verify_wallet(wallet, "GET", "/api/v1/components")
                .is_ok()
                && wallet_authorized(&row.access_policy, wallet)
        })
        .collect();

    Ok(Json(rows))
}

fn wallet_authorized(access_policy: &serde_json::Value, wallet: &str) -> bool {
    access_policy
        .get("owner_wallet")
        .and_then(|value| value.as_str())
        == Some(wallet)
        || access_policy
            .get("allowed_wallets")
            .and_then(|value| value.as_array())
            .is_some_and(|allowed| allowed.iter().any(|value| value.as_str() == Some(wallet)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ApiAuthority;
    use crate::storage::StorageBackend;
    use ed25519_dalek::{Signer, SigningKey};
    use std::sync::Arc;
    use terra_geo::component_package::BoundingVolume;

    fn base_package() -> ComponentPackage {
        ComponentPackage {
            component_id: "comp-001".into(),
            physical_entity_id: "parcel-001".into(),
            parent_id: None,
            related_ids: None,
            component_type: "roof".into(),
            geometry_artifact: "https://storage.example.com/roof.glb".into(),
            format: "model/gltf-binary".into(),
            format_version: "2.0".into(),
            geometry_digest: "0e5ac8e42361d78feabaedf8aa9dbb5173285f879f40e9fb2cf93f2943146674"
                .into(),
            digest_algorithm_version: "terra-geom-v2".into(),
            crs: "EPSG:4326".into(),
            elevation_ref: "EGM2008".into(),
            vertical_units: "m".into(),
            local_frame: None,
            global_transform: None,
            bounding_volume: BoundingVolume {
                min: vec![8.5, 47.4, 390.0],
                max: vec![8.6, 47.5, 410.0],
            },
            spatial_extent: serde_json::json!({
                "type": "Polygon",
                "coordinates": [[
                    [8.5, 47.4, 400.0],
                    [8.6, 47.4, 401.0],
                    [8.6, 47.5, 402.0],
                    [8.5, 47.5, 401.0],
                    [8.5, 47.4, 400.0],
                ]]
            }),
            resolution: 0.1,
            lod: 2,
            source_evidence_refs: vec!["capture-001".into()],
            reconstruction_engine_id: "photogrammetry-engine".into(),
            reconstruction_engine_version: "1.2.0".into(),
            processing_configuration: None,
            creation_timestamp: "2026-11-02T20:00:00Z".into(),
            resource_usage_receipt: None,
            accuracy: Some(0.05),
            uncertainty: Some(0.1),
            coverage: None,
            confidence: Some(0.95),
            provenance_classification: "reconstructed".into(),
            dependencies: None,
            overlap_ids: None,
            privacy_classification: "public".into(),
            access_policy: serde_json::json!({ "read": "public" }),
            validation_state: "pending".into(),
            version_lineage: serde_json::json!({ "previous_version_id": null }),
        }
    }

    fn signed_request() -> SignedRequest {
        let key = SigningKey::from_bytes(&[13u8; 32]);
        SignedRequest {
            signature: key.sign(b"artifact-test"),
            timestamp: 0,
            body_hash_hex: String::new(),
        }
    }

    fn artifact_headers(content_type: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            content_type.parse().expect("valid content type"),
        );
        headers
    }

    #[tokio::test]
    async fn artifact_upload_and_retrieval_roundtrip() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed component artifact test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        let dir = tempfile::tempdir().expect("temp storage dir");
        let state = AppState {
            pool: pool.clone(),
            geo: None,
            api_authority: ApiAuthority(None),
            storage: Arc::new(StorageBackend::Local {
                root: dir.path().to_path_buf(),
            }),
            zk: None,
        };

        let mut pkg = base_package();
        pkg.component_id = format!("comp-artifact-{}", Uuid::new_v4());
        let (status, Json(row)) =
            submit_component(State(state.clone()), Json(SubmitComponent { package: pkg }))
                .await
                .unwrap();
        assert_eq!(status, StatusCode::CREATED);

        let bytes = Bytes::from_static(b"glTF artifact bytes");
        let expected = ContentAddress::of(&bytes).to_string();
        let (status, Json(first)) = upload_component_artifact(
            State(state.clone()),
            Path(row.id),
            signed_request(),
            artifact_headers("model/gltf-binary"),
            bytes.clone(),
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert!(first.created);
        assert_eq!(first.content_address, expected);

        let (status, Json(second)) = upload_component_artifact(
            State(state.clone()),
            Path(row.id),
            signed_request(),
            artifact_headers("model/gltf-binary"),
            bytes.clone(),
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(!second.created);

        let response = get_component_artifact(
            State(state.clone()),
            Path(row.id),
            OptionalSignedRequest(None),
            HeaderMap::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "model/gltf-binary"
        );
        assert_eq!(
            response
                .headers()
                .get("x-content-sha256")
                .unwrap()
                .to_str()
                .unwrap(),
            expected
        );
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), bytes.as_ref());

        let err = upload_component_artifact(
            State(state.clone()),
            Path(row.id),
            signed_request(),
            artifact_headers("model/gltf-binary"),
            Bytes::from_static(b"different artifact"),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));

        sqlx::query("DELETE FROM component_packages WHERE id = $1")
            .bind(row.id)
            .execute(&pool)
            .await
            .expect("clean up test component");
    }

    #[tokio::test]
    async fn artifact_upload_rejects_declared_digest_mismatch() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed artifact digest test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        let dir = tempfile::tempdir().expect("temp storage dir");
        let state = AppState {
            pool: pool.clone(),
            geo: None,
            api_authority: ApiAuthority(None),
            storage: Arc::new(StorageBackend::Local {
                root: dir.path().to_path_buf(),
            }),
            zk: None,
        };

        let mut pkg = base_package();
        pkg.component_id = format!("comp-artifact-digest-{}", Uuid::new_v4());
        let (_, Json(row)) =
            submit_component(State(state.clone()), Json(SubmitComponent { package: pkg }))
                .await
                .unwrap();

        let mut headers = artifact_headers("model/gltf-binary");
        headers.insert(
            "x-content-sha256",
            ContentAddress::of(b"something else")
                .to_string()
                .parse()
                .unwrap(),
        );
        let err = upload_component_artifact(
            State(state.clone()),
            Path(row.id),
            signed_request(),
            headers,
            Bytes::from_static(b"actual artifact"),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));

        sqlx::query("DELETE FROM component_packages WHERE id = $1")
            .bind(row.id)
            .execute(&pool)
            .await
            .expect("clean up test component");
    }
}
