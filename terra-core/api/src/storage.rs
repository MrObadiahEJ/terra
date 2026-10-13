use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Storage adapter — abstracts where uploaded evidence bytes live.
// ---------------------------------------------------------------------------

/// Where uploaded bytes are persisted.
pub enum StorageBackend {
    /// Write files to a local directory tree.
    Local { root: PathBuf },
}

impl StorageBackend {
    /// Short identifier written to the `storage_backend` DB column.
    pub fn backend_name(&self) -> &'static str {
        match self {
            StorageBackend::Local { .. } => "local",
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
        }
    }

    /// Load the raw bytes previously stored under `storage_ref`.
    /// Returns `Ok(None)` when the ref no longer resolves (deleted file).
    pub async fn load(&self, storage_ref: &str) -> Result<Option<Vec<u8>>> {
        match self {
            StorageBackend::Local { root } => load_local(root, storage_ref).await,
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
// Builder
// ---------------------------------------------------------------------------

pub fn from_env() -> Result<StorageBackend> {
    let root =
        std::env::var("EVIDENCE_STORAGE_DIR").unwrap_or_else(|_| "/tmp/terra-evidence".to_string());
    let root = PathBuf::from(root);
    std::fs::create_dir_all(&root).context("creating evidence storage root")?;
    tracing::info!(root = %root.display(), "evidence storage: local filesystem");
    Ok(StorageBackend::Local { root })
}
