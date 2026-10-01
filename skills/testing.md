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
- Anything that adds to the list: whether item rules apply (voice always,
  manual only when opted in, merges never) — `tests/item_rules_applied.rs`
- WebSocket events: one session, several sessions, a dropped session
- Errors: an internal failure must not leak its cause into the body
- Store integration: never a real store. Mappings are unit-tested against
  `backend/tests/fixtures/<store>/` (trimmed real responses); the clients run
  against `wiremock` in `tests/<store>_client.rs`; routes use the fake
  catalogue — `TestApp` sets `STORE_CLIENTS=fake` via
  `common::fake_store_settings()`. A query containing `outage` makes the fake
  Coles fail, for the one-store-down path.

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
make up && make e2e                   # full stack required
cd frontend && pnpm e2e               # the same thing
cd frontend && pnpm e2e --project=desktop -g "commit"   # one project, by name
cd frontend && pnpm e2e:report        # last HTML report
```

Playwright is a **frontend-only** tool in this repo. The backend's store
automation uses `chromiumoxide` — see `backend/skills/store-integration.md`.

E2e tests live in `frontend/e2e/`. Cover the happy path plus the key error
states per feature. Two projects run every spec: `desktop` (1280px) and
`mobile` (Pixel 7), which is how the spec's "must display correctly at all
widths" gets checked.

### The shared fixture does three things
`frontend/e2e/fixtures.ts` replaces Playwright's `page`, and every spec should
import `test` from there rather than from `@playwright/test`:

1. **Resets state** — releases a committed list, deletes every item, rejects
   every pending intake request, and deletes every item rule (a leftover rule
   would put chips on another test's items). Tests then start from nothing without
   touching the database directly.
2. **Fails on browser trouble** — a `pageerror`, a console error, or any 5xx
   from the backend fails the test. Chrome logs a console line for every 4xx
   and this app provokes 4xx deliberately (the duplicate-item 409 is a
   feature), so "Failed to load resource" lines are skipped and status codes
   judged instead.
3. **Waits for hydration** — `gotoList(page)` navigates and waits for
   `#app[data-hydrated="true"]`. The app is server-rendered: markup is on
   screen before React attaches a handler, and a click or keystroke in that
   window is silently lost. This is the usual cause of a "locator timed out on
   a button that is clearly there".

### Locators
Use exact labels. Playwright matches a label by substring, so `getByLabel
("Item")` also matches the form's `aria-label="Add a grocery item"`, and
`getByLabel("Quantity")` matches a `Quantity 2` badge. Address a list item by
`itemCard(page, name)`, which uses `data-item-name`: with an editor open the
name is an input's value, which text filtering cannot see.

### No browser to download?
`E2E_CHROMIUM_PATH=/path/to/chrome pnpm e2e` uses an already-installed
Chromium instead of the one Playwright manages, for a machine where
`npx playwright install` cannot reach the network.

## Never in CI
Real Alexa requests, real store requests, a real Google account, a real LLM
provider, or the development database. Every one of those is either a
credential in CI or a flake.
