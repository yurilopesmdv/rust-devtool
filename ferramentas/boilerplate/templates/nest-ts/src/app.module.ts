import { Controller, Get, Module } from '@nestjs/common';
import { ExampleModule } from './modules/example/example.module.js';
@Controller()
class HealthController {
  @Get('health')
  get() {
    return { status: 'ok' };
  }

  @Get('ready')
  ready() {
    return { status: 'ok' };
  }
}

@Module({ imports: [ExampleModule], controllers: [HealthController], providers: [] })
export class AppModule {}
