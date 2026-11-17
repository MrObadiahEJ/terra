use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use hmac::Mac;
use reqwest::multipart;
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Storage adapter — abstracts where uploaded evidence bytes live.
//
// IMPORTANT: Only the local filesystem backend is fully implemented.
// IPFS pinning is wired but requires IPFS_API_URL to point at a running
// IPFS node (local go-ipfs, Infura, Pinata, etc.). Until a remote storage
// backend is added, all evidence bytes live on local disk and are NOT
// replicated. Do not deploy this backend to production without adding
// an IPFS pinning or S3-compatible backend and enabling >=2 replicas
// (see replication-audit endpoint in routes/mirror.rs).
//
// To add a new backend: add a variant to `StorageBackend`, implement
// `store`/`store_content_addressed`/`load`, update `backend_name`, and
// update `from_env` to select it via environment variable.
// ---------------------------------------------------------------------------

/// Where uploaded bytes are persisted.
pub enum StorageBackend {
    /// Write files to a local directory tree.
    Local { root: PathBuf },
    /// Pin bytes to an IPFS node via its HTTP API (e.g. http://localhost:5001).
    /// The storage_ref returned is the IPFS CID.
    Ipfs { api_url: String, gateway_url: Option<String> },
    /// Store bytes in an S3-compatible bucket (Cloudflare R2, AWS S3, MinIO, etc.).
    /// The storage_ref returned is the object key.
    S3 {
        bucket: String,
        region: String,
        endpoint: String,
        access_key: String,
        secret_key: String,
    },
}

impl StorageBackend {
    /// Short identifier written to the `storage_backend` DB column.
    pub fn backend_name(&self) -> &'static str {
        match self {
            StorageBackend::Local { .. } => "local",
            StorageBackend::Ipfs { .. } => "ipfs",
            StorageBackend::S3 { .. } => "s3",
        }
    }

    /// Store raw bytes, compute SHA-256, return (content_hash, storage_ref).
    pub async fn store(
        &self,
        filename: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<StoredArtifact> {
        let content_hash: [u8; 32] = Sha256::digest(data).into();

        match self {
            StorageBackend::Local { root } => store_local(root, filename, content_type, data)
                .await
                .map(|storage_ref| StoredArtifact {
                    content_hash,
                    storage_ref,
                    size_bytes: data.len() as u64,
                    backend_name: self.backend_name().to_string(),
                }),
            StorageBackend::Ipfs {
                api_url,
                gateway_url,
            } => {
                // IPFS is inherently content-addressed; pin via store_ipfs.
                let (storage_ref, size_bytes) =
                    store_ipfs(api_url, gateway_url.as_deref(), filename, content_type, data)
                        .await?;
                Ok(StoredArtifact {
                    content_hash,
                    storage_ref,
                    size_bytes,
                    backend_name: self.backend_name().to_string(),
                })
            }
            StorageBackend::S3 {
                bucket,
                region,
                endpoint,
                access_key,
                secret_key,
            } => {
                // S3 is inherently content-addressed when keyed by content hash.
                let hex = hex::encode(content_hash);
                let storage_ref = format!("content/sha256/{}/{}/{}", &hex[..2], &hex[2..4], hex);
                let s3_key = storage_ref.clone();
                let (_stored_ref, size_bytes) = store_s3(
                    bucket, region, endpoint, access_key, secret_key,
                    &s3_key, content_type, data,
                )
                .await?;
                Ok(StoredArtifact {
                    content_hash,
                    storage_ref: _stored_ref,
                    size_bytes,
                    backend_name: self.backend_name().to_string(),
                })
            }
        }
        }

    /// Store bytes under a content-addressed path (`content/sha256/aa/bb/<hex>`).
    /// Re-storing identical bytes is idempotent and returns the same reference.
    pub async fn store_content_addressed(
        &self,
        content_type: &str,
        data: &[u8],
    ) -> Result<StoredArtifact> {
        let content_hash: [u8; 32] = Sha256::digest(data).into();

        match self {
            StorageBackend::Local { root } => {
                let hex = hex::encode(content_hash);
                let storage_ref = format!("content/sha256/{}/{}/{}", &hex[..2], &hex[2..4], hex);
                let dest = root.join(&storage_ref);
                let parent = dest.parent().context("resolving artifact parent")?;
                tokio::fs::create_dir_all(parent)
                    .await
                    .context("creating artifact directory")?;
                if tokio::fs::metadata(&dest).await.is_err() {
                    tokio::fs::write(&dest, data)
                        .await
                        .context("writing content-addressed artifact")?;
                }
                Ok(StoredArtifact {
                    content_hash,
                    storage_ref,
                    size_bytes: data.len() as u64,
                    backend_name: self.backend_name().to_string(),
                })
            }
            StorageBackend::Ipfs {
                api_url,
                gateway_url,
            } => {
                // IPFS is inherently content-addressed; pin via store_ipfs.
                let (storage_ref, size_bytes) =
                    store_ipfs(api_url, gateway_url.as_deref(), "artifact", "application/octet-stream", data)
                        .await?;
                Ok(StoredArtifact {
                    content_hash,
                    storage_ref,
                    size_bytes,
                    backend_name: self.backend_name().to_string(),
                })
            }
            StorageBackend::S3 {
                bucket,
                region,
                endpoint,
                access_key,
                secret_key,
            } => {
                // S3 is inherently content-addressed when keyed by content hash.
                let hex = hex::encode(content_hash);
                let storage_ref = format!("content/sha256/{}/{}/{}", &hex[..2], &hex[2..4], hex);
                let s3_key = storage_ref.clone();
                let (_stored_ref, size_bytes) = store_s3(
                    bucket, region, endpoint, access_key, secret_key,
                    &s3_key, content_type, data,
                ).await?;
                Ok(StoredArtifact {
                    content_hash,
                    storage_ref,
                    size_bytes,
                    backend_name: self.backend_name().to_string(),
                })
            }
        }
    }

    /// Load the raw bytes previously stored under `storage_ref`.
    /// Returns `Ok(None)` when the ref no longer resolves (deleted file).
    pub async fn load(&self, storage_ref: &str) -> Result<Option<Vec<u8>>> {
        match self {
            StorageBackend::Local { root } => load_local(root, storage_ref).await,
            StorageBackend::Ipfs {
                api_url,
                gateway_url,
            } => load_ipfs(api_url, gateway_url.as_deref(), storage_ref).await,
            StorageBackend::S3 {
                bucket,
                region,
                endpoint,
                access_key,
                secret_key,
            } => load_s3(bucket, region, endpoint, access_key, secret_key, storage_ref).await,
        }
    }
}

