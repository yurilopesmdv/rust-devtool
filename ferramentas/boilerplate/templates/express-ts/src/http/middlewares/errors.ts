import type { ErrorRequestHandler } from 'express';

export const errors: ErrorRequestHandler = (error: unknown, _req, res, _next) => {
  let status = 500;
  let message = 'Erro interno';
  if (typeof error === 'object' && error !== null && 'type' in error) {
    if (error.type === 'entity.parse.failed') {
      status = 400;
      message = 'JSON inválido';
    } else if (error.type === 'entity.too.large') {
      status = 413;
      message = 'Payload excede o limite';
    }
  }
  if (status === 500) console.error('Falha interna durante a requisição');
  res.status(status).json({ error: message });
};
