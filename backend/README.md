# Backend

The Rust and Axum service owns grocery-list rules, household authentication,
store integrations, Google Tasks polling, and REST/WebSocket endpoints. It uses
PostgreSQL through sqlx with literal SQL and migrations embedded in the binary.
The Alexa verification process is a separate Python sidecar.

## Development

Run `make dev` from the repository root for the complete local stack. It rebuilds
this service when backend files change. The printed API address is bound to
loopback. `make dev-stop` removes development containers while retaining volumes.

For direct host execution, run `cargo run --bin grocery-backend` here with a
reachable PostgreSQL server and a host-appropriate `DATABASE_URL`.
The Compose development database is not published to the host.
Startup loads configuration before logging and applies embedded migrations.
`make update-db` applies migrations without starting HTTP; it uses the selected
`DATABASE_URL`, so select the intended database first. `make build-backend`
compiles the release binary with embedded migrations into `build/backend`;
`make build` includes it in the systemd server bundle at `build/production`.

## Environment variables required

The loader discovers the root `.env` or uses the required `GROCERY_ENV_FILE`.
Exported values override file values; `op://` references resolve through the host
1Password CLI. See [configuration sources](../docs/features/FEATURE_CONFIGURATION.md)
and [`.env.example`](../.env.example) for the full settings contract.

| Variable | Purpose |
|---|---|
| `DATABASE_URL` | PostgreSQL connection. Use explicit credentials for the intended database. |
| `CREDENTIAL_ENCRYPTION_KEY` | Base64-encoded 32-byte AES key for stored integration credentials. |
| `CLERK_JWKS_URL` | Verify household JWTs. Local development can explicitly use `DEV_AUTH_BYPASS=true`. |
| `VOICE_WEBHOOK_SECRET` | Authenticate the generic intake webhook. |
| `ALEXA_BRIDGE_SECRET` | Authenticate forwarded Alexa requests. |
| `STORE_TAB_SECRET` | Authenticate store bookmark handoffs. |

`BIND_ADDRESS` defaults to `0.0.0.0:8000`; `CORS_ORIGINS` lists exact permitted
browser origins. `STORE_CLIENTS=fake` selects the offline catalogue. LLM and
Google Tasks settings are optional integration configuration. Missing integration
credentials leave those integrations unavailable; never publish an auth-bypassed API.

## Checks and tests

From the root, `make format-backend` applies rustfmt, `make lint-backend` applies
Clippy fixes, and `make type-backend` runs cargo check. `make check` applies
fixes, checks file lengths and types, and runs all repository tests.
`make test-backend` owns a disposable PostgreSQL
Testcontainer and ignores application database settings. Docker and the root
pnpm dependencies are required. Add `human=1` for full runner output.
SQLx creates a fresh database per integration
test. Direct `cargo test` requires a separate test PostgreSQL role that can create
databases; use the owned runner for routine development.

## Source ownership

| Path | Responsibility |
|---|---|
| `src/main.rs`, `src/lib.rs`, `src/state.rs` | Process startup and application wiring. |
| `src/environment/`, `src/config/`, `src/config.rs` | Configuration sources and typed settings. |
| `src/auth/` | Clerk verification and shared-secret authentication. |
| `src/routes/` | HTTP/WebSocket boundaries, grouped by domain. |
| `src/models/` | Database rows and request/response schemas. |
| `src/services/` | Business rules, provider adapters and SQL repositories. |
| `migrations/` | Embedded schema history. |
| `tests/` | Router, authentication, database and integration coverage. |

Household routes and WebSocket sessions require authentication; intake endpoints
use their channel's shared secret. See [AGENTS.md](AGENTS.md) for architecture
rules and the [feature specs](../docs/features/) for endpoint contracts.
