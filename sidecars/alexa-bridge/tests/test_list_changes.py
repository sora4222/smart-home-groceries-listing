"""Tests for the remove/undo client: what it sends, and how answers map."""

import json

import pytest
import requests

from alexa_bridge.backend import BackendUnavailable
from alexa_bridge.config import Settings
from alexa_bridge.list_changes import ChangeRefused, ListChange, ListChangeClient, NotOnList
from alexa_bridge.parsing import ParsedRemoval

SETTINGS = Settings(
    skill_id="amzn1.ask.skill.test",
    backend_base_url="http://backend:8000/",
    bridge_secret="test-bridge-secret",
    backend_timeout_seconds=1.0,
)

REMOVED = {
    "id": "c1",
    "kind": "removed",
    "item_name": "Milk",
    "quantity_before": 2,
    "quantity_after": 0,
    "undone": False,
}


class FakeSession:
    """A stand-in for `requests.Session`: records calls, answers as told."""

    def __init__(self, status_code=201, body=None, raises=None):
        self.status_code = status_code
        self.body = REMOVED if body is None else body
        self.raises = raises
        self.calls = []

    def post(self, url, json=None, headers=None, timeout=None):
        if self.raises:
            raise self.raises
        self.calls.append({"url": url, "json": json, "headers": headers, "timeout": timeout})
        return _Response(self.status_code, self.body)


class _Response:
    def __init__(self, status_code, body):
        self.status_code = status_code
        self._body = body

    @property
    def ok(self):
        return 200 <= self.status_code < 300

    def json(self):
        return self._body


def test_remove_posts_the_item_without_a_quantity_when_none_was_said():
    session = FakeSession()
    ListChangeClient(SETTINGS, session=session).remove_item(
        ParsedRemoval(item="milk", quantity=None), request_id="req-1"
    )
    call = session.calls[0]
    assert call["url"] == "http://backend:8000/api/intake/alexa/remove"
    assert call["json"] == {"item": "milk", "external_id": "req-1"}
    assert call["timeout"] == 1.0


def test_remove_posts_the_quantity_when_one_was_said():
    session = FakeSession()
    ListChangeClient(SETTINGS, session=session).remove_item(
        ParsedRemoval(item="milk", quantity=2), request_id="req-2"
    )
    assert session.calls[0]["json"]["quantity"] == 2


def test_remove_returns_the_backends_account_of_the_change():
    change = ListChangeClient(SETTINGS, session=FakeSession()).remove_item(
        ParsedRemoval(item="milk", quantity=None), request_id="req-3"
    )
    assert change == ListChange(
        kind="removed", item_name="Milk", quantity_before=2, quantity_after=0
    )


def test_undo_posts_alexas_request_id():
    session = FakeSession()
    ListChangeClient(SETTINGS, session=session).undo(request_id="req-4")
    call = session.calls[0]
    assert call["url"] == "http://backend:8000/api/intake/alexa/undo"
    assert call["json"] == {"external_id": "req-4"}


def test_the_secret_travels_only_in_the_header():
    session = FakeSession()
    ListChangeClient(SETTINGS, session=session).undo(request_id="req-5")
    call = session.calls[0]
    assert call["headers"]["X-Bridge-Secret"] == "test-bridge-secret"
    assert "test-bridge-secret" not in call["url"]
    assert "test-bridge-secret" not in json.dumps(call["json"])


@pytest.mark.parametrize(
    ("status", "error"),
    [(404, NotOnList), (409, ChangeRefused), (401, BackendUnavailable), (500, BackendUnavailable)],
)
def test_error_statuses_map_to_what_the_skill_says(status, error):
    client = ListChangeClient(SETTINGS, session=FakeSession(status_code=status, body={}))
    with pytest.raises(error):
        client.undo(request_id="req-6")


def test_a_network_failure_is_unavailable_not_success():
    session = FakeSession(raises=requests.ConnectionError("refused"))
    with pytest.raises(BackendUnavailable):
        ListChangeClient(SETTINGS, session=session).remove_item(
            ParsedRemoval(item="milk", quantity=None), request_id="req-7"
        )
