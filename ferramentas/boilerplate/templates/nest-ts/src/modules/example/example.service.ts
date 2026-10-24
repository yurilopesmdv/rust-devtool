import { Injectable } from '@nestjs/common';

@Injectable()
export class ExampleService {
  get() {
    return { message: 'API {{PROJECT_NAME}} funcionando' };
  }
}
