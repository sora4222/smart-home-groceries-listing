"""Fixtures shared by unit and integration tests (no Docker required)."""

import uuid

import pytest

TEST_WEBHOOK_SECRET = "test-webhook-secret"


@pytest.fixture
def webhook_headers() -> dict[str, str]:
    return {"x-webhook-secret": TEST_WEBHOOK_SECRET}


@pytest.fixture
def new_item_name() -> str:
    return f"test-item-{uuid.uuid4().hex[:8]}"
