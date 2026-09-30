# Testing

## Backend (Rust)
```bash
make test-backend              # compact, for agents
make hm-test-backend           # verbose, for humans
cd backend && cargo test       # needs DATABASE_URL
cd backend && cargo test --lib                    # unit tests only
cd backend && cargo test --test voice_requests    # one integration file
cd backend && cargo test normalise                # by name
```

`DATABASE_URL` must point at a PostgreSQL server whose user may
`CREATE DATABASE`. `make up` provides one.

### `#[sqlx::test]` replaces testcontainers
No Docker daemon is needed. The attribute creates a fresh database per test,
applies `backend/migrations/`, passes a `PgPool`, and drops the database
afterwards. Tests are therefore isolated and run in parallel.

```rust
// backend/tests/voice_requests.rs
mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn webhook_creates_pending_request(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.post_webhook(&json!({ "item": "milk", "quantity": 2 })).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["status"], "pending");
}
```

`TestApp` (in `backend/tests/common/mod.rs`) builds the **real** router,
middleware included, and drives it with `tower::ServiceExt::oneshot`. A test
therefore exercises the same stack a browser hits — CORS, body limits,
extractors and all. Use `TestApp::serve()` when a test needs a real socket
(the WebSocket tests do).

Never share a database between tests, and never point `DATABASE_URL` at the
development database while running them.

### Unit tests
Pure logic is tested in a `#[cfg(test)] mod tests` beside the code:
`normalise`, `parse_csv`, the shared-secret comparison, the encryptor, the
event's JSON shape. If something needs a database to test, it probably wants
splitting into a pure part and a storage part — that is why
`services/voice/` has `mod.rs` for rules and `repository.rs` for SQL.

### What to test per feature
- Every service function with a rule in it (unit or `#[sqlx::test]`)
- Every route: happy path + auth failure + invalid input (integration)
- Every intake channel: authentication, validation, **and re-delivery** — a
  channel that retries must not create a second card
- WebSocket events: one session, several sessions, a dropped session
- Errors: an internal failure must not leak its cause into the body
- Store integration: mocked HTTP (`wiremock`), never a real store

## Alexa bridge sidecar (Python)
```bash
make test-alexa
cd sidecars/alexa-bridge && uv run pytest -q
```

Nothing in these tests reaches Amazon or the backend. Inject a fake HTTP
session (`RecordingSession` in `tests/test_backend.py`), and assert that an
**unsigned** skill request is refused — forging a valid signature would need
Amazon's private key, so rejection is the property worth testing.

```python
def test_sends_the_bridge_secret_as_a_header():
    session = RecordingSession()
    BackendClient(SETTINGS, session=session).record_item(
        ParsedItem(item="rice", quantity=1), request_id="req-1", raw_text=None
    )
    assert session.calls[0]["headers"]["X-Bridge-Secret"] == "test-bridge-secret"
```

## Frontend unit (vitest)
```bash
cd frontend && pnpm test:run          # single pass
cd frontend && pnpm test              # watch mode
```

Test components in isolation. Mock API calls and the WebSocket. No real
network.

```tsx
// src/components/grocery/grocery-item.test.tsx
import { render, screen } from '@testing-library/react'
import { GroceryItem } from './grocery-item'

test('shows item name and quantity', () => {
  render(<GroceryItem name="Milk" quantity={2} />)
  expect(screen.getByText('Milk')).toBeInTheDocument()
  expect(screen.getByText('×2')).toBeInTheDocument()
})
```

## Frontend e2e (Playwright)
```bash
make up && cd frontend && pnpm e2e    # full stack required
```

Playwright is a **frontend-only** tool in this repo. The backend's store
automation uses `chromiumoxide` — see `backend/skills/store-integration.md`.

E2e tests live in `frontend/e2e/`. Cover the happy path plus the key error
states per feature.

## Never in CI
Real Alexa requests, real store requests, a real Google account, a real LLM
provider, or the development database. Every one of those is either a
credential in CI or a flake.
