import { createClient } from 'redis';
import { readConfig } from '../../config/env.js';
let client: ReturnType<typeof createClient> | undefined;
export async function initialize() {
  const url = readConfig('REDIS_URL');
  client = createClient({
    url,
    socket: { connectTimeout: 2000, reconnectStrategy: false },
    disableOfflineQueue: true,
  });
  client.on('error', () => {
    console.error('Cache indisponível; seguindo sem cache');
  });
  try {
    await client.connect();
  } catch {
    if (client.isOpen) client.destroy();
    client = undefined;
  }
}
export async function get(key: string): Promise<string | null> {
  try {
    return client?.isReady ? await client.get(key) : null;
  } catch {
    return null;
  }
}
export async function set(key: string, value: string) {
  try {
    if (client?.isReady) await client.set(key, value, { EX: 30 });
  } catch { /* Cache não é fonte primária. */ }
}
export async function remove(key: string) {
  try {
    if (client?.isReady) await client.del(key);
  } catch { /* Mantém TTL limitado em uma falha. */ }
}
export async function close() {
  if (client?.isOpen) client.destroy();
}
