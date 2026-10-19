# Google Home Grocery List

A self-hosted household grocery manager: say *"Alexa, add milk to the shopping
list"*, review and confirm what was heard in a web app, and (eventually) have
it purchase from Woolworths/Coles at the lowest total cost. Proof-of-concept,
single home server: systemd services in production, Docker Compose for
development, Cloudflare Tunnel. Full product spec: the
"Google Home Grocery List — Application Specification" doc in the project's
docs.

**Currently implemented:**
- **Voice intake** — the generic webhook, the Alexa channel, the confirmation
  queue and Pending Requests (`docs/features/FEATURE_VOICE.md`).
- **LLM triage** — a local model (Ollama, llama.cpp or vLLM) checks every
  intake item ("would a supermarket sell this?") before the household sees it; the `/triage` page holds what the
  checker rejected or was unsure of (`FEATURE_TRIAGE.md`).
- **The grocery list** — add, edit, annotate, chip and commit items
  (`FEATURE_GROCERY_LIST.md`), with **item rules** that chip items
  automatically (`FEATURE_ITEM_RULES.md`).
- **Price comparison** — each item's products at Woolworths and Coles,
  cheapest per unit first, with unit prices, specials and multibuys
  (`FEATURE_STORE_INTEGRATION.md`), and choosing one product per item
  (`FEATURE_PRODUCT_CHOICE.md`).
- **Send to Woolworths** — puts every chosen Woolworths product into your own
  Woolworths trolley through a bookmark you press on woolworths.com.au; you
  pay on Woolworths (`FEATURE_TROLLEY_HANDOFF.md`,
  `docs/FEAT_WOOLWORTHS_ACCESS.md`).
- **Access logs** — every request is logged to a daily file and the
  database; the `/logs` page shows who asked for what
  (`FEATURE_ACCESS_LOGS.md`).

Ordering, checkout and spending analysis are not built yet; their
`FEATURE_*.md` files are placeholders.

> Despite the repository name, Google Home cannot be integrated directly — the
> Smart Home Action / OAuth account-linking route was dropped. Alexa is the
> working voice channel; Google Tasks and Google Keep are planned. See
> `docs/features/FEATURE_VOICE.md`.

## First-time setup
**Start with [`docs/human-setup.md`](docs/human-setup.md).** It covers every
manual provisioning step (Clerk, Cloudflare Tunnel, the Alexa skill and its
interaction model, `.env`, Woolworths), ordered by dependency. `make setup-env` fills in the random secrets.

For everyday jobs (a Woolworths shop, changing the delivery time, fixes) see
[`docs/using-the-app.md`](docs/using-the-app.md).

## Using the app on a Mac
There is no packaged macOS app (no `.dmg`, Tauri or Electron build, and no
PWA manifest). On a Mac you run the web app locally with `make web`; it talks
to the backend on the home server, which does the Woolworths and Coles
lookups. A browser shortcut then gives it its own Dock icon and window.

