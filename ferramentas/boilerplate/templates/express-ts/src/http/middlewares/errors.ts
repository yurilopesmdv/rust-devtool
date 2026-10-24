import type { ErrorRequestHandler } from 'express';
export const errors: ErrorRequestHandler = (error, _req, res, _next) => {
  console.error(error);
  res.status(500).json({ error: 'Erro interno' });
};
