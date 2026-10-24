import { createApp } from './app.js';
const port = Number(process.env.PORT ?? 3000);
if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('PORT inválida');
const app = await createApp();
await app.listen(port, '0.0.0.0');
console.log(`API: http://localhost:${port}`);
