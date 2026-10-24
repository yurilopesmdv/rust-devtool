import { MongoClient } from 'mongodb';
import { readConfig } from '../config.js';
let client: MongoClient | undefined;
function collection() { if (!client) throw new Error('Banco não inicializado'); return client.db().collection<{_id:string,message:string}>('examples'); }
export async function initialize() { client = new MongoClient(readConfig('DATABASE_URL')!, { serverSelectionTimeoutMS: 5000 }); await client.connect(); await ready(); }
export async function ready() { if (!client) throw new Error('Banco não inicializado'); await client.db().command({ ping: 1 }); }
export async function close() { await client?.close(); }
export async function prepare() {
  await collection().createIndex({ message: 1 });
  await collection().updateOne({ _id: 'default' }, { $setOnInsert: { message: 'API funcionando' } }, { upsert: true });
}
export async function read() {
  const entry = await collection().findOne({ _id: 'default' });
  if (!entry) throw new Error('Execute db:prepare');
  return { message: entry.message };
}
export async function write(message: string) { await collection().updateOne({ _id: 'default' }, { $set: { message } }, { upsert: true }); return { message }; }
