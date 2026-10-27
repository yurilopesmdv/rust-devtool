import assert from 'node:assert/strict';
import test from 'node:test';
import { createApp } from '../dist/app.js';
test('health e serviço da API', async () => {
  const app = await createApp();
  await app.listen(0, '127.0.0.1');
  try {
    const url = await app.getUrl();
    assert.equal((await fetch(`${url}/health`)).status, 200);
    assert.equal((await fetch(`${url}/ready`)).status, 200);
    assert.equal((await fetch(`${url}/api/v1/example`)).status, 200);
  } finally {
    await app.close();
  }
});