The home server must already be published on its own hostname, with sign-in
on and `http://localhost:3000` in `CORS_ORIGINS`. Those steps are in
[`docs/human-setup.md`](docs/human-setup.md#optional-web-app-on-your-own-computer).

1. Install the tools (Homebrew from [brew.sh](https://brew.sh)):

   ```bash
   xcode-select --install       # git and make
   brew install node pnpm       # Node 22 or newer
   ```

2. Clone the repository and create `.env` in its root with only these keys:

   | Key | Value |
   |---|---|
   | `VITE_API_BASE_URL` | `https://grocery-api.<your-domain>` |
   | `VITE_WS_URL` | empty (becomes `wss://grocery-api.<your-domain>/ws`) |
   | `VITE_CLERK_PUBLISHABLE_KEY`, `CLERK_PUBLISHABLE_KEY` | the Clerk development instance's `pk_test_…` key |
   | `CLERK_SECRET_KEY` | the matching `sk_test_…` key |

   Clerk `pk_live_…` keys do not work on `localhost`.

3. Run `make web` and open [localhost:3000](http://localhost:3000). Sign in.
   The Pending Requests page should load with no errors in the browser
   console.

4. Add it to the Dock:
   - **Safari** (macOS 14+): **File → Add to Dock**.
   - **Chrome**: **⋮ → Cast, save and share → Install page as app**.

   The Dock icon only opens the page. `make web` must be running in a
   terminal for it to load; stop it with `Ctrl+C`.

## Stack
| Layer | Tech |
|---|---|
| Frontend | React 19 + TanStack Start (SSR) + Shadcn-pattern UI + Tailwind v4 |
| Backend | **Rust + Axum** (REST + WebSocket) |
| Database | PostgreSQL + `sqlx` (migrations embedded in the binary) |
| Auth | Clerk (JWT), abstracted behind `backend/src/auth/mod.rs` |
| Alexa | `sidecars/alexa-bridge` — Python `ask-sdk`, the only Python here |
| Stores | `wreq` (Chrome TLS emulation) for search · `chromiumoxide` for checkout (not yet built) |
| Infra | Production: systemd services, no Docker (`docs/production.md`) · Development: Docker Compose · Cloudflare Tunnel |

The backend is a single compiled binary with its migrations embedded: no
interpreter, virtualenv or migration files on the home server.

## Running it
On the home server (no Docker): `make build && sudo make install`. See
[`docs/production.md`](docs/production.md).

For development:

```bash
make setup-env              # makes .env and fills the random secrets
                            # then fill the rest from docs/human-setup.md
make up                     # Postgres, backend, Alexa bridge, Cloudflare Tunnel
make migrate                # optional — the backend migrates itself at startup

cd frontend && pnpm install && pnpm dev   # http://localhost:3000
```

`make hm-up` runs the containers in the foreground with full logs.

To run only the web app on your own computer, against a backend on the home
server, see [Using the app on a Mac](#using-the-app-on-a-mac).

To run the backend on the host instead of in its container:

```bash
make up                                   # Postgres only is enough
cd backend && cargo run                   # reads .env via the environment
```

Set `STORE_CLIENTS=fake` to use a small built-in catalogue instead of the
real Woolworths and Coles websites (development, tests, or a machine that
should not reach the stores).

Without Clerk configured yet, set `DEV_AUTH_BYPASS=true` in `.env` so the
backend accepts requests without a JWT. It logs a warning at startup — see
`docs/human-setup.md` for why this must never be set on a host reachable from
the internet.

## Testing
```bash
pnpm install       # root Testcontainers tooling
cd frontend && pnpm install && pnpm exec playwright install chromium && cd ..
make test          # Rust + Vitest + pytest + tooling + desktop/mobile Playwright
make hm-test       # same, verbose
```

Tests require a running Docker daemon. Testcontainers starts a fresh PostgreSQL
16 server for each backend or browser run, with random credentials, a random
loopback port, and no application volume. SQLx creates a separate database per
backend integration test. Cleanup runs after success, failure, and interrupts.
Tests ignore the application's `.env`, database credentials, and Compose
profiles. They never need a 1Password account or access to a household database.

`make test-backend` and `make e2e` own their test infrastructure independently.
See [database modes](docs/features/FEATURE_DATABASE.md) and the
[testing skill](.claude/skills/testing/SKILL.md).

## Repository layout
```
backend/          Rust + Axum API — see backend/AGENTS.md
frontend/         TanStack Start app — see frontend/AGENTS.md
sidecars/
  alexa-bridge/   Python ask-sdk bridge — see its AGENTS.md
docs/
  features/       One spec-and-status doc per major goal
  README.md       Which doc is for whom
  human-setup.md  One-time provisioning (Clerk, Cloudflare, Alexa skill, .env)
  using-the-app.md  Recurring workflows (voice intake, a shop, undo, fixes)
skills/           Reference notes for agents working in this repo — read before
                  touching migrations, the Makefile, git conventions, or
                  store integrations
```

## Development rules
See `AGENTS.md` for the non-negotiables (file length limits, commit
convention, TDD, architecture boundaries, no Python in the backend).
`backend/AGENTS.md`, `frontend/AGENTS.md` and
`sidecars/alexa-bridge/AGENTS.md` cover their own layers.
