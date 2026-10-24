import { Injectable, Module, type OnApplicationShutdown, type OnModuleInit } from '@nestjs/common';
import { close, initialize } from './lifecycle.js';
@Injectable()
class IntegrationLifecycle implements OnModuleInit, OnApplicationShutdown {
  async onModuleInit() {
    await initialize();
  }
  async onApplicationShutdown() {
    await close();
  }
}

@Module({ providers: [IntegrationLifecycle] })
export class InfrastructureModule {}
