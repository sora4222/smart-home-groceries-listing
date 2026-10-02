# AGENTS.md — Google Home Grocery List

Must support independent execution across separate git worktrees on the same
machine for concurrent multi-agent development.

## Purpose
Self-hosted household grocery manager. Spoken item → web confirmation →
Woolworths/Coles purchase at lowest total cost (items + delivery). Unix home
server, Docker Compose, Cloudflare Tunnel.

Full feature specs: `docs/features/FEATURE_*.md` — short feature
specifications that agents read and write to stop regression.

## Developer note
The developer may express tasks unclearly due to language generation
difficulties. **Ask one focused clarifying question before proceeding** when
intent or scope is ambiguous. Do not assume on non-trivial changes.

## Stack
| Layer | Tech |
|---|---|
| Frontend | React 19 + TanStack Start (SSR) + Shadcn/ui + Radix + Tailwind v4 |
| Backend | **Rust + Axum** (REST + WebSocket) |
| DB | PostgreSQL · `sqlx` migrations · `sqlx` queries (no ORM) |
| Auth | Clerk → `backend/src/auth/mod.rs` (swap provider here only) |
| Automation | `chromiumoxide` (CDP) · `wreq` (TLS fingerprint emulation) |
| Alexa | `sidecars/alexa-bridge` — Python `ask-sdk`, the only Python in the repo |
| Infra | Docker Compose · Cloudflare Tunnel |

**The backend is Rust. There is no Python in it.** The single Python process
is the Alexa bridge sidecar, isolated in `sidecars/alexa-bridge`, which
verifies Alexa's request signature and forwards the item over local HTTP. It
contains no business logic and holds no database credentials. See "Sidecars".

## Repo layout
```
/
├── frontend/
│   ├── src/routes/          # TanStack Router file-based routes
│   ├── src/components/      # Shadcn + compound components
│   ├── src/lib/             # api.ts + api/ (one module per domain), ws.ts
│   ├── e2e/                 # Playwright e2e tests (frontend only)
│   └── AGENTS.md            # Frontend-specific rules
├── backend/
│   ├── src/routes/          # Axum routers (one module per domain)
│   ├── src/auth/            # AuthProvider trait + Clerk impl + shared secrets
│   ├── src/models/          # db.rs (rows) + schemas/ (request/response)
│   ├── src/services/        # Business logic (no HTTP types, no pool creation)
│   ├── migrations/          # sqlx migrations, embedded in the binary
│   ├── tests/               # Integration tests over the real router
│   └── AGENTS.md            # Backend-specific rules
├── sidecars/
│   └── alexa-bridge/        # Python ask-sdk bridge — AGENTS.md of its own
├── skills/                  # Agent reference files ← read before acting
├── docs/
│   ├── features/            # FEATURE_*.md specs
│   ├── human-setup.md       # Steps a person does once — plain words
│   └── using-the-app.md     # Steps a person repeats — plain words
├── Makefile
├── docker-compose.yml
├── .env.example
└── AGENTS.md                ← this file
```

## Make targets
| Target | Action |
|---|---|
| `make up` | Start all services; adapt the port for multi-agent development |
| `make down` | Stop all |
| `make test` | All tests (compact output) |
| `make migrate` | Apply pending `sqlx` migrations |
| `make migration-new name="desc"` | Create a new migration file |
| `make lint` | `cargo clippy` + `cargo fmt --check` + `ruff` + `biome` |
| `make lint-fix` | Apply what can be fixed automatically |
| `make hm-up` | Human-readable startup with logs |
| `make hm-test` | Human-readable test output |

## Rules (non-negotiable)

1. **File length** — Rust ≤ 300 lines · Python ≤ 300 lines · TypeScript ≤ 400 lines
2. **Single responsibility** — one purpose per function/module/file. Check at task end.
3. **TDD** — tests alongside or before code
4. **Commits** — `feat: <name>` per feature. Multi-feature task → commit after each, staging only that feature's files.
5. **Axum routes** — `backend/src/routes/`, one module per domain, merged in `routes/mod.rs`. Feature clusters → subdirectory.
6. **Tailwind** — existing theme classes only. No custom additions.
7. **React** — no global context unless required. Compound components. Limit prop drilling.
8. **Docs** — document all public functions, modules and files.
9. **Dependencies** — can get any needed. Use Context7 MCP for current versions and APIs.
10. **Skills** — update the relevant `skills/` file after a change to that area.
11. **No Python in the backend** — if a task seems to need it, it belongs in a
    sidecar, and only when no maintained Rust crate exists. Say so and ask first.
