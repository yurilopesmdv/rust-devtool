import { getExample } from './api.js';
document.querySelector('#load').addEventListener('click', async () => {
  try {
    document.querySelector('#result').textContent = (await getExample()).message;
  } catch {
    document.querySelector('#result').textContent = 'API indisponível';
  }
});
