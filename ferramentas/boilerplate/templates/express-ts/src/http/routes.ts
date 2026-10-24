import { Router } from 'express';
import { exampleController } from '../modules/example/controllers/example.controller.js';
export const routes = Router();
routes.get('/example', exampleController);
