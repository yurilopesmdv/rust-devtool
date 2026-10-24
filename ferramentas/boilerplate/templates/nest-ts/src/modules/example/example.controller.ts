import { Controller, Get, Inject } from '@nestjs/common';
import { ExampleService } from './example.service.js';
@Controller('api/v1/example')
export class ExampleController {
 constructor(@Inject(ExampleService) private readonly service: ExampleService) {}
 @Get() get() { return this.service.get(); }
}
