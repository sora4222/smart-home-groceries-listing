# Production server (no Docker)

Runs the app on an Ubuntu 26.04 home server as systemd services, without
Docker. `make build` compiles everything as your user; `sudo make install`
installs it, sets up the PostgreSQL role, Ollama and cloudflared, and starts
the services. Re-running both is also how an update is deployed.

Do the vendor-console parts of [`human-setup.md`](human-setup.md) first
(Clerk, Cloudflare Tunnel token, Alexa skill). This page replaces its
section 6 and "Optional: PostgreSQL installed on the server" on the server.
Developer machines keep Docker Compose (`make up`) and Testcontainers.

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
| Desktop app | `sudo make install OP_DESKTOP_USER=<login>` | Someone approves the 1Password prompt on each start |

Save a token without putting it in shell history or command arguments:

```bash
sudo install -d -m 700 /etc/1password
sudo sh -c 'umask 077; cat > /etc/1password/grocery.token'   # paste, Enter, Ctrl-D
```

The service account needs read access to the project vault only.

## 2. `.env`

**Where:** the repository checkout on the server. Make it with
`make setup-env` if it does not exist yet. `sudo make install` copies it to
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

`make install` stops with a clear message if `DATABASE_URL` or
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
sudo make install   # /opt/grocery, PostgreSQL role, Ollama, cloudflared, services
```

On each run, `make install`:

- creates the `DATABASE_URL` role and database if they are missing, and
  re-sets the role's password from 1Password. The password is sent as a
  SCRAM verifier, never as plain text;
- installs Ollama with its official installer if it is missing, and pulls
  the model;
- adds Cloudflare's signed apt repository for cloudflared, so the server's
  scheduled apt upgrades keep cloudflared up to date. Ollama is not an apt
  package: to update it, re-run its installer
  (`curl -fsSL https://ollama.com/install.sh | sh`).

**Verify:** `make install` ends with `backend is healthy`, `make status`
shows each unit `active (running)`, and the Cloudflare tunnel reports
**Healthy**. On the server, [localhost:3000](http://localhost:3000) redirects
to `/sign-in`.

## Updating

```bash
make update        # as yourself; asks for your sudo password once
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
     `make update ALLOW_DESTRUCTIVE=1`.
   - the database has migrations the checkout does not know, which means
     an older branch. Its code would run against a newer schema.
4. Backs the database up with `pg_dump -Fc` to `/var/backups/grocery/`
   (root only, newest 10 kept).
5. Runs `make install`. The new backend binary applies the pending
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

Then check out the matching commit and run `make update` again.

## Operating

| Task | Command |
|---|---|
| State of every unit | `make status` |
| Recent logs | `make logs`, or `journalctl -u grocery-backend -f` |
| Deploy the checked-out branch | `make update` |
| Restart everything | `sudo systemctl restart grocery.target` |
| Stop everything | `sudo systemctl stop grocery.target` |
| Remove (keeps `/etc/grocery`, the database and logs) | `sudo make uninstall` |

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
| `make install`: `DATABASE_URL names "db"` | Set the host to `localhost` (section 2). |
| Backend logs `password authentication failed` | Re-run `sudo make install`. It re-sets the role's password from `DATABASE_URL`. |
| Backend logs `no pg_hba.conf entry` | Ubuntu's default `host all all 127.0.0.1/32 scram-sha-256` line was removed. Restore it and restart PostgreSQL. |
| Alexa says the skill cannot be reached | Check that the tunnel route points at `http://localhost:8081` (section 3) and that `grocery-tunnel` is running. |
| `make build`: `missing …` | Install the listed tool (section 1). |
| Triage says "The checker could not be reached" | `systemctl status ollama`, then `ollama list` should show the model. |
