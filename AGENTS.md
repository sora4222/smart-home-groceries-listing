# AGENTS.md: Google Home Grocery List

Must support independent execution across separate git worktrees on the same
machine for concurrent multi-agent development.

## Purpose
Self-hosted household grocery manager. Spoken item → web confirmation →
Woolworths/Coles purchase at lowest total cost (items + delivery). Unix home
server: systemd services in production (`docs/production.md`), Docker Compose
for development, Cloudflare Tunnel.

Full feature specs: `docs/features/FEATURE_*.md`. These are short feature
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
| Alexa | `sidecars/alexa-bridge`. Python `ask-sdk`, the only Python in the repo |
| Infra | Production: systemd units in `deploy/` (no Docker) · Dev: Docker Compose · Cloudflare Tunnel |

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
│   └── alexa-bridge/        # Python ask-sdk bridge, with its own AGENTS.md
├── desktop/                 # Tauri client, loads the server web app
├── .claude/skills/           # Agent reference files, read before acting
├── docs/
│   ├── features/            # FEATURE_*.md specs
│   ├── human-setup.md       # One-time manual provisioning (developer audience)
│   └── using-the-app.md     # Recurring manual workflows (developer audience)
├── deploy/                # Production: build/install scripts, systemd units
├── Makefile
├── docker-compose.yml
├── .env.example
└── AGENTS.md
```

## Make targets
`make dev`, `make check` and `make test` are the common ones. The README lists
the rest, and the `Makefile` is the source of truth.

## Rules (non-negotiable)

1. **File length.** Rust ≤ 500 lines · Python ≤ 500 lines · TypeScript ≤ 500 lines
2. **Single responsibility.** One purpose per function/module/file. Check at task end.
3. **TDD.** Tests alongside or before code
4. **Commits.** `feat: <name>` per feature. Multi-feature task → commit after each, staging only that feature's files.
5. **Axum routes.** `backend/src/routes/`, one module per domain, merged in `routes/mod.rs`. Feature clusters → subdirectory.
6. **Tailwind.** Existing theme classes only. No custom additions.
7. **React.** No global context unless required. Compound components. Limit prop drilling.
8. **Docs.** Document all public functions, modules and files in short concise code comments
9. **Dependencies.** Can get any needed. Use Context7 MCP for current versions and APIs.
10. **No Python in the backend.** If a task seems to need it, it belongs in a
    sidecar, and only when no maintained Rust crate exists. Say so and ask first.
11. **No dynamic SQL.** Every query is a literal string with bind parameters.
    Never reach for sqlx's `AssertSqlSafe`.
12. **Steps for a person.** Anything only a person can do (sign up, paste a
    value, log in, run a bookmarklet, allow a prompt) goes in
    `docs/human-setup.md` or `docs/using-the-app.md`
    grouped by place, in the order the values are needed. **All docs are
    written for software developers, in their terminology.** Not for
    telling the developer to run tests.

## Toolchains

**Rust:** `cargo` · `rustfmt` · `clippy` · `sqlx` · `#[sqlx::test]`
**Python (sidecar only):** `uv` · `ruff` · `pytest`
**TypeScript:** `pnpm` · `biome` · `vitest` (unit) · `playwright` (e2e)

## Key decisions

- **Backend language:** Rust + Axum. One compiled binary with migrations
  embedded, and no interpreter on the home server.
- **No ORM:** `sqlx` with literal SQL. Queries are visible where they run, and
  sqlx refuses a runtime-assembled query string outright.
- **Intake:** Google Home cannot be integrated directly. The Smart Home
  Action / OAuth account-linking design is dropped.
  `POST /api/voice-requests` remains as a generic shared-secret webhook (Home
  Assistant, IFTTT, `curl`, tests). `POST /api/intake/alexa` takes items from
  the Alexa bridge sidecar; `/api/intake/alexa/remove` and `/undo` take
  items off the list (applied at once, undoable by voice). Google Tasks is
  polled over OAuth from Rust (`docs/features/FEATURE_GOOGLE_TASKS.md`);
  "Hey Google, add to my shopping list" does not reach it, because Google keeps that
  list to itself. Google Keep is paused, not built. See
  `docs/features/FEATURE_VOICE.md`.
- **Nothing reaches the list unasked:** every intake item lands as `pending`
  and needs a household member to accept it, however confident the channel
  was. LLM triage (`INTAKE_LLM_PROVIDER`, `FEATURE_TRIAGE.md`) only decides
  whether a person sees it as a Pending request or on the Pending page's Held tab. Reducing marketing-driven buying is a project goal: no part of this
  system suggests, recommends or upsells an item.
- **Store integration priority:** official API → internal XHR/JSON endpoints →
  headless-browser HTML scrape. Both stores are behind bot protection.
  Product search uses each website's own JSON call through `wreq` emulating
  Chrome (built, see `docs/features/FEATURE_STORE_INTEGRATION.md`). If that is
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
  Split-store evaluated. Delivery fees are typed in Settings. Live delivery
  windows are not built.
- **Auth:** Clerk JWT on every backend request, verified against Clerk's JWKS.
  Intake endpoints use shared secrets instead, since they cannot carry a
  session. All of it behind `backend/src/auth/mod.rs`.
- **Provider SDKs stay put:** no route or service imports an auth or LLM
  provider SDK directly. Only the one module that owns that provider does.

## Sidecars

A sidecar is allowed only when a required SDK has no maintained Rust
equivalent. It must: authenticate or verify at its own boundary, forward to a
backend intake endpoint using its **own** shared secret, hold no database
credentials, and contain no rules about the grocery list.

| Sidecar | Why | Status |
|---|---|---|
| `alexa-bridge` | Amazon's request signing + skill model are `ask-sdk` Python only | built |
| Google Keep | `gkeepapi` is Python-only and unofficial (master-token auth) | not built |

Google Tasks needs no sidecar. It is a plain OAuth REST API and belongs in
Rust.

## Code Review Rules

Use the thermo-nuclear-code-review skill when performing a code review.
Always give simple to understand language.