12. **No dynamic SQL** — every query is a literal string with bind parameters.
    Never reach for sqlx's `AssertSqlSafe`.
13. **Steps for a person** — anything only a person can do (sign up, paste a
    value, log in, press a bookmark, allow a prompt) goes in
    `docs/human-setup.md` or `docs/using-the-app.md`, written with the
    `human-instructions` skill (`.claude/skills/human-instructions/`): plain
    words, grouped by place, in the order the values are needed. Not for
    telling the developer to run tests.

## Toolchains

**Rust:** `cargo` · `rustfmt` · `clippy` · `sqlx` · `#[sqlx::test]`
**Python (sidecar only):** `uv` · `ruff` · `pytest`
**TypeScript:** `pnpm` · `biome` · `vitest` (unit) · `playwright` (e2e)

## Key decisions

- **Backend language:** Rust + Axum, replacing Python + FastAPI. One compiled
  binary with migrations embedded, no interpreter or virtualenv on the home
  server, and the request contract unchanged so the frontend was untouched.
- **No ORM:** `sqlx` with literal SQL. Queries are visible where they run, and
  sqlx refuses a runtime-assembled query string outright.
- **Intake:** Google Home cannot be integrated directly — the Smart Home
  Action / OAuth account-linking design is dropped.
  `POST /api/voice-requests` remains as a generic shared-secret webhook (Home
  Assistant, IFTTT, `curl`, tests). `POST /api/intake/alexa` takes items from
  the Alexa bridge sidecar. Google Tasks and Google Keep are planned, not
  built — see `docs/features/FEATURE_VOICE.md`.
  **Verify the assumptions in the intake brief before building on them.**
- **Nothing reaches the list unasked:** every intake item lands as `pending`
  and needs a household member to accept it, however confident the channel
  was. LLM triage (`INTAKE_LLM_PROVIDER`, `FEATURE_TRIAGE.md`) only decides
  whether a person sees it in Pending Requests or on `/triage`. Reducing marketing-driven buying is a project goal: no part of this
  system suggests, recommends or upsells an item.
- **Store integration priority:** official API → internal XHR/JSON endpoints →
  headless-browser HTML scrape. Both stores are behind bot protection.
  Product search uses each website's own JSON call through `wreq` emulating
  Chrome (built — `docs/features/FEATURE_STORE_INTEGRATION.md`). If that is
  refused: human logs in via browser → `chromiumoxide` reuses the session
  cookie until expiry. Never use plain `reqwest` against a store URL, and
  never retry a refused store request in a loop. `STORE_CLIENTS=fake` for
  development and tests.
- **Checkout:** automated to the payment page only. Never stores or enters
  card details.
- **Credentials:** AES-256-GCM encrypted in PostgreSQL
  (`backend/src/services/encryption.rs`). Key in an env var. Never logged,
  never sent to the frontend.
- **Optimisation:** minimise (item prices × qty) + all delivery fees.
  Split-store evaluated. Delivery windows fetched live (15-min hold). User
  constraints set in Settings.
- **Auth:** Clerk JWT on every backend request, verified against Clerk's JWKS.
  Intake endpoints use shared secrets instead, since they cannot carry a
  session. All of it behind `backend/src/auth/mod.rs`.
- **Provider SDKs stay put:** no route or service imports an auth or LLM
  provider SDK directly — only the one module that owns that provider does.

## Sidecars
A sidecar is allowed only when a required SDK has no maintained Rust
equivalent. It must: authenticate or verify at its own boundary, forward to a
backend intake endpoint using its **own** shared secret, hold no database
credentials, and contain no rules about the grocery list.

| Sidecar | Why | Status |
|---|---|---|
| `alexa-bridge` | Amazon's request signing + skill model are `ask-sdk` Python only | built |
| Google Keep | `gkeepapi` is Python-only and unofficial (master-token auth) | not built |

Google Tasks needs no sidecar — it is a plain OAuth REST API and belongs in
Rust.
