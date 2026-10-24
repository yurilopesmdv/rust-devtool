export function readConfig(key: string, required = true): string | undefined {
  const value = process.env[key];
  if (required && !value) throw new Error(`${key} ausente`);
  return value;
}
