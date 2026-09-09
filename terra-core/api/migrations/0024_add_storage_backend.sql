-- Add storage_backend column to evidence_uploads.
-- Identifies which backend stored the file (local, ipfs, s3, etc.).
-- Default 'local' for rows that predate this migration.

ALTER TABLE evidence_uploads ADD COLUMN storage_backend VARCHAR(31) NOT NULL DEFAULT 'local';
