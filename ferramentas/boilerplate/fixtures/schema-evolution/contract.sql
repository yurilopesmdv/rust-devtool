-- migrate:up
ALTER TABLE examples ALTER COLUMN display_name SET NOT NULL;
-- migrate:down
DO $$ BEGIN RAISE EXCEPTION 'Reversão exige revisão da janela de compatibilidade'; END $$;
