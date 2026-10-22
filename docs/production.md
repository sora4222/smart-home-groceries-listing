# Production server (no Docker)

Runs the app on an Ubuntu 26.04 home server as systemd services, without
Docker. `make build` compiles everything as your user, using sudo to resolve
frontend configuration. `sudo make hm-install-server` installs it, sets up the PostgreSQL role, Ollama and cloudflared, and starts
the services. Re-running both is also how an update is deployed.

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

`grocery.target` groups the four grocery units and starts them at boot.

**Security model:**

- Each grocery service runs as a throwaway user systemd creates at start
  (`DynamicUser=`). It has no login, home, sudo, capabilities or group
  memberships.
- Each service sees a read-only system, and every listener is on loopback.
- Shared hardening lives in `/etc/systemd/system/grocery-.service.d/10-hardening.conf`.
  Run `systemd-analyze security grocery-backend` to review it.
- Before each start, `/opt/grocery/libexec/grocery-env` runs as root
  (`ExecStartPre=+`). It reads the 1Password token from its file, resolves
  `op://` values, and writes only that service's keys to
  `/run/grocery-secrets/<service>.env` (root, `0600`).
- systemd reads that file into the service environment, and the unit deletes
  it once the program is running.
- The token never enters a service's environment, command line or readable
  files. Per-service key lists: `scripts/service-components.mjs`.

## 1. Home server shell: prerequisites

Toolchains for `make build`:

```bash
sudo apt install build-essential cmake clang libclang-dev pkg-config perl git python3 curl
```

You also need Rust (rustup), Node 22 with pnpm, and uv, all on your
`PATH`. Node must be installed system-wide (for example `/usr/bin/node`),
not under a home directory, because the web app service runs it.

1Password, for `op://` values in `.env`:

