"""Load only bridge configuration, keeping secret-provider details out of settings."""

import os
import subprocess
from pathlib import Path

from dotenv import dotenv_values

KEYS = (
    "ALEXA_SKILL_ID",
    "ALEXA_BRIDGE_SECRET",
    "BACKEND_BASE_URL",
    "BACKEND_TIMEOUT_SECONDS",
    "LOG_LEVEL",
    "PORT",
)


def read_secret(reference: str) -> str:
    """Read with desktop integration or the exported service-account token."""
    return subprocess.run(
        ["op", "read", "--no-newline", reference],
        check=True,
        capture_output=True,
        text=True,
    ).stdout


def load_environment() -> dict[str, str]:
    """Read the nearest .env, apply exported overrides, and resolve bridge references."""
    configured = os.environ.get("GROCERY_ENV_FILE")
    file = (
        Path(configured)
        if configured
        else next(
            (
                parent / ".env"
                for parent in (Path.cwd(), *Path.cwd().parents)
                if (parent / ".env").is_file()
            ),
            None,
        )
    )
    try:
        values = dotenv_values(file, interpolate=False) if file else {}
    except (OSError, UnicodeError):
        raise RuntimeError("Cannot read environment file") from None
    result = {}
    for key in KEYS:
        value = os.environ.get(key, values.get(key))
        if value is None:
            continue
        if value.startswith("op://"):
            try:
                value = read_secret(value)
            except (OSError, UnicodeError, subprocess.SubprocessError):
                raise RuntimeError(
                    f"Cannot load {key} from 1Password; check CLI access and the reference"
                ) from None
            if not value or value.startswith("op://"):
                raise RuntimeError(f"1Password returned no resolved value for {key}")
        result[key] = value
    return result
