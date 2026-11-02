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
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/scene/{manifest_id}", get(resolve_scene))
        .route("/scene/{manifest_id}/{lod}", get(resolve_scene_at_lod))
        .route("/assets/{component_id}", get(retrieve_asset))
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
        description: "Globe representation, major boundaries, low-resolution terrain, region manifests",
        max_distance_m: 1_000_000.0,
    },
    LodLevel {
        level: 1,
        name: "regional",
        description: "Regional manifests, terrain tiles, administrative boundaries, major infrastructure",
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
    let levels: Vec<LodLevel> = LOD_LEVELS.iter().filter(|l| l.level <= max).copied().collect();
    Ok(Json(levels))
}

// ---------------------------------------------------------------------------
// GET /runtime/scene/{manifest_id}
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
struct ManifestObjectForScene {
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

const MANIFEST_OBJECTS_FOR_SCENE: &str = "SELECT id, manifest_id, object_id, \
    component_version_id, storage_reference, validation_state, \
    available_lods, spatial_extent FROM manifest_objects WHERE manifest_id = $1 ORDER BY object_id";

const INSERT_SCENE_GRAPH: &str = "INSERT INTO scene_graphs (manifest_id, region_id, \
    lod_level, node_count, status) VALUES ($1, $2, $3, $4, $5) RETURNING id";

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

async fn resolve_scene_internal(
    state: &AppState,
    manifest_id: Uuid,
    lod: i32,
) -> Result<Json<SceneWithNodes>, AppError> {
    // Verify manifest exists and get its region.
    let manifest_region: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, region_id FROM world_manifests WHERE id = $1",
    )
    .bind(manifest_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("manifest query failed: {e}")))?;

    let (_, region_id) = manifest_region
        .ok_or_else(|| AppError::not_found(format!("manifest {manifest_id} not found")))?;

    // Get objects for this manifest at this LOD (filter by available_lods contains lod).
    let objects = sqlx::query_as::<_, ManifestObjectForScene>(
        "SELECT id, manifest_id, object_id, component_version_id, \
         storage_reference, validation_state, available_lods \
         FROM manifest_objects WHERE manifest_id = $1 \
         AND $2 = ANY(available_lods) ORDER BY object_id",
    )
    .bind(manifest_id)
    .bind(lod as i32)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("objects query failed: {e}")))?;

    // Create scene graph.
    let scene = sqlx::query_as::<_, SceneGraphRow>(INSERT_SCENE_GRAPH)
        .bind(manifest_id)
        .bind(&region_id)
        .bind(lod)
        .bind(objects.len() as i32)
        .bind(if objects.is_empty() { "incomplete" } else { "ready" })
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("scene graph insert failed: {e}")))?;

    // Create scene nodes.
    let mut nodes = Vec::with_capacity(objects.len());
    for obj in &objects {
        let node = sqlx::query_as::<_, SceneNodeRow>(
            r#"
            INSERT INTO scene_nodes (
                scene_graph_id, object_id, component_version_id, parent_node_id,
                transform, lod_level, visibility, loading_state
            )
            VALUES ($1, $2, $3, NULL, NULL, $4, 'visible', 'pending')
            RETURNING id, scene_graph_id, object_id, component_version_id,
                      parent_node_id, transform, lod_level, visibility,
                      loading_state, created_at
            "#,
        )
        .bind(scene.id)
        .bind(&obj.object_id)
        .bind(&obj.component_version_id)
        .bind(lod)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("node insert failed: {e}")))?;
        nodes.push(node);
    }

    Ok(Json(SceneWithNodes {
        scene,
        nodes,
    }))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct AssemblyObjectRow {
    pub id: Uuid,
    pub manifest_id: Uuid,
    pub object_id: String,
    pub component_version_id: String,
    pub storage_reference: String,
    pub validation_state: String,
    pub available_lods: Vec<i32>,
}

// ---------------------------------------------------------------------------
// GET /runtime/assets/{component_id}
// ---------------------------------------------------------------------------

pub async fn retrieve_asset(
    State(state): State<AppState>,
    Path((component_id, component_version)): Path<(String, String)>,
) -> Result<Json<AssetRetrieval>, AppError> {
    let row = sqlx::query_as::<_, AssetRetrievalRow>(
        r#"
        SELECT pr.component_id, pr.component_version, sp.id as provider_id, \
               sp.name as provider_name, pr.storage_reference, \
               pr.latency_ms, sp.reliability_score
        FROM provider_replicas pr
        JOIN storage_providers sp ON sp.id = pr.provider_id
        WHERE pr.component_id = $1 AND pr.component_version = $2 \
          AND pr.is_available = true AND sp.is_active = true
        ORDER BY sp.reliability_score DESC, pr.latency_ms ASC NULLS LAST
        LIMIT 1
        "#,
    )
    .bind(&component_id)
    .bind(&component_version)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("asset query failed: {e}")))?;

    let row = row.ok_or_else(|| {
        AppError::not_found(format!("no available replica for component {component_id} version {component_version}"))
    })?;

    Ok(Json(AssetRetrieval {
        component_id: row.component_id,
        component_version: row.component_version,
        provider_id: row.provider_id,
        provider_name: row.provider_name,
        storage_reference: row.storage_reference,
        latency_ms: row.latency_ms,
        reliability_score: row.reliability_score,
    }))
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
