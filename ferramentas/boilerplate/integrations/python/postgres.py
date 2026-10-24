from psycopg_pool import AsyncConnectionPool


class Database:
    def __init__(self, url: str):
        self.pool = AsyncConnectionPool(url, min_size=1, max_size=10, timeout=5, open=False)

    async def open(self):
        await self.pool.open(wait=True, timeout=5)

    async def ready(self):
        async with self.pool.connection() as connection:
            await connection.execute("SELECT 1")

    async def close(self):
        await self.pool.close()
