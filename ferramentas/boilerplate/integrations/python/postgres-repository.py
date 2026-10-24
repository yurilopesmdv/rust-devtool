from app.infrastructure.database import Database
from app.modules.example.schemas import ExampleResponse


class ExampleRepository:
    def __init__(self, database: Database):
        self.database = database

    async def read(self) -> ExampleResponse:
        async with self.database.pool.connection() as connection:
            cursor = await connection.execute(
                "SELECT message FROM examples WHERE key = %s", ("default",)
            )
            row = await cursor.fetchone()
        if row is None:
            raise RuntimeError("Execute python -m app.infrastructure.prepare")
        return ExampleResponse(message=row[0])

    async def write(self, message: str) -> ExampleResponse:
        async with self.database.pool.connection() as connection:
            await connection.execute(
                "INSERT INTO examples(key, message) VALUES (%s, %s) "
                "ON CONFLICT(key) DO UPDATE SET message = EXCLUDED.message",
                ("default", message),
            )
        return ExampleResponse(message=message)
