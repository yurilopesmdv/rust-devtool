const base = 'http://localhost:3000';
export async function getExample() { const response = await fetch(`${base}/api/v1/example`); if (!response.ok) throw new Error('Falha na API'); return response.json(); }
