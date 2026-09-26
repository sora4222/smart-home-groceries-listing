# Google Home Grocery List

A self-hosted household grocery manager: say *"Hey Google, add milk to the
shopping list"*, review and confirm what Google heard in a web app, and
(eventually) have it purchase from Woolworths/Coles at the lowest total
cost. Proof-of-concept, single home server, Docker Compose + Cloudflare
Tunnel. Full product spec: the "Google Home Grocery List — Application
Specification" doc in the project's docs.

**Currently implemented: the Google Home voice-intake goal** — the webhook,
confirmation queue, and Pending Requests web UI. See `FEATURE_VOICE.md` for
what exists and what's still a stub. Other `FEATURE_*.md` files are
placeholders for the goals that come after this one.

## First-time setup
**If you haven't signed up for anything yet (Clerk, Cloudflare, Google
Home), start with [`docs/human-setup.md`](docs/human-setup.md).** It's the
only part of this that a human has to do — everything else is `make up`.

## Stack
| Layer | Tech |
|---|---|
| Frontend | React 19 + TanStack Start (SSR) + Shadcn-pattern UI + Tailwind v4 |
| Backend | Python 3.12 + FastAPI (REST + WebSocket) |
| Database | PostgreSQL + Alembic + SQLAlchemy (async) |
| Auth | Clerk (JWT), abstracted behind `backend/app/auth/provider.py` |
| Automation | Playwright (store checkout — not yet built) |
| Infra | Docker Compose (Postgres, Cloudflare Tunnel) + host-run frontend/backend |

## Running it
```bash
cp .env.example .env        # fill in the values from docs/human-setup.md
make up                     # Postgres + Cloudflare Tunnel containers
make migrate                # apply the Alembic schema

cd backend && uv sync && uv run uvicorn app.main:app --reload --port 8000
cd frontend && pnpm install && pnpm dev   # http://localhost:3000
```
`make hm-up` runs the containers in the foreground with full logs, if you'd
rather watch them than background them.

Without Clerk configured yet, set `DEV_AUTH_BYPASS=true` in `.env` so the
backend accepts requests without a JWT — see `docs/human-setup.md` for why
this must never be set in a deployment reachable from the internet.

## Testing
```bash
make test          # backend (pytest, needs Docker for the Postgres
                    # testcontainers fixture — this is why test-backend
                    # runs inside the backend container) + frontend (vitest)
make hm-test        # same, verbose
```
See `skills/testing.md` for the full pattern (unit vs. integration vs. e2e).

## Repository layout
```
backend/    FastAPI app — see backend/AGENTS.md
frontend/   TanStack Start app — see frontend/AGENTS.md
docs/       Human setup instructions (signups, secrets, one-time config)
skills/     Reference notes for agents working in this repo — read before
            touching migrations, the Makefile, git commit conventions, or
            store integrations
FEATURE_*.md   One spec-and-status doc per major goal
```

## Development rules
See `AGENTS.md` for the non-negotiables (file length limits, commit
convention, TDD, architecture boundaries). `backend/AGENTS.md` and
`frontend/AGENTS.md` cover their own layers.
