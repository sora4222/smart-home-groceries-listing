"""Tests for the backend client: what it sends, and how it fails."""

import json

import pytest
import requests

from alexa_bridge.backend import BackendClient, BackendUnavailable
from alexa_bridge.config import Settings
from alexa_bridge.parsing import ParsedItem

SETTINGS = Settings(
    skill_id="amzn1.ask.skill.test",
    backend_base_url="http://backend:8000",
    bridge_secret="test-bridge-secret",
    backend_timeout_seconds=1.0,
)


class RecordingSession:
    """A stand-in for `requests.Session` that records the call."""

    def __init__(self, status_code=201, raises=None):
        self.status_code = status_code
        self.raises = raises
        self.calls = []

    def post(self, url, json=None, headers=None, timeout=None):
        if self.raises:
            raise self.raises
        self.calls.append({"url": url, "json": json, "headers": headers, "timeout": timeout})
        return _Response(self.status_code)


class _Response:
    def __init__(self, status_code):
        self.status_code = status_code

    @property
    def ok(self):
        return 200 <= self.status_code < 300


def test_posts_the_item_to_the_intake_endpoint():
    session = RecordingSession()
    BackendClient(SETTINGS, session=session).record_item(
        ParsedItem(item="oat milk", quantity=2),
        request_id="amzn1.echo-api.request.abc",
        raw_text="two oat milk",
    )

    call = session.calls[0]
    assert call["url"] == "http://backend:8000/api/intake/alexa"
    assert call["json"] == {
        "item": "oat milk",
        "quantity": 2,
        "external_id": "amzn1.echo-api.request.abc",
        "raw_text": "two oat milk",
    }


def test_sends_the_bridge_secret_as_a_header():
    session = RecordingSession()
    BackendClient(SETTINGS, session=session).record_item(
        ParsedItem(item="rice", quantity=1), request_id="req-1", raw_text=None
    )

    headers = session.calls[0]["headers"]
    assert headers["X-Bridge-Secret"] == "test-bridge-secret"
    # The secret must never travel in the URL or the body, where it would be
    # logged by proxies and access logs.
    assert "test-bridge-secret" not in session.calls[0]["url"]
    assert "test-bridge-secret" not in json.dumps(session.calls[0]["json"])


def test_passes_alexas_request_id_through_so_retries_deduplicate():
    session = RecordingSession()
    client = BackendClient(SETTINGS, session=session)
    parsed = ParsedItem(item="butter", quantity=1)

    client.record_item(parsed, request_id="same-id", raw_text=None)
    client.record_item(parsed, request_id="same-id", raw_text=None)

    assert [c["json"]["external_id"] for c in session.calls] == ["same-id", "same-id"]


def test_omits_raw_text_when_there_is_none():
    session = RecordingSession()
    BackendClient(SETTINGS, session=session).record_item(
        ParsedItem(item="bread", quantity=1), request_id="req-2", raw_text=None
    )
    assert "raw_text" not in session.calls[0]["json"]


def test_applies_the_configured_timeout():
    session = RecordingSession()
    BackendClient(SETTINGS, session=session).record_item(
        ParsedItem(item="bread", quantity=1), request_id="req-3", raw_text=None
    )
    assert session.calls[0]["timeout"] == 1.0


@pytest.mark.parametrize("status", [401, 422, 500, 503])
def test_an_error_status_is_never_reported_as_success(status):
    session = RecordingSession(status_code=status)
    with pytest.raises(BackendUnavailable):
        BackendClient(SETTINGS, session=session).record_item(
            ParsedItem(item="rice", quantity=1), request_id="req-4", raw_text=None
        )


def test_a_network_failure_raises_rather_than_passing_silently():
    session = RecordingSession(raises=requests.ConnectionError("refused"))
    with pytest.raises(BackendUnavailable):
        BackendClient(SETTINGS, session=session).record_item(
            ParsedItem(item="rice", quantity=1), request_id="req-5", raw_text=None
        )


def test_a_timeout_raises_rather_than_passing_silently():
    session = RecordingSession(raises=requests.Timeout("too slow"))
    with pytest.raises(BackendUnavailable):
        BackendClient(SETTINGS, session=session).record_item(
            ParsedItem(item="rice", quantity=1), request_id="req-6", raw_text=None
        )
