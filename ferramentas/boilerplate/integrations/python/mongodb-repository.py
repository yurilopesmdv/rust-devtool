from app.infrastructure.database import Database
from app.modules.example.schemas import ExampleResponse


class ExampleRepository:
    def __init__(self, database: Database):
        self.collection = database.database["examples"]

    async def read(self) -> ExampleResponse:
        entry = await self.collection.find_one({"_id": "default"})
        if entry is None:
            raise RuntimeError("Execute python -m app.infrastructure.prepare")
        return ExampleResponse(message=entry["message"])

    async def write(self, message: str) -> ExampleResponse:
        await self.collection.update_one(
            {"_id": "default"}, {"$set": {"message": message}}, upsert=True
        )
        return ExampleResponse(message=message)
