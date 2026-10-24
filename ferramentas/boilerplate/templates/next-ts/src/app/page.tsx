'use client';
import React, { useState } from 'react';
export default function Home() {
  const [message, setMessage] = useState('{{PROJECT_NAME}}');
  return (
    <main>
      <h1>{message}</h1>
      <button
        onClick={async () => {
          try {
            const response = await fetch(
              `${process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:3000'}/api/v1/example`,
            );
            if (!response.ok) throw new Error();
            setMessage((await response.json()).message);
          } catch {
            setMessage('API indisponível');
          }
        }}
      >
        Consultar API
      </button>
    </main>
  );
}
