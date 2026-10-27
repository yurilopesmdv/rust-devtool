-- migrate:up
ALTER TABLE examples ADD COLUMN display_name TEXT;
-- migrate:down
DO $$ BEGIN RAISE EXCEPTION 'Reversão exige revisão da janela de compatibilidade'; END $$;
