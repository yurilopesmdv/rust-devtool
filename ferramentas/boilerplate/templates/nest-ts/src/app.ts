import 'reflect-metadata';
import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module.js';
export async function createApp() { const app = await NestFactory.create(AppModule, { logger: false }); app.enableCors({ origin: process.env.FRONTEND_ORIGIN ?? 'http://localhost:{{WEB_PORT}}' }); app.enableShutdownHooks(); return app; }
