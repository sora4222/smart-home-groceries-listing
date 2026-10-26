# Home Grocery List

A self-hosted grocery list for one household. It is a monorepo with four
parts: a Rust and Axum backend with PostgreSQL (sqlx migrations), a React 19
and TanStack Start web app with Shadcn and Tailwind, a small Python Flask
sidecar that uses Amazon's `ask-sdk` for Alexa, and an optional Tauri
desktop app. Development runs everything in Docker Compose with live reload
(`make dev`). Production builds a bundle (`make build`) and installs it on a
Unix home server as systemd services, without Docker (`make install-server`).

The app collects grocery items from voice and typed lists, and keeps one
shared list. It finds the cheapest way to buy that list from Woolworths and
Coles, counting item prices, deals and delivery fees. It never suggests items
to buy.

How it works:

1. You say an item to Alexa, or type it into Google Tasks.
2. The item waits in the web app until a person accepts it.
3. The web app shows the price of each item at Woolworths and Coles.
4. The web app puts the products you chose into your Woolworths or Coles
   trolley. You pay on the store's website. This app never pays.

| Part | What it does | Where it runs |
|---|---|---|
| Backend | Rust and Axum. Stores the list and gets store prices | The home server |
| Database | PostgreSQL | The home server |
| Web app | React and TanStack Start. The pages you use | The home server, or your own computer |
| Alexa bridge | A small Python program that passes Alexa requests to the backend | The home server |
| Desktop app | Tauri. Opens the web app and fills the Woolworths trolley | Your own computer (optional) |

Google Home is not connected. Google does not allow it. Google Keep is paused.
See [`docs/features/`](docs/features/) for what each feature does and what is
not built.

## Which guide to read

| You want to | Read |
|---|---|
| Set up Clerk, Cloudflare, Alexa and the store bookmarks | [`docs/human-setup.md`](docs/human-setup.md) |
| Install the app on the home server | [`docs/production.md`](docs/production.md) |
| Do a shop, undo a mistake, or fix a problem | [`docs/using-the-app.md`](docs/using-the-app.md) |
| Know how one feature works | [`docs/features/`](docs/features/) |
| Follow the code rules | [`AGENTS.md`](AGENTS.md) |
| Know the steps from setup to release | [`CONTRIBUTING.md`](CONTRIBUTING.md) |

## Run everything on your own computer

You need Node 22 or newer, pnpm, Rust, uv, and Docker with Compose.

```sh
make setup       # installs the dependencies and Chromium
make setup-env   # makes .env and fills in the random secrets
make dev         # starts the whole app
```

Fill in the rest of `.env` with [`docs/human-setup.md`](docs/human-setup.md).
`make dev` prints the web app address. Press Ctrl-C or run `make dev-stop` to
stop it. Your database is kept.

### Environment variables required

`make setup-env` makes `.env` from [`.env.example`](.env.example) and fills in
the random secrets. Values can be plain text or 1Password `op://` references.

| Part | Variables |
|---|---|
| Database | `DATABASE_URL`, and `POSTGRES_PASSWORD` for the Docker database |
| Sign-in | `CLERK_JWKS_URL`, `CLERK_SECRET_KEY`, `VITE_CLERK_PUBLISHABLE_KEY` |
| Stored secrets | `CREDENTIAL_ENCRYPTION_KEY` |
| Intake and bookmarks | `VOICE_WEBHOOK_SECRET`, `ALEXA_BRIDGE_SECRET`, `STORE_TAB_SECRET` |
| Alexa | `ALEXA_SKILL_ID` |
| Tunnel | `CLOUDFLARE_TUNNEL_TOKEN` |
| Web app address | `VITE_API_BASE_URL` (only when the web app runs apart from the backend) |

LLM triage and Google Tasks settings are optional. Only `VITE_*` values reach
the browser, so never put secrets in them.

Two `.env` settings help while you develop:

| Setting | What it does |
|---|---|
| `STORE_CLIENTS=fake` | Uses a small fake shop. The backend does not call Woolworths or Coles |
| `DEV_AUTH_BYPASS=true` | Turns sign-in off. Never use it on a server that the internet can reach |

Each Git worktree gets its own ports and containers, so you can run several at
once.

## Use only the web app, with the home server's backend

```sh
make hm-dev-frontend
```

Set `VITE_API_BASE_URL` in `.env` first. See
[web app on your own computer](docs/human-setup.md#optional-web-app-on-your-own-computer).

## Use the desktop app

```sh
make hm-dev-desktop   # run it
make build-desktop    # make an installer in build/desktop
```

The app asks for the web app address the first time. See
[desktop setup](docs/human-setup.md#desktop-client-through-tailscale-or-cloudflare).

## Run the checks and tests

You need Docker running. The tests start their own database and never touch
yours.

```sh
make check   # formats, lints, checks types and file lengths, runs all tests
make test    # runs all tests
```

Add `human=1` to see the full output. Focused suites are `test-backend`,
`test-frontend`, `test-alexa`, `test-desktop`, `test-tooling` and
`test-frontend-playwright`.

## Database changes

```sh
make migration-new name="description"   # new migration file
make update-db                          # apply migrations, keep data
make reset-db confirm=reset             # delete this worktree's local data
```

## Install on the home server

```sh
make build
sudo make install-server args='--config /protected/application.env'
```

Production uses systemd, not Docker. See [`docs/production.md`](docs/production.md).

## Folders

| Folder | What is in it |
|---|---|
| [`backend/`](backend/README.md) | The backend, in Rust |
| [`frontend/`](frontend/README.md) | The web app, in React |
| [`sidecars/alexa-bridge/`](sidecars/alexa-bridge/README.md) | The Alexa bridge, in Python |
| [`desktop/`](desktop/README.md) | The desktop app, in Tauri |
| `deploy/` | Files that install the app on the home server |
| `scripts/` | Scripts that `make` runs |
| `docs/` | Setup, use, and feature docs |

Never commit secret values. Keep them in `.env` or 1Password.
