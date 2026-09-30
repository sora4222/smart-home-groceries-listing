"""Tests for settings loading — it must fail closed, not default."""

import pytest

from alexa_bridge.config import ConfigError, load_settings

COMPLETE = {
    "ALEXA_SKILL_ID": "amzn1.ask.skill.test",
    "ALEXA_BRIDGE_SECRET": "s3cret",
}


def test_loads_a_complete_environment():
    settings = load_settings(COMPLETE)
    assert settings.skill_id == "amzn1.ask.skill.test"
    assert settings.bridge_secret == "s3cret"
    assert settings.backend_base_url == "http://backend:8000"
    assert settings.backend_timeout_seconds == 3.0


def test_builds_the_intake_url():
    settings = load_settings({**COMPLETE, "BACKEND_BASE_URL": "http://127.0.0.1:8000/"})
    assert settings.intake_url == "http://127.0.0.1:8000/api/intake/alexa"


@pytest.mark.parametrize("missing", ["ALEXA_SKILL_ID", "ALEXA_BRIDGE_SECRET"])
def test_a_missing_required_value_stops_the_process(missing):
    env = {k: v for k, v in COMPLETE.items() if k != missing}
    with pytest.raises(ConfigError, match=missing):
        load_settings(env)


@pytest.mark.parametrize("blank", ["", "   "])
def test_a_blank_secret_is_treated_as_missing(blank):
    # Otherwise the backend would be called with an empty secret and the
    # endpoint would look configured while being unauthenticated.
    with pytest.raises(ConfigError, match="ALEXA_BRIDGE_SECRET"):
        load_settings({**COMPLETE, "ALEXA_BRIDGE_SECRET": blank})


def test_the_timeout_is_configurable():
    settings = load_settings({**COMPLETE, "BACKEND_TIMEOUT_SECONDS": "1.5"})
    assert settings.backend_timeout_seconds == 1.5
