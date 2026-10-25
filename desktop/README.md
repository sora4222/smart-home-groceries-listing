# Desktop client

A Tauri 2 Rust client loads the household web app from a saved server URL and
provides native notifications and a Woolworths store window. React/TanStack Start
screens, Clerk authentication and grocery data run on the server. This client
needs the Rust and pnpm toolchains plus Tauri's native OS build dependencies;
it needs no local PostgreSQL or Docker stack.

## Environment variables required

No environment variables are required. `GROCERY_DESKTOP_SERVER_URL` optionally
sets a public HTTP(S) origin at launch or build time. The app's saved URL takes
priority. Never include credentials; build artifacts contain this URL.
The shared loader supports the root `.env`, exported overrides and an explicit
`GROCERY_ENV_FILE`, including `op://` references.

## Development and packaging

Run commands from the repository root:

| Command | Purpose |
|---|---|
| `make setup` | Install locked dependencies and fetch native Rust crates. |
| `make hm-dev-desktop` | Run Tauri development with a separate identity for this worktree. |
| `make build-desktop` | Build the native client and replace installers in `build/desktop`. |
| `make create-application-desktop` | Alias for `build-desktop`. |
| `make format-desktop` / `make lint-desktop` | Apply rustfmt or Clippy fixes. |
| `make type-desktop` / `make test-desktop` | Check native types or run unit tests. |

Add `human=1` for full build/test output. `make build` packages server components;
use `build-desktop` explicitly for installers for the current OS. Native tests
and a build do not verify authenticated store, Clerk, Tailscale or Cloudflare flows.

See [desktop setup](../docs/human-setup.md#desktop-client-through-tailscale-or-cloudflare)
for server connectivity and [the feature contract](../docs/features/FEATURE_DESKTOP_APP.md)
for behavior and validation limits.
