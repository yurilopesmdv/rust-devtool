import assert from 'node:assert/strict';
import test from 'node:test';
import { app } from '../dist/app.js';
test('health, API e CORS funcionam', async () => {
  const server = app.listen(0, '127.0.0.1');
  await new Promise(resolve => server.once('listening', resolve));
  try {
    const base = `http://127.0.0.1:${server.address().port}`;
    const response = await fetch(`${base}/health`);
    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), { status: 'ok' });
    const example = await fetch(`${base}/api/v1/example`, {
      headers: { Origin: 'http://localhost:{{WEB_PORT}}' },
    });
    assert.equal(
      example.headers.get('access-control-allow-origin'),
      'http://localhost:{{WEB_PORT}}',
    );
    assert.equal(example.status, 200);
  } finally {
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  }
});
