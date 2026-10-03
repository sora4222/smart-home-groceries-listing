---
name: testing
description: Add and run focused Rust, Alexa bridge, React, or Playwright tests for this grocery application. Use when changing behavior or investigating a regression.
---

# Testing the grocery application

Start with focused coverage, then run `make lint` and `make test` when the
change warrants the full suite. Report any unavailable infrastructure or
skipped coverage rather than treating it as passing.

For repository tooling, add a matching `scripts/*.test.sh` test and include
it in `make test-tooling`; workflow-only rules should use the same script.

## Backend

Use nearby `#[cfg(test)]` modules for pure rules and `#[sqlx::test]` for
database or router behavior. The latter creates a fresh database per test;
never use a development database. `backend/tests/common::TestApp` exercises
the real router and middleware. Cover new routes' success, authentication, and
invalid-input paths; add a CORS preflight test for a new HTTP method.
Add every new household route to `backend/tests/auth_required.rs`; real
Clerk tokens are tested in `tests/clerk_tokens.rs` with a mock JWKS.

Mock store clients and use trimmed fixtures or `wiremock`; do not reach real
stores, accounts, Alexa, or LLM providers. Poll background outcomes with a
deadline rather than sleeping.

## Alexa bridge

Run `make test-alexa` or `uv run pytest -q` in `sidecars/alexa-bridge`. Inject
the HTTP session; verify unsigned Alexa requests are rejected and the bridge
forwards only its shared secret.

## Frontend

Use Vitest with mocked API and WebSocket calls for component behavior. Put
browser coverage in `frontend/e2e/` and import its shared fixture, which resets
state, catches browser/server failures, and waits for hydration. Cover desktop
and mobile, use exact accessible labels or existing item helpers, and do not
interact before hydration completes.

E2E runs with sign-in off: the backend with `DEV_AUTH_BYPASS=true` and the web
app with `VITE_AUTH_MODE=off pnpm dev`, or a Clerk key in `.env` sends every
test to the sign-in page.
