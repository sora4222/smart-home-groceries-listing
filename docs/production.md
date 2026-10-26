# Production server (no Docker)

Runs the app on an Ubuntu 26.04 home server as systemd services, without
Docker. `make build` compiles and packages everything as your user without sudo.
`sudo make install-server` installs
that local bundle using reviewed fixed control code. The server repository
provides the verified release updater for unattended production deployment.

Do the vendor-console parts of [`human-setup.md`](human-setup.md) first
(Clerk, Cloudflare Tunnel token, Alexa skill). This page replaces its
section 6 and "Optional: PostgreSQL installed on the server" on the server.
Developer machines keep Docker Compose (`make dev`) and Testcontainers.

## What runs

| Unit | Program | Listens on |
|---|---|---|
| `grocery-backend.service` | `/opt/grocery/bin/grocery-backend` | `127.0.0.1:8000` |
| `grocery-alexa-bridge.service` | gunicorn in `/opt/grocery/alexa-bridge/venv` | `127.0.0.1:8081` |
| `grocery-frontend.service` | Node, `/opt/grocery/frontend/.output` | `127.0.0.1:3000` |
| `grocery-tunnel.service` | `/usr/bin/cloudflared` (Cloudflare apt repo) | outbound only |
| `ollama.service` | Ollama's own unit (when it is the triage provider) | `127.0.0.1:11434` |
| `postgresql.service` | the server's PostgreSQL | `localhost:5432` |

`grocery.target` starts its enabled units at boot. The installer enables the
backend and frontend, and Alexa when its required settings are supplied.
Configure and enable the optional tunnel separately.

**Security model:**

- Each grocery service runs as a throwaway user systemd creates at start
  (`DynamicUser=`). It has no login, home, sudo, capabilities or supplementary group
  memberships.
- Each service sees a read-only system, and every listener is on loopback.
- Shared hardening lives in `/etc/systemd/system/grocery-.service.d/10-hardening.conf`.
  Run `systemd-analyze security grocery-backend` to review it.
- Before each start, the fixed root-owned resolver
  `/usr/local/lib/grocery-service/libexec/grocery-env` runs as root
  (`ExecStartPre=+`). It reads the 1Password token from its file, resolves
  `op://` values, and writes only that service's keys to
  `/run/grocery-secrets/<service>.env` (root, `0600`).
- systemd reads that file into the service environment, and the unit deletes
  it once the program is running.
- The token never enters a service's environment, command line or readable
  files. Per-service key lists: `scripts/service-components.mjs`.
- Bundles cannot replace installer modules, resolvers or units. Control-code
  changes require a separately reviewed source installation. Local installation
  trusts the supplied source/bundle; `grocery-update` verifies provenance.

## 1. Home server shell: prerequisites

For a prebuilt bundle, install only the runtime prerequisites:

```bash
sudo bash deploy/service/install-dependencies.sh
```

This installs Ubuntu runtime packages, including Node, Python and PostgreSQL,
and enables PostgreSQL at boot. No Docker, compiler or 1Password account is
required. PostgreSQL may serve other applications; use a dedicated grocery
application role and database.

To build locally, also install the build tools:

```bash
sudo apt install build-essential cmake clang libclang-dev pkg-config perl git python3 curl
```

You also need Rust (rustup), Node 22 with pnpm, and uv on your `PATH`.
Services use system-wide `/usr/bin/node` and `/usr/bin/python3`.
Install optional Ollama and cloudflared separately using their official
instructions. The application installer does not download these tools or
pull models; see the triage and tunnel setup in [`human-setup.md`](human-setup.md).

### Optional 1Password

