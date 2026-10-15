//! Spatial stories (RFC-014 Story Engine).
//!
//! CRUD for versioned, shareable scene specifications: ordered stops, a
//! deterministic camera timeline, optional navigation paths and captions.
//! Stories are presentation-layer documents — opening or replaying one never
//! writes chain state. The definition is stored as JSONB with an explicit
//! `schema_version` so old links keep playing as the frontend evolves.
//!
//! Video exports (v0) are client-rendered: the browser captures the same
//! deterministic timeline the player uses; `story_exports` is the auditable
//! receipt ledger, not a server renderer.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::SignedRequest;
use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stories", get(list_stories).post(create_story))
        .route("/stories/{id}", get(get_story).put(update_story).delete(delete_story))
        .route("/stories/{id}/exports", get(list_exports).post(create_export))
        .route("/exports/{id}", put(update_export))
}

// ---------------------------------------------------------------------------
// Story document schema (schema_version = 1) — mirrored in terra-web
// src/story/types.ts. Keep both in sync.
// ---------------------------------------------------------------------------

const SCHEMA_VERSION: u32 = 1;
const MAX_STOPS: usize = 20;
const MAX_DURATION_S: f64 = 300.0;
const MAX_KEYFRAMES: usize = 400;
const MAX_ROUTE_POINTS: usize = 5_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraKeyframe {
    pub time_s: f64,
    pub lon: f64,
    pub lat: f64,
    pub height_m: f64,
    pub heading_deg: f64,
    pub pitch_deg: f64,
    #[serde(default)]
    pub roll_deg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraTrack {
    pub keyframes: Vec<CameraKeyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryStop {
    pub id: String,
    pub label: String,
    /// start | waypoint | arrival | version
    pub kind: String,
    pub time_s: f64,
    pub lon: f64,
    pub lat: f64,
    pub height_m: f64,
    #[serde(default)]
    pub heading_deg: Option<f64>,
    #[serde(default)]
    pub pitch_deg: Option<f64>,
    #[serde(default)]
    pub parcel_id: Option<String>,
    #[serde(default)]
    pub asset_pubkey: Option<String>,
    #[serde(default)]
    pub geometry_version: Option<u32>,
    #[serde(default)]
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationPath {
    pub id: String,
    /// Honest method label: road-graph | great-circle | drawn.
    pub method: String,
    pub coordinates: Vec<[f64; 2]>,
    pub length_m: f64,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptionCue {
    pub time_s: f64,
    pub duration_s: f64,
    pub text: String,
    #[serde(default)]
    pub stop_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryDefinition {
    pub schema_version: u32,
    pub id: String,
    /// journey | history | discovery
    pub story_type: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub stops: Vec<StoryStop>,
    pub camera_track: CameraTrack,
    #[serde(default)]
    pub routes: Vec<NavigationPath>,
    #[serde(default)]
    pub captions: Vec<CaptionCue>,
    pub duration_s: f64,
}

fn finite_in(v: f64, lo: f64, hi: f64) -> bool {
    v.is_finite() && v >= lo && v <= hi
}

/// Validate a definition against schema v1 rules. Errors are explicit so a
/// creator learns exactly why a story was rejected — never silently coerced.
pub fn validate_definition(def: &StoryDefinition) -> Result<(), AppError> {
    if def.schema_version != SCHEMA_VERSION {
        let found = def.schema_version;
        return Err(AppError::bad_request(format!(
            "unsupported schema_version {found} (supported: {SCHEMA_VERSION})"
        )));
    }
    if !matches!(def.story_type.as_str(), "journey" | "history" | "discovery") {
        return Err(AppError::bad_request(
            "story_type must be journey, history or discovery",
        ));
    }
    let title = def.title.trim();
    if title.is_empty() || title.len() > 200 {
        return Err(AppError::bad_request("title must be 1..=200 characters"));
    }
    if !(1.0..=MAX_DURATION_S).contains(&def.duration_s) || !def.duration_s.is_finite() {
        return Err(AppError::bad_request(format!(
            "duration_s must be within 1..={MAX_DURATION_S}"
        )));
    }
    if def.stops.len() < 2 || def.stops.len() > MAX_STOPS {
        return Err(AppError::bad_request(format!(
            "stops must be 2..={MAX_STOPS} (product limit, not a protocol maximum)"
        )));
    }
    if def.camera_track.keyframes.len() < 2
        || def.camera_track.keyframes.len() > MAX_KEYFRAMES
    {
        return Err(AppError::bad_request(format!(
            "camera_track needs 2..={MAX_KEYFRAMES} keyframes"
        )));
    }

    let mut seen_stop_ids = std::collections::HashSet::new();
    for (i, stop) in def.stops.iter().enumerate() {
        if stop.id.trim().is_empty() || !seen_stop_ids.insert(stop.id.clone()) {
            return Err(AppError::bad_request(format!(
                "stops[{i}].id must be non-empty and unique"
            )));
        }
        if !matches!(
            stop.kind.as_str(),
            "start" | "waypoint" | "arrival" | "version"
        ) {
            return Err(AppError::bad_request(format!(
                "stops[{i}].kind must be start, waypoint, arrival or version"
            )));
        }
        if !finite_in(stop.time_s, 0.0, def.duration_s) {
            return Err(AppError::bad_request(format!(
                "stops[{i}].time_s must lie within the story duration"
            )));
        }
        if !finite_in(stop.lon, -180.0, 180.0) || !finite_in(stop.lat, -90.0, 90.0) {
            return Err(AppError::bad_request(format!(
                "stops[{i}] has out-of-range lon/lat"
            )));
        }
        if !finite_in(stop.height_m, -500.0, 100_000.0) {
            return Err(AppError::bad_request(format!(
                "stops[{i}].height_m out of range"
            )));
        }
    }

    let mut prev_t = f64::NEG_INFINITY;
    for (i, kf) in def.camera_track.keyframes.iter().enumerate() {
        if !kf.time_s.is_finite() || kf.time_s <= prev_t {
            return Err(AppError::bad_request(format!(
                "camera keyframes must have strictly increasing time_s (index {i})"
            )));
        }
        prev_t = kf.time_s;
        if !finite_in(kf.time_s, 0.0, def.duration_s) {
            return Err(AppError::bad_request(format!(
                "camera keyframe {i} time_s outside story duration"
            )));
        }
        if !finite_in(kf.lon, -180.0, 180.0) || !finite_in(kf.lat, -90.0, 90.0) {
            return Err(AppError::bad_request(format!(
                "camera keyframe {i} has out-of-range lon/lat"
            )));
        }
        if !finite_in(kf.height_m, -500.0, 100_000.0) {
            return Err(AppError::bad_request(format!(
                "camera keyframe {i} height_m out of range"
            )));
        }
        if !finite_in(kf.heading_deg, -360.0, 360.0)
            || !finite_in(kf.pitch_deg, -90.0, 90.0)
        {
            return Err(AppError::bad_request(format!(
                "camera keyframe {i} heading/pitch out of range"
            )));
        }
    }

    for (i, route) in def.routes.iter().enumerate() {
        if !matches!(
            route.method.as_str(),
            "road-graph" | "great-circle" | "drawn"
        ) {
            return Err(AppError::bad_request(format!(
                "routes[{i}].method must be road-graph, great-circle or drawn"
            )));
        }
        if route.coordinates.len() < 2 || route.coordinates.len() > MAX_ROUTE_POINTS {
            return Err(AppError::bad_request(format!(
                "routes[{i}] needs 2..={MAX_ROUTE_POINTS} coordinates"
            )));
        }
        if !route.length_m.is_finite() || route.length_m < 0.0 {
            return Err(AppError::bad_request(format!(
                "routes[{i}].length_m must be >= 0"
            )));
        }
        for (j, [lon, lat]) in route.coordinates.iter().enumerate() {
            if !finite_in(*lon, -180.0, 180.0) || !finite_in(*lat, -90.0, 90.0) {
                return Err(AppError::bad_request(format!(
                    "routes[{i}].coordinates[{j}] out of range"
                )));
            }
        }
    }

    for (i, cue) in def.captions.iter().enumerate() {
        if !finite_in(cue.time_s, 0.0, def.duration_s)
            || !finite_in(cue.duration_s, 0.1, 60.0)
            || cue.time_s + cue.duration_s > def.duration_s + 0.001
        {
            return Err(AppError::bad_request(format!(
                "captions[{i}] window outside story duration"
            )));
        }
        if cue.text.trim().is_empty() || cue.text.len() > 500 {
            return Err(AppError::bad_request(format!(
                "captions[{i}].text must be 1..=500 characters"
            )));
        }
    }

    match def.story_type.as_str() {
        "journey" => {
            if !def.stops.iter().any(|s| s.kind == "start")
                || !def.stops.iter().any(|s| s.kind == "arrival")
            {
                return Err(AppError::bad_request(
                    "journey stories need at least one start and one arrival stop",
                ));
            }
        }
        "history" => {
            if !def
                .stops
                .iter()
                .any(|s| s.geometry_version.is_some() || s.asset_pubkey.is_some())
            {
                return Err(AppError::bad_request(
                    "history stories must reference geometry versions or a spatial asset",
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn entity_refs(def: &StoryDefinition) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    for stop in &def.stops {
        if let Some(p) = &stop.parcel_id {
            if !refs.contains(p) {
                refs.push(p.clone());
            }
        }
        if let Some(a) = &stop.asset_pubkey {
            if !refs.contains(a) {
                refs.push(a.clone());
            }
        }
    }
    refs
}

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Story {
    pub id: String,
    pub schema_version: i32,
    pub story_type: String,
    pub title: String,
    pub description: Option<String>,
    pub author_wallet: Option<String>,
    pub duration_s: f64,
    pub stop_count: i32,
    pub entity_refs: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoryWithDefinition {
    #[serde(flatten)]
    pub story: Story,
    pub definition: StoryDefinition,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct StoryExport {
    pub id: String,
    pub story_id: String,
    pub aspect: String,
    pub width: i32,
    pub height: i32,
    pub fps: i32,
    pub duration_s: f64,
    pub status: String,
    pub progress: f32,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

const SELECT_STORY: &str = "SELECT id, schema_version, story_type, title, description, author_wallet, duration_s, stop_count, entity_refs, created_at, updated_at";
const SELECT_EXPORT: &str = "SELECT id, story_id, aspect, width, height, fps, duration_s, status, progress, error, created_at, updated_at";

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

fn caller_wallet(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("x-wallet")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

#[derive(Debug, Deserialize)]
pub struct StoryListQuery {
    pub story_type: Option<String>,
    pub entity: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct StoryDbRow {
    id: String,
    schema_version: i32,
    story_type: String,
    title: String,
    description: Option<String>,
    author_wallet: Option<String>,
    duration_s: f64,
    stop_count: i32,
    entity_refs: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    definition: serde_json::Value,
}

impl StoryDbRow {
    fn into_parts(self) -> Result<(Story, StoryDefinition), AppError> {
        let definition: StoryDefinition = serde_json::from_value(self.definition)
            .map_err(|e| AppError::internal(format!("stored story definition corrupt: {e}")))?;
        Ok((
            Story {
                id: self.id,
                schema_version: self.schema_version,
                story_type: self.story_type,
                title: self.title,
                description: self.description,
                author_wallet: self.author_wallet,
                duration_s: self.duration_s,
                stop_count: self.stop_count,
                entity_refs: self.entity_refs,
                created_at: self.created_at,
                updated_at: self.updated_at,
            },
            definition,
        ))
    }
}

async fn story_row(
    pool: &sqlx::PgPool,
    id: &str,
) -> Result<StoryWithDefinition, AppError> {
    let row = sqlx::query_as::<_, StoryDbRow>(&format!(
        "{SELECT_STORY}, definition FROM stories WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::not_found("story not found"))?;
    let (story, definition) = row.into_parts()?;
    Ok(StoryWithDefinition {
        story,
        definition,
    })
}

pub async fn list_stories(
    State(state): State<AppState>,
    Query(params): Query<StoryListQuery>,
) -> Result<Json<Vec<Story>>, AppError> {
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let rows = match (&params.story_type, &params.entity) {
        (Some(t), Some(e)) => {
            sqlx::query_as::<_, Story>(&format!(
                "{SELECT_STORY} FROM stories WHERE story_type = $1 AND $2 = ANY(entity_refs)
                 ORDER BY updated_at DESC LIMIT {limit}"
            ))
            .bind(t)
            .bind(e)
            .fetch_all(&state.pool)
            .await?
        }
        (Some(t), None) => {
            sqlx::query_as::<_, Story>(&format!(
                "{SELECT_STORY} FROM stories WHERE story_type = $1
                 ORDER BY updated_at DESC LIMIT {limit}"
            ))
            .bind(t)
            .fetch_all(&state.pool)
            .await?
        }
        (None, Some(e)) => {
            sqlx::query_as::<_, Story>(&format!(
                "{SELECT_STORY} FROM stories WHERE $1 = ANY(entity_refs)
                 ORDER BY updated_at DESC LIMIT {limit}"
            ))
            .bind(e)
            .fetch_all(&state.pool)
            .await?
        }
        (None, None) => {
            sqlx::query_as::<_, Story>(&format!(
                "{SELECT_STORY} FROM stories ORDER BY updated_at DESC LIMIT {limit}"
            ))
            .fetch_all(&state.pool)
            .await?
        }
    };
    Ok(Json(rows))
}

pub async fn get_story(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StoryWithDefinition>, AppError> {
    Ok(Json(story_row(&state.pool, &id).await?))
}

fn check_author(story_author: &Option<String>, headers: &axum::http::HeaderMap) -> Result<(), AppError> {
    // Unattributed stories are open to any signed caller (v0 demo posture);
    // attributed stories only accept the author's wallet.
    let Some(expected) = story_author else {
        return Ok(());
    };
    let provided = caller_wallet(headers)
        .ok_or_else(|| AppError::unauthorized("x-wallet required for attributed stories"))?;
    if provided != *expected {
        return Err(AppError::unauthorized("only the story author may modify it"));
    }
    Ok(())
}

pub async fn create_story(
    State(state): State<AppState>,
    auth: SignedRequest,
    headers: axum::http::HeaderMap,
    Json(mut def): Json<StoryDefinition>,
) -> Result<(StatusCode, Json<StoryWithDefinition>), AppError> {
    let _ = auth; // structural signature required; wallet binding optional below
    let mut author: Option<String> = None;
    if let Some(wallet) = caller_wallet(&headers) {
        auth.verify_wallet(&wallet, "POST", "/api/v1/stories")?;
        author = Some(wallet);
    }
    validate_definition(&def)?;
    if def.id.trim().is_empty() {
        def.id = Uuid::new_v4().to_string();
    }
    if !def
        .id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        || def.id.len() < 3
        || def.id.len() > 64
    {
        return Err(AppError::bad_request(
            "story id must be 3..=64 chars of [A-Za-z0-9_-]",
        ));
    }

    let refs = entity_refs(&def);
    let def_json = serde_json::to_value(&def)
        .map_err(|e| AppError::internal(format!("serialize definition: {e}")))?;

    let result = sqlx::query(
        "INSERT INTO stories (
             id, schema_version, story_type, title, description, author_wallet,
             duration_s, stop_count, entity_refs, definition
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(&def.id)
    .bind(def.schema_version as i32)
    .bind(&def.story_type)
    .bind(def.title.trim())
    .bind(&def.description)
    .bind(&author)
    .bind(def.duration_s)
    .bind(def.stops.len() as i32)
    .bind(&refs)
    .bind(&def_json)
    .execute(&state.pool)
    .await;

    if let Err(e) = result {
        if e.to_string().contains("duplicate key") {
            return Err(AppError::conflict("story id already exists"));
        }
        return Err(e.into());
    }

    let story = sqlx::query_as::<_, Story>(&format!(
        "{SELECT_STORY} FROM stories WHERE id = $1"
    ))
    .bind(&def.id)
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(StoryWithDefinition {
            story,
            definition: def,
        }),
    ))
}

pub async fn update_story(
    State(state): State<AppState>,
    auth: SignedRequest,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(def): Json<StoryDefinition>,
) -> Result<Json<StoryWithDefinition>, AppError> {
    let _ = auth;
    let existing = story_row(&state.pool, &id).await?;
    check_author(&existing.story.author_wallet, &headers)?;
    if def.id != id {
        return Err(AppError::bad_request("definition id must match the URL"));
    }
    validate_definition(&def)?;

    let refs = entity_refs(&def);
    let def_json = serde_json::to_value(&def)
        .map_err(|e| AppError::internal(format!("serialize definition: {e}")))?;
    sqlx::query(
        "UPDATE stories SET
             schema_version = $2, story_type = $3, title = $4, description = $5,
             duration_s = $6, stop_count = $7, entity_refs = $8, definition = $9,
             updated_at = now()
         WHERE id = $1",
    )
    .bind(&id)
    .bind(def.schema_version as i32)
    .bind(&def.story_type)
    .bind(def.title.trim())
    .bind(&def.description)
    .bind(def.duration_s)
    .bind(def.stops.len() as i32)
    .bind(&refs)
    .bind(&def_json)
    .execute(&state.pool)
    .await?;

    Ok(Json(story_row(&state.pool, &id).await?))
}

pub async fn delete_story(
    State(state): State<AppState>,
    auth: SignedRequest,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let _ = auth;
    let existing = story_row(&state.pool, &id).await?;
    check_author(&existing.story.author_wallet, &headers)?;
    sqlx::query("DELETE FROM stories WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Export receipts (client-rendered video, v0)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct NewExport {
    /// 9:16 | 16:9 | 1:1
    pub aspect: String,
    pub width: i32,
    pub height: i32,
    pub fps: i32,
    pub duration_s: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExport {
    /// running | done | failed | cancelled
    pub status: String,
    #[serde(default)]
    pub progress: Option<f32>,
    #[serde(default)]
    pub error: Option<String>,
}

fn validate_export(input: &NewExport) -> Result<(), AppError> {
    if !matches!(input.aspect.as_str(), "9:16" | "16:9" | "1:1") {
        return Err(AppError::bad_request("aspect must be 9:16, 16:9 or 1:1"));
    }
    if !(160..=1920).contains(&input.width) || !(160..=1920).contains(&input.height) {
        return Err(AppError::bad_request("width/height must be within 160..=1920"));
    }
    if !(10..=60).contains(&input.fps) {
        return Err(AppError::bad_request("fps must be within 10..=60"));
    }
    if !(1.0..=MAX_DURATION_S).contains(&input.duration_s) {
        return Err(AppError::bad_request(format!(
            "duration_s must be within 1..={MAX_DURATION_S}"
        )));
    }
    Ok(())
}

pub async fn create_export(
    State(state): State<AppState>,
    auth: SignedRequest,
    Path(id): Path<String>,
    Json(input): Json<NewExport>,
) -> Result<(StatusCode, Json<StoryExport>), AppError> {
    let _ = auth;
    story_row(&state.pool, &id).await?; // 404 when the story is missing
    validate_export(&input)?;
    let export_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO story_exports (id, story_id, aspect, width, height, fps, duration_s)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&export_id)
    .bind(&id)
    .bind(&input.aspect)
    .bind(input.width)
    .bind(input.height)
    .bind(input.fps)
    .bind(input.duration_s)
    .execute(&state.pool)
    .await?;
    let row = sqlx::query_as::<_, StoryExport>(&format!(
        "{SELECT_EXPORT} FROM story_exports WHERE id = $1"
    ))
    .bind(&export_id)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn update_export(
    State(state): State<AppState>,
    auth: SignedRequest,
    Path(id): Path<String>,
    Json(input): Json<UpdateExport>,
) -> Result<Json<StoryExport>, AppError> {
    let _ = auth;
    if !matches!(
        input.status.as_str(),
        "running" | "done" | "failed" | "cancelled"
    ) {
        return Err(AppError::bad_request(
            "status must be running, done, failed or cancelled",
        ));
    }
    let progress = input.progress.unwrap_or(if input.status == "done" { 1.0 } else { 0.0 });
    if !(0.0..=1.0).contains(&progress) {
        return Err(AppError::bad_request("progress must be within 0..=1"));
    }
    let touched = sqlx::query_scalar::<_, String>(
        "UPDATE story_exports
         SET status = $2, progress = $3, error = $4, updated_at = now()
         WHERE id = $1
         RETURNING id",
    )
    .bind(&id)
    .bind(&input.status)
    .bind(progress)
    .bind(&input.error)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("export not found"))?;
    let updated = sqlx::query_as::<_, StoryExport>(&format!(
        "{SELECT_EXPORT} FROM story_exports WHERE id = $1"
    ))
    .bind(touched)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(updated))
}

pub async fn list_exports(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<StoryExport>>, AppError> {
    story_row(&state.pool, &id).await?;
    let rows = sqlx::query_as::<_, StoryExport>(&format!(
        "{SELECT_EXPORT} FROM story_exports WHERE story_id = $1 ORDER BY created_at DESC LIMIT 50"
    ))
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_journey() -> StoryDefinition {
        StoryDefinition {
            schema_version: 1,
            id: "test-journey-1".into(),
            story_type: "journey".into(),
            title: "Basel courtyard walk".into(),
            description: None,
            stops: vec![
                StoryStop {
                    id: "s0".into(),
                    label: "Start".into(),
                    kind: "start".into(),
                    time_s: 0.0,
                    lon: 7.59,
                    lat: 47.55,
                    height_m: 400.0,
                    heading_deg: Some(30.0),
                    pitch_deg: Some(-25.0),
                    parcel_id: None,
                    asset_pubkey: None,
                    geometry_version: None,
                    caption: Some("At the gate".into()),
                },
                StoryStop {
                    id: "s1".into(),
                    label: "Arrival".into(),
                    kind: "arrival".into(),
                    time_s: 12.0,
                    lon: 7.595,
                    lat: 47.555,
                    height_m: 420.0,
                    heading_deg: Some(120.0),
                    pitch_deg: Some(-30.0),
                    parcel_id: None,
                    asset_pubkey: None,
                    geometry_version: None,
                    caption: None,
                },
            ],
            camera_track: CameraTrack {
                keyframes: vec![
                    CameraKeyframe {
                        time_s: 0.0,
                        lon: 7.59,
                        lat: 47.55,
                        height_m: 400.0,
                        heading_deg: 30.0,
                        pitch_deg: -25.0,
                        roll_deg: 0.0,
                    },
                    CameraKeyframe {
                        time_s: 12.0,
                        lon: 7.595,
                        lat: 47.555,
                        height_m: 420.0,
                        heading_deg: 120.0,
                        pitch_deg: -30.0,
                        roll_deg: 0.0,
                    },
                ],
            },
            routes: vec![],
            captions: vec![CaptionCue {
                time_s: 0.5,
                duration_s: 3.0,
                text: "Welcome".into(),
                stop_id: Some("s0".into()),
            }],
            duration_s: 12.0,
        }
    }

    #[test]
    fn minimal_journey_is_valid() {
        assert!(validate_definition(&minimal_journey()).is_ok());
    }

    #[test]
    fn rejects_unknown_schema_version() {
        let mut d = minimal_journey();
        d.schema_version = 2;
        assert!(validate_definition(&d).is_err());
    }

    #[test]
    fn rejects_too_many_stops() {
        let mut d = minimal_journey();
        for i in 2..25 {
            d.stops.push(StoryStop {
                id: format!("s{i}"),
                label: format!("stop {i}"),
                kind: "waypoint".into(),
                time_s: 1.0,
                lon: 7.59,
                lat: 47.55,
                height_m: 400.0,
                heading_deg: None,
                pitch_deg: None,
                parcel_id: None,
                asset_pubkey: None,
                geometry_version: None,
                caption: None,
            });
        }
        assert!(validate_definition(&d).is_err());
    }

    #[test]
    fn rejects_non_monotonic_keyframes() {
        let mut d = minimal_journey();
        d.camera_track.keyframes[1].time_s = 0.0; // not strictly increasing
        assert!(validate_definition(&d).is_err());
    }

    #[test]
    fn rejects_journey_without_arrival() {
        let mut d = minimal_journey();
        d.stops[1].kind = "waypoint".into();
        assert!(validate_definition(&d).is_err());
    }

    #[test]
    fn history_needs_version_reference() {
        let mut d = minimal_journey();
        d.story_type = "history".into();
        assert!(validate_definition(&d).is_err());
        d.stops[1].geometry_version = Some(3);
        assert!(validate_definition(&d).is_ok());
    }

    #[test]
    fn rejects_caption_beyond_duration() {
        let mut d = minimal_journey();
        d.captions[0].time_s = 11.0;
        d.captions[0].duration_s = 5.0; // 16 > 12
        assert!(validate_definition(&d).is_err());
    }

    #[test]
    fn entity_refs_dedup() {
        let mut d = minimal_journey();
        d.stops[0].parcel_id = Some("P1".into());
        d.stops[1].parcel_id = Some("P1".into());
        d.stops[1].asset_pubkey = Some("A1".into());
        assert_eq!(entity_refs(&d), vec!["P1".to_string(), "A1".to_string()]);
    }

    #[test]
    fn story_row_serializes() {
        let row = Story {
            id: "s-1".into(),
            schema_version: 1,
            story_type: "journey".into(),
            title: "Walk".into(),
            description: None,
            author_wallet: Some("Wallet111".into()),
            duration_s: 12.0,
            stop_count: 2,
            entity_refs: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"story_type\":\"journey\""));
    }
}
