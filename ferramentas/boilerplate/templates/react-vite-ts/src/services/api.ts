const base = import.meta.env?.VITE_API_URL ?? 'http://localhost:3000';
export async function getExample(): Promise<{ message: string }> {
  const response = await fetch(`${base}/api/v1/example`);
  if (!response.ok) throw new Error('Falha na API');
  return response.json();
}
