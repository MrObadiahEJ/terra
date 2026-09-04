-- AuthorityRegistry + IPFS document-anchor mirrors.
--
-- The frontend has always called /authority-registry/* and /ipfs-docs/* but
-- no backend served them (every call 404'd). These tables close that gap,
-- mirroring the on-chain AuthorityRegistry / ValidatorEndorsement accounts
-- and the off-chain document-anchor log.

-- On-chain AuthorityRegistry mirror (PDA: ["authority_registry"]).
CREATE TABLE IF NOT EXISTS authority_registries (
    id                    BIGSERIAL PRIMARY KEY,
    pubkey                TEXT NOT NULL UNIQUE,
    admin                 TEXT NOT NULL,
    validators            TEXT[] NOT NULL DEFAULT '{}',
    required_endorsements SMALLINT NOT NULL DEFAULT 1,
    mode                  SMALLINT NOT NULL DEFAULT 0, -- 0=bootstrap, 1=peer-consensus
    version               INT NOT NULL DEFAULT 0,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- On-chain ValidatorEndorsement mirror (proposal to add a validator).
CREATE TABLE IF NOT EXISTS registry_endorsements (
    id              BIGSERIAL PRIMARY KEY,
    registry_pubkey TEXT NOT NULL REFERENCES authority_registries(pubkey) ON DELETE CASCADE,
    proposed        TEXT NOT NULL,
    endorsers       TEXT[] NOT NULL DEFAULT '{}',
    required        SMALLINT NOT NULL DEFAULT 1,
    added_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (registry_pubkey, proposed)
);

-- Off-chain IPFS document-anchor log (content-addressed deed/survey anchors
-- bound to an attestation pubkey).
CREATE TABLE IF NOT EXISTS document_anchors (
    id                BIGSERIAL PRIMARY KEY,
    attestation_pubkey TEXT NOT NULL,
    cid               TEXT NOT NULL,
    content_hash      TEXT NOT NULL,
    category          TEXT NOT NULL,
    registered_by     TEXT NOT NULL DEFAULT '',
    registered_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_authority_registries_admin ON authority_registries (admin);
CREATE INDEX IF NOT EXISTS idx_registry_endorsements_registry ON registry_endorsements (registry_pubkey);
CREATE INDEX IF NOT EXISTS idx_document_anchors_attestation ON document_anchors (attestation_pubkey);
CREATE INDEX IF NOT EXISTS idx_document_anchors_cid ON document_anchors (cid);
