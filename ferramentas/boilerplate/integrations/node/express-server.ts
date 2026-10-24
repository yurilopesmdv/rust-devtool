import { app } from './app.js';
import { config } from './config/index.js';
import { close, initialize } from './infrastructure/lifecycle.js';
async function main() {
  await initialize();
  const server = app.listen(
    config.port,
    '0.0.0.0',
    () => console.log(`API: http://localhost:${config.port}`),
  );
  for (const signal of ['SIGINT', 'SIGTERM']) {
    process.on(signal, () => {
      server.close(() => {
        void close().finally(() => process.exit(0));
      });
    });
  }
}
void main().catch(() => {
  console.error('Falha no bootstrap; confira configuração e dependências');
  process.exitCode = 1;
});
