-- 0047_runtime_signed_manifest_binding.sql
--
-- Binds a mutable runtime manifest (`world_manifests`) to a verified,
-- publisher-signed `terra.manifest/v1` envelope stored in `signed_manifests`
-- (migration 0044). When a manifest carries a `signed_content_address`, scene
-- resolution (`GET /runtime/scene/{manifest_id}`) is grounded in the signed
-- envelope: objects are read from the re-verified payload rather than the
-- mutable `manifest_objects` rows, and each scene node carries the object's
-- content address so clients can reject bytes that do not match the publisher's
-- signed manifest. Nullable and additive: manifests without a binding keep the
-- legacy manifest_objects path.

ALTER TABLE world_manifests
    ADD COLUMN IF NOT EXISTS signed_content_address TEXT;

CREATE INDEX IF NOT EXISTS idx_world_manifests_signed_content_address
    ON world_manifests (signed_content_address)
    WHERE signed_content_address IS NOT NULL;
