# Google Home Grocery List

A self-hosted household grocery manager with a React 19 and TanStack Start web
app, a Rust and Axum API, PostgreSQL, a Flask/ask-sdk Python Alexa verification
sidecar, and an optional Tauri desktop client.
Review incoming items, compare Woolworths and Coles products, choose how to split
a shop, and fill the stores' trolleys through bookmarks in your own browser.
Production runs as systemd services on a Unix home server. Development uses
Docker Compose.

## What works

- Alexa custom intents, a shared-secret webhook, and Google Tasks intake feed
  the confirmation queue. Every added item needs a household member's acceptance.
- Local LLM triage separates ordinary requests from items that need closer review.
- The list supports quantities, notes, item rules, product choices and dislikes.
- Product search and the order planner compare chosen products and configured
  delivery fees across Woolworths and Coles.
- Store bookmarks fill trolleys and offer to open checkout. Payment stays on the
  store website. A successful trolley fill records an undoable purchase, which
  feeds purchase history and spending analysis. It does not prove payment.

Google Home's own shopping list has no integration here. Google Tasks is built
for typed tasks, with fake and mock coverage; its real-account flow remains
unverified. Google Keep is paused. Live delivery windows, order tracking and
server-side checkout are not built. See the [feature specs](docs/features/)
for each feature's status and validation limits.

## Setup and development

Install Node.js 22 or newer, pnpm, Rust, uv, and Docker with Compose Watch support.
Rust's native build dependencies are listed in [production setup](docs/production.md).
`all` runs checks, tests and the production build when those prerequisites are ready.
Use the [provisioning guide](docs/human-setup.md) for Clerk, Alexa, Cloudflare and
store bookmarks, then:

```sh
make setup
make setup-env
# Complete the required configuration described below.
make dev
```

`dev` builds and starts the complete stack, waits for backend and frontend health,
and prints its local URLs. Frontend source changes reload through Vite. Backend
and Alexa changes rebuild their containers. Ctrl-C removes this worktree's
containers and network. `make dev-stop` repeats that cleanup safely and retains
all database and model volumes. `make dev human=1` streams full Compose output.
The existing `stop-all` and `hm-dev` names remain aliases.
Compact mode saves logs in a private temporary directory and prints its location.

Development uses `docker-compose.dev.yml` over the base Compose file. API and web
ports bind to loopback; PostgreSQL has no published development port. Each
worktree gets a default Compose project name and ports derived from its path.
Override `COMPOSE_PROJECT_NAME`, `WEB_PORT`, or `BACKEND_PORT` when needed; explicit
project names must be unique across concurrent worktrees. A port collision fails
startup. Use separate tunnel tokens/routes across worktrees when remote intake
must stay isolated. The new default development project uses its own volumes; existing
Compose volumes remain available under their original project name.

