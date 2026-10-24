import { database } from './client.js';
function collection() {
  return database().collection<{ _id: string; message: string }>('examples');
}
export async function prepare() {
  await collection().createIndex({ message: 1 });
  await collection().updateOne(
    { _id: 'default' },
    { $setOnInsert: { message: 'API funcionando' } },
    { upsert: true },
  );
}
