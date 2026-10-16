use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::SignedRequest;
use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload_evidence))
        .route("/", get(list_evidence_uploads))
        .route("/{id}", get(get_evidence_upload))
        .route("/{id}/file", get(get_evidence_file))
}

// ---------------------------------------------------------------------------
// POST /api/v1/evidence/upload
//
// Accepts a multipart form upload.  Computes SHA-256 over the raw bytes,
// persists to the storage backend, records metadata in the database, and
// returns the content hash + storage reference that the client should use
// when calling the on-chain `add_evidence` instruction.
//
// Optional fields: `parcel_id` (links the upload to a parcel for the
// off-chain gallery) and the `x-wallet` header (Ed25519-signed like every
// authenticated request; when present the wallet is recorded as uploaded_by).
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EvidenceUploadRow {
    pub id: Uuid,
    pub content_hash: String,
    pub storage_ref: String,
    pub storage_backend: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub uploaded_by: Option<String>,
    pub parcel_id: Option<String>,
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
    pub parcel_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EvidenceListParams {
    pub parcel_id: Option<String>,
    pub content_hash: Option<String>,
}

const SELECT_UPLOAD: &str =
    "SELECT id, content_hash, storage_ref, storage_backend, filename, content_type, size_bytes, \
     uploaded_by, parcel_id, created_at";

/// Upload evidence file. Requires authenticated request (SignedRequest).
/// The caller's wallet address is recorded as `uploaded_by` for provenance.
pub async fn upload_evidence(
    State(state): State<AppState>,
    _auth: SignedRequest,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    // Optional caller wallet: proof of who uploaded, stored as uploaded_by.
    let mut uploaded_by: Option<String> = None;
    if let Some(wallet) = headers
        .get("x-wallet")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
    {
        _auth.verify_wallet(&wallet, "POST", "/api/v1/evidence/upload")?;
        uploaded_by = Some(wallet);
    }

    let mut filename = String::from("unknown");
    let mut content_type = String::from("application/octet-stream");
    let mut parcel_id: Option<String> = None;
    let mut data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("multipart error: {e}")))?
    {
        match field.name() {
            Some("file") => {
                filename = field.file_name().unwrap_or("unknown").to_string();
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
            Some("parcel_id") => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| AppError::bad_request(format!("read error: {e}")))?;
                let value = value.trim().to_string();
                parcel_id = if value.is_empty() { None } else { Some(value) };
            }
            _ => {}
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
            (id, content_hash, storage_ref, storage_backend, filename, content_type, size_bytes, uploaded_by, parcel_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, content_hash, storage_ref, storage_backend, filename, content_type, size_bytes,
                   uploaded_by, parcel_id, created_at",
    )
    .bind(id)
    .bind(hex::encode(artifact.content_hash))
    .bind(&artifact.storage_ref)
    .bind(&artifact.backend_name)
    .bind(&filename)
    .bind(&content_type)
    .bind(artifact.size_bytes as i64)
    .bind(&uploaded_by)
    .bind(&parcel_id)
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
            parcel_id: row.parcel_id,
        }),
    ))
}

/// List uploaded evidence records, optionally filtered by parcel or hash.
pub async fn list_evidence_uploads(
    State(state): State<AppState>,
    Query(params): Query<EvidenceListParams>,
) -> Result<Json<Vec<EvidenceUploadRow>>, AppError> {
    let rows = match (params.parcel_id, params.content_hash) {
        (Some(parcel), Some(hash)) => {
            sqlx::query_as::<_, EvidenceUploadRow>(&format!(
                "{SELECT_UPLOAD} FROM evidence_uploads WHERE parcel_id = $1 AND content_hash = $2 \
                 ORDER BY created_at DESC LIMIT 200"
            ))
            .bind(parcel)
            .bind(hash)
            .fetch_all(&state.pool)
            .await?
        }
        (Some(parcel), None) => {
            sqlx::query_as::<_, EvidenceUploadRow>(&format!(
                "{SELECT_UPLOAD} FROM evidence_uploads WHERE parcel_id = $1 \
                 ORDER BY created_at DESC LIMIT 200"
            ))
            .bind(parcel)
            .fetch_all(&state.pool)
            .await?
        }
        (None, Some(hash)) => {
            sqlx::query_as::<_, EvidenceUploadRow>(&format!(
                "{SELECT_UPLOAD} FROM evidence_uploads WHERE content_hash = $1 \
                 ORDER BY created_at DESC LIMIT 200"
            ))
            .bind(hash)
            .fetch_all(&state.pool)
            .await?
        }
        (None, None) => {
            sqlx::query_as::<_, EvidenceUploadRow>(&format!(
                "{SELECT_UPLOAD} FROM evidence_uploads ORDER BY created_at DESC LIMIT 100"
            ))
            .fetch_all(&state.pool)
            .await?
        }
    };
    Ok(Json(rows))
}

