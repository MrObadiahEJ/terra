-- RFC-012 legacy sweep: the on-chain parcel `Attestation` model
-- (attest / rotate_validators / register_document / migrate_attestations /
-- migrate_attestation_to_claim) was removed from terra_registry. Drop the
-- off-chain mirror that existed only to shadow it.
--
-- Kept: subdivision_records.surveyor_attestation_id (historical data
-- reference, no FK) and the document/evidence tables used by the
-- verification pipeline (evidence lives under its own routes).
DROP TABLE IF EXISTS validator_rotations;
DROP TABLE IF EXISTS validations;
DROP TABLE IF EXISTS attestations;
DROP TABLE IF EXISTS documents;
ALTER TABLE subdivision_records DROP COLUMN IF EXISTS attestations_migrated;
