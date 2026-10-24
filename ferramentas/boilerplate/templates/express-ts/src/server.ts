import { app } from './app.js';
import { config } from './config/index.js';
const server = app.listen(
  config.port,
  '0.0.0.0',
  () => console.log(`API: http://localhost:${config.port}`),
);
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => {
    server.close(() => process.exit(0));
  });
}