1. Install [1Password CLI](https://developer.1password.com/docs/cli/get-started/)
   system-wide (`/usr/bin/op`).
2. Choose one way in. `grocery-env` uses the first one it finds:

| Option | File | Use |
|---|---|---|
| Per-app service account | `/etc/1password/grocery.token` | Unattended boot (recommended) |
| Shared service account | `/etc/1password/token` | Unattended boot |
| Desktop app | `sudo make hm-install-server OP_DESKTOP_USER=<login>` | Someone approves the 1Password prompt on each start |

A 1Password Developer Environment can supply settings too. Install it once
with `sudo make hm-install-server OP_ENVIRONMENT_ID=<id>`; this writes
`/etc/grocery/op-environment-id`. `grocery-env` then reads the Environment
through `op run --environment` (1Password CLI 2.41 or later) with the same
access option above. Environment values override `.env`, which may then be
absent. The frontend build reads the same ID file, so `make build` and the
services use one source. A 1Password community report says service accounts
could not read Environments in CLI 2.33 beta; verify the token on the server.

Save a token without putting it in shell history or command arguments:

```bash
sudo install -d -m 700 /etc/1password
sudo sh -c 'umask 077; cat > /etc/1password/grocery.token'   # paste, Enter, Ctrl-D
```

The service account needs read access to the project vault only.

The frontend build uses this same resolver and token-file priority through
sudo, reading the checkout's `.env`. Only resolved `CLERK_*` and `VITE_*`
settings reach the unprivileged build; the token stays in the resolver.
Configuration resolution stays in the foreground so sudo can prompt in the
terminal. The compiler runs through the compact log runner after resolution.
A token-file authentication failure stops the build or service startup without
falling back to a desktop account. Local development retains desktop fallback.
No token file permissions need to be relaxed or token exported in your shell.

## 2. `.env`

**Where:** the repository checkout on the server. Make it with
`make setup-env` if it does not exist yet. `sudo make hm-install-server` copies it to
`/etc/grocery/grocery.env` (root, `0600`) every time. Edit `.env`, not the
copy.

| `.env` key | Production value |
|---|---|
| `DATABASE_URL` | `postgres://grocery:<password>@localhost:5432/grocery` (or an `op://` reference). The host must be `localhost`, not `db` or `host.docker.internal`. |
| `INTAKE_LLM_PROVIDER` | `ollama` (default), or `openai` / `off` |
| `INTAKE_LLM_BASE_URL` | `http://127.0.0.1:11434/v1` for the local Ollama |
| `INTAKE_LLM_MODEL` | Optional; `llama3.2` when empty |
| `CLOUDFLARE_TUNNEL_TOKEN` | From human-setup section 3. When empty, no tunnel starts. |
| `VITE_API_BASE_URL`, `VITE_WS_URL` | Leave as `http://localhost:8000` and `ws://localhost:8000/ws` |
| `DEV_AUTH_BYPASS` | `false` |

These keys are ignored in production, so you can leave them as they are:

- `COMPOSE_PROFILES`, `POSTGRES_PORT` and `BIND_ADDRESS`.
- `ACCESS_LOG_DIR`: logs go to `/var/log/grocery`.

`make hm-install-server` stops with a clear message if `DATABASE_URL` or
`INTAKE_LLM_BASE_URL` still names a Docker Compose host.

`VITE_*` values are built into the web app by `make build`, so re-run
`make build` after changing them. Everything else is read at each service
start.

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

```bash
make build          # as yourself: cargo, uv, pnpm into build/production
sudo make install-server   # /opt/grocery, PostgreSQL role, configured services
```

If the configured Ollama or cloudflared executable is missing, install it first
or explicitly allow the installer to download it:

```bash
sudo make install-server ALLOW_DEPENDENCY_INSTALL=1
```

The option also works with `make hm-update-server`. Existing `hm-install-server`
and `hm-uninstall-server` commands remain aliases. The selected 1Password
Environment ID, desktop user and root-only configuration remain in `/etc/grocery`
for subsequent runs.

On each run, `make install-server`:

- creates the `DATABASE_URL` role and database if they are missing, and
  re-sets the role's password from 1Password. The password is sent as a
  SCRAM verifier, never as plain text;
- installs a missing Ollama with its official installer only when
  `ALLOW_DEPENDENCY_INSTALL=1`, then pulls the configured model;
- when cloudflared is missing and installation is explicitly allowed, adds
  Cloudflare's signed apt repository for cloudflared, so the server's
  scheduled apt upgrades keep cloudflared up to date. Ollama is not an apt
  package: to update it, re-run its installer
  (`curl -fsSL https://ollama.com/install.sh | sh`).

**Verify:** `make hm-install-server` ends with `backend is healthy`, `make hm-status`
shows each unit `active (running)`, and the Cloudflare tunnel reports
**Healthy**. On the server, [localhost:3000](http://localhost:3000) redirects
to `/sign-in`.

## Updating

```bash
make hm-update-server        # as yourself; asks for your sudo password once
```

Deploys the branch that is checked out:

1. Fast-forwards it to its upstream. It stops if the checkout has
   uncommitted changes or the branch has diverged.
2. Runs `make build`.
3. Compares the build's sqlx migrations with `_sqlx_migrations` in the
   database and lists the pending ones. It stops before changing anything
   when:
   - a pending migration contains `DROP TABLE/COLUMN/SCHEMA`, `TRUNCATE`,
     `DELETE FROM` or a column type change. Read the file, then run
     `make hm-update-server ALLOW_DESTRUCTIVE=1`.
   - the database has migrations the checkout does not know, which means
     an older branch. Its code would run against a newer schema.
4. Backs the database up with `pg_dump -Fc` to `/var/backups/grocery/`
   (root only, newest 10 kept).
5. Runs `make hm-install-server`. The new backend binary applies the pending
   migrations (`--migrate-only`, as `nobody`) *before* `/opt/grocery` is
   switched over. If a migration fails, sqlx rolls it back and the running
   version is left untouched.

sqlx applies each pending file once, in order, inside a transaction. It
refuses to start if an already-applied migration file was edited, so fix
mistakes with a new migration, never by editing an old one.

**Restore a backup:**

```bash
sudo systemctl stop grocery.target
sudo cat /var/backups/grocery/<file>.dump | sudo -u postgres pg_restore --clean --if-exists -d grocery
```

Then check out the matching commit and run `make hm-update-server` again.

## Operating

| Task | Command |
|---|---|
| State of every unit | `make hm-status` |
| Recent logs | `make hm-logs`, or `journalctl -u grocery-backend -f` |
| Deploy the checked-out branch | `make hm-update-server` |
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
| `grocery-env: cannot load KEY from 1Password` in `journalctl -u grocery-<unit>` | The token file is missing or lacks vault access, or the reference is wrong. Test it with `sudo /opt/grocery/libexec/grocery-env json <unit>` (this prints secrets; clear the terminal afterwards). The unit retries every 15 s. |
| `make hm-install-server`: `DATABASE_URL names "db"` | Set the host to `localhost` (section 2). |
| Backend logs `password authentication failed` | Re-run `sudo make hm-install-server`. It re-sets the role's password from `DATABASE_URL`. |
| Backend logs `no pg_hba.conf entry` | Ubuntu's default `host all all 127.0.0.1/32 scram-sha-256` line was removed. Restore it and restart PostgreSQL. |
| Alexa says the skill cannot be reached | Check that the tunnel route points at `http://localhost:8081` (section 3) and that `grocery-tunnel` is running. |
| `make build`: `missing …` | Install the listed tool (section 1). |
| Triage says "The checker could not be reached" | `systemctl status ollama`, then `ollama list` should show the model. |

## Grocery production Service on Tailscale

The approved Service is `svc:grocery-list-prod` at
`grocery-list-prod.tail33d20b.ts.net`. HTTPS port 8100 forwards to the frontend
on `127.0.0.1:3000`. HTTPS port 8101 forwards to the API on
`127.0.0.1:8000`, including `/ws`. These routes leave the existing loopback
listeners and Cloudflare intake tunnel in place.

The Service host configuration is:

```sh
sudo tailscale serve --service=svc:grocery-list-prod --https=8100 http://127.0.0.1:3000
sudo tailscale serve --service=svc:grocery-list-prod --https=8101 http://127.0.0.1:8000
tailscale serve status
```

Before advertising a replacement host, define Service endpoints `tcp:8100`
and `tcp:8101`, approve that host in Tailscale's Service hosts, and grant access
to the intended household devices. Do not reset all Serve configuration or
rename the shared server. This Service uses its own DNS name and virtual IP.
See [Tailscale Services](https://tailscale.com/docs/features/tailscale-services).

Production Environment changes approved and saved on 2026-10-22:

| Variable | Value |
|---|---|
| `VITE_API_BASE_URL` | `https://grocery-list-prod.tail33d20b.ts.net:8101` |
| `VITE_WS_URL` | `wss://grocery-list-prod.tail33d20b.ts.net:8101/ws` |
| `SERVER_API_BASE_URL` | `http://127.0.0.1:8000` |
| `GROCERY_DESKTOP_SERVER_URL` | `https://grocery-list-prod.tail33d20b.ts.net:8100` |
| `CORS_ORIGINS` | `http://localhost:3000,https://grocery-list-prod.tail33d20b.ts.net:8100` |

The CORS proposal retains the live server's existing localhost origin. If more
origins have been added since this snapshot, merge them before applying it.
Update the production 1Password Environment with the MCP server, rebuild the
frontend so its public URLs change, then restart the affected services.
Keep authentication enabled. Verify the exact CORS response and WebSocket
handshake from the desktop computer after deployment. A backend health response
alone does not establish that sign-in or authenticated trolley operations work.

The Service routes and HTTPS backend health check passed from the Mac on
2026-10-22. All five approved settings are saved in the production 1Password
Environment, with one variable per name. Existing URL and CORS variables were
edited in place through the desktop UI because the MCP append operation had
previously created duplicates. The existing localhost CORS origin was retained.
Deployment completed on 2026-10-22 at 20:03 Melbourne time after the owner
approved 1Password CLI access. Application commit `f49fad4` was built from an
isolated detached checkout at
`/home/sora/projects/grocery-desktop-deploy-20261022-D4tQHy/checkout`.
The existing server checkout's staged and unstaged work was preserved.

That deployment preserved a manually installed loader that read the
Developer Environment from `/etc/grocery/op-environment-id`, because the
branch's loader could not. The versioned loader now reads the same file (see
section 1), so `make build` and `sudo make hm-install-server` no longer need
that overlay. This has not yet been run on the server. Before the next install,
confirm that the Environment ID file exists there, then check that the built
browser assets contain the approved API and WebSocket URLs.

The normal migration check found only the additive
`20261018100000_selection_images.sql` migration. The database was backed up to
`/var/backups/grocery/grocery-20261022-200341-f49fad4.dump` before applying it;
the archive is readable and sqlx records the migration as successful.
Pre-deployment application and configuration snapshots remain under
`/var/backups/grocery/desktop-before-f49fad4/`.

All four grocery services are active. Checks of the running frontend and
backend confirm the approved configuration and absence of 1Password provider
credentials. Built browser assets contain the approved API and WebSocket URLs.
From the Mac, the frontend returns 307 to sign-in, API health returns 200,
and the `/api/grocery-items` preflight allows the exact Tailscale frontend
origin with credentials. Unauthenticated grocery API and WebSocket upgrade
requests both return 401. These checks do not establish a successful
authenticated WebSocket session. Desktop UI inspection was blocked by the
locked Mac; live Clerk sign-in and authenticated store fills remain unverified.

Branch validation on 2026-10-22 passed 635 backend tests, 344 frontend tests,
92 Alexa tests, 136 browser tests and 22 native tests. Repository checks and
launcher/build tooling tests passed. Both the frontend production build with
isolated test configuration and the macOS app bundle built successfully.
The first browser run had one module-fetch failure after dependency installation
changed during the run; the complete rerun with the final lockfile passed.
Live Clerk sign-in and authenticated store fills remain unverified. Windows
and Linux native builds have not been run locally.

After merging current main (`c70e157`) into the desktop branch on 2026-10-22,
`make check`, tooling tests, 359 frontend unit tests, 22 desktop tests and
12 price-comparison browser tests passed. The broader suite counts above are
from the earlier branch validation, not a rerun after that merge.
PR #40's initial GitHub functional, build, native, CodeQL and secret checks
passed. Its four failing audit/Compose checks prompted a separate follow-up:
`rustls-webpki` 0.103.16, webpki roots 1.0.10, `certifi` 2026.10.19 and
`launch-editor` 2.14.2, plus Compose 5.6.1 pinned in both tooling jobs.
The shared Compose test validates profiles, watched source sync and loopback
port mappings using only the committed example configuration. These follow-up
changes are in the PR; the deployed application remains at `f49fad4`.
