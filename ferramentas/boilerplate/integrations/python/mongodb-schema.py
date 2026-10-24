from app.infrastructure.database import Database


async def prepare(database: Database):
    collection = database.database["examples"]
    await collection.update_one(
        {"_id": "default"},
        {"$setOnInsert": {"message": "API funcionando"}},
        upsert=True,
    )
