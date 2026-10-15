-- Spatial stories (RFC-014 Story Engine): versioned, shareable scene specs.
--
-- A story is a presentation-layer document: ordered stops, a deterministic
-- camera timeline, optional navigation paths and captions. It references
-- spatial identities (parcels, geometry versions) but is NOT a protocol
-- fact — opening a story never writes chain state and never mutates
-- geometry history. The definition lives here as JSONB with an explicit
-- schema_version so old links keep playing as the frontend evolves.
--
-- Story exports are client-rendered video receipts (v0): the browser
-- captures the same deterministic timeline the player uses; this table is
-- the auditable ledger of what was exported, not a server renderer.

CREATE TABLE IF NOT EXISTS stories (
    id              VARCHAR(64) PRIMARY KEY,          -- stable story id (URL slug)
    schema_version  INTEGER NOT NULL DEFAULT 1,
    story_type      TEXT NOT NULL,                    -- journey | history | discovery
    title           TEXT NOT NULL,
    description     TEXT,
    author_wallet   TEXT,                             -- creator wallet when known
    duration_s      DOUBLE PRECISION NOT NULL,
    stop_count      INTEGER NOT NULL DEFAULT 0,
    entity_refs     TEXT[] NOT NULL DEFAULT '{}',     -- parcel / asset pubkeys referenced
    definition      JSONB NOT NULL,                   -- full StoryDefinition document
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_stories_type ON stories (story_type, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_stories_entity ON stories USING GIN (entity_refs);

CREATE TABLE IF NOT EXISTS story_exports (
    id           VARCHAR(64) PRIMARY KEY,
    story_id     VARCHAR(64) NOT NULL REFERENCES stories (id) ON DELETE CASCADE,
    aspect       TEXT NOT NULL,                       -- 9:16 | 16:9 | 1:1
    width        INTEGER NOT NULL,
    height       INTEGER NOT NULL,
    fps          INTEGER NOT NULL,
    duration_s   DOUBLE PRECISION NOT NULL,
    status       TEXT NOT NULL DEFAULT 'running',     -- running | done | failed | cancelled
    progress     REAL NOT NULL DEFAULT 0,
    error        TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_story_exports_story ON story_exports (story_id, created_at DESC);
