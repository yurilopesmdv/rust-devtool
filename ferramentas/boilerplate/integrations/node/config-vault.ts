import { readFileSync } from 'node:fs';
export function readConfig(key: string, required = true): string | undefined {
  const direct = process.env[key];
  const path = process.env[`${key}_FILE`];
  if (direct !== undefined && path !== undefined) {
    throw new Error(`${key}: configure valor OU arquivo, não ambos`);
  }
  const value = path ? readFileSync(path, 'utf8').trim() : direct;
  if (required && !value) throw new Error(`${key} ausente`);
  return value;
}
