# Backend

Rust + Axum. REST and WebSocket, PostgreSQL via `sqlx`. No Python.

```
src/
├── main.rs             # read config, open the pool, migrate, serve
├── lib.rs              # build_app: state, routes, middleware
├── config.rs           # Settings from the environment
├── error.rs            # ApiError → HTTP, in the {"detail": ...} shape the web app parses
├── state.rs            # AppState handed to every handler
├── auth/
│   ├── mod.rs          # AuthUser extractor + AuthProvider trait — swap provider here only
│   ├── clerk.rs        # Clerk JWKS verification (current implementation)
│   └── secret.rs       # constant-time shared-secret checks for the intake endpoints
├── db/mod.rs           # pool + embedded migrations
├── models/
│   ├── db.rs           # row types
│   └── schemas.rs      # request/response bodies, with validation limits
├── routes/             # one module per domain, merged in routes/mod.rs
│   ├── voice.rs        # /api/voice-requests — generic intake webhook + confirmation queue
│   ├── alexa.rs        # /api/intake/alexa — from the Alexa bridge sidecar
│   ├── grocery.rs      # /api/grocery-items
│   ├── ws.rs           # /ws
│   ├── health.rs       # /api/health
│   └── extract.rs      # validating body extractors
└── services/
    ├── voice/          # confirmation-queue rules (mod.rs) and SQL (repository.rs)
    ├── ws_hub.rs       # broadcast fan-out to browser sessions
    └── encryption.rs   # AES-256-GCM for store credentials
migrations/             # sqlx migrations, embedded into the binary
tests/                  # integration tests over the real router (#[sqlx::test])
```

See `AGENTS.md` here for the patterns, and the repository root `AGENTS.md` for
the project-wide rules.

## Running it

```bash
make up                 # Postgres (+ Cloudflare Tunnel) from the repo root
cargo run               # reads .env at the repo root via the environment
```

Migrations run automatically at startup. To apply them without starting the
server, use `make migrate` from the repository root.

## Tests

```bash
make test-backend       # from the repository root
cargo test              # here, with DATABASE_URL set
```

`#[sqlx::test]` creates a fresh database per test, applies `migrations/`, and
drops it afterwards. It needs a PostgreSQL server at `DATABASE_URL` whose user
may `CREATE DATABASE` — no Docker daemon required, unlike the previous
testcontainers setup.

## Endpoints

| Method | Path | Auth |
|---|---|---|
| `GET` | `/api/health` | none |
| `POST` | `/api/voice-requests` | `X-Webhook-Secret` |
| `POST` | `/api/intake/alexa` | `X-Bridge-Secret` (from the sidecar) |
| `GET` | `/api/voice-requests` | Clerk JWT |
| `POST` | `/api/voice-requests/{id}/accept` | Clerk JWT |
| `POST` | `/api/voice-requests/{id}/reject` | Clerk JWT |
| `GET` | `/api/grocery-items` | Clerk JWT |
| `GET` | `/ws` | none (read-only event stream) |
