-- migrate:up
UPDATE examples SET display_name = message WHERE display_name IS NULL;
SELECT 1/0;
-- migrate:down
