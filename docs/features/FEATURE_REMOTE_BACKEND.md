# Feature: Web app on another computer

Status: **implemented** — the web app can run on a household member's own
computer while the backend and PostgreSQL stay on the home server, reached
through a Cloudflare Tunnel hostname. Steps for people:
[`human-setup.md`](../human-setup.md#optional-web-app-on-your-own-computer).

## How the browser finds the backend
The browser calls the backend directly. Server-rendered route loaders also
call the API; optional `SERVER_API_BASE_URL` selects an internal address for
those requests without changing browser configuration. `frontend/src/lib/backend-url.ts` turns two `VITE_*`
values into addresses, and `lib/config.ts` exports them for this build.

| Key | Rule |
|---|---|
| `VITE_API_BASE_URL` | `/` → current page origin in the browser, local backend during SSR unless `SERVER_API_BASE_URL` overrides it. Blank → `http://localhost:8000`. Otherwise requires `http://` or `https://`; trailing slashes dropped. |
| `VITE_WS_URL` | Blank → the API address with `http` → `ws` (so `https` → `wss`) plus `/ws`. Must start `ws://` or `wss://`. |

`VITE_*` values are fixed when Vite starts (`make hm-dev-frontend`, `make dev`) or builds
(`make build`).

For a reverse proxy, prefer `VITE_API_BASE_URL=/` and a blank `VITE_WS_URL`.
Each frontend origin must proxy `/api`, `/api/*`, and `/ws` to the backend.
This keeps browser navigation, preloads, and WebSockets on the page's origin
across domain aliases and ports. SSR still uses `SERVER_API_BASE_URL`.
Separate frontend/API origins require an exact CORS entry for every frontend
origin, including standard HTTPS separately from any explicit port. A successful
SSR page refresh does not verify browser CORS or WebSocket routing.

## Running only the web app
`make hm-dev-frontend` installs frontend packages and runs `vite dev` on `WEB_PORT`
(default 3000, `--strictPort`). It reads the checkout root `.env`, needs no
Docker, database or backend settings, and accepts only the `frontend`
component's keys from `scripts/service-components.mjs` (`CLERK_*`, `VITE_*`
and `SERVER_API_BASE_URL`), the same rule the production build uses.

Full-stack `make dev` supplies its own local API and socket URLs. Use
`hm-dev-frontend` for a remote backend. Its separately launched host process
is stopped with Ctrl-C in its terminal, outside Compose ownership.

## Server side
- CORS: the web app's origin (`http://localhost:<WEB_PORT>`) must be in the
  backend's `CORS_ORIGINS`. The default already holds `http://localhost:3000`.
- Auth is unchanged: every household route needs a Clerk session token, and
  `/ws` takes it as `?token=` (`FEATURE_AUTH.md`). Publishing the API is safe
  only with `DEV_AUTH_BYPASS=false`.
- Clerk production keys are bound to their domain, so `localhost` needs the
  development instance's keys.

## Tests
`frontend/src/lib/__tests__/backend-url.test.ts`: defaults, blank values,
`https` → `wss`, trailing slashes, an explicit socket address, and both
missing-scheme errors.
Same-origin tests cover HTTPS, explicit ports, HTTP development, and SSR selection.
