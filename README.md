# Google Home Grocery List

A self-hosted household grocery manager: say *"Alexa, add milk to the shopping
list"*, review and confirm what was heard in a web app, and (eventually) have
it purchase from Woolworths/Coles at the lowest total cost. Proof-of-concept,
single home server, Docker Compose + Cloudflare Tunnel. Full product spec: the
"Google Home Grocery List — Application Specification" doc in the project's
docs.

**Currently implemented: voice intake** — the generic webhook, the Alexa
channel, the confirmation queue, and the Pending Requests web UI. See
`docs/features/FEATURE_VOICE.md` for what exists and what is still a stub.
Other `FEATURE_*.md` files are placeholders for the goals that come after.

> Despite the repository name, Google Home cannot be integrated directly — the
> Smart Home Action / OAuth account-linking route was dropped. Alexa is the
> working voice channel; Google Tasks and Google Keep are planned. See
> `docs/features/FEATURE_VOICE.md`.

## First-time setup
**If you haven't signed up for anything yet (Clerk, Cloudflare, Amazon
developer), start with [`docs/human-setup.md`](docs/human-setup.md).** It's the
only part of this that a human has to do — everything else is `make up`.

## Stack
| Layer | Tech |
|---|---|
| Frontend | React 19 + TanStack Start (SSR) + Shadcn-pattern UI + Tailwind v4 |
| Backend | **Rust + Axum** (REST + WebSocket) |
| Database | PostgreSQL + `sqlx` (migrations embedded in the binary) |
| Auth | Clerk (JWT), abstracted behind `backend/src/auth/mod.rs` |
| Alexa | `sidecars/alexa-bridge` — Python `ask-sdk`, the only Python here |
| Automation | `chromiumoxide` (store checkout — not yet built) |
| Infra | Docker Compose (Postgres, backend, Alexa bridge, Cloudflare Tunnel) |

The backend is a single compiled binary with its migrations embedded: no
interpreter, virtualenv or migration files on the home server.

## Running it
```bash
cp .env.example .env        # fill in the values from docs/human-setup.md
make up                     # Postgres, backend, Alexa bridge, Cloudflare Tunnel
make migrate                # optional — the backend migrates itself at startup

cd frontend && pnpm install && pnpm dev   # http://localhost:3000
```

`make hm-up` runs the containers in the foreground with full logs.

To run the backend on the host instead of in its container:

```bash
make up                                   # Postgres only is enough
cd backend && cargo run                   # reads .env via the environment
```

Without Clerk configured yet, set `DEV_AUTH_BYPASS=true` in `.env` so the
backend accepts requests without a JWT. It logs a warning at startup — see
`docs/human-setup.md` for why this must never be set on a host reachable from
the internet.

## Testing
```bash
make test          # backend (cargo test) + frontend (vitest) + sidecar (pytest)
make hm-test       # same, verbose
```

The backend's tests need a PostgreSQL server at `DATABASE_URL` whose user may
create databases — `make up` provides one. They do **not** need a Docker
daemon: `#[sqlx::test]` creates and drops a database per test, replacing the
old testcontainers fixture.

See `skills/testing.md` for the full pattern (unit vs. integration vs. e2e).

## Repository layout
```
backend/          Rust + Axum API — see backend/AGENTS.md
frontend/         TanStack Start app — see frontend/AGENTS.md
sidecars/
  alexa-bridge/   Python ask-sdk bridge — see its AGENTS.md
docs/
  features/       One spec-and-status doc per major goal
  human-setup.md  Human setup instructions (signups, secrets, one-time config)
skills/           Reference notes for agents working in this repo — read before
                  touching migrations, the Makefile, git conventions, or
                  store integrations
```

## Development rules
See `AGENTS.md` for the non-negotiables (file length limits, commit
convention, TDD, architecture boundaries, no Python in the backend).
`backend/AGENTS.md`, `frontend/AGENTS.md` and
`sidecars/alexa-bridge/AGENTS.md` cover their own layers.
