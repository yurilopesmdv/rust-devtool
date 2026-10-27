-- migrate:up
CREATE TABLE IF NOT EXISTS examples (key TEXT PRIMARY KEY, message TEXT NOT NULL);

-- migrate:down
DO $$ BEGIN RAISE EXCEPTION 'Rollback destrutivo não distribuído; use migration corretiva revisada'; END $$;
