CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE parcels (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    owner       TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending',
    geometry    geometry(Polygon, 4326) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_parcels_geometry ON parcels USING GIST (geometry);
CREATE INDEX idx_parcels_owner ON parcels (owner);
