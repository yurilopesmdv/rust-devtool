import { database } from '../../../infrastructure/database/client.js';
function collection() {
  return database().collection<{ _id: string; message: string; display_name?: string }>('examples');
}
export async function read() {
  const row = await collection().findOne({ _id: 'default' });
  if (!row) throw new Error('Exemplo ausente');
  return { message: row.message, display_name: row.display_name ?? row.message };
}
export async function write(message: string) {
  await collection().updateOne(
    { _id: 'default' },
    { $set: { message, display_name: message } },
    { upsert: true },
  );
  return { message, display_name: message };
}
