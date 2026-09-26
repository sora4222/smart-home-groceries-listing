# AGENT.md — Google Home Grocery List

## Purpose
Self-hosted household grocery manager. Google Home voice → web confirmation → Woolworths/Coles purchase at lowest total cost (items + delivery). Unix home server, Docker Compose, Cloudflare Tunnel.

Full feature specs: `FEATURE_*.md`. Full spec: `FEATURE_SPEC.md`.

## Developer note
The developer may express tasks unclearly due to language generation difficulties.
**Ask one focused clarifying question before proceeding** when intent or scope is ambiguous.
Do not assume on non-trivial changes.

## Stack
| Layer | Tech |
|---|---|
| Frontend | React + TanStack Router + Shadcn/ui + Radix + Tailwind |
| Backend | Python 3.12 + FastAPI (REST + WebSocket) + PydanticAI |
| DB | PostgreSQL · Alembic migrations · SQLAlchemy ORM |
| Auth | Clerk → `backend/app/auth/provider.py` (swap here only) |
| Automation | Playwright · `curl_cffi` (Akamai TLS spoofing) |
| Infra | Docker Compose · Cloudflare Tunnel (Google Home webhook) |

## Repo layout
```
/
├── frontend/
│   ├── src/routes/          # TanStack Router file-based routes
│   ├── src/components/      # Shadcn + compound components
│   ├── src/lib/             # api.ts, ws.ts
│   ├── e2e/                 # Playwright e2e tests
│   └── AGENT.md             # Frontend-specific rules
├── backend/
│   ├── app/routes/          # FastAPI routers (one per domain)
│   ├── app/auth/            # provider.py — auth abstraction
│   ├── app/models/          # SQLAlchemy + Pydantic schemas
│   ├── app/services/        # Business logic (no HTTP, no DB session)
│   ├── app/db/              # Session factory, Base
│   ├── alembic/             # Migrations
│   └── AGENT.md             # Backend-specific rules
├── skills/                  # Agent reference files ← read before acting
├── Makefile
├── docker-compose.yml
├── .env.example
├── AGENT.md                 ← this file
└── FEATURE_*.md             # Feature specs
```

## Make targets
| Target | Action |
|---|---|
| `make up` | Start all services, adapt port used for multi-agent development in Claude |
| `make down` | Stop all |
| `make test` | All tests (compact output) |
| `make migrate` | `alembic upgrade head` |
| `make lint` | `ruff` + `ty` + `eslint` + `prettier` |
| `make hm-up` | Human-readable startup with logs |
| `make hm-test` | Human-readable test output |
| `make migration-new name="desc"` | Create Alembic migration |

## Rules (non-negotiable)
1. **File length** — Python ≤ 300 lines · TypeScript ≤ 400 lines
2. **Single responsibility** — one purpose per function/class/file. Check at task end.
3. **TDD** — tests alongside or before code
4. **Commits** — `feat: <name>` per feature. Multi-feature task → commit after each, staging only that feature's files.
5. **FastAPI routes** — `backend/app/routes/`. Feature clusters → subdirectory.
6. **Tailwind** — existing theme classes only. No custom additions.
7. **React** — no global context unless required. Compound components. Limit prop drilling.
8. **Docs** — document all public functions and files.
9. **Dependencies** — Can get any needed. Use Context7 MCP for information
10. **Skills** — update the relevant `skills/` file after change to that area.

## Toolchains
**Python:** `uv` · `ruff` · `ty` · `pydantic` · `alembic` · `testcontainers`
**TypeScript:** `pnpm` · `prettier` · `eslint` · `vitest` (unit) · `playwright` (e2e)

## Key decisions
- **Store integration priority:** official API → internal XHR/JSON endpoints → Playwright HTML scrape.
  Both stores protected by **Akamai**. Strategy: human logs in via browser → Playwright reuses session cookie until expiry. TLS spoofing fallback: `curl_cffi` or `tls-client`. Never use bare `requests`/`httpx` against store URLs.
- **Checkout:** Playwright automates to payment page only. Never stores or enters card details.
- **Credentials:** AES-256 encrypted in PostgreSQL. Key in env var. Never logged or sent to frontend.
- **Optimisation:** minimise (item prices × qty) + all delivery fees. Split-store evaluated. Delivery windows fetched live (15-min hold). User constraints set in Settings.
- **Voice:** Google Smart Home webhook → `/api/voice-requests` → unconfirmed → user accepts/rejects in web app.
- **Auth:** Clerk JWT on every backend request. Webhook uses shared secret header. Abstracted via `provider.py`.