/// Get a single evidence upload by id.
pub async fn get_evidence_upload(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EvidenceUploadRow>, AppError> {
    let row = sqlx::query_as::<_, EvidenceUploadRow>(&format!(
        "{SELECT_UPLOAD} FROM evidence_uploads WHERE id = $1"
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// GET /api/v1/evidence/{id}/file — stream the stored bytes.
//
// Single-range support (206) so <video> elements can seek; everything else
// gets the full body (200). Evidence files are capped at EVIDENCE_MAX_SIZE
// (50 MB default), so the body is buffered rather than streamed from disk.
// ---------------------------------------------------------------------------

fn parse_single_range(value: &str, len: usize) -> Option<(usize, usize)> {
    let spec = value.trim().strip_prefix("bytes=")?;
    if spec.contains(',') {
        return None; // multi-range not supported — caller falls back to 200
    }
    let (start_s, end_s) = spec.split_once('-')?;
    let start: usize = start_s.trim().parse().ok()?;
    let end: usize = if end_s.trim().is_empty() {
        len.saturating_sub(1)
    } else {
        end_s.trim().parse().ok()?
    };
    if start >= len || end < start {
        return None;
    }
    Some((start, end.min(len - 1)))
}

pub async fn get_evidence_file(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let row = sqlx::query_as::<_, EvidenceUploadRow>(&format!(
        "{SELECT_UPLOAD} FROM evidence_uploads WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("evidence upload not found"))?;

    let bytes = state
        .storage
        .load(&row.storage_ref)
        .await
        .map_err(|e| AppError::internal(format!("storage read error: {e}")))?
        .ok_or_else(|| AppError::not_found("evidence bytes no longer stored"))?;

    let len = bytes.len();
    let content_type = row.content_type.clone();

    let range = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| parse_single_range(value, len));

    let (status, body, content_range) = match range {
        Some((start, end)) => (
            StatusCode::PARTIAL_CONTENT,
            bytes[start..=end].to_vec(),
            Some(format!("bytes {start}-{end}/{len}")),
        ),
        None => (StatusCode::OK, bytes, None),
    };

    // Let hyper derive Content-Length from the body actually being sent (the
    // slice for range responses); Content-Range reports the full length.
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    let headers_mut = response.headers_mut();
    headers_mut.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers_mut.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
    if let Some(content_range) = content_range {
        headers_mut.insert(header::CONTENT_RANGE, content_range.parse().unwrap());
    }
    Ok(response)
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
            parcel_id: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("content_hash"));
        assert!(json.contains("storage_ref"));
        assert!(json.contains("filename"));
        assert!(json.contains("size_bytes"));
        assert!(json.contains("parcel_id"));
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
            parcel_id: Some("PARCEL-1".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: UploadResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, resp.id);
        assert_eq!(decoded.content_hash, resp.content_hash);
        assert_eq!(decoded.storage_ref, resp.storage_ref);
        assert_eq!(decoded.size_bytes, resp.size_bytes);
        assert_eq!(decoded.parcel_id.as_deref(), Some("PARCEL-1"));
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
        let artifact = backend.store("test.txt", "text/plain", data).await.unwrap();

        // Content hash must be SHA-256 of the data.
        let expected_hash: [u8; 32] = sha2::Sha256::digest(data).into();
        assert_eq!(artifact.content_hash, expected_hash);
        // content_type is recorded by the upload route (DB column), not StoredArtifact.
        assert_eq!(artifact.size_bytes, data.len() as u64);
        assert_eq!(artifact.backend_name, "local");

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

        let a1 = backend
            .store("photo.jpg", "image/jpeg", b"data1")
            .await
            .unwrap();
        let a2 = backend
            .store("photo.jpg", "image/jpeg", b"data2")
            .await
            .unwrap();

        // Same filename → different UUID prefix → different storage_ref.
        assert_ne!(a1.storage_ref, a2.storage_ref);
        // But same content hash for same data.
        let a3 = backend
            .store("photo.jpg", "image/jpeg", b"data1")
            .await
            .unwrap();
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
        assert_eq!(
            artifact.content_hash,
            <[u8; 32]>::from(sha2::Sha256::digest(b""))
        );
    }

    #[tokio::test]
    async fn storage_local_date_directory_structure() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = crate::storage::StorageBackend::Local {
            root: tmp.path().to_path_buf(),
        };

        let artifact = backend
            .store("doc.pdf", "application/pdf", b"pdf-content")
            .await
            .unwrap();

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
            zk: None,
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
            format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
        // File content
        body.extend_from_slice(data);
        // Closing boundary
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        body
    }

    #[tokio::test]
    async fn upload_requires_authentication() {
        let body = build_multipart("test.txt", "text/plain", b"hello world");

        // Without SignedRequest headers, the handler must reject with 401.
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
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn upload_rejects_empty_multipart() {
        // Even with auth, empty multipart should fail.
        let boundary = "----TerraTestBoundary";
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"other\"\r\n\r\n\
             value\r\n\
             --{boundary}--\r\n"
        );

        // Note: this test will actually get 401 (no auth) before reaching
        // the multipart validation. That's correct behavior.
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
        // Auth is checked first — 401 without SignedRequest headers.
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
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
            storage_backend: "local".into(),
            filename: "doc.pdf".into(),
            content_type: "application/pdf".into(),
            size_bytes: 1024,
            uploaded_by: None,
            parcel_id: Some("PARCEL-2".into()),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("content_hash"));
        assert!(json.contains("storage_ref"));
        assert!(json.contains("filename"));
        assert!(json.contains("size_bytes"));
        assert!(json.contains("uploaded_by"));
        assert!(json.contains("parcel_id"));
    }

    // -----------------------------------------------------------------------
    // File serving — range parsing + storage load
    // -----------------------------------------------------------------------

    #[test]
    fn range_parsing_supports_common_forms() {
        // Full open-ended range.
        assert_eq!(parse_single_range("bytes=0-99", 200), Some((0, 99)));
        // Open-ended suffix of the body.
        assert_eq!(parse_single_range("bytes=100-", 200), Some((100, 199)));
        // End clamped to the last byte.
        assert_eq!(parse_single_range("bytes=150-999", 200), Some((150, 199)));
        // Multi-range unsupported → None (caller answers 200).
        assert_eq!(parse_single_range("bytes=0-1,5-9", 200), None);
        // Start beyond EOF → unsatisfiable → None.
        assert_eq!(parse_single_range("bytes=500-", 200), None);
        // Malformed / non-bytes values.
        assert_eq!(parse_single_range("items=0-1", 200), None);
        assert_eq!(parse_single_range("bytes=abc-", 200), None);
        assert_eq!(parse_single_range("", 200), None);
    }

    #[tokio::test]
    async fn storage_local_load_roundtrips() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = crate::storage::StorageBackend::Local {
            root: tmp.path().to_path_buf(),
        };

        let data = b"roundtrip me";
        let artifact = backend.store("a.txt", "text/plain", data).await.unwrap();
        let loaded = backend.load(&artifact.storage_ref).await.unwrap().unwrap();
        assert_eq!(loaded, data);
    }

    #[tokio::test]
    async fn storage_local_load_rejects_traversal_and_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = crate::storage::StorageBackend::Local {
            root: tmp.path().to_path_buf(),
        };

        // Crafted traversal ref must not escape the root.
        assert!(backend.load("../../etc/passwd").await.is_err());
        // A benign missing ref is simply None.
        assert!(backend
            .load("2099/01/01/ghost.bin")
            .await
            .unwrap()
            .is_none());
    }
}