For a frontend on this computer against the home server, use
`make hm-dev-frontend`. It retains the separate frontend workflow and its
`WEB_PORT` default of 3000. See [remote backend setup](docs/human-setup.md#optional-web-app-on-your-own-computer).

## Environment variables required

[`make setup-env`](Makefile) creates the ignored root `.env` from
[`.env.example`](.env.example) and fills empty generated secrets without replacing
existing values. Exported values override the file. `GROCERY_ENV_FILE` selects
another file, which must exist. Literal values and `op://` references work for
both launchers and direct application startup. References require the host's
1Password CLI access; application containers never receive the loader's `OP_*`
credentials. See [configuration sources](docs/features/FEATURE_CONFIGURATION.md).

| Component | Configuration |
|---|---|
| Database | `DATABASE_URL`; `POSTGRES_PASSWORD` when using the Compose database. The URL's credentials must match the database. |
| API authentication | `CLERK_JWKS_URL`. `DEV_AUTH_BYPASS=true` is only for isolated local development. |
| Encryption | `CREDENTIAL_ENCRYPTION_KEY`, a base64-encoded 32-byte key. |
| Intake and store bookmarks | `VOICE_WEBHOOK_SECRET`, `ALEXA_BRIDGE_SECRET`, `STORE_TAB_SECRET`. |
| Alexa bridge | `ALEXA_SKILL_ID` and the matching `ALEXA_BRIDGE_SECRET`. |
| Web sign-in | `VITE_CLERK_PUBLISHABLE_KEY`; server Clerk settings as described in the provisioning guide. |
| Cloudflare | `CLOUDFLARE_TUNNEL_TOKEN`; development starts the tunnel by default. |
| Optional integrations | LLM provider and Google OAuth settings in `.env.example`. `STORE_CLIENTS=fake` uses the built-in catalogue. |

Full-stack development supplies its local API/WebSocket URLs and CORS origin.
The separate frontend workflow uses `VITE_API_BASE_URL` and optional `VITE_WS_URL`.
Only `VITE_*` values are public browser configuration; never put secrets in them.

## Commands and validation

| Target | Purpose |
|---|---|
| `setup` | Install locked JavaScript/Python dependencies, fetch Rust crates, install Chromium. |
| `setup-env` | Create the ignored `.env` and fill empty generated secrets. |
| `dev` / `dev-stop` | Start or stop the complete local development stack; retain volumes. |
| `check` | Apply format/lint fixes, check types and file lengths, and run all tests. |
| `format` / `lint` | Apply rustfmt/Biome/Ruff formatting or supported lint fixes. Component targets use `format-<component>` and `lint-<component>`. |
| `test` | Backend, frontend unit, Alexa, desktop, tooling, and Playwright suites. |
| `test-backend`, `test-frontend`, `test-alexa`, `test-desktop`, `test-tooling`, `test-frontend-playwright` | Focused suites. |
| `test human=1` | Full test output. `hm-test` and existing component aliases do the same. |
| `update-db` / `migration-new name="description"` | Apply embedded migrations without serving HTTP, or create a migration. Startup also applies migrations. |
| `reset-db confirm=reset` | Delete and migrate only this worktree's running Compose database; stop its backend first. |
| `hm-compose args='logs backend'` | Service commands using this worktree's Compose context. |
| `build-backend`, `build-frontend`, `build-alexa` | Build individual server components into `build/<component>`. |
| `build` | Build those components and assemble the installable bundle in `build/production`. |
| `build-desktop` / `create-application-desktop` | Package the native client for this OS into `build/desktop`. |
| `install-server` / `uninstall-server` | Install or remove systemd services; existing `hm-` aliases remain available. |
| `all` | Run `check`, `test` and `build`; Make runs each test target once. |

`check`, `format`, and `lint` can change source files. `check-format` remains a
read-only formatting check. Component names are `backend`, `frontend`, `alexa` and
`desktop`; type checks are `type-backend`, `type-frontend` and `type-desktop`.
Commands default to compact output; use `human=1` to pass `--human` to the runners.
Compact-mode failures retain private logs and print the log location.

`reset-db` refuses to run without `confirm=reset`. It never follows `DATABASE_URL`
to a remote database. It requires the local Compose `db` service to be running,
stops the backend, deletes its public schema and reapplies embedded SQLx migrations.
Run `make dev` afterwards. For updates that retain records, use `update-db`.
Worktree project names and ports already derive from the checkout path, so no
`setup-env-worktree` copy step is needed.

Tests require Docker. Backend and browser runners own disposable PostgreSQL 16
containers on random ports. Unit and integration runners ignore household
configuration and 1Password credentials. They never use a household database.
Missing results, failures, unexpected
skips and empty suites fail the gates. See the [testing guidance](.claude/skills/testing/SKILL.md).

## Production and repository layout

On the server, use `make build` and `sudo make install-server`.
The build also creates a configuration-free runtime bundle; installation consumes
that bundle. Supply startup settings with `args='--config /protected/application.env'`
on a fresh host. Optional 1Password inputs use protected files or exports.
`make hm-update-server` calls the server repository's verifier for the latest
completed `release-N.N.N` release; it does not deploy the checked-out branch.
See [production operations](docs/production.md) for prerequisites, GitHub rules,
backups, status and recovery. Install optional Ollama/cloudflared dependencies
separately. Updates preserve configuration and database identity; production uses
systemd, without Docker. Removal retains configuration, databases and shared packages.

| Directory | Responsibility |
|---|---|
| [frontend](frontend/README.md) | Web interface, SSR, Vitest and browser tests. |
| [backend](backend/README.md) | Authenticated REST/WebSocket API, grocery rules, integrations and migrations. |
| [sidecars/alexa-bridge](sidecars/alexa-bridge/README.md) | Verify Amazon requests and forward intents to the API. |
| [desktop](desktop/README.md) | Tauri client that loads the server web app and packages native installers. |
| [scripts](scripts/) | Configuration, lifecycle, test and validation runners. |
| [deploy](deploy/) | Production build, installation and systemd units. |
| [docs](docs/README.md) | Feature contracts, provisioning and everyday workflows. |

Project conventions are in [AGENTS.md](AGENTS.md), with layer-specific guidance
in each application directory. Agent reference skills live in `.claude/skills/`
and the component `skills/` directories. See [CONTRIBUTING.md](CONTRIBUTING.md)
for the software lifecycle and focused commands.

## Desktop client

The Tauri client loads the web app from one saved server URL. Use the frontend's
Tailscale address or Cloudflare HTTPS hostname. The server hosts the screens,
API and database; the desktop client needs no local database or Docker stack.
Woolworths can fill its trolley in a native store window. Coles retains the
browser bookmark workflow.

After `make setup`, use `make hm-dev-desktop` or `make build-desktop`.
`GROCERY_DESKTOP_SERVER_URL` optionally supplies a default at launch or build.
Leave it blank to choose in the app. A saved address takes priority. Build
artifacts contain this public URL, so never include credentials in it.
Development instances use separate application identities for each worktree.
`make test-desktop` runs native unit tests; `make check` includes native lint,
type checks and tests. Installers are copied to `build/desktop` after a successful
build. The server build leaves desktop packaging to `build-desktop`.

The server's browser API and WebSocket URLs must be reachable from the desktop
computer. `SERVER_API_BASE_URL` can keep SSR requests on server loopback while
`VITE_API_BASE_URL` points to a reachable API hostname. See
[desktop setup](docs/human-setup.md#desktop-client-through-tailscale-or-cloudflare)
and [the desktop contract](docs/features/FEATURE_DESKTOP_APP.md).
