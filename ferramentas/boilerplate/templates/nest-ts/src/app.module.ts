import { Controller, Get, Module } from '@nestjs/common';
import { ExampleModule } from './modules/example/example.module.js';
@Controller('health')
class HealthController {
  @Get()
  get() {
    return { status: 'ok' };
  }
}

@Module({ imports: [ExampleModule], controllers: [HealthController], providers: [] })
export class AppModule {}
