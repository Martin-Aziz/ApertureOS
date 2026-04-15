import base64

import pytest
from fastapi.testclient import TestClient

from api import routes
from core.background_removal import ModelStatus
from main import app


class FakeBackgroundRemovalService:
    def status(self) -> ModelStatus:
        return ModelStatus(ready=True, provider="fake")

    async def remove_background(self, image_bytes: bytes) -> bytes:
        return image_bytes + b"-processed"


@pytest.fixture(autouse=True)
def patch_background_service(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        routes,
        "background_service",
        FakeBackgroundRemovalService(),
    )


@pytest.fixture()
def client() -> TestClient:
    return TestClient(app)


def test_health_live_should_return_ok(client: TestClient) -> None:
    response = client.get("/health/live")
    assert response.status_code == 200
    assert response.json()["status"] == "live"


def test_remove_background_should_require_service_secret(
    client: TestClient,
) -> None:
    payload = {"image_base64": base64.b64encode(b"input").decode("ascii")}
    response = client.post("/v1/remove-background", json=payload)

    assert response.status_code == 401


def test_metrics_should_return_prometheus_payload(client: TestClient) -> None:
    response = client.get("/metrics")

    assert response.status_code == 200
    assert "pixelforge_ai_service_info" in response.text
    assert "pixelforge_ai_model_ready" in response.text


def test_remove_background_should_return_processed_image(
    client: TestClient,
) -> None:
    payload = {"image_base64": base64.b64encode(b"input").decode("ascii")}

    response = client.post(
        "/v1/remove-background",
        json=payload,
        headers={
            "x-ai-service-secret": (
                "development-service-secret-change-before-production"
            )
        },
    )

    assert response.status_code == 200
    body = response.json()
    assert body["provider"] == "fake"
    assert base64.b64decode(body["image_base64"]) == b"input-processed"
