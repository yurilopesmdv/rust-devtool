import logging

from redis.asyncio import Redis
from redis.exceptions import RedisError

logger = logging.getLogger(__name__)


class Cache:
    def __init__(self, url: str):
        self.client = Redis.from_url(
            url, socket_connect_timeout=2, socket_timeout=2, decode_responses=True
        )

    async def get(self, key: str) -> str | None:
        try:
            return await self.client.get(key)
        except (RedisError, OSError):
            logger.warning("Cache indisponível; seguindo pela origem")
            return None

    async def set(self, key: str, value: str):
        try:
            await self.client.set(key, value, ex=30)
        except (RedisError, OSError):
            logger.warning("Cache indisponível; seguindo pela origem")

    async def remove(self, key: str):
        try:
            await self.client.delete(key)
        except (RedisError, OSError):
            logger.warning("Cache indisponível; mantendo TTL limitado")

    async def close(self):
        await self.client.aclose()
