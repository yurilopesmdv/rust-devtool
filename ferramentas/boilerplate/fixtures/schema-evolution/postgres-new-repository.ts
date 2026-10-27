import { pool } from '../../../infrastructure/database/client.js';
export async function read() {
  const result = await pool().query<{ message: string; display_name: string | null }>(
    'SELECT message, display_name FROM examples WHERE key = $1',
    ['default'],
  );
  const row = result.rows[0];
  if (!row) throw new Error('Exemplo ausente');
  return { message: row.message, display_name: row.display_name ?? row.message };
}
export async function write(message: string) {
  await pool().query(
    'INSERT INTO examples(key,message,display_name) VALUES ($1,$2,$2) ON CONFLICT(key) DO UPDATE SET message=EXCLUDED.message,display_name=EXCLUDED.display_name',
    ['default', message],
  );
  return { message, display_name: message };
}
