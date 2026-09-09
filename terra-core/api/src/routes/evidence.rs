use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload_evidence))
        .route("/", get(list_evidence_uploads))
        .route("/{id}", get(get_evidence_upload))
}

// ---------------------------------------------------------------------------
// POST /api/v1/evidence/upload
//
// Accepts a multipart form upload.  Computes SHA-256 over the raw bytes,
// persists to the storage backend, records metadata in the database, and
// returns the content hash + storage reference that the client should use
// when calling the on-chain `add_evidence` instruction.
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EvidenceUploadRow {
    pub id: Uuid,
    pub content_hash: String,
    pub storage_ref: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub uploaded_by: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub id: Uuid,
    pub content_hash: String,
    pub storage_ref: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: u64,
}

/// Upload evidence file.
pub async fn upload_evidence(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    let mut filename = String::from("unknown");
    let mut content_type = String::from("application/octet-stream");
    let mut data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("multipart error: {e}")))?
    {
        if let Some(name) = field.name() {
            if name == "file" {
                filename = field
                    .file_name()
                    .unwrap_or("unknown")
                    .to_string();
                content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::bad_request(format!("read error: {e}")))?
                        .to_vec(),
                );
            }
        }
    }

    let bytes = data.ok_or_else(|| AppError::bad_request("missing 'file' field"))?;

    if bytes.is_empty() {
        return Err(AppError::bad_request("empty file"));
    }

    let max_size: usize = std::env::var("EVIDENCE_MAX_SIZE_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(50 * 1024 * 1024); // 50 MB default

    if bytes.len() > max_size {
        return Err(AppError::bad_request(format!(
            "file too large: {} bytes (max {})",
            bytes.len(),
            max_size,
        )));
    }

    let artifact = state
        .storage
        .store(&filename, &content_type, &bytes)
        .await
        .map_err(|e| AppError::bad_request(format!("storage error: {e}")))?;

    let id = Uuid::new_v4();
    let row = sqlx::query_as::<_, EvidenceUploadRow>(
        "INSERT INTO evidence_uploads
            (id, content_hash, storage_ref, filename, content_type, size_bytes, uploaded_by)
         VALUES ($1, $2, $3, $4, $5, $6, NULL)
         RETURNING id, content_hash, storage_ref, filename, content_type, size_bytes,
                   uploaded_by, created_at",
    )
    .bind(id)
    .bind(hex::encode(artifact.content_hash))
    .bind(&artifact.storage_ref)
    .bind(&filename)
    .bind(&artifact.content_type)
    .bind(artifact.size_bytes as i64)
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(UploadResponse {
            id: row.id,
            content_hash: row.content_hash,
            storage_ref: row.storage_ref,
            filename: row.filename,
            content_type: row.content_type,
            size_bytes: artifact.size_bytes,
        }),
    ))
}

/// List uploaded evidence records.
pub async fn list_evidence_uploads(
    State(state): State<AppState>,
) -> Result<Json<Vec<EvidenceUploadRow>>, AppError> {
    let rows = sqlx::query_as::<_, EvidenceUploadRow>(
        "SELECT id, content_hash, storage_ref, filename, content_type, size_bytes,
                uploaded_by, created_at
         FROM evidence_uploads ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

/// Get a single evidence upload by id.
pub async fn get_evidence_upload(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EvidenceUploadRow>, AppError> {
    let row = sqlx::query_as::<_, EvidenceUploadRow>(
        "SELECT id, content_hash, storage_ref, filename, content_type, size_bytes,
                uploaded_by, created_at
         FROM evidence_uploads WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_response_serializes() {
        let resp = UploadResponse {
            id: Uuid::nil(),
            content_hash: "aabb".into(),
            storage_ref: "2026/09/09/test.txt".into(),
            filename: "test.txt".into(),
            content_type: "text/plain".into(),
            size_bytes: 42,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("content_hash"));
        assert!(json.contains("storage_ref"));
    }
}
