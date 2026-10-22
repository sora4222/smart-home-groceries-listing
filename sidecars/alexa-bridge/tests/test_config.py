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


def test_direct_startup_reads_dotenv_without_inheriting_database_credentials(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    monkeypatch.delenv("GROCERY_ENV_FILE", raising=False)
    monkeypatch.delenv("ALEXA_SKILL_ID", raising=False)
    monkeypatch.delenv("ALEXA_BRIDGE_SECRET", raising=False)
    (tmp_path / ".env").write_text(
        "ALEXA_SKILL_ID=amzn1.ask.skill.dotenv\nALEXA_BRIDGE_SECRET=local-secret\n"
        "DATABASE_URL=op://Development/unrelated/url\n"
    )
    settings = load_settings()
    assert settings.skill_id == "amzn1.ask.skill.dotenv"
    assert settings.bridge_secret == "local-secret"


@pytest.mark.parametrize("token", ["", "fixture-service-token"])
def test_direct_startup_resolves_references_with_either_cli_auth_method(
    tmp_path, monkeypatch, token
):
    monkeypatch.chdir(tmp_path)
    monkeypatch.delenv("GROCERY_ENV_FILE", raising=False)
    monkeypatch.setenv("PATH", str(tmp_path))
    monkeypatch.setenv("OP_SERVICE_ACCOUNT_TOKEN", token)
    monkeypatch.delenv("ALEXA_SKILL_ID", raising=False)
    monkeypatch.setenv("ALEXA_BRIDGE_SECRET", "exported-secret")
    (tmp_path / ".env").write_text(
        "ALEXA_SKILL_ID=op://Development/skill/id\n"
        "ALEXA_BRIDGE_SECRET=op://Development/unavailable/secret\n"
        "DATABASE_URL=op://Development/unrelated/url\n"
    )
    op = tmp_path / "op"
    op.write_text(
        '#!/bin/sh\n[ "$1" = read ] && [ "$2" = --no-newline ] || exit 2\n'
        'printf "skill-%s" "${OP_SERVICE_ACCOUNT_TOKEN:-desktop}"\n'
    )
    op.chmod(0o700)
    settings = load_settings()
    assert settings.skill_id == f"skill-{token or 'desktop'}"
    assert settings.bridge_secret == "exported-secret"


@pytest.mark.parametrize("token", ["", "denied-fixture-token"])
def test_provider_errors_do_not_disclose_values_or_cli_output(tmp_path, monkeypatch, token):
    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv("PATH", str(tmp_path))
    monkeypatch.setenv("OP_SERVICE_ACCOUNT_TOKEN", token)
    monkeypatch.setenv("ALEXA_SKILL_ID", "op://private/skill/id")
    monkeypatch.setenv("ALEXA_BRIDGE_SECRET", "literal")
    op = tmp_path / "op"
    op.write_text("#!/bin/sh\necho sensitive-provider-output >&2\nexit 1\n")
    op.chmod(0o700)
    with pytest.raises(ConfigError) as error:
        load_settings()
    assert str(error.value) == (
        "Cannot load ALEXA_SKILL_ID from 1Password; check CLI access and the reference"
    )


@pytest.mark.parametrize("service_result", ["exit 7", "exit 0", "printf op://unresolved"])
def test_failed_service_account_retries_with_desktop_authentication(
    tmp_path, monkeypatch, service_result
):
    monkeypatch.setenv("PATH", str(tmp_path))
    (tmp_path / ".env").write_text("")
    monkeypatch.setenv("GROCERY_ENV_FILE", str(tmp_path / ".env"))
    monkeypatch.setenv("ALEXA_SKILL_ID", "op://Development/skill/id")
    monkeypatch.setenv("ALEXA_BRIDGE_SECRET", "fixture-secret")
    monkeypatch.setenv("OP_SERVICE_ACCOUNT_TOKEN", "denied-fixture-token")
    for key in ("OP_CONNECT_HOST", "OP_CONNECT_TOKEN", "OP_SESSION", "OP_SESSION_fixture"):
        monkeypatch.setenv(key, "competing-auth")
    monkeypatch.setenv("OP_BIOMETRIC_UNLOCK_ENABLED", "false")
    trace = tmp_path / "trace"
    monkeypatch.setenv("AUTH_TRACE", str(trace))
    (tmp_path / "op").write_text(
        '#!/bin/sh\n[ "$1" = read ] && [ "$2" = --no-newline ] || exit 2\n'
        '[ -z "$OP_CONNECT_HOST$OP_CONNECT_TOKEN$OP_SESSION$OP_SESSION_fixture" ] || exit 3\n'
        'if [ -n "$OP_SERVICE_ACCOUNT_TOKEN" ]; then\n'
        '  [ "$OP_BIOMETRIC_UNLOCK_ENABLED" = false ] || exit 4\n'
        '  printf "service\\n" >> "$AUTH_TRACE"\n'
        f"  echo sensitive-provider-output >&2\n  {service_result}\n  exit 0\nfi\n"
        '[ "$OP_BIOMETRIC_UNLOCK_ENABLED" = true ] || exit 5\n'
        'printf "desktop\\n" >> "$AUTH_TRACE"\nprintf desktop-skill\n'
    )
    (tmp_path / "op").chmod(0o700)

    settings = load_settings()

    assert settings.skill_id == "desktop-skill"
    assert trace.read_text() == "service\ndesktop\n"
