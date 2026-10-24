import { MongoClient } from 'mongodb';
import { readConfig } from '../../config/env.js';
let client: MongoClient | undefined;
export function database() {
  if (!client) throw new Error('Banco não inicializado');
  return client.db();
}
export async function initialize() {
  client = new MongoClient(readConfig('DATABASE_URL')!, {
    serverSelectionTimeoutMS: 5000,
    socketTimeoutMS: 5000,
  });
  await client.connect();
  await ready();
}
export async function ready() {
  if (!client) throw new Error('Banco não inicializado');
  await client.db().command({ ping: 1 });
}
export async function close() {
  await client?.close();
  client = undefined;
}