pub struct StoredArtifact {
    /// SHA-256 digest of the uploaded bytes.
    pub content_hash: [u8; 32],
    /// Where the file can be retrieved (e.g. local path, IPFS CID, S3 key).
    pub storage_ref: String,
    /// Byte length of the original upload.
    pub size_bytes: u64,
    /// Backend identifier (e.g. "local", "ipfs", "s3").
    pub backend_name: String,
}

// ---------------------------------------------------------------------------
// Local filesystem backend
// ---------------------------------------------------------------------------

async fn store_local(
    root: &Path,
    filename: &str,
    _content_type: &str,
    data: &[u8],
) -> Result<String> {
    let date_dir = chrono::Utc::now().format("%Y/%m/%d").to_string();
    let dir = root.join(&date_dir);
    tokio::fs::create_dir_all(&dir)
        .await
        .context("creating evidence upload directory")?;

    // Use a short random prefix to avoid collisions when multiple uploads share a name.
    let id = uuid::Uuid::new_v4();
    let safe_name = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>();
    let dest = dir.join(format!("{id}_{safe_name}"));

    tokio::fs::write(&dest, data)
        .await
        .context("writing evidence file")?;

    // Return a relative path the IPFS adapter would otherwise turn into a CID.
    Ok(format!("{date_dir}/{id}_{safe_name}"))
}

async fn load_local(root: &Path, storage_ref: &str) -> Result<Option<Vec<u8>>> {
    // storage_ref is a relative YYYY/MM/DD/… path written by store_local;
    // canonicalize both sides so a crafted ref cannot escape the root.
    let candidate = root.join(storage_ref);
    let canonical_root = root
        .canonicalize()
        .context("canonicalizing evidence storage root")?;
    match tokio::fs::canonicalize(&candidate).await {
        Ok(path) if path.starts_with(&canonical_root) => tokio::fs::read(&path)
            .await
            .context("reading evidence file")
            .map(Some),
        Ok(_) => anyhow::bail!("storage_ref escapes the evidence root"),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).context("resolving evidence storage_ref"),
    }
}

// ---------------------------------------------------------------------------
// IPFS backend (HTTP API)
// ---------------------------------------------------------------------------

async fn store_ipfs(
    api_url: &str,
    gateway_url: Option<&str>,
    filename: &str,
    content_type: &str,
    data: &[u8],
) -> Result<(String, u64)> {
    let client = reqwest::Client::new();

    // Build multipart form for IPFS /api/v0/add
    let part = reqwest::multipart::Part::bytes(data.to_vec())
        .file_name(filename.to_string())
        .mime_str(content_type)
        .expect("valid content type");
    let form = multipart::Form::new().part("file", part);

    let resp = client
        .post(format!("{}/api/v0/add", api_url))
        .multipart(form)
        .send()
        .await
        .context("ipfs add request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("ipfs add returned {status}: {body}");
    }

    let json: serde_json::Value = resp.json().await.context("parsing ipfs add response")?;
    let cid = json["Hash"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("ipfs add response missing Hash field"))?;

    let size_bytes = json["Size"]
        .as_u64()
        .unwrap_or(data.len() as u64);

    let storage_ref = if let Some(gw) = gateway_url {
        format!("{}/ipfs/{}", gw.trim_end_matches('/'), cid)
    } else {
        format!("ipfs://{}", cid)
    };

    Ok((storage_ref, size_bytes))
}

