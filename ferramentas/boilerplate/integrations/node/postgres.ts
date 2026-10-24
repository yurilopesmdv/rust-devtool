import pg from 'pg';
import { readConfig } from '../config.js';
let pool: pg.Pool | undefined;
export async function initialize() {
  pool = new pg.Pool({ connectionString: readConfig('DATABASE_URL'), connectionTimeoutMillis: 5000, max: 10 });
  await pool.query('SELECT 1');
}
export async function ready() { if (!pool) throw new Error('Banco não inicializado'); await pool.query('SELECT 1'); }
export async function close() { await pool?.end(); }
export async function prepare() {
  if (!pool) throw new Error('Banco não inicializado');
  await pool.query('CREATE TABLE IF NOT EXISTS examples (key TEXT PRIMARY KEY, message TEXT NOT NULL)');
  await pool.query("INSERT INTO examples(key,message) VALUES ('default','API funcionando') ON CONFLICT DO NOTHING");
}
export async function read() {
  if (!pool) throw new Error('Banco não inicializado');
  const result = await pool.query<{message:string}>("SELECT message FROM examples WHERE key = $1", ['default']);
  if (!result.rows[0]) throw new Error('Execute db:prepare');
  return result.rows[0];
}
export async function write(message: string) {
  if (!pool) throw new Error('Banco não inicializado');
  await pool.query("INSERT INTO examples(key,message) VALUES ($1,$2) ON CONFLICT(key) DO UPDATE SET message=EXCLUDED.message", ['default',message]);
  return { message };
}
