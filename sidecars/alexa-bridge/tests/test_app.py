"""Tests for the Flask wiring, including that verification is really on.

These do not forge an Alexa signature — that would mean holding Amazon's
private key. What they check is that an unsigned request cannot reach a
handler, which is the property that matters.
"""

import pytest

from alexa_bridge.app import SKILL_ROUTE, create_app
from alexa_bridge.config import Settings

SETTINGS = Settings(
    skill_id="amzn1.ask.skill.test",
    backend_base_url="http://127.0.0.1:8000",
    bridge_secret="test-bridge-secret",
)


@pytest.fixture
def client():
    app = create_app(SETTINGS)
    app.config.update(TESTING=True)
    return app.test_client()


def test_health_reports_ok(client):
    response = client.get("/health")
    assert response.status_code == 200
    assert response.get_json() == {"status": "ok"}


def test_an_unsigned_request_is_rejected(client):
    # No Signature / SignatureCertChainUrl headers: ask-sdk must refuse it
    # before any handler runs.
    response = client.post(
        SKILL_ROUTE,
        json={
            "version": "1.0",
            "session": {"new": True, "sessionId": "s", "application": {"applicationId": "x"}},
            "request": {
                "type": "IntentRequest",
                "requestId": "r",
                "timestamp": "2026-09-30T00:00:00Z",
                "intent": {"name": "AddItemIntent", "slots": {}},
            },
        },
    )
    assert response.status_code in (400, 403), response.data


def test_a_request_with_a_bogus_signature_is_rejected(client):
    response = client.post(
        SKILL_ROUTE,
        headers={
            "Signature": "bogus",
            "SignatureCertChainUrl": "https://s3.amazonaws.com/echo.api/echo-api-cert.pem",
        },
        json={"version": "1.0", "request": {"type": "LaunchRequest", "requestId": "r"}},
    )
    assert response.status_code in (400, 403), response.data


def test_the_skill_route_rejects_a_get(client):
    assert client.get(SKILL_ROUTE).status_code == 405
