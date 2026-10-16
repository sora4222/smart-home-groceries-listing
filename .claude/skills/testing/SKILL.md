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

Configuration tests use fixture dotenv files and a fake `op` executable.
Cover exported overrides and denied access without displaying real values.
Cover service-account success, a failed service read followed by desktop
success, and failure of both methods. Fake CLI checks should reject competing
Connect/session credentials and verify desktop integration is enabled for the
fallback. Keep real credentials out of test fixtures and output.
The Node suite checks Compose wiring and needs Node 22+ and the Compose CLI,
but no running containers or 1Password account.

## Backend

Use nearby `#[cfg(test)]` modules for pure rules and `#[sqlx::test]` for
database or router behavior. The latter creates a fresh database per test;
never use a development database. `backend/tests/common::TestApp` exercises
the real router and middleware. Cover new routes' success, authentication, and
invalid-input paths; add a CORS preflight test for a new HTTP method.
`make test-backend` owns a ready PostgreSQL Testcontainer on a random port.
Install root tooling with `pnpm install`; keep Docker available. Do not derive
test connections from `.env`, `POSTGRES_*`, or `TEST_DATABASE_URL`.
Add every new household route to `backend/tests/auth_required.rs`; real
Clerk tokens are tested in `tests/clerk_tokens.rs` with a mock JWKS.

Mock store clients and use trimmed fixtures or `wiremock`; do not reach real
stores, accounts, Alexa, or LLM providers. Poll background outcomes with a
deadline rather than sleeping.

## Alexa bridge

Run `make test-alexa` or `uv run pytest -q` in `sidecars/alexa-bridge`. Inject
the HTTP session; verify unsigned Alexa requests are rejected and the bridge
forwards only its shared secret.
For direct startup configuration, cover the port and logging passed to Flask
and logging setup from dotenv, resolved references, exported overrides, and
defaults. Replace the server runner and secret reader with fakes.

## Frontend

Use Vitest with mocked API and WebSocket calls for component behavior. Put
browser coverage in `frontend/e2e/` and import its shared fixture, which resets
state, catches browser/server failures, and waits for hydration. Cover desktop
and mobile, use exact accessible labels or existing item helpers, and do not
interact before hydration completes.

E2E runs with sign-in off: the backend with `DEV_AUTH_BYPASS=true` and the web
app with `VITE_AUTH_MODE=off pnpm dev`, or a Clerk key in `.env` sends every
test to the sign-in page.
Use `make e2e` for an isolated database and local servers with fake providers.
The runner sets random ports and the matching OAuth callback. Raw `pnpm e2e`
assumes an already isolated test stack and resets its data. Verify resource
cleanup when changing either runner; never reuse a household application stack.
Keep Vite's automatic dotenv loading disabled for an explicit
`GROCERY_ENV_FILE`, including the empty file supplied by the browser runner.

Use `test-results.mjs` as the verdict policy for every output mode. Adapters
read dedicated Vitest/Playwright JSON, pytest hook JSON, or libtest result
records. Application stdout is diagnostic only. Missing or malformed reports,
failures, unexpected skips, and an empty aggregate suite fail the gate.
Cargo discovers binaries through compiler-artifact JSON and preserves doc tests
on full runs. Its stable `--logfile` adapter is deprecated upstream; keep it
isolated until stable Rust offers structured output without nightly flags.

Extend `scripts/environment-contract.json` for configuration source policy,
then exercise it in all three loaders. Explicit files must be readable even
when exports supply every key; only absent default discovery is optional.
