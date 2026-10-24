import { close, initialize } from './client.js';
import { prepare } from './schema.js';
async function main() {
  try {
    await initialize();
    await prepare();
  } finally {
    await close();
  }
}
void main().catch(() => {
  console.error('Falha ao preparar banco; confira configuração e conectividade');
  process.exitCode = 1;
});
