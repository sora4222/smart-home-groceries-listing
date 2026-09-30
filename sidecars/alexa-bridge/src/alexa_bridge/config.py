"""Settings, read from the environment.

No secret here is logged. `ALEXA_BRIDGE_SECRET` is the credential the Rust
backend checks, and `ALEXA_SKILL_ID` is what ask-sdk checks each request
against, so a different skill pointed at this endpoint is rejected.
"""

from __future__ import annotations

import os
from dataclasses import dataclass


class ConfigError(RuntimeError):
    """A required setting is missing, so the process must not start."""


@dataclass(frozen=True)
class Settings:
    """Runtime configuration for the bridge."""

    #: Amazon skill id, e.g. `amzn1.ask.skill.<uuid>`. Verified per request.
    skill_id: str
    #: Base URL of the Rust backend, normally on loopback or the Compose network.
    backend_base_url: str
    #: Shared secret the backend authenticates this sidecar with.
    bridge_secret: str
    #: Seconds to wait for the backend. Alexa abandons a skill response after
    #: about 8 seconds, so this stays well inside that budget.
    backend_timeout_seconds: float = 3.0

    @property
    def intake_url(self) -> str:
        """Full URL of the backend's Alexa intake endpoint."""
        return f"{self.backend_base_url.rstrip('/')}/api/intake/alexa"


def load_settings(env: dict[str, str] | None = None) -> Settings:
    """Builds settings from the environment, failing fast on anything missing.

    Raises:
        ConfigError: if a required variable is unset or blank. Starting without
            a skill id or bridge secret would leave the endpoint effectively
            unauthenticated, so this fails closed rather than defaulting.
    """
    source = os.environ if env is None else env

    def required(name: str) -> str:
        value = (source.get(name) or "").strip()
        if not value:
            raise ConfigError(f"{name} is not set")
        return value

    timeout = (source.get("BACKEND_TIMEOUT_SECONDS") or "").strip()
    return Settings(
        skill_id=required("ALEXA_SKILL_ID"),
        backend_base_url=(source.get("BACKEND_BASE_URL") or "http://backend:8000").strip(),
        bridge_secret=required("ALEXA_BRIDGE_SECRET"),
        backend_timeout_seconds=float(timeout) if timeout else 3.0,
    )
