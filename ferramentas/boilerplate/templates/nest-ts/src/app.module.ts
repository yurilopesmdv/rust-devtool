import { Module, Controller, Get } from '@nestjs/common';
import { ExampleController } from './modules/example/example.controller.js';
import { ExampleService } from './modules/example/example.service.js';
@Controller('health')
class HealthController { @Get() get() { return { status: 'ok' }; } }
@Module({ controllers: [HealthController, ExampleController], providers: [ExampleService] })
export class AppModule {}
