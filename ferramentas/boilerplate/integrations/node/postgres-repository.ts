import { pool } from '../../../infrastructure/database/client.js';
export async function read() {
  const result = await pool().query<{ message: string }>(
    'SELECT message FROM examples WHERE key = $1',
    ['default'],
  );
  if (!result.rows[0]) throw new Error('Execute db:prepare');
  return result.rows[0];
}
export async function write(message: string) {
  await pool().query(
    'INSERT INTO examples(key,message) VALUES ($1,$2) ON CONFLICT(key) DO UPDATE SET message=EXCLUDED.message',
    ['default', message],
  );
  return { message };
}
