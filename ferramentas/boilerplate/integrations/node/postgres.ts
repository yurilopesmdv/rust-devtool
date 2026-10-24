import pg from 'pg';
import { readConfig } from '../../config/env.js';
let connection: pg.Pool | undefined;
export async function initialize() {
  connection = new pg.Pool({
    connectionString: readConfig('DATABASE_URL'),
    connectionTimeoutMillis: 5000,
    query_timeout: 5000,
    max: 10,
  });
  await pool().query('SELECT 1');
}
export async function ready() {
  if (!connection) throw new Error('Banco não inicializado');
  await pool().query('SELECT 1');
}
export async function close() {
  await connection?.end();
  connection = undefined;
}

export function pool(): pg.Pool {
  if (!connection) throw new Error('Banco não inicializado');
  return connection;
}
