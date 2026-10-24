import { database } from '../../../infrastructure/database/client.js';
function collection() {
  return database().collection<{ _id: string; message: string }>('examples');
}
export async function read() {
  const entry = await collection().findOne({ _id: 'default' });
  if (!entry) throw new Error('Execute db:prepare');
  return { message: entry.message };
}
export async function write(message: string) {
  await collection().updateOne({ _id: 'default' }, { $set: { message } }, { upsert: true });
  return { message };
}
