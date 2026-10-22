# Frontend

The household web interface uses React 19, TanStack Start for server rendering,
TanStack Router for file-based routes, and Tailwind v4 with Shadcn/Radix components.
It handles intake review, the grocery list, product choices, trolley handoff,
spending analysis and settings. Grocery rules and persistence belong to the
[Rust backend](../backend/README.md).

## Development

From the repository root, `make dev` starts the complete stack and prints its
local URLs. Source changes reload through Vite; configuration and dependency
changes rebuild the frontend container. `make dev-stop` stops the stack and
retains database volumes.

For only the web app, `make hm-dev-frontend` installs locked dependencies and
starts Vite on `WEB_PORT`, default 3000. Set `VITE_API_BASE_URL` to the intended
backend and allow the web origin in its `CORS_ORIGINS`. See
[remote backend setup](../docs/human-setup.md#optional-web-app-on-your-own-computer).
This workflow starts no backend or database.

Direct commands here are `pnpm dev`, `pnpm build`, and `pnpm start`.
The build outputs the SSR bundle in `.output/`; `start` serves it through the
shared environment loader. Root `make build-frontend` copies the SSR bundle to
`build/frontend/.output`. `make build` also packages it with the backend and
bridge in `build/production`. Production frontend builds use the privileged
configuration resolver described in [production setup](../docs/production.md).

## Environment variables required

Vite loads the root `.env`, exported overrides, or an explicitly selected
`GROCERY_ENV_FILE`. It resolves `CLERK_*`, `VITE_*` and `SERVER_API_BASE_URL` references through the
shared loader. See [configuration sources](../docs/features/FEATURE_CONFIGURATION.md).

| Variable | Purpose |
|---|---|
| `VITE_CLERK_PUBLISHABLE_KEY` | Enable Clerk sign-in with the intended instance's public key. |
| `CLERK_SECRET_KEY` | Server-side Clerk configuration when using Clerk. Never expose it through `VITE_*`. |
| `VITE_API_BASE_URL` | Browser API address. Defaults to `http://localhost:8000` outside full-stack development. |
| `SERVER_API_BASE_URL` | Optional internal API address for server rendering only. Compose supplies `http://backend:8000`; the browser keeps `VITE_API_BASE_URL`. |
| `VITE_WS_URL` | Optional WebSocket address. Defaults to the API scheme/host plus `/ws`. |
| `VITE_AUTH_MODE` | Explicit `off` for isolated development/tests with matching backend auth bypass. |

Full-stack `dev` supplies the local API and WebSocket addresses. API and socket
requests go directly from the browser to the backend. `VITE_*` values become
public browser configuration at startup/build time. Restart or rebuild after
changing them. PostgreSQL and intake secrets do not belong in this application.

## Checks and tests

Root `make format-frontend` applies Biome formatting, `make lint-frontend` applies
Biome fixes, and `make type-frontend` checks TypeScript. `make check` applies
fixes and runs all repository tests. Use `make test-frontend` for Vitest and
`make test-frontend-playwright` for desktop/mobile browser coverage. The browser
runner owns a disposable database and local servers with fake providers; Docker
and Chromium are required. `make setup` installs the tooling and browser.
Add `human=1` for full runner output.
Raw `pnpm e2e` assumes an already isolated test stack and resets its data.

## Source ownership

| Path | Responsibility |
|---|---|
| `src/routes/` | Pages and route layouts; `routeTree.gen.ts` is generated. |
| `src/components/` | Feature components and shared UI primitives. |
| `src/lib/api/` | API modules grouped by domain. |
| `src/lib/auth*.ts`, `src/components/auth/` | Clerk-specific client integration. |
| `src/lib/store-tab/` | Store-tab handoff and bookmark behavior. |
| `src/hooks/` | Shared interactions and live updates. |
| `e2e/` | Playwright tests and isolated browser fixtures. |

See [AGENTS.md](AGENTS.md) for component, authentication and routing rules, and
[feature specs](../docs/features/) for behavior contracts.
