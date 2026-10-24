import type { Request, Response } from 'express';
import { getExample } from '../services/example.service.js';
export function exampleController(_request: Request, response: Response) { response.json(getExample()); }