Literal application configuration requires no 1Password account or CLI.
For `op://` references or a Developer Environment, install
[1Password CLI](https://developer.1password.com/docs/cli/get-started/)
system-wide. Use an application-scoped service account with only the necessary
read access and manage Developer Environments through their MCP server.

Pass an existing protected token file with `--op-token-file PATH` and select
an Environment with `--op-environment-id ID`. Startup exports
`OP_SERVICE_ACCOUNT_TOKEN` and `OP_ENVIRONMENT_ID` are supported when explicitly
preserved through sudo. Never put token values in command arguments.

Startup saves the token as `/etc/1password/grocery.token` and the optional ID
as `/etc/grocery/op-environment-id`, both root-only. Omitted inputs retain
existing selections. The resolver prefers the per-app token, then an existing
`/etc/1password/token`. Legacy desktop integration through
`/etc/grocery/op-desktop-user` needs a person at startup; use a service account
for unattended boot. Token-file authentication failure never falls back to
the desktop account.

Environment values override retained local settings at every service start;
omitted Environment keys retain local fallbacks. Builds select an Environment
separately, but release bundles need no production configuration. See
[build configuration](using-the-app.md#configuration-and-1password).

## 2. Application configuration

Pass a mode-`0600` dotenv file explicitly with `--config PATH`, or preserve only
intended application exports through sudo. Startup stores supplied settings in
`/etc/grocery/grocery.env` with mode `0600`. Exports override supplied file
values; omitted settings retain previous values. A checkout's `.env` is not
copied automatically. If using `make setup-env`, pass its file explicitly and
retain existing database credentials and the encryption key during migration.

| Key | Production value |
|---|---|
| `DATABASE_URL` | An application-role PostgreSQL URL or `op://` reference. Use `localhost` for this server, not Compose's `db` or `host.docker.internal`. |
| `CREDENTIAL_ENCRYPTION_KEY` | The existing key for an existing database, required to decrypt stored credentials. |
| `INTAKE_LLM_PROVIDER` | `ollama`, `openai` or `off` |
| `INTAKE_LLM_BASE_URL` | `http://127.0.0.1:11434/v1` for a separately managed local Ollama |
| `INTAKE_LLM_MODEL` | Your installed model; `llama3.2` when empty |
| `CLOUDFLARE_TUNNEL_TOKEN` | Optional token from human setup; enable the tunnel service separately. |
| `VITE_API_BASE_URL`, `VITE_WS_URL` | Prefer `/` and an explicitly empty socket URL behind a proxy routing `/api` and `/ws`. Separate HTTPS/WSS API URLs require an exact frontend origin in `CORS_ORIGINS`. |
| `SERVER_API_BASE_URL` | Normally `http://127.0.0.1:8000` for server-side rendering |
| `DEV_AUTH_BYPASS` | `false` |

Service-owned `BIND_ADDRESS` and `ACCESS_LOG_DIR` cannot override the units'
loopback listeners or `/var/log/grocery` logging. Compose settings such as
`COMPOSE_PROFILES` do not configure native services.

The installer rejects changed database targets and encryption keys. Move data
and keys deliberately rather than pointing an update at a new empty database.
Password rotation for the same target is allowed; changing configuration does
not rotate an existing database role's password.

Four public frontend settings take effect after a frontend restart without a
rebuild: `VITE_API_BASE_URL`, `VITE_WS_URL`, `VITE_AUTH_MODE` and
`VITE_CLERK_PUBLISHABLE_KEY`. Restart the backend after changing its settings.
Other `VITE_*` values are outside this runtime configuration contract.

## 3. Cloudflare dashboard: tunnel routes

The connector now runs on the host. In **Networking → Tunnels →
grocery-list → Routes**, change the service URLs:

| Route path | Service URL |
|---|---|
| `alexa` | `http://localhost:8081` (was `http://alexa-bridge:8081`) |
| `api/voice-requests` (optional webhook) | `http://localhost:8000` (was `http://backend:8000`) |

Do not route anything else to `localhost:8000`, except the separate API
hostname for
[the web app on your own computer](human-setup.md#optional-web-app-on-your-own-computer).

## 4. Home server shell: build and install

Build/package as yourself. An explicit empty build configuration keeps the
runtime independent of site settings:

```bash
GROCERY_ENV_FILE=/dev/null make build
sudo make install-server args='--config /protected/application.env'
```

`make build` creates `build/production` and
`build/grocery-linux-<architecture>.json.gz`; `make package-server` is an alias.
`install-server` selects that packaged bundle by default. For another bundle:

```bash
sudo make install-server bundle=/protected/runtime.json.gz args='--config /protected/application.env'
```

Add optional `--op-token-file PATH --op-environment-id ID` inside `args` when
using 1Password. `hm-install-server` and `hm-uninstall-server` remain aliases.
Local installation trusts its source and bundle; use the verified updater for
downloaded production releases. Existing settings are retained when omitted.

To install the fixed control layer while retaining the existing application,
back up its configuration and database, then run:

```bash
sudo bash deploy/start.sh --adopt-existing
```

Installation provisions a local database/role only on a fresh installation,
checks the schema, backs up data, and runs migrations as `grocery-migrate`
with application database credentials. It switches `/opt/grocery` to a
root-owned release, retains old releases/configuration, enables the core
services at boot, and checks health.

If using the optional tunnel, install cloudflared and configure its token
before enabling it:

```bash
sudo systemctl enable --now grocery-tunnel.service
```

Verify `systemctl is-enabled grocery.target grocery-backend grocery-frontend`
and `systemctl is-active grocery-backend grocery-frontend`. API health at
`http://127.0.0.1:8000/api/health` should succeed. Test sign-in and a grocery
operation through private ingress, and check the tunnel's health if used.
Local health alone does not prove authentication or external integrations.

## Updating

Install the server repository's updater and configure
[GitHub release protections](human-setup.md#github-release-protections) before
enabling unattended deployment. Merge reviewed code into main and push a new
`release-N.N.N` tag. The native-service Action builds and tests installation,
update and recovery across four configuration modes before publishing an
attested runtime. Branches and pull requests run tests without publishing.

The updater retains site configuration. It does not deploy the server
repository's Caddyfile, change Tailscale Serve routes, or update 1Password
Environment values. Apply those changes separately and restart affected
services. A successful release installation or local health check does not
prove browser API or WebSocket routing.

```bash
make hm-update-server                       # latest completed stable release
make hm-update-server release=release-1.2.3 # one exact release
```

These call `sudo grocery-update`; its timer checks automatically. Updates
verify release provenance before installation. They do not depend on a local
branch, pull source or build application code on the server.

The installer rejects destructive pending migrations and schema downgrades,
stops application writes, takes a root-only PostgreSQL dump under
`/var/backups/grocery/`, and migrates before switching the runtime. There is
no destructive-migration bypass flag; plan intentional data removal separately.
The SQL check is a guard, not proof arbitrary application code cannot alter data.

sqlx applies migrations in transactions and rejects edits to already-applied
files. Fix mistakes with a new migration. Old releases and backups are retained;
choose retention after testing recovery. Migration or health failures require
deliberate recovery: a file rollback does not reverse database migrations.

**Recovering a failed update:** stop `grocery.target`, retain logs and the
failed release, and inspect the migration state. Restore the selected dump
into a separate recovery database first and verify its contents. Replacing
live data overwrites current contents; take another backup and plan that
operation explicitly. Recover matching application/configuration and the
original encryption key together. The installed data-identity guard prevents
accidental database/key replacement; intentional moves need a coordinated
migration. Restart and verify application operations before resuming updates.

## Operating

| Task | Command |
|---|---|
| State of every unit | `make hm-status` |
| Recent logs | `make hm-logs`, or `journalctl -u grocery-backend -f` |
| Deploy the latest verified release | `make hm-update-server` |
| Install a reviewed local build | `make build`, then `sudo make install-server` |
| Restart everything | `sudo systemctl restart grocery.target` |
| Stop everything | `sudo systemctl stop grocery.target` |
| Remove (keeps `/etc/grocery`, the database and logs) | `sudo make hm-uninstall-server` |

**Reaching the web app from another device:** the backend and web app
listen on loopback, and the browser calls the backend at
`VITE_API_BASE_URL` directly. Use an SSH tunnel
(`ssh -L 3000:localhost:3000 -L 8000:localhost:8000 <server>`).
Alternatively, publish both ports privately with Tailscale Serve and set
`VITE_API_BASE_URL` and `CORS_ORIGINS` to those addresses. To run the web
app on your own computer instead, against the backend through the tunnel,
see [human setup](human-setup.md#optional-web-app-on-your-own-computer).

## Troubleshooting

| Symptom | Fix |
|---|---|
| A component cannot resolve a setting | Run `sudo /usr/local/lib/grocery-service/libexec/grocery-env check <component>` without printing values. Check token access, Environment selection and references. Units retry every 15 seconds. |
| Browser preflight has no `Access-Control-Allow-Origin` | With separate API URLs, include the exact page origin, including its port, in `CORS_ORIGINS` and restart the backend. For same-origin mode, first proxy `/api` and `/ws` on every frontend origin, then set `VITE_API_BASE_URL=/`, clear `VITE_WS_URL` and restart the frontend. A release retains old site settings. |
| `make hm-install-server`: `DATABASE_URL names "db"` | Set the host to `localhost` (section 2). |
| Backend logs `password authentication failed` | Coordinate the database role password and configured URL. Reinstalling does not rotate an existing role's password. |
| Backend logs `no pg_hba.conf entry` | Ubuntu's default `host all all 127.0.0.1/32 scram-sha-256` line was removed. Restore it and restart PostgreSQL. |
| Alexa says the skill cannot be reached | Check that the tunnel route points at `http://localhost:8081` (section 3) and that `grocery-tunnel` is running. |
| `make build`: `missing …` | Install the listed tool (section 1). |
| Triage says "The checker could not be reached" | `systemctl status ollama`, then `ollama list` should show the model. |

## Private ingress with Tailscale

### Same-origin proxy

Prefer a frontend HTTPS origin that routes `/api`, `/api/*` and `/ws` to
`127.0.0.1:8000`, and other paths to `127.0.0.1:3000`. Preserve paths and
WebSocket upgrades. The server repository owns the Caddy configuration and
its ingress verification scripts.

Before changing frontend settings, install and reload the proxy configuration.
Route every frontend endpoint through it, including any older Tailscale port
used by desktop clients. Forwarding that port directly to Node on 3000 bypasses
the API proxy. A loopback-only Caddy listener can provide the shared routing
when Tailscale Serve terminates HTTPS. Preserve its incoming Host header.

Manage production Environment settings through the 1Password MCP server.
When its available tool only appends variables, edit existing keys in the
1Password editor and verify that each name occurs once. Set
`VITE_API_BASE_URL=/`, `VITE_WS_URL=` and
`SERVER_API_BASE_URL=http://127.0.0.1:8000`. Restart `grocery-frontend`.
These public runtime settings need no rebuild. Retain the existing API endpoint
and required `CORS_ORIGINS` entries for older clients; restart the backend if
its allowlist changes.

Verify the public runtime settings injected into the served sign-in HTML,
authenticated navigation between Pending and Triage, and the `/ws` route.
Unauthenticated protected API requests and WebSocket upgrades should return
401, rather than frontend HTML or a sign-in redirect. The server repository's
access and frontend checks cover those deployment boundaries without tokens.

### Separate frontend and API origins

For a Tailscale Service, configure separate frontend and API HTTPS endpoints
under your own service name. The loopback listeners and any existing intake
tunnel remain in place. For example, after replacing `<service-name>`:

```sh
sudo tailscale serve --service=<service-name> --https=8100 http://127.0.0.1:3000
sudo tailscale serve --service=<service-name> --https=8101 http://127.0.0.1:8000
tailscale serve status
```

Define the Service's `tcp:8100` and `tcp:8101` endpoints, approve the intended
Service host, and grant access only to intended household devices. Do not
reset unrelated Serve configuration or rename a shared server. See
[Tailscale Services](https://tailscale.com/docs/features/tailscale-services).

Set `VITE_API_BASE_URL` to your Service's API HTTPS URL, `VITE_WS_URL` to its
WSS `/ws` URL, and `SERVER_API_BASE_URL` to `http://127.0.0.1:8000`.
Set `CORS_ORIGINS` to the exact permitted frontend origins, retaining any
existing required origins. Set the desktop client's `GROCERY_DESKTOP_SERVER_URL`
to the frontend HTTPS URL when applicable. Keep authentication enabled.

Update a production Environment through the 1Password MCP server, then restart
affected services. Supported public frontend URLs take effect without a
rebuild. From another device verify sign-in, the exact CORS response, an
authenticated WebSocket session and a grocery operation. These checks are
separate from local process/health checks; record their results privately
rather than committing hostnames, account identifiers or deployment snapshots.
