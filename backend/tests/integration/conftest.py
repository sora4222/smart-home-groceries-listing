"""Integration-test fixtures: a real Postgres via testcontainers, migrated
with Alembic, and an httpx AsyncClient wired to the FastAPI app.

Requires a Docker daemon on the host running these tests.
"""

import os

import pytest
import pytest_asyncio
from alembic.config import Config
from httpx import ASGITransport, AsyncClient
from testcontainers.postgres import PostgresContainer

from alembic import command
from tests.conftest import TEST_WEBHOOK_SECRET


@pytest.fixture(scope="session")
def postgres_container():
    with PostgresContainer("postgres:16") as container:
        yield container


@pytest.fixture(scope="session")
def database_url(postgres_container) -> str:
    url = postgres_container.get_connection_url()
    # testcontainers returns a psycopg2-style URL; swap the driver for asyncpg.
    return url.replace("postgresql+psycopg2", "postgresql+asyncpg")


@pytest.fixture(scope="session", autouse=True)
def _configure_env(database_url):
    os.environ["DATABASE_URL"] = database_url
    os.environ["VOICE_WEBHOOK_SECRET"] = TEST_WEBHOOK_SECRET
    os.environ["DEV_AUTH_BYPASS"] = "true"

    from app.config import get_settings

    get_settings.cache_clear()

    alembic_cfg = Config("alembic.ini")
    alembic_cfg.set_main_option("sqlalchemy.url", database_url)
    command.upgrade(alembic_cfg, "head")
    yield


@pytest_asyncio.fixture
async def client(_configure_env):
    from app.main import app

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as ac:
        yield ac
