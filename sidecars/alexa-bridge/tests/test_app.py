"""Tests for the Flask wiring, including that verification is really on.

These do not forge an Alexa signature — that would mean holding Amazon's
private key. What they check is that an unsigned request cannot reach a
handler, which is the property that matters.
"""

from unittest.mock import Mock

import pytest

from alexa_bridge import app as bridge_app
from alexa_bridge import environment
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


@pytest.mark.parametrize("source", ["defaults", "dotenv", "reference", "exported"])
def test_direct_startup_applies_resolved_port_and_logging(tmp_path, monkeypatch, source):
    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv("GROCERY_ENV_FILE", str(tmp_path / ".env"))
    monkeypatch.setenv("ALEXA_SKILL_ID", SETTINGS.skill_id)
    monkeypatch.setenv("ALEXA_BRIDGE_SECRET", SETTINGS.bridge_secret)
    monkeypatch.delenv("PORT", raising=False)
    monkeypatch.delenv("LOG_LEVEL", raising=False)
    values = {"PORT": "9092", "LOG_LEVEL": "DEBUG"}
    if source == "reference":
        references = {f"op://Development/bridge/{key}": value for key, value in values.items()}
        monkeypatch.setattr(environment, "read_secret", references.__getitem__)
        values = {key: f"op://Development/bridge/{key}" for key in values}
    if source != "defaults":
        (tmp_path / ".env").write_text("".join(f"{key}={value}\n" for key, value in values.items()))
    if source == "exported":
        monkeypatch.setenv("PORT", "9093")
        monkeypatch.setenv("LOG_LEVEL", "WARNING")
    flask_app = Mock()
    create_app = Mock(return_value=flask_app)
    configure_logging = Mock()
    monkeypatch.setattr(bridge_app, "create_app", create_app)
    monkeypatch.setattr(bridge_app.logging, "basicConfig", configure_logging)

    bridge_app.main()

    expected_port = {"defaults": 8081, "exported": 9093}.get(source, 9092)
    expected_level = {"defaults": "INFO", "exported": "WARNING"}.get(source, "DEBUG")
    configure_logging.assert_called_once_with(level=expected_level)
    flask_app.run.assert_called_once_with(host="127.0.0.1", port=expected_port)
