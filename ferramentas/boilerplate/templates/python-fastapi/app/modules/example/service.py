from app.modules.example.schemas import ExampleResponse


class ExampleService:
    async def get(self) -> ExampleResponse:
        return ExampleResponse(message="API {{PROJECT_NAME}} funcionando")


def get_service() -> ExampleService:
    return ExampleService()