async fn load_ipfs(api_url: &str, gateway_url: Option<&str>, storage_ref: &str) -> Result<Option<Vec<u8>>> {
    let client = reqwest::Client::new();

    // Extract CID from storage_ref (either "ipfs://<cid>" or "<gateway>/ipfs/<cid>")
    let cid = if storage_ref.starts_with("ipfs://") {
        storage_ref.trim_start_matches("ipfs://").to_string()
    } else if let Some(gw) = gateway_url {
        storage_ref.trim_start_matches(&format!("{}/ipfs/", gw.trim_end_matches('/'))).to_string()
    } else {
        storage_ref.to_string()
    };

    // Try gateway first for faster retrieval
    if let Some(gw) = gateway_url {
        let url = format!("{}/ipfs/{}", gw.trim_end_matches('/'), cid);
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                return Ok(Some(resp.bytes().await.context("reading ipfs gateway bytes")?.to_vec()));
            }
        }
    }

    // Fall back to API
    let url = format!("{}/api/v0/cat?arg={}", api_url, cid);
    let resp = client.get(&url).send().await.context("ipfs cat request failed")?;

    if resp.status() == 404 {
        return Ok(None);
    }

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("ipfs cat returned {status}: {body}");
    }

    Ok(Some(resp.bytes().await.context("reading ipfs cat bytes")?.to_vec()))
}

// ---------------------------------------------------------------------------
// S3-compatible backend (Cloudflare R2, AWS S3, MinIO, etc.)
//
// Uses AWS Signature V4 for request signing. R2 is S3-compatible, so this
// backend works with R2 endpoints as well.
// ---------------------------------------------------------------------------

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    type HmacSha256 = hmac::Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn aws_signature_v4(
    access_key: &str,
    secret_key: &str,
    region: &str,
    service: &str,
    method: &str,
    uri: &str,
    query: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs();
    let date = time::OffsetDateTime::from_unix_timestamp(now as i64)
        .expect("invalid unix timestamp");
    let amz_date = format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        date.year(),
        date.month() as u8,
        date.day() as u8,
        date.hour() as u8,
        date.minute() as u8,
        date.second() as u8,
    );
    let date_str = &amz_date[..8];

    // Canonical request
    let mut canonical_headers: Vec<String> = headers
        .iter()
        .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v))
        .collect();
    canonical_headers.sort();
    let canonical_headers = canonical_headers.join("\n") + "\n";

    let signed_headers: String = headers
        .iter()
        .map(|(k, _)| k.to_lowercase())
        .collect::<Vec<_>>()
        .join(";");

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method,
        uri,
        query,
        canonical_headers,
        signed_headers,
        sha256_hex(body),
    );

    // String to sign
    let credential_scope = format!("{}/{}/{}/aws4_request", date_str, region, service);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        sha256_hex(canonical_request.as_bytes()),
    );

    // Signing key
    let k_date = hmac_sha256(
        format!("AWS4{}", secret_key).as_bytes(),
        date_str.as_bytes(),
    );
    let k_region = hmac_sha256(&k_date, region.as_bytes());
                let _k_service = hmac_sha256(&k_region, service.as_bytes());
    let k_signing = hmac_sha256(&k_region, b"aws4_request");

    let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

    format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        access_key, credential_scope, signed_headers, signature
    )
}

