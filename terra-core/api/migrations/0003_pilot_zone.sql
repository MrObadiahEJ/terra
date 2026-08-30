-- Seed the Soa/Biteng pilot zone (Cameroon, Centre Region).
-- Soa town centre: lat 3.9833, lon 11.6000. Buffer ~1.5 km around it as the survey area.
INSERT INTO pilot_zones (id, name, description, geometry)
SELECT
    'e8c9a000-0000-4000-8000-000000000001'::uuid,
    'Soa/Biteng',
    'Pilot survey zone around Soa and Biteng (Cameroon). Baseline OSM + drone photogrammetry fusion.',
    ST_Buffer(ST_SetSRID(ST_MakePoint(11.6000, 3.9833), 4326), 0.014)
ON CONFLICT DO NOTHING;
