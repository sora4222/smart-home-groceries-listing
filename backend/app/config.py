"""Application settings, loaded from environment variables / .env file."""

from functools import lru_cache

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Runtime configuration for the FastAPI backend.

    All values are sourced from environment variables (see `.env.example`
    at the repo root). Nothing here should be hardcoded in application code.
    """

    model_config = SettingsConfigDict(
        env_file=("../.env", ".env"), extra="ignore"
    )

    # Database
    database_url: str = "postgresql+asyncpg://grocery:changeme@localhost:5432/grocery"

    # Clerk auth
    clerk_secret_key: str = ""
    clerk_publishable_key: str = ""
    clerk_jwks_url: str = ""

    # Google Home webhook
    voice_webhook_secret: str = ""

    # Credential encryption
    credential_encryption_key: str = ""

    # CORS — comma separated list of allowed origins for the frontend
    cors_origins: str = "http://localhost:3000"

    # When true, auth is bypassed and a fixed dev user is injected.
    # Never enable this in a deployment reachable outside the home network.
    dev_auth_bypass: bool = False

    @property
    def cors_origin_list(self) -> list[str]:
        return [o.strip() for o in self.cors_origins.split(",") if o.strip()]


@lru_cache
def get_settings() -> Settings:
    """Return a cached Settings instance (env is only read once per process)."""
    return Settings()
