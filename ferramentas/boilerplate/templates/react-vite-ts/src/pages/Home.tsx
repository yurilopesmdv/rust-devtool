import { useState } from 'react';
import { getExample } from '../services/api.js';
export function Home() {
  const [message, setMessage] = useState('{{PROJECT_NAME}}');
  return (
    <main>
      <h1>{message}</h1>
      <button
        onClick={() =>
          getExample().then(data => setMessage(data.message)).catch(() =>
            setMessage('API indisponível')
          )}
      >
        Consultar API
      </button>
    </main>
  );
}
