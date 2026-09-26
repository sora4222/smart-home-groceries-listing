# Testing

## Backend (pytest)
```bash
make test-backend              # compact, for agents
make hm-test-backend           # verbose, for humans
uv run pytest -x -q            # fast-fail inside container
uv run pytest tests/unit/      # unit only
uv run pytest tests/integration/  # integration (spins up DB via testcontainers)
```

### Testcontainers pattern
Never use a shared dev DB in tests. Use testcontainers for any test that touches PostgreSQL.

```python
# tests/conftest.py
import pytest
from testcontainers.postgres import PostgresContainer
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession

@pytest.fixture(scope="session")
def pg_url():
    with PostgresContainer("postgres:16") as pg:
        yield pg.get_connection_url().replace("postgresql://", "postgresql+asyncpg://")

@pytest.fixture
async def db_session(pg_url):
    engine = create_async_engine(pg_url)
    async with AsyncSession(engine) as session:
        yield session
        await session.rollback()
```

### Route tests
```python
# tests/integration/test_voice_routes.py
from httpx import AsyncClient
import pytest

@pytest.mark.asyncio
async def test_post_voice_request(client: AsyncClient):
    resp = await client.post(
        "/api/voice-requests",
        json={"item": "milk", "quantity": 2},
        headers={"X-Webhook-Secret": "test-secret"},
    )
    assert resp.status_code == 201
    assert resp.json()["status"] == "pending"
```

## Frontend unit (vitest)
```bash
pnpm test:run          # single pass
pnpm test              # watch mode
```

Test components in isolation. Mock API calls and WebSocket. No real network.

```typescript
// src/components/grocery/GroceryItem.test.tsx
import { render, screen } from '@testing-library/react'
import { GroceryItem } from './GroceryItem'

test('shows item name and quantity', () => {
  render(<GroceryItem name="Milk" quantity={2} />)
  expect(screen.getByText('Milk')).toBeInTheDocument()
  expect(screen.getByText('×2')).toBeInTheDocument()
})
```

## Frontend e2e (Playwright)
```bash
make up && pnpm e2e    # full stack required
```

E2e tests live in `frontend/e2e/`. Cover happy path + key error states per feature.

```typescript
// frontend/e2e/pending.spec.ts
import { test, expect } from '@playwright/test'

test('accept voice request moves item to grocery list', async ({ page }) => {
  await page.goto('/pending')
  await page.getByRole('button', { name: 'Accept' }).first().click()
  await page.goto('/')
  await expect(page.getByText('Milk')).toBeVisible()
})
```

## What to test per feature
- Every service function (unit)
- Every API route: happy path + auth failure + invalid input (integration)
- Every page: load, core interaction, error state (e2e)
- WebSocket events: connection, disconnect, message received
- Store integration: mock HTTP responses for price fetch and checkout flow
