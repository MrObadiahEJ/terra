-- Evidence uploads: off-chain file storage records.
-- Each row corresponds to a file uploaded through POST /api/v1/evidence/upload.
-- The content_hash is the SHA-256 of the raw bytes — it becomes the on-chain
-- Evidence.content_hash that clients include when calling `add_evidence`.

CREATE TABLE IF NOT EXISTS evidence_uploads (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_hash  VARCHAR(64) NOT NULL,       -- hex(32) SHA-256
    storage_ref   TEXT NOT NULL,               -- local path / IPFS CID / S3 key
    filename      TEXT NOT NULL,
    content_type  VARCHAR(127) NOT NULL,
    size_bytes    BIGINT NOT NULL,
    uploaded_by   VARCHAR(64),                 -- optional base58 wallet
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_evidence_uploads_hash ON evidence_uploads (content_hash);
CREATE INDEX IF NOT EXISTS idx_evidence_uploads_created ON evidence_uploads (created_at DESC);
