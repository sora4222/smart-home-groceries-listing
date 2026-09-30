"""The one place that talks to the Rust backend."""

from __future__ import annotations

import logging

import requests

from alexa_bridge.config import Settings
from alexa_bridge.parsing import ParsedItem

logger = logging.getLogger(__name__)


class BackendUnavailable(RuntimeError):
    """The backend could not be reached, or answered with an error."""


class BackendClient:
    """Posts verified items to the backend's Alexa intake endpoint."""

    def __init__(self, settings: Settings, session: requests.Session | None = None) -> None:
        self._settings = settings
        self._session = session or requests.Session()

    def record_item(self, parsed: ParsedItem, request_id: str, raw_text: str | None) -> None:
        """Records one spoken item as a pending request.

        `request_id` is Alexa's own request id, passed through as the backend's
        `external_id`. Alexa retries a skill endpoint it believes timed out and
        reuses the id when it does, so the backend can recognise the retry and
        not raise a second confirmation card.

        Raises:
            BackendUnavailable: on a network failure, a timeout, or any
                non-success status. The caller turns this into a spoken
                apology — never a claim that the item was added.
        """
        payload: dict[str, object] = {
            "item": parsed.item,
            "quantity": parsed.quantity,
            "external_id": request_id,
        }
        if raw_text:
            payload["raw_text"] = raw_text[:2000]

        try:
            response = self._session.post(
                self._settings.intake_url,
                json=payload,
                headers={"X-Bridge-Secret": self._settings.bridge_secret},
                timeout=self._settings.backend_timeout_seconds,
            )
        except requests.RequestException as exc:
            # The message may include the URL but never the secret, which
            # travels in a header requests does not echo here.
            raise BackendUnavailable(f"could not reach the backend: {exc}") from exc

        if not response.ok:
            raise BackendUnavailable(f"backend answered {response.status_code} for item intake")

        logger.info(
            "recorded item for review (status=%s, quantity=%s)",
            response.status_code,
            parsed.quantity,
        )
