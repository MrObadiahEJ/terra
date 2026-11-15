//! Chapter 12 — Distributed Runtime API (REQ-028 dependent).
//!
//! The runtime resolves what data is needed for a view, discovers where it is
//! stored, retrieves it, manages versions, and supplies consistent information
//! to visualization clients.
//!
//! Endpoints:
//!   GET    /runtime/scene/{manifest_id}      resolve scene graph for a manifest
//!   GET    /runtime/scene/{manifest_id}/{lod} resolve at specific LOD
//!   GET    /runtime/assets/{component_id}     retrieve asset with provider fallback
//!   GET    /runtime/lod                       list LOD levels
//!   POST   /runtime/cache                     record cached asset

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use terra_geo::{ContentAddress, SignedManifest, ValidationState};
use uuid::Uuid;

use super::components::enforce_component_access;
use crate::auth::OptionalSignedRequest;
use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/scene/{manifest_id}", get(resolve_scene))
        .route("/scene/{manifest_id}/{lod}", get(resolve_scene_at_lod))
        .route(
            "/assets/{component_id}/{component_version}",
            get(retrieve_asset),
        )
        .route("/lod", get(list_lod_levels))
        .route("/cache", post(record_cache))
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListLodParams {
    pub max_level: Option<i32>,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct LodLevel {
    pub level: i32,
    pub name: &'static str,
    pub description: &'static str,
    pub max_distance_m: f64,
}

const LOD_LEVELS: [LodLevel; 5] = [
    LodLevel {
        level: 0,
        name: "global_overview",
        description:
            "Globe representation, major boundaries, low-resolution terrain, region manifests",
        max_distance_m: 1_000_000.0,
    },
    LodLevel {
        level: 1,
        name: "regional",
        description:
            "Regional manifests, terrain tiles, administrative boundaries, major infrastructure",
        max_distance_m: 100_000.0,
    },
    LodLevel {
        level: 2,
        name: "city_district",
        description: "District manifests, road networks, parcel boundaries, building footprints",
        max_distance_m: 10_000.0,
    },
    LodLevel {
        level: 3,
        name: "parcel_building",
        description: "Detailed parcel geometry, building components, roof and facade models",
        max_distance_m: 1_000.0,
    },
    LodLevel {
        level: 4,
        name: "interior_detail",
        description: "Floors, apartments, rooms, interior models, detailed meshes, textures",
        max_distance_m: 100.0,
    },
];

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SceneGraphRow {
    pub id: Uuid,
    pub manifest_id: Uuid,
    pub region_id: String,
    pub lod_level: i32,
    pub node_count: i32,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SceneNodeRow {
    pub id: Uuid,
    pub scene_graph_id: Uuid,
    pub object_id: String,
    pub component_version_id: String,
    pub parent_node_id: Option<Uuid>,
    pub transform: Option<serde_json::Value>,
    pub lod_level: i32,
    pub visibility: String,
    pub loading_state: String,
    pub validation_state: String,
    pub spatial_extent: serde_json::Value,
    /// SHA-256 content address of the component package bytes, as recorded in
    /// the publisher-signed manifest that grounded this scene. `None` when the
    /// scene was resolved from mutable `manifest_objects` rows (legacy path)
    /// and the bytes cannot be tied to a signed manifest.
    pub content_address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct SceneWithNodes {
    pub scene: SceneGraphRow,
    pub nodes: Vec<SceneNodeRow>,
}

#[derive(Debug, Serialize)]
pub struct AssetRetrieval {
    pub component_id: String,
    pub component_version: String,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub storage_reference: String,
    pub latency_ms: Option<i32>,
    pub reliability_score: f64,
    pub global_transform: Option<serde_json::Value>,
    pub crs: Option<String>,
    /// SHA-256 of the raw artifact bytes (`sha256:<hex>`), from the Terra
    /// data-plane copy when available, otherwise the provider-declared
    /// replica checksum. `None` means the client cannot verify the bytes.
    pub content_address: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AssetRetrievalRow {
    pub component_id: String,
    pub component_version: String,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub storage_reference: String,
    pub latency_ms: Option<i32>,
    pub reliability_score: f64,
    pub global_transform: Option<serde_json::Value>,
    pub crs: Option<String>,
    pub content_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordCacheRequest {
    pub component_id: String,
    pub component_version: String,
    pub storage_reference: String,
    pub cache_path: String,
    pub size_bytes: Option<i64>,
}

// ---------------------------------------------------------------------------
// GET /runtime/lod
// ---------------------------------------------------------------------------

pub async fn list_lod_levels(
    Query(params): Query<ListLodParams>,
) -> Result<Json<Vec<LodLevel>>, AppError> {
    let max = params.max_level.unwrap_or(4).min(4);
    let levels: Vec<LodLevel> = LOD_LEVELS
        .iter()
        .filter(|l| l.level <= max)
        .copied()
        .collect();
    Ok(Json(levels))
}

// ---------------------------------------------------------------------------
// GET /runtime/scene/{manifest_id}
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ManifestObjectForScene {
    pub id: Uuid,
    pub manifest_id: Uuid,
    pub object_id: String,
    pub component_version_id: String,
    pub storage_reference: String,
    pub validation_state: String,
    pub available_lods: Vec<i32>,
    pub spatial_extent: serde_json::Value,
}

// ---------------------------------------------------------------------------
// GET /runtime/scene/{manifest_id}
// ---------------------------------------------------------------------------

fn normalize_validation_state(state: &str) -> String {
    match state {
        "missing" => "missing".to_string(),
        "independently_valid" => "independently_valid".to_string(),
        _ => "preliminary".to_string(),
    }
}

pub async fn resolve_scene(
    State(state): State<AppState>,
    Path(manifest_id): Path<Uuid>,
) -> Result<Json<SceneWithNodes>, AppError> {
    resolve_scene_internal(&state, manifest_id, 0).await
}

pub async fn resolve_scene_at_lod(
    State(state): State<AppState>,
    Path((manifest_id, lod)): Path<(Uuid, i32)>,
) -> Result<Json<SceneWithNodes>, AppError> {
    if !(0..=4).contains(&lod) {
        return Err(AppError::bad_request("lod must be between 0 and 4"));
    }
    resolve_scene_internal(&state, manifest_id, lod).await
}

/// Fetch a stored signed envelope by content address, deserialize it, and
/// re-verify the signature. Returns the manifest only if the stored bytes still
/// match their canonical content address. Shared by `GET /manifests/signed/{address}`
/// and signed scene resolution.
async fn load_verified_signed_manifest(
    state: &AppState,
    address: &ContentAddress,
) -> Result<SignedManifest, AppError> {
    let envelope = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT envelope FROM signed_manifests WHERE content_address = $1 \
         ORDER BY publisher LIMIT 1",
    )
    .bind(address.to_string())
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("query signed manifest failed: {e}")))?
    .ok_or_else(|| AppError::not_found(format!("signed manifest {address} not found")))?;

    let manifest: SignedManifest = serde_json::from_value(envelope)
        .map_err(|e| AppError::internal(format!("stored envelope is malformed: {e}")))?;
    let actual = manifest
        .verify()
        .map_err(|e| AppError::internal(format!("stored envelope failed verification: {e}")))?;
    if actual != *address {
        return Err(AppError::internal(
            "stored envelope content address does not match its key",
        ));
    }
    Ok(manifest)
}

/// Build scene nodes from a verified signed manifest payload. Objects are
/// grounded in the publisher's signature: each node carries the object's
/// content address so clients can reject bytes that do not match.
fn build_scene_nodes_from_signed(
    scene_graph_id: Uuid,
    lod: i32,
    manifest: &SignedManifest,
) -> Vec<SceneNodeRow> {
    let mut nodes = Vec::with_capacity(manifest.payload.objects.len());
    for obj in &manifest.payload.objects {
        let validation_state = match obj.validation_state {
            ValidationState::Pending => "pending",
            ValidationState::TechnicallyValid => "technically_valid",
            ValidationState::IndependentlyValid => "independently_valid",
            ValidationState::Rejected => "rejected",
        };
        let extent = serde_json::json!({
            "type": "Polygon",
            "coordinates": [[
                [obj.bounds.min_lon, obj.bounds.min_lat],
                [obj.bounds.max_lon, obj.bounds.min_lat],
                [obj.bounds.max_lon, obj.bounds.max_lat],
                [obj.bounds.min_lon, obj.bounds.max_lat],
                [obj.bounds.min_lon, obj.bounds.min_lat],
            ]]
        });
        nodes.push(SceneNodeRow {
            id: Uuid::new_v4(),
            scene_graph_id,
            object_id: obj.object_id.clone(),
            component_version_id: obj.component_version_id.clone(),
            parent_node_id: None,
            transform: None,
            lod_level: lod,
            visibility: "visible".into(),
            loading_state: "pending".into(),
            validation_state: validation_state.to_string(),
            spatial_extent: extent,
            content_address: Some(obj.content.to_string()),
            created_at: chrono::Utc::now(),
        });
    }
    nodes
}

// ---------------------------------------------------------------------------
// GET /runtime/scene/{manifest_id}
// ---------------------------------------------------------------------------

pub async fn resolve_scene_internal(
    state: &AppState,
    manifest_id: Uuid,
    lod: i32,
) -> Result<Json<SceneWithNodes>, AppError> {
    // Verify manifest exists and get its region, plus any signed-envelope
    // binding that grounds this scene in a publisher-signed manifest.
    let manifest_row: Option<(Uuid, String, Option<String>)> = sqlx::query_as(
        "SELECT id, region_id, signed_content_address FROM world_manifests WHERE id = $1",
    )
    .bind(manifest_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("manifest query failed: {e}")))?;

    let (_, region_id, signed_content_address) = manifest_row
        .ok_or_else(|| AppError::not_found(format!("manifest {manifest_id} not found")))?;

    // Signed path: resolve from the re-verified publisher-signed envelope.
    // Fail closed — a bound manifest whose envelope is missing or no longer
    // verifies yields an error, never a silent fall back to mutable rows.
    let nodes = match signed_content_address {
        Some(address) => {
            let address = address.parse::<ContentAddress>().map_err(|e| {
                AppError::bad_request(format!("invalid signed content address: {e}"))
            })?;
            let manifest = load_verified_signed_manifest(state, &address).await?;
            build_scene_nodes_from_signed(manifest_id, lod, &manifest)
        }
        None => {
            // Legacy path: read objects from mutable manifest_objects rows.
            // Read-only: scene-graph resolution must not write on GET.
            let objects = sqlx::query_as::<_, ManifestObjectForScene>(
                "SELECT id, manifest_id, object_id, component_version_id, \
                 storage_reference, validation_state, available_lods, spatial_extent \
                 FROM manifest_objects WHERE manifest_id = $1 \
                 AND $2 = ANY(available_lods) ORDER BY object_id",
            )
            .bind(manifest_id)
            .bind(lod)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::internal(format!("objects query failed: {e}")))?;

            let mut nodes = Vec::with_capacity(objects.len());
            for obj in &objects {
                nodes.push(SceneNodeRow {
                    id: Uuid::new_v4(),
                    scene_graph_id: manifest_id,
                    object_id: obj.object_id.clone(),
                    component_version_id: obj.component_version_id.clone(),
                    parent_node_id: None,
                    transform: None,
                    lod_level: lod,
                    visibility: "visible".into(),
                    loading_state: "pending".into(),
                    validation_state: normalize_validation_state(&obj.validation_state),
                    spatial_extent: obj.spatial_extent.clone(),
                    content_address: None,
                    created_at: chrono::Utc::now(),
                });
            }
            nodes
        }
    };

    // Build scene graph row without inserting (read-only resolve).
    let scene = SceneGraphRow {
        id: manifest_id,
        manifest_id,
        region_id,
        lod_level: lod,
        node_count: nodes.len() as i32,
        status: if nodes.is_empty() {
            "incomplete"
        } else {
            "ready"
        }
        .to_string(),
        created_at: chrono::Utc::now(),
    };

    Ok(Json(SceneWithNodes { scene, nodes }))
}

