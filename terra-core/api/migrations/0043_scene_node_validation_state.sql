ALTER TABLE scene_nodes
    ADD COLUMN IF NOT EXISTS validation_state TEXT NOT NULL DEFAULT 'preliminary';

ALTER TABLE scene_nodes DROP CONSTRAINT IF EXISTS scene_nodes_validation_check;
ALTER TABLE scene_nodes
    ADD CONSTRAINT scene_nodes_validation_check
    CHECK (validation_state IN ('missing', 'preliminary', 'independently_valid'));
