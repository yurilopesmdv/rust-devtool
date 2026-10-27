-- migrate:up
ALTER TABLE examples ALTER COLUMN display_name DROP NOT NULL;
-- migrate:down
DO $$ BEGIN RAISE EXCEPTION 'Recuperação não deve ser desfeita sem nova revisão'; END $$;