// ---------------------------------------------------------------------------
// GET /runtime/assets/{component_id}
// ---------------------------------------------------------------------------

/// `GET /api/v1/runtime/assets/{component_id}/{component_version}`
///
/// Returns up to `MAX_REPLICAS` ranked provider retrievals so the client can
/// fall back through them if the first choice is unreachable. Each retrieval
/// carries the component's raw-artifact content address (SHA-256 of the bytes),
/// preferring Terra's own data-plane record over the provider-declared replica
/// checksum, so a client can reject bytes that do not match — a trust anchor
/// that does not depend on the provider telling the truth at retrieval time.
pub async fn retrieve_asset(
    State(state): State<AppState>,
    Path((component_id, component_version)): Path<(String, String)>,
    auth: OptionalSignedRequest,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<AssetRetrieval>>, AppError> {
    let privacy: Option<(String, serde_json::Value)> = sqlx::query_as(
        "SELECT privacy_classification, access_policy \
         FROM component_packages \
         WHERE component_id = $1 AND component_version = $2 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&component_id)
    .bind(&component_version)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("privacy query failed: {e}")))?;

    if let Some((privacy_classification, access_policy)) = privacy {
        enforce_component_access(
            &privacy_classification,
            &access_policy,
            auth.0.as_ref(),
            &headers,
            "GET",
            &format!("/api/v1/runtime/assets/{component_id}/{component_version}"),
        )?;
    }

    const MAX_REPLICAS: i64 = 5;
    let rows = sqlx::query_as::<_, AssetRetrievalRow>(
        r#"
        SELECT pr.component_id, pr.component_version, sp.id as provider_id,
               sp.name as provider_name, pr.storage_reference,
               pr.latency_ms, sp.reliability_score,
               cp.global_transform, cp.crs,
               COALESCE(ca.content_address, pr.checksum) as content_address
        FROM provider_replicas pr
        JOIN storage_providers sp ON sp.id = pr.provider_id
        LEFT JOIN component_packages cp
          ON cp.component_id = pr.component_id
         AND cp.component_version = pr.component_version
        LEFT JOIN component_artifacts ca
          ON ca.component_package_id = cp.id
        WHERE pr.component_id = $1 AND pr.component_version = $2
          AND pr.is_available = true AND sp.is_active = true
        ORDER BY sp.reliability_score DESC NULLS LAST, pr.latency_ms ASC NULLS LAST
        LIMIT $3
        "#,
    )
    .bind(&component_id)
    .bind(&component_version)
    .bind(MAX_REPLICAS)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("asset query failed: {e}")))?;

    if rows.is_empty() {
        return Err(AppError::not_found(format!(
            "no available replica for component {component_id} version {component_version}"
        )));
    }

    Ok(Json(
        rows.into_iter()
            .map(|row| AssetRetrieval {
                component_id: row.component_id,
                component_version: row.component_version,
                provider_id: row.provider_id,
                provider_name: row.provider_name,
                storage_reference: row.storage_reference,
                latency_ms: row.latency_ms,
                reliability_score: row.reliability_score,
                global_transform: row.global_transform,
                crs: row.crs,
                content_address: row.content_address,
            })
            .collect(),
    ))
}

