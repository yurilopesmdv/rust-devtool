import os

import pytest
from fastapi.testclient import TestClient

from app.main import create_app


@pytest.mark.skipif(
    not (os.environ.get("DATABASE_URL") or os.environ.get("DATABASE_URL_FILE")),
    reason="Requer banco de teste disponível",
)
def test_closed_database_affects_readiness_but_not_liveness():
    app = create_app()

    # This route belongs only to this in-memory test app, never the generated server.
    @app.post("/_test/close-database", include_in_schema=False)
    async def close_database():
        await app.state.database.close()

    with TestClient(app) as client:
        assert client.get("/ready").status_code == 200
        assert client.post("/_test/close-database").status_code == 200
        assert client.get("/ready").status_code == 503
        assert client.get("/health").status_code == 200