async fn store_s3(
    bucket: &str,
    region: &str,
    endpoint: &str,
    access_key: &str,
    secret_key: &str,
    key: &str,
    content_type: &str,
    data: &[u8],
) -> Result<(String, u64)> {
    let client = reqwest::Client::new();
    let url = format!("{}/{}/{}", endpoint.trim_end_matches('/'), bucket, key);

    let date = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs();
    let amz_date = time::OffsetDateTime::from_unix_timestamp(date as i64)
        .expect("invalid unix timestamp")
        .format(&time::format_description::well_known::Rfc3339)
        .expect("invalid date format");

    let headers = vec![
        ("host".to_string(), format!("{}/{}", endpoint.trim_end_matches('/'), bucket)),
        ("x-amz-date".to_string(), amz_date.clone()),
        ("x-amz-content-sha256".to_string(), sha256_hex(data)),
    ];

    let authorization = aws_signature_v4(
        access_key,
        secret_key,
        region,
        "s3",
        "PUT",
        &format!("/{}/{}", bucket, key),
        "",
        &headers,
        data,
    );

    let resp = client
        .put(&url)
        .header("host", format!("{}/{}", endpoint.trim_end_matches('/'), bucket))
        .header("x-amz-date", &amz_date)
        .header("x-amz-content-sha256", sha256_hex(data))
        .header("authorization", &authorization)
        .header("content-type", content_type)
        .header("content-length", data.len())
        .body(data.to_vec())
        .send()
        .await
        .context("s3 put request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("s3 put returned {status}: {body}");
    }

    let storage_ref = format!("{}/{}/{}", endpoint.trim_end_matches('/'), bucket, key);
    Ok((storage_ref, data.len() as u64))
}

async fn load_s3(
    bucket: &str,
    region: &str,
    endpoint: &str,
    access_key: &str,
    secret_key: &str,
    key: &str,
) -> Result<Option<Vec<u8>>> {
    let client = reqwest::Client::new();
    let url = format!("{}/{}/{}", endpoint.trim_end_matches('/'), bucket, key);

    let date = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs();
    let amz_date = time::OffsetDateTime::from_unix_timestamp(date as i64)
        .expect("invalid unix timestamp")
        .format(&time::format_description::well_known::Rfc3339)
        .expect("invalid date format");

    let headers = vec![
        ("host".to_string(), format!("{}/{}", endpoint.trim_end_matches('/'), bucket)),
        ("x-amz-date".to_string(), amz_date.clone()),
    ];

    let authorization = aws_signature_v4(
        access_key,
        secret_key,
        region,
        "s3",
        "GET",
        &format!("/{}/{}", bucket, key),
        "",
        &headers,
        &[],
    );

    let resp = client
        .get(&url)
        .header("host", format!("{}/{}", endpoint.trim_end_matches('/'), bucket))
        .header("x-amz-date", &amz_date)
        .header("authorization", &authorization)
        .send()
        .await
        .context("s3 get request failed")?;

    if resp.status() == 404 {
        return Ok(None);
    }

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("s3 get returned {status}: {body}");
    }

    Ok(Some(resp.bytes().await.context("reading s3 bytes")?.to_vec()))
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

pub fn from_env() -> Result<StorageBackend> {
    // IPFS takes precedence when IPFS_API_URL is set.
    if let Ok(api_url) = std::env::var("IPFS_API_URL") {
        if !api_url.is_empty() {
            let gateway_url = std::env::var("IPFS_GATEWAY_URL").ok();
            tracing::info!(api_url = %api_url, "evidence storage: ipfs");
            return Ok(StorageBackend::Ipfs { api_url, gateway_url });
        }
    }

    let root =
        std::env::var("EVIDENCE_STORAGE_DIR").unwrap_or_else(|_| "/tmp/terra-evidence".to_string());
    let root = PathBuf::from(root);
    std::fs::create_dir_all(&root).context("creating evidence storage root")?;
    tracing::info!(root = %root.display(), "evidence storage: local filesystem");
    Ok(StorageBackend::Local { root })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn content_addressed_store_is_idempotent_and_loads() {
        let dir = tempfile::tempdir().unwrap();
        let backend = StorageBackend::Local {
            root: dir.path().to_path_buf(),
        };
        let bytes = b"component artifact bytes";

        let first = backend
            .store_content_addressed("model/gltf-binary", bytes)
            .await
            .unwrap();
        let second = backend
            .store_content_addressed("model/gltf-binary", bytes)
            .await
            .unwrap();

        assert_eq!(first.content_hash, second.content_hash);
        assert_eq!(first.storage_ref, second.storage_ref);
        assert_eq!(first.size_bytes, bytes.len() as u64);

        let hex = hex::encode(first.content_hash);
        assert_eq!(
            first.storage_ref,
            format!("content/sha256/{}/{}/{}", &hex[..2], &hex[2..4], hex)
        );
        assert_eq!(
            backend.load(&first.storage_ref).await.unwrap().as_deref(),
            Some(bytes.as_slice())
        );
    }

    #[tokio::test]
    async fn different_bytes_get_different_addresses() {
        let dir = tempfile::tempdir().unwrap();
        let backend = StorageBackend::Local {
            root: dir.path().to_path_buf(),
        };

        let a = backend
            .store_content_addressed("text/plain", b"a")
            .await
            .unwrap();
        let b = backend
            .store_content_addressed("text/plain", b"b")
            .await
            .unwrap();

        assert_ne!(a.content_hash, b.content_hash);
        assert_ne!(a.storage_ref, b.storage_ref);
    }
}
