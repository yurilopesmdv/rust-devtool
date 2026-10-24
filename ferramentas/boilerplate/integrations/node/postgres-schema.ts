import { pool } from './client.js';
export async function prepare() {
  await pool().query(
    'CREATE TABLE IF NOT EXISTS examples (key TEXT PRIMARY KEY, message TEXT NOT NULL)',
  );
  await pool().query(
    "INSERT INTO examples(key,message) VALUES ('default','API funcionando') ON CONFLICT DO NOTHING",
  );
}