// ---------------------------------------------------------------------------
// POST /runtime/cache
// ---------------------------------------------------------------------------

pub async fn record_cache(
    State(state): State<AppState>,
    Json(req): Json<RecordCacheRequest>,
) -> Result<(axum::http::StatusCode, Json<CacheRow>), AppError> {
    let row = sqlx::query_as::<_, CacheRow>(
        r#"
        INSERT INTO asset_cache (
            component_id, component_version, storage_reference, cache_path,
            size_bytes
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (component_id, component_version) DO UPDATE SET
            storage_reference = EXCLUDED.storage_reference,
            cache_path = EXCLUDED.cache_path,
            size_bytes = EXCLUDED.size_bytes,
            cached_at = now(),
            last_accessed_at = now(),
            access_count = asset_cache.access_count + 1
        RETURNING id, component_id, component_version, storage_reference,
                  cache_path, size_bytes, cached_at, last_accessed_at,
                  access_count
        "#,
    )
    .bind(&req.component_id)
    .bind(&req.component_version)
    .bind(&req.storage_reference)
    .bind(&req.cache_path)
    .bind(req.size_bytes)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("cache insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CacheRow {
    pub id: Uuid,
    pub component_id: String,
    pub component_version: String,
    pub storage_reference: String,
    pub cache_path: String,
    pub size_bytes: Option<i64>,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed_at: chrono::DateTime<chrono::Utc>,
    pub access_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ApiAuthority;
    use crate::storage::StorageBackend;
    use ed25519_dalek::SigningKey;
    use serde_json::json;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tempfile::tempdir;
    use terra_geo::{
        ContentAddress, GeoBounds, ManifestObject, ManifestPayload, QuadCell, ValidationState,
    };

    /// Builds a publisher-signed manifest with one object for `region_id`,
    /// signed with a fixed test key. The region id is unique per call so the
    /// test is safe to run in parallel with other tests touching `cm-centre`.
    fn signed_test_manifest(region_id: &str) -> SignedManifest {
        let cell = QuadCell::from_point(11.5, 3.85, 8).unwrap();
        let bounds = GeoBounds::new(11.49, 3.84, 11.51, 3.86).unwrap();
        let object = ManifestObject {
            object_id: "parcel-1".to_string(),
            component_version_id: "parcel-1-v1".to_string(),
            assembly_version_id: None,
            content: ContentAddress::of(b"component-bytes-parcel-1"),
            storage_reference: "https://storage.example.com/parcel-1.glb".to_string(),
            bounds,
            available_lods: vec![0, 1, 2],
            validation_state: ValidationState::TechnicallyValid,
        };
        let payload = ManifestPayload::new(
            region_id,
            "Signed Test Region",
            cell,
            1,
            vec![object],
            1_700_000_000,
        );
        SignedManifest::sign(payload, &SigningKey::from_bytes(&[7u8; 32])).unwrap()
    }

    fn test_state(pool: PgPool) -> AppState {
        let dir = tempdir().expect("temp storage dir");
        // Leak the tempdir for the duration of the test process; these tests
        // only exercise read paths that do not touch storage.
        let root = dir.keep();
        AppState {
            pool,
            geo: None,
            api_authority: ApiAuthority(None),
            storage: Arc::new(StorageBackend::Local { root }),
            zk: None,
        }
    }

    #[tokio::test]
    async fn scene_resolution_is_read_only_and_returns_spatial_extent() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed runtime scene test: DATABASE_URL is not set");
            return;
        };
        let pool = PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        let state = test_state(pool.clone());

        let manifest_id = Uuid::new_v4();
        let region_id = format!("runtime-region-{}", Uuid::new_v4());
        let extent = json!({
            "type": "Polygon",
            "coordinates": [[
                [8.5, 47.4], [8.6, 47.4], [8.6, 47.5], [8.5, 47.5], [8.5, 47.4]
            ]]
        });

        sqlx::query(
            "INSERT INTO world_manifests (id, region_id, region_name, version) \
             VALUES ($1, $2, $3, 1)",
        )
        .bind(manifest_id)
        .bind(&region_id)
        .bind("Runtime Test Region")
        .execute(&pool)
        .await
        .expect("insert manifest");

        sqlx::query(
            "INSERT INTO manifest_objects (manifest_id, object_id, component_version_id, \
             spatial_extent, storage_reference, available_lods, validation_state) \
             VALUES ($1, $2, $3, $4, $5, '{0}', 'technically_valid')",
        )
        .bind(manifest_id)
        .bind("object-runtime-1")
        .bind("component-runtime-1@v1")
        .bind(&extent)
        .bind("https://storage.example.com/runtime.glb")
        .execute(&pool)
        .await
        .expect("insert manifest object");

        let before: i64 = sqlx::query_scalar("SELECT count(*) FROM scene_graphs")
            .fetch_one(&pool)
            .await
            .expect("count scene graphs before");
        let before_nodes: i64 = sqlx::query_scalar("SELECT count(*) FROM scene_nodes")
            .fetch_one(&pool)
            .await
            .expect("count scene nodes before");

        let Json(scene) = resolve_scene_internal(&state, manifest_id, 0)
            .await
            .expect("resolve scene");

        assert_eq!(scene.scene.manifest_id, manifest_id);
        assert_eq!(scene.scene.region_id, region_id);
        assert_eq!(scene.nodes.len(), 1);
        assert_eq!(scene.nodes[0].object_id, "object-runtime-1");
        assert_eq!(scene.nodes[0].spatial_extent, extent);
        assert_eq!(scene.nodes[0].validation_state, "preliminary");

        let after: i64 = sqlx::query_scalar("SELECT count(*) FROM scene_graphs")
            .fetch_one(&pool)
            .await
            .expect("count scene graphs after");
        let after_nodes: i64 = sqlx::query_scalar("SELECT count(*) FROM scene_nodes")
            .fetch_one(&pool)
            .await
            .expect("count scene nodes after");
        assert_eq!(before, after, "GET must not insert scene graphs");
        assert_eq!(before_nodes, after_nodes, "GET must not insert scene nodes");

        sqlx::query("DELETE FROM world_manifests WHERE id = $1")
            .bind(manifest_id)
            .execute(&pool)
            .await
            .expect("clean up manifest");
    }

    #[tokio::test]
    async fn asset_retrieval_returns_ranked_replicas_with_content_address() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed runtime asset test: DATABASE_URL is not set");
            return;
        };
        let pool = PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        let state = test_state(pool.clone());

        let provider_uuid = Uuid::new_v4();
        let provider_key = format!("runtime-provider-{}", Uuid::new_v4());
        let component_id = format!("runtime-component-{}", Uuid::new_v4());
        let checksum = format!("sha256:{}", "ab".repeat(32));

        sqlx::query(
            "INSERT INTO storage_providers (id, provider_id, name, provider_type, base_url, reliability_score) \
             VALUES ($1, $2, $3, 'object_storage', $4, 0.9)",
        )
        .bind(provider_uuid)
        .bind(&provider_key)
        .bind("Runtime Test Provider")
        .bind("https://provider.example.com")
        .execute(&pool)
        .await
        .expect("insert provider");

        sqlx::query(
            "INSERT INTO provider_replicas (provider_id, component_id, component_version, \
             storage_reference, version, checksum, latency_ms) \
             VALUES ($1, $2, 'v1', $3, 'v1', $4, 42)",
        )
        .bind(provider_uuid)
        .bind(&component_id)
        .bind("https://provider.example.com/component.glb")
        .bind(&checksum)
        .execute(&pool)
        .await
        .expect("insert replica");

        let Json(assets) = retrieve_asset(
            State(state.clone()),
            Path((component_id.clone(), "v1".to_string())),
            OptionalSignedRequest(None),
            axum::http::HeaderMap::new(),
        )
        .await
        .expect("retrieve asset");

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].component_id, component_id);
        assert_eq!(assets[0].provider_name, "Runtime Test Provider");
        assert_eq!(
            assets[0].content_address.as_deref(),
            Some(checksum.as_str())
        );

        sqlx::query("DELETE FROM storage_providers WHERE id = $1")
            .bind(provider_uuid)
            .execute(&pool)
            .await
            .expect("clean up provider");
    }

    #[tokio::test]
    async fn resolve_scene_grounded_in_signed_manifest() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed signed scene test: DATABASE_URL is not set");
            return;
        };
        let pool = PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        let state = test_state(pool.clone());

        let region_id = format!("cm-centre-{}", Uuid::new_v4());
        let manifest = signed_test_manifest(&region_id);
        let address = manifest.content_address().unwrap().to_string();

        let envelope = serde_json::to_value(&manifest).expect("serialize signed manifest");
        sqlx::query(
            "INSERT INTO signed_manifests (content_address, publisher, region_id, region_name, \
             version, region_cell, signature, envelope) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&address)
        .bind(&manifest.publisher)
        .bind(&manifest.payload.region_id)
        .bind(&manifest.payload.region_name)
        .bind(i64::from(manifest.payload.version))
        .bind(manifest.payload.region_cell.to_string())
        .bind(&manifest.signature)
        .bind(envelope)
        .execute(&pool)
        .await
        .expect("insert signed manifest");

        let manifest_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO world_manifests (id, region_id, region_name, version, signed_content_address) \
             VALUES ($1, $2, $3, 1, $4)",
        )
        .bind(manifest_id)
        .bind(&manifest.payload.region_id)
        .bind("Signed Test Region")
        .bind(&address)
        .execute(&pool)
        .await
        .expect("insert world manifest");

        let Json(scene) = resolve_scene_internal(&state, manifest_id, 0)
            .await
            .expect("resolve scene");

        assert_eq!(scene.scene.manifest_id, manifest_id);
        assert_eq!(scene.scene.region_id, region_id);
        assert_eq!(scene.nodes.len(), 1);
        assert_eq!(scene.nodes[0].object_id, "parcel-1");
        assert_eq!(scene.nodes[0].component_version_id, "parcel-1-v1");
        assert_eq!(
            scene.nodes[0].content_address,
            Some(ContentAddress::of(b"component-bytes-parcel-1").to_string())
        );
        assert_eq!(scene.nodes[0].validation_state, "technically_valid");
        assert_eq!(scene.nodes[0].loading_state, "pending");

        let extent: serde_json::Value = serde_json::json!({
            "type": "Polygon",
            "coordinates": [[
                [11.49, 3.84], [11.51, 3.84], [11.51, 3.86], [11.49, 3.86], [11.49, 3.84],
            ]]
        });
        assert_eq!(scene.nodes[0].spatial_extent, extent);

        sqlx::query("DELETE FROM world_manifests WHERE id = $1")
            .bind(manifest_id)
            .execute(&pool)
            .await
            .expect("clean up manifest");
        sqlx::query("DELETE FROM signed_manifests WHERE content_address = $1")
            .bind(&address)
            .execute(&pool)
            .await
            .expect("clean up signed manifest");
    }

    #[tokio::test]
    async fn resolve_scene_bound_to_missing_envelope_errors() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed signed scene test: DATABASE_URL is not set");
            return;
        };
        let pool = PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        let state = test_state(pool.clone());

        let manifest_id = Uuid::new_v4();
        let region_id = format!("cm-centre-{}", Uuid::new_v4());
        let address = format!("sha256:{}", "00".repeat(32));
        sqlx::query(
            "INSERT INTO world_manifests (id, region_id, region_name, version, signed_content_address) \
             VALUES ($1, $2, $3, 1, $4)",
        )
        .bind(manifest_id)
        .bind(&region_id)
        .bind("Signed Test Region")
        .bind(&address)
        .execute(&pool)
        .await
        .expect("insert world manifest");

        let err = resolve_scene_internal(&state, manifest_id, 0)
            .await
            .expect_err("bound manifest with no envelope must fail closed");
        assert!(matches!(err, AppError::NotFound(_)));

        sqlx::query("DELETE FROM world_manifests WHERE id = $1")
            .bind(manifest_id)
            .execute(&pool)
            .await
            .expect("clean up manifest");
    }
}
