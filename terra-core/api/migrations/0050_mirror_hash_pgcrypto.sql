-- 0050_mirror_hash_pgcrypto.sql
--
-- Migration 0048 computes per-row mirror hashes with `digest(..., 'sha256')`.
-- That function is provided by pgcrypto, not by PostgreSQL core, so the
-- extension must exist before any trigger fires. CI's postgis image ships the
-- extension but does not enable it by default; enable it explicitly here so a
-- fresh database and an already-migrated database behave identically.

CREATE EXTENSION IF NOT EXISTS pgcrypto;
