ALTER TABLE ownership_roots
    ADD COLUMN IF NOT EXISTS verification_key_hash TEXT NOT NULL DEFAULT '';
