use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
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
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use sha2::Digest;
    use tower::ServiceExt;

    // -----------------------------------------------------------------------
    // UploadResponse serialization
    // -----------------------------------------------------------------------

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
        assert!(json.contains("filename"));
        assert!(json.contains("size_bytes"));
    }

    #[test]
    fn upload_response_roundtrips() {
        let resp = UploadResponse {
            id: Uuid::new_v4(),
            content_hash: hex::encode([0xAB; 32]),
            storage_ref: "2026/09/09/abc_photo.jpg".into(),
            filename: "photo.jpg".into(),
            content_type: "image/jpeg".into(),
            size_bytes: 1024 * 512,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: UploadResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, resp.id);
        assert_eq!(decoded.content_hash, resp.content_hash);
        assert_eq!(decoded.storage_ref, resp.storage_ref);
        assert_eq!(decoded.size_bytes, resp.size_bytes);
    }

    // -----------------------------------------------------------------------
    // Storage adapter — local filesystem
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn storage_local_writes_file_and_returns_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = crate::storage::StorageBackend::Local {
            root: tmp.path().to_path_buf(),
        };

        let data = b"hello terra evidence";
        let artifact = backend
            .store("test.txt", "text/plain", data)
            .await
            .unwrap();

        // Content hash must be SHA-256 of the data.
        let expected_hash: [u8; 32] = sha2::Sha256::digest(data).into();
        assert_eq!(artifact.content_hash, expected_hash);
        assert_eq!(artifact.content_type, "text/plain");
        assert_eq!(artifact.size_bytes, data.len() as u64);

        // Storage ref must be a relative path under a date directory.
        assert!(artifact.storage_ref.contains("test.txt"));
        let full_path = tmp.path().join(&artifact.storage_ref);
        assert!(full_path.exists(), "file should exist on disk");

        // File contents must match.
        let written = tokio::fs::read(&full_path).await.unwrap();
        assert_eq!(written, data);
    }

    #[tokio::test]
    async fn storage_local_deduplicates_by_uuid_prefix() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = crate::storage::StorageBackend::Local {
            root: tmp.path().to_path_buf(),
        };

        let a1 = backend.store("photo.jpg", "image/jpeg", b"data1").await.unwrap();
        let a2 = backend.store("photo.jpg", "image/jpeg", b"data2").await.unwrap();

        // Same filename → different UUID prefix → different storage_ref.
        assert_ne!(a1.storage_ref, a2.storage_ref);
        // But same content hash for same data.
        let a3 = backend.store("photo.jpg", "image/jpeg", b"data1").await.unwrap();
        assert_eq!(a1.content_hash, a3.content_hash);
    }

    #[tokio::test]
    async fn storage_local_sanitizes_path_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = crate::storage::StorageBackend::Local {
            root: tmp.path().to_path_buf(),
        };

        let artifact = backend
            .store("../../../etc/passwd", "text/plain", b"nope")
            .await
            .unwrap();

        // Slash characters must be stripped — no raw path traversal.
        assert!(!artifact.storage_ref.contains("/../../../"));
        // The file must live under the root temp dir, not elsewhere.
        let full_path = tmp.path().join(&artifact.storage_ref);
        assert!(full_path.starts_with(tmp.path()));
    }

    #[tokio::test]
    async fn storage_local_empty_data() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = crate::storage::StorageBackend::Local {
            root: tmp.path().to_path_buf(),
        };

        let artifact = backend.store("empty.txt", "text/plain", b"").await.unwrap();
        assert_eq!(artifact.size_bytes, 0);
        assert_eq!(artifact.content_hash, <[u8; 32]>::from(sha2::Sha256::digest(b"")));
    }

    #[tokio::test]
    async fn storage_local_date_directory_structure() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = crate::storage::StorageBackend::Local {
            root: tmp.path().to_path_buf(),
        };

        let artifact = backend.store("doc.pdf", "application/pdf", b"pdf-content").await.unwrap();

        // Storage ref should start with YYYY/MM/DD/
        let today = chrono::Utc::now().format("%Y/%m/%d").to_string();
        assert!(artifact.storage_ref.starts_with(&today));
    }

    // -----------------------------------------------------------------------
    // Multipart integration — handler via axum test
    // -----------------------------------------------------------------------

    use crate::auth::ApiAuthority;
    use crate::storage::StorageBackend;
    use std::sync::Arc;
    fn evidence_app() -> axum::Router {
        let pool = sqlx::PgPool::connect_lazy("postgres://x:x@localhost/x").unwrap();
        let state = AppState {
            pool,
            geo: None,
            api_authority: ApiAuthority(None),
            storage: Arc::new(StorageBackend::Local {
                root: std::path::PathBuf::from("/tmp/terra-test-evidence"),
            }),
        };
        axum::Router::new()
            .route("/upload", post(upload_evidence))
            .with_state(state)
    }

    /// Build a minimal multipart/form-data body with one file field.
    fn build_multipart(filename: &str, content_type: &str, data: &[u8]) -> Vec<u8> {
        let boundary = "----TerraTestBoundary";
        let mut body = Vec::new();

        // Opening boundary
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        // Content-Disposition header
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(
            format!("Content-Type: {content_type}\r\n\r\n").as_bytes(),
        );
        // File content
        body.extend_from_slice(data);
        // Closing boundary
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        body
    }

    #[tokio::test]
    async fn upload_rejects_empty_multipart() {
        // Multipart with no file field.
        let boundary = "----TerraTestBoundary";
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"other\"\r\n\r\n\
             value\r\n\
             --{boundary}--\r\n"
        );

        let resp = evidence_app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/upload")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn upload_accepts_valid_file() {
        let body = build_multipart("test.txt", "text/plain", b"hello world");

        // This will fail at the DB layer (no pool), but it validates multipart
        // parsing succeeds — the error will be a database error, not a parse error.
        let resp = evidence_app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/upload")
                    .header(
                        "content-type",
                        "multipart/form-data; boundary=----TerraTestBoundary",
                    )
                .body(Body::from(body))
                .unwrap(),
            )
            .await
            .unwrap();

        // Without a DB pool the handler will return 500 (database error),
        // not 400 (bad request) — this proves multipart parsing succeeded.
        assert_ne!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // -----------------------------------------------------------------------
    // EvidenceUploadRow serialization
    // -----------------------------------------------------------------------

    #[test]
    fn evidence_row_serializes() {
        let row = EvidenceUploadRow {
            id: Uuid::nil(),
            content_hash: hex::encode([0xAA; 32]),
            storage_ref: "2026/09/09/test".into(),
            filename: "doc.pdf".into(),
            content_type: "application/pdf".into(),
            size_bytes: 1024,
            uploaded_by: None,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("content_hash"));
        assert!(json.contains("storage_ref"));
        assert!(json.contains("filename"));
        assert!(json.contains("size_bytes"));
        assert!(json.contains("uploaded_by"));
    }
}
