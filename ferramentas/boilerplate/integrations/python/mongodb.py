from pymongo import AsyncMongoClient


class Database:
    def __init__(self, url: str):
        self.client = AsyncMongoClient(url, serverSelectionTimeoutMS=5000, socketTimeoutMS=5000)
        self.database = self.client.get_default_database()

    async def open(self):
        await self.ready()

    async def ready(self):
        await self.database.command("ping")

    async def close(self):
        await self.client.close()
