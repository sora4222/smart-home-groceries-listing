"""Run the same source-selection fixtures as JavaScript and Rust."""

import json
from pathlib import Path

import pytest

from alexa_bridge.environment import KEYS, load_environment

FIXTURES = json.loads((Path(__file__).parents[3] / "scripts/environment-contract.json").read_text())


@pytest.mark.parametrize("fixture", FIXTURES, ids=lambda fixture: fixture["name"])
def test_source_contract(fixture, tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    for key in (*KEYS, "GROCERY_ENV_FILE"):
        monkeypatch.delenv(key, raising=False)
    file = tmp_path / ".env"
    if fixture["contents"] is not None:
        file.write_text(fixture["contents"])
    if fixture["source"] != "default":
        selected = {
            "empty": "",
            "directory": str(tmp_path),
        }.get(fixture["source"], str(file))
        monkeypatch.setenv("GROCERY_ENV_FILE", selected)
    for key, value in fixture["inherited"].items():
        monkeypatch.setenv(key, value)
    if fixture.get("error"):
        with pytest.raises(RuntimeError, match="Cannot read environment file"):
            load_environment()
    else:
        assert load_environment()["ALEXA_SKILL_ID"] == fixture["expected"]
