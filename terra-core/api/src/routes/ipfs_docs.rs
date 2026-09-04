use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// IPFS document-anchor log (content-addressed deed/survey anchors)
// ---------------------------------------------------------------------------

const ANCHOR_SELECT: &str = r#"
    SELECT
        id, attestation_pubkey, cid, content_hash,
        category, registered_by, registered_at
    FROM document_anchors
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DocumentAnchor {
    pub id: i64,
    pub attestation_pubkey: String,
    pub cid: String,
    pub content_hash: String,
    pub category: String,
    pub registered_by: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterAnchorRequest {
    pub attestation_pubkey: String,
    pub cid: String,
    pub content_hash: String,
    pub category: String,
    #[serde(default)]
    pub registered_by: String,
}

#[derive(Debug, Deserialize)]
pub struct ListAnchorsParams {
    pub attestation: Option<String>,
}

async fn register_anchor(
    State(state): State<AppState>,
    Json(req): Json<RegisterAnchorRequest>,
) -> Result<(StatusCode, Json<DocumentAnchor>), AppError> {
    if req.cid.trim().is_empty() {
        return Err(AppError::bad_request("cid is required"));
    }
    if req.content_hash.trim().is_empty() {
        return Err(AppError::bad_request("content_hash is required"));
    }
    if req.category.trim().is_empty() {
        return Err(AppError::bad_request("category is required"));
    }
    let row: DocumentAnchor = sqlx::query_as(
        "INSERT INTO document_anchors
            (attestation_pubkey, cid, content_hash, category, registered_by)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, attestation_pubkey, cid, content_hash,
                   category, registered_by, registered_at",
    )
    .bind(&req.attestation_pubkey)
    .bind(&req.cid)
    .bind(&req.content_hash)
    .bind(&req.category)
    .bind(&req.registered_by)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_anchor(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<DocumentAnchor>, AppError> {
    let row: DocumentAnchor = sqlx::query_as(&format!("{ANCHOR_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("document anchor not found"))?;
    Ok(Json(row))
}

async fn list_anchors(
    State(state): State<AppState>,
    Query(params): Query<ListAnchorsParams>,
) -> Result<Json<Vec<DocumentAnchor>>, AppError> {
    let rows: Vec<DocumentAnchor> = match params.attestation {
        Some(att) => {
            sqlx::query_as(&format!(
                "{ANCHOR_SELECT} WHERE attestation_pubkey = $1 ORDER BY registered_at DESC"
            ))
            .bind(att)
            .fetch_all(&state.pool)
            .await?
        }
        None => {
            sqlx::query_as(&format!("{ANCHOR_SELECT} ORDER BY registered_at DESC"))
                .fetch_all(&state.pool)
                .await?
        }
    };
    Ok(Json(rows))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_anchors).post(register_anchor))
        .route("/{id}", get(get_anchor))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_requires_cid_content_hash_category() {
        fn valid(cid: &str, hash: &str, cat: &str) -> bool {
            !cid.trim().is_empty() && !hash.trim().is_empty() && !cat.trim().is_empty()
        }
        assert!(valid("QmX", "ab12", "deed"));
        assert!(!valid("", "ab12", "deed"));
        assert!(!valid("QmX", "", "deed"));
        assert!(!valid("QmX", "ab12", ""));
    }
}
