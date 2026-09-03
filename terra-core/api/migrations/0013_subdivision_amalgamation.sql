-- Parcel subdivision & amalgamation records (RFC-008)
CREATE TABLE IF NOT EXISTS subdivision_records (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_parcel_id       UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    sub_parcel_id            UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    original_geometry_hash   TEXT NOT NULL,
    new_geometry_hash        TEXT NOT NULL,
    surveyor_attestation_id  UUID,
    rights_migrated          BOOLEAN NOT NULL DEFAULT false,
    attestations_migrated    BOOLEAN NOT NULL DEFAULT false,
    initiated_by             TEXT NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at             TIMESTAMPTZ,
    status                   TEXT NOT NULL DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS amalgamation_records (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    result_parcel_id         UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    source_parcel_id         UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    source_geometry_hash     TEXT NOT NULL,
    result_geometry_hash     TEXT NOT NULL,
    rights_merged            BOOLEAN NOT NULL DEFAULT false,
    initiated_by             TEXT NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at             TIMESTAMPTZ,
    status                   TEXT NOT NULL DEFAULT 'pending'
);

CREATE INDEX IF NOT EXISTS idx_subdiv_original  ON subdivision_records (original_parcel_id);
CREATE INDEX IF NOT EXISTS idx_subdiv_sub       ON subdivision_records (sub_parcel_id);
CREATE INDEX IF NOT EXISTS idx_amalg_result     ON amalgamation_records (result_parcel_id);
CREATE INDEX IF NOT EXISTS idx_amalg_source     ON amalgamation_records (source_parcel_id);
