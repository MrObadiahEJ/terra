-- Link evidence uploads to a parcel. Optional: the on-chain content_hash
-- join through EvidenceManifest artifacts remains the authoritative path
-- once manifests are mirrored; this column serves the off-chain demo flow
-- and quick "media for this land" queries.
ALTER TABLE evidence_uploads ADD COLUMN IF NOT EXISTS parcel_id VARCHAR(64);

CREATE INDEX IF NOT EXISTS idx_evidence_uploads_parcel
    ON evidence_uploads (parcel_id, created_at DESC)
    WHERE parcel_id IS NOT NULL;
