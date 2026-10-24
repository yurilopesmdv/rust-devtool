from app.infrastructure.database import Database


async def prepare(database: Database):
    async with database.pool.connection() as connection:
        await connection.execute(
            "CREATE TABLE IF NOT EXISTS examples (key TEXT PRIMARY KEY, message TEXT NOT NULL)"
        )
        await connection.execute(
            "INSERT INTO examples(key, message) VALUES ('default', 'API funcionando') "
            "ON CONFLICT DO NOTHING"
        )
