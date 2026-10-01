"""Guards the packaging metadata, which no behavioural test exercises.

hatchling rejects a `name @ url` dependency unless the project opts in, and the
failure only appears when the package is built (`uv run`, `docker build`).
"""

import tomllib
from pathlib import Path

PYPROJECT = Path(__file__).resolve().parents[1] / "pyproject.toml"


def _load() -> dict:
    return tomllib.loads(PYPROJECT.read_text())


def test_direct_url_dependencies_are_allowed_by_the_build_backend():
    project = _load()
    direct = [d for d in project["project"]["dependencies"] if " @ " in d]
    allowed = (
        project.get("tool", {})
        .get("hatch", {})
        .get("metadata", {})
        .get("allow-direct-references", False)
    )

    assert not direct or allowed, f"{direct} need allow-direct-references = true"


def test_direct_url_dependencies_are_pinned_to_a_commit():
    # A branch or tag can move; the oscrypto pin exists for reproducibility.
    for dep in _load()["project"]["dependencies"]:
        if " @ git+" in dep:
            assert "@" in dep.split(" @ git+", 1)[1], f"{dep} is not pinned to a revision"
