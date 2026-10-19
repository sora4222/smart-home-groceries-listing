# Feature: Web app on another computer

Status: **implemented** — the web app can run on a household member's own
computer while the backend and PostgreSQL stay on the home server, reached
through a Cloudflare Tunnel hostname. Steps for people:
[`human-setup.md`](../human-setup.md#optional-web-app-on-your-own-computer).

## How the browser finds the backend
The browser calls the backend directly; the web app's own server never
proxies API calls. `frontend/src/lib/backend-url.ts` turns two `VITE_*`
values into addresses, and `lib/config.ts` exports them for this build.

| Key | Rule |
|---|---|
| `VITE_API_BASE_URL` | Blank → `http://localhost:8000`. Trailing slashes dropped. Must start `http://` or `https://`, else the app throws at load with the key's name. |
| `VITE_WS_URL` | Blank → the API address with `http` → `ws` (so `https` → `wss`) plus `/ws`. Must start `ws://` or `wss://`. |

`VITE_*` values are fixed when Vite starts (`make web`, `make up`) or builds
(`make build`).

## Running only the web app
`make web` installs frontend packages and runs `vite dev` on `WEB_PORT`
(default 3000, `--strictPort`). It reads the checkout root `.env`, needs no
Docker, database or backend settings, and accepts only `CLERK_*` and
`VITE_*` keys (`frontend/vite.config.ts`).

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
