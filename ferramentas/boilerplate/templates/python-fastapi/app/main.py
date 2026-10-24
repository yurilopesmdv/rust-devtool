import os

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.modules.example.router import router


def create_app() -> FastAPI:
    app = FastAPI(title="{{PROJECT_NAME}}")
    app.add_middleware(
        CORSMiddleware,
        allow_origins=[os.environ.get("FRONTEND_ORIGIN", "http://localhost:{{WEB_PORT}}")],
        allow_methods=["GET", "POST", "OPTIONS"],
        allow_headers=["Content-Type"],
    )
    app.include_router(router)

    @app.get("/health")
    async def health() -> dict[str, str]:
        return {"status": "ok"}

    @app.get("/ready")
    async def ready() -> dict[str, str]:
        return {"status": "ok"}

    return app
