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
    """Try the exported service account, then retry once through the desktop app."""
    desktop = {
        key: value
        for key, value in os.environ.items()
        if key not in ("OP_SERVICE_ACCOUNT_TOKEN", "OP_SESSION")
        and not key.startswith(("OP_CONNECT_", "OP_SESSION_"))
    }
    desktop["OP_BIOMETRIC_UNLOCK_ENABLED"] = "true"

    def read(env: dict[str, str]) -> str:
        value = subprocess.run(
            ["op", "read", "--no-newline", reference],
            env=env,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        if not value or value.startswith("op://"):
            raise RuntimeError("Unresolved 1Password value")
        return value

    if token := os.environ.get("OP_SERVICE_ACCOUNT_TOKEN"):
        service = {
            **desktop,
            "OP_SERVICE_ACCOUNT_TOKEN": token,
            "OP_BIOMETRIC_UNLOCK_ENABLED": "false",
        }
        service.pop("OP_ACCOUNT", None)
        try:
            return read(service)
        except (OSError, UnicodeError, subprocess.SubprocessError, RuntimeError):
            pass
    return read(desktop)


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
            except (OSError, UnicodeError, subprocess.SubprocessError, RuntimeError):
                raise RuntimeError(
                    f"Cannot load {key} from 1Password; check CLI access and the reference"
                ) from None
            if not value or value.startswith("op://"):
                raise RuntimeError(f"1Password returned no resolved value for {key}")
        result[key] = value
    return result
