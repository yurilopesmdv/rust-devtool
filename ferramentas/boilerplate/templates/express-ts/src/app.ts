import cors from 'cors';
import express from 'express';
import { config } from './config/index.js';
import { errors } from './http/middlewares/errors.js';
import { routes } from './http/routes.js';
export const app = express();
app.disable('x-powered-by');
app.use(cors({ origin: config.frontendOrigin }));
app.use(express.json());
app.get('/health', (_req, res) => {
  res.json({ status: 'ok' });
});
app.use('/api/v1', routes);
app.use(errors);
