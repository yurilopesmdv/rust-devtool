from fastapi.testclient import TestClient

from app.main import create_app


def test_health_and_example():
    with TestClient(create_app()) as client:
        assert client.get("/health").status_code == 200
        assert client.get("/ready").status_code == 200
        assert isinstance(client.get("/api/v1/example").json()["message"], str)
