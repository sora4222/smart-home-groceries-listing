"""Talking to the backend's remove and undo endpoints.

Separate from `backend.py` because these calls answer with a body the skill
reads back, and fail in two ways the skill speaks about (no such item, or
the list is locked) as well as the usual "could not reach it".
"""

from __future__ import annotations

import logging
from dataclasses import dataclass

import requests

from alexa_bridge.backend import BackendUnavailable
from alexa_bridge.config import Settings
from alexa_bridge.parsing import ParsedRemoval

logger = logging.getLogger(__name__)


class NotOnList(LookupError):
    """No item on the list matched (404), or there was nothing to undo."""


class ChangeRefused(RuntimeError):
    """The backend would not make the change (409): the list is locked, or
    the item changed since."""


@dataclass(frozen=True)
class ListChange:
    """The backend's account of one remove, reduce or undo."""

    kind: str
    item_name: str
    quantity_before: int
    quantity_after: int

    @classmethod
    def from_json(cls, body: dict[str, object]) -> ListChange:
        """Reads the backend's `VoiceListChangeResponse`."""
        return cls(
            kind=str(body["kind"]),
            item_name=str(body["item_name"]),
            quantity_before=int(body["quantity_before"]),
            quantity_after=int(body["quantity_after"]),
        )


class ListChangeClient:
    """Posts removes and undos to the backend's Alexa endpoints."""

    def __init__(self, settings: Settings, session: requests.Session | None = None) -> None:
        self._settings = settings
        self._session = session or requests.Session()

    def remove_item(self, parsed: ParsedRemoval, request_id: str) -> ListChange:
        """Takes an item off the list, or lowers its quantity.

        `request_id` is Alexa's own; a retry with the same id changes nothing
        twice.

        Raises:
            NotOnList: no item on the list matched the name.
            ChangeRefused: the item is committed for purchase.
            BackendUnavailable: the backend could not be reached or failed.
        """
        payload: dict[str, object] = {"item": parsed.item, "external_id": request_id}
        if parsed.quantity is not None:
            payload["quantity"] = parsed.quantity
        return self._post(self._settings.remove_url, payload)

    def undo(self, request_id: str) -> ListChange:
        """Reverses the newest voice change.

        Raises:
            NotOnList: there is nothing recent to undo.
            ChangeRefused: the item changed since and cannot be put back.
            BackendUnavailable: the backend could not be reached or failed.
        """
        return self._post(self._settings.undo_url, {"external_id": request_id})

    def _post(self, url: str, payload: dict[str, object]) -> ListChange:
        """Sends one request and turns the answer into a result or an error."""
        try:
            response = self._session.post(
                url,
                json=payload,
                headers={"X-Bridge-Secret": self._settings.bridge_secret},
                timeout=self._settings.backend_timeout_seconds,
            )
        except requests.RequestException as exc:
            raise BackendUnavailable(f"could not reach the backend: {exc}") from exc

        if response.status_code == 404:
            raise NotOnList()
        if response.status_code == 409:
            raise ChangeRefused()
        if not response.ok:
            raise BackendUnavailable(f"backend answered {response.status_code} for a list change")

        change = ListChange.from_json(response.json())
        logger.info("list change applied (kind=%s, status=%s)", change.kind, response.status_code)
        return change
