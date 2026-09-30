"""The Flask application: one verified Alexa endpoint and a health check.

Request verification is Amazon's own, via `flask-ask-sdk`'s `SkillAdapter`,
which checks the signature, the certificate chain, the timestamp and the
skill id before any handler runs. That verification is the whole reason this
process is Python.
"""

from __future__ import annotations

import logging
import os

from ask_sdk_core.skill_builder import SkillBuilder
from flask import Flask, jsonify
from flask_ask_sdk.skill_adapter import SkillAdapter

from alexa_bridge.backend import BackendClient
from alexa_bridge.config import Settings, load_settings
from alexa_bridge.handlers import (
    AddItemIntentHandler,
    CancelAndStopIntentHandler,
    CatchAllExceptionHandler,
    HelpIntentHandler,
    LaunchRequestHandler,
    SessionEndedRequestHandler,
)

#: Path Alexa is pointed at in the skill's endpoint configuration.
SKILL_ROUTE = "/alexa"

logger = logging.getLogger(__name__)


def build_skill(client: BackendClient) -> SkillBuilder:
    """Registers every handler. The order is the dispatch order."""
    builder = SkillBuilder()
    builder.add_request_handler(AddItemIntentHandler(client))
    builder.add_request_handler(LaunchRequestHandler())
    builder.add_request_handler(HelpIntentHandler())
    builder.add_request_handler(CancelAndStopIntentHandler())
    builder.add_request_handler(SessionEndedRequestHandler())
    builder.add_exception_handler(CatchAllExceptionHandler())
    return builder


def create_app(settings: Settings | None = None) -> Flask:
    """Builds the Flask application.

    `settings` is injectable so tests can supply their own without touching
    the process environment.
    """
    resolved = settings or load_settings()
    app = Flask(__name__)

    skill = build_skill(BackendClient(resolved)).create()
    adapter = SkillAdapter(skill=skill, skill_id=resolved.skill_id, app=app)
    adapter.register(app=app, route=SKILL_ROUTE)

    @app.get("/health")
    def health() -> tuple[object, int]:
        """Liveness only — says nothing about the backend or Alexa."""
        return jsonify({"status": "ok"}), 200

    logger.info("alexa bridge ready (backend=%s)", resolved.backend_base_url)
    return app


def main() -> None:
    """Development entrypoint. Production runs this under gunicorn."""
    logging.basicConfig(level=os.environ.get("LOG_LEVEL", "INFO"))
    port = int(os.environ.get("PORT", "8081"))
    create_app().run(host="127.0.0.1", port=port)


if __name__ == "__main__":
    main()
