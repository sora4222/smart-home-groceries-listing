# Setup

One-time provisioning of the external services and the home-server
deployment. Audience: a software developer comfortable with a shell, `.env`
files, Docker Compose and vendor developer consoles. Sections are ordered by
dependency, so the stack is started once, at the end. Every value written to
`.env` persists, so you can stop between sections.

| § | Where | Produces | Time |
|---|---|---|---|
| [1. Generate `.env`](#1-generate-env) | Home server shell | `.env` with generated secrets | 5 min |
| [2. Clerk](#2-clerk) | dashboard.clerk.com | Clerk keys + JWKS URL | 10 min |
| [3. Cloudflare Tunnel](#3-cloudflare-tunnel) | dash.cloudflare.com | Tunnel token + public skill endpoint | 15 min |
| [4. Alexa skill](#4-alexa-skill) | Alexa developer console | Skill ID; interaction model deployed | 20 min |
| [5. Intake triage model](#5-intake-triage-model) | `.env` | LLM provider config | 5 min |
| [6. Start the stack](#6-start-the-stack) | Home server shell, Cloudflare | Running services, healthy tunnel | 10 min |
| [7. Smoke-test the skill](#7-smoke-test-the-skill) | Echo device, web app | End-to-end check | 5 min |
| [8. Woolworths bookmarklet](#8-woolworths-bookmarklet) | Chrome | Bookmarklet installed, store session | 10 min |
| [9. Coles bookmarklet](#9-coles-bookmarklet) | Chrome | Bookmarklet installed, store session | 10 min |

Day-to-day operation: [`using-the-app.md`](using-the-app.md).

**Production server without Docker:** do sections 1–5 and 7–9 here, but
start the app with [`production.md`](production.md) instead of section 6.
In that setup the tunnel routes use `localhost` (`http://localhost:8081`),
not the Compose hostnames below.

## Prerequisites

- Shell access to the home server, in a checkout of this repository, with
  Docker with Compose v5.6.1 or later (the version pinned in CI, supporting
  watched development's `initial_sync`), `make`, `openssl`, Node 22 and `pnpm`.
- A domain whose DNS is managed by Cloudflare (Cloudflare Registrar works).
- The Amazon account the target Echo device is registered to.
- Woolworths credentials (passkey or password manager).
- Coles credentials (saved in Chrome is fine; the app never sees them).

---

## 1. Generate `.env`

```bash
make setup-env
```

Copies `.env.example` to `.env` and fills every generated secret
(`ALEXA_BRIDGE_SECRET`, `VOICE_WEBHOOK_SECRET`, `STORE_TAB_SECRET`,
`CREDENTIAL_ENCRYPTION_KEY`, the Postgres password). Idempotent: existing
values are never overwritten. Keep `.env` open; sections 2–5 write to it.

The default `.env` runs PostgreSQL in Docker (`db` in `COMPOSE_PROFILES`).
On a server with PostgreSQL already installed, do
[Optional: PostgreSQL installed on the server](#optional-postgresql-installed-on-the-server)
before section 6.

**Verify:** the script exits printing `Done.`
**Troubleshooting:** `openssl: not found` → install OpenSSL and re-run.

### 1Password

Plain `.env` values work without 1Password CLI. Any value can instead be a
complete reference, such as
`CLERK_SECRET_KEY='op://Development/grocery-clerk/secret key'`.
Single-quote values containing spaces, `$`, or `#`. When moving database
credentials, retain their existing values. `DATABASE_URL` may reference an
entire connection URL; supplied `POSTGRES_*` settings can each reference
their own fields. Loading them does not rotate database credentials.

**Desktop application, on the development computer:**

1. Install [1Password CLI](https://www.1password.dev/cli/get-started).
2. Unlock 1Password and enable **Settings → Developer → Integrate with
   1Password CLI**.
3. Store project secrets in **Development**. Copy each field's secret reference
   from its menu into the matching `.env` assignment. Set both publishable-key
   variables to the same Clerk key.
4. Start the app and approve the CLI request in 1Password. With multiple
   accounts, export `OP_ACCOUNT` to select the account for this shell.

**Service account, on a headless server (optional):**

1. Create a [service account](https://www.1password.dev/service-accounts/)
   with read access only to the project vault. Desktop development needs none.
2. Supply its token as `OP_SERVICE_ACCOUNT_TOKEN` in the server's protected
   process environment before startup. Keep it out of committed files,
   command arguments, and containers. The shared loader removes `OP_*`
   variables before starting its child.
3. Use the same references and startup commands. The loader first tries the
   exported service-account token. If that read fails or returns no resolved
   value, it retries once using desktop app integration. Without a token it
   goes straight to the desktop app. A headless server needs a working token;
   when both methods fail, startup stops.

**Verify:** startup succeeds without printing values. If a reference fails,
correct the named variable's item, vault access, or desktop authorization.

---

## 2. Clerk

**Console:** [dashboard.clerk.com](https://dashboard.clerk.com). Clerk is the
auth provider; the backend verifies its JWTs against the JWKS endpoint
(`backend/src/auth/clerk.rs`).

1. Create an application (free tier). Enable the **Google** and **Email**
   sign-in strategies, with **Password** on (under **User &
   authentication**; may be labelled **Email, phone, username**).
2. Open **API Keys** and copy:

| Clerk value | `.env` key |
|---|---|
| Publishable key (`pk_…`) | `CLERK_PUBLISHABLE_KEY` and `VITE_CLERK_PUBLISHABLE_KEY` |
| Secret key (`sk_…`) | `CLERK_SECRET_KEY` |
| JWKS URL (ends `/.well-known/jwks.json`; may be under **Advanced**) | `CLERK_JWKS_URL` |

The publishable key is `VITE_`-prefixed because the web app reads it in the
browser; setting it turns sign-in on (`FEATURE_AUTH.md`).

**Alternative: Clerk CLI** (on a machine with a browser; the login is an
OAuth redirect to `127.0.0.1`, so it cannot run on a headless host). The
household's application is `app_3Jqu9YFwyO4a1ZmUuMOFRA7t5Nl`. From the repo
root:

```bash
npm install -g clerk          # or: clerk update --yes
clerk auth login
clerk link --app app_3Jqu9YFwyO4a1ZmUuMOFRA7t5Nl
clerk env pull --app app_3Jqu9YFwyO4a1ZmUuMOFRA7t5Nl --file .env.clerk
```

Copy the two keys from `.env.clerk` into `.env` under the names in the
table above, then delete `.env.clerk`. The JWKS URL still comes from the
dashboard. Do not run `clerk init`: the SDK, middleware and sign-in pages
are already in `frontend/` and init would scaffold them again.
`cd frontend && clerk doctor` checks the login, the link and the keys.

3. Restrict sign-up to the household. Every signed-in user shares the list,
   so an open sign-up would expose it. Open **Allowlist** (under **Protect**
   or **Restrictions**), toggle **Enable allowlist**, add each household
   email address, and **Save**.

**Verify:** all three keys are non-empty and the allowlist lists the
household addresses.

---

## 3. Cloudflare Tunnel

**Console:** [dash.cloudflare.com](https://dash.cloudflare.com). The tunnel
gives Amazon an HTTPS ingress to the Alexa bridge sidecar without opening a
port on the router. The `cloudflared` container in `docker-compose.yml`
runs the connector, so do not install it on the host.

1. Add your domain to Cloudflare if it is not there already.
2. **Networking → Tunnels → Create Tunnel** (older UI: **Zero Trust →
   Networks → Tunnels**). Connector type **Cloudflared**, name
   `grocery-list`.
3. From the install page, copy only the connector token (the `eyJ…` string)
   into `CLOUDFLARE_TUNNEL_TOKEN`. Skip the install command.
4. **Routes → Add route → Published application** (older UI: **Public
   Hostnames → Add a public hostname**):

| Field | Value |
|---|---|
| Subdomain | `grocery` |
| Domain | your domain |
| Path | `alexa` |
| Service URL | `http://alexa-bridge:8081` |

The resulting skill endpoint is `https://grocery.<your-domain>/alexa`; section
4 needs it.

**Security:** expose only the sidecar here. Route to `backend:8000` only
for [the web app on your own computer](#optional-web-app-on-your-own-computer),
and only with `DEV_AUTH_BYPASS=false`: the route publishes the whole API,
which with the bypass on trusts nothing.

**Verify:** the tunnel is listed. Status is **Down**/**Inactive** until
section 6 starts the connector.

---

## 4. Alexa skill

**Console:**
[developer.amazon.com/alexa/console/ask](https://developer.amazon.com/alexa/console/ask).
**Sign in with the Amazon account the Echo is registered to.** A skill in
development is only enabled on that account's devices.

1. **Create Skill** with these settings:

| Setting | Value |
|---|---|
| Skill name | `Grocery List` |
| Primary locale | **English (AU)** |
| Experience / model | **Custom** |
| Hosting | **Provision your own** |
| Template | **Start from scratch** |

2. **Build → Invocation:** invocation name `grocery list`. Save.
3. **Build → Interaction Model → JSON Editor.** Append these four intents
   to `interactionModel.languageModel.intents` (keep the built-in
   `AMAZON.*` intents already there):

```json
{
  "name": "AddItemIntent",
  "slots": [
    { "name": "item", "type": "AMAZON.Food" },
    { "name": "quantity", "type": "AMAZON.NUMBER" }
  ],
  "samples": [
    "add {quantity} {item} to the shopping list",
    "add {item} to the shopping list",
    "put {item} on the shopping list",
    "add {quantity} {item}",
    "add {item}"
  ]
},
{
  "name": "RemoveItemIntent",
  "slots": [
    { "name": "item", "type": "AMAZON.Food" },
    { "name": "quantity", "type": "AMAZON.NUMBER" }
  ],
  "samples": [
    "remove {item}",
    "remove {item} from the shopping list",
    "remove {quantity} {item}",
    "remove {quantity} {item} from the shopping list",
    "take {item} off the shopping list",
    "take {quantity} {item} off the shopping list",
    "delete {item} from the shopping list"
  ]
},
{
  "name": "ReduceItemIntent",
  "slots": [
    { "name": "item", "type": "AMAZON.Food" },
    { "name": "quantity", "type": "AMAZON.NUMBER" }
  ],
  "samples": [
    "reduce {item}",
    "reduce {item} by {quantity}",
    "lower {item} by {quantity}",
    "decrease {item} by {quantity}"
  ]
},
{
  "name": "UndoIntent",
  "slots": [],
  "samples": [
    "undo",
    "undo that",
    "undo the last change",
    "put it back"
  ]
}
```

   Intent semantics (implemented in `sidecars/alexa-bridge` and
   `backend/src/services/voice_changes/`):
   - `RemoveItemIntent` without `quantity` deletes the item; with `quantity`
     it decrements by that amount.
   - `ReduceItemIntent` decrements by `quantity`, default 1.
   - A decrement to ≤ 0 deletes the item.
   - `UndoIntent` reverts the newest remove/reduce from the last 30 minutes.
     Repeat it to step further back.

   Spec: [`features/FEATURE_VOICE.md`](features/FEATURE_VOICE.md#removing-reducing-and-undo-by-voice).

4. **Save**, then **Build skill** and wait for the model build to succeed.
5. **Build → Endpoint:** **HTTPS**; Default Region = the skill endpoint from
   section 3; SSL certificate type = **My development endpoint is a
   sub-domain of a domain that has a wildcard certificate from a certificate
   authority**. **Save Endpoints**.
6. Copy the Skill ID (`amzn1.ask.skill.…`, shown under **View Skill ID**)
   into `ALEXA_SKILL_ID`. The sidecar rejects requests for any other skill ID.
7. **Test** tab: set skill testing to **Development**.

**Verify:** endpoint saved, model build succeeded, `ALEXA_SKILL_ID` set.

---

## 5. Intake triage model

**File:** `.env`. Every intake item is classified by an LLM ("would a
supermarket sell this?") before it reaches Pending Requests; low-confidence
or non-grocery items are held on the Pending page's Held tab. Spec:
[`features/FEATURE_TRIAGE.md`](features/FEATURE_TRIAGE.md). The local
providers run as Compose profiles, so item names never leave the server.

| Provider | Use when | `.env` |
|---|---|---|
| **Ollama** (default) | Any host | `INTAKE_LLM_PROVIDER=ollama`<br>`COMPOSE_PROFILES=ollama,db`<br>`INTAKE_LLM_BASE_URL=http://ollama:11434/v1` |
| **llama.cpp** | Low-spec host | `INTAKE_LLM_PROVIDER=llamacpp`<br>`COMPOSE_PROFILES=llamacpp,db`<br>`INTAKE_LLM_BASE_URL=http://llamacpp:8080/v1` |
| **vLLM** | NVIDIA GPU host (needs the driver and NVIDIA Container Toolkit) | `INTAKE_LLM_PROVIDER=vllm`<br>`COMPOSE_PROFILES=vllm,db`<br>`INTAKE_LLM_BASE_URL=http://vllm:8000/v1` |
| **Off** | No triage; everything goes straight to Pending Requests | `INTAKE_LLM_PROVIDER=off` |

`db` starts the Docker PostgreSQL; drop it when PostgreSQL is installed on
the server ([see below](#optional-postgresql-installed-on-the-server)).

- **OpenAI** (hosted, paid): `INTAKE_LLM_PROVIDER=openai`, no model server
  in `COMPOSE_PROFILES` (only `db`, if used), empty `INTAKE_LLM_BASE_URL`,
  and an API key from
  [platform.openai.com/api-keys](https://platform.openai.com/api-keys) in
  `OPENAI_API_KEY`.
- **A model server already running on the host:** leave the model server
  out of `COMPOSE_PROFILES` and set `INTAKE_LLM_BASE_URL=http://host.docker.internal:<port>/v1`
  (Ollama's default port is `11434`).

---

## 6. Start the stack

Keep `DEV_AUTH_BYPASS=false`. The web app signs people in with Clerk
(section 2); the bypass is for local development and e2e only.

```bash
make dev
```

With the Ollama profile, pull the model once (~2 GB). llama.cpp and vLLM
download theirs on first start.

```bash
make hm-compose args='exec ollama ollama pull llama3.2'
```

The frontend is included in `dev`. Keep it attached to its terminal and use a
second terminal for service commands. Open the web URL printed after readiness.
Set `WEB_PORT=3000` and `BACKEND_PORT=8000` in the selected configuration to keep
fixed URLs, or use the worktree defaults. Any Google OAuth redirect must match
the selected web port.

**Verify:** the tunnel reports **Healthy** in Cloudflare, and the printed web
address redirects to `/sign-in`. The first
time, use **Sign up** (`/sign-up`) with an allowlisted address; after that,
sign in with **Continue with Google** or email and password. The Grocery
List loads with your avatar top right.

**Troubleshooting:**
- Tunnel not Healthy → check `CLOUDFLARE_TUNNEL_TOKEN` for stray whitespace,
  then `make stop-all && make dev`.
- No sign-in page, and a **Sign-in is off** banner → `VITE_CLERK_PUBLISHABLE_KEY`
  is empty; set it and restart `make dev`.
- "Not allowed to sign up" → add the address to the Clerk allowlist
  (section 2, step 3).
- Anything else → `make hm-compose args='logs <service>'`.

---

## 7. Smoke-test the skill

On the Echo, then in the web app:

| Utterance | Expected |
|---|---|
| "Alexa, ask grocery list to add milk." | A `milk` card in **Pending Requests**. **Accept** it; it moves to the **Grocery List**. |
| "Alexa, ask grocery list to remove milk." | Alexa confirms; `milk` is gone from the Grocery List (reload the page). |
| "Undo" (in the same session) or "Alexa, ask grocery list to undo." | `milk` is back with its original quantity, note and filter terms. |

Removes and reduces apply immediately, unlike adds, which always wait in
Pending Requests. The rationale is in
[`features/FEATURE_VOICE.md`](features/FEATURE_VOICE.md#removing-reducing-and-undo-by-voice).

**Troubleshooting:**
- No card in Pending Requests → check **Triage**; a "could not be reached"
  reason means the model is not running (sections 5–6).
- "I couldn't find the skill" → skill testing is not set to Development
  (section 4, step 7).
- Remove/undo intents are not recognised → the interaction model was not
  rebuilt after adding them (section 4, steps 3–4).
- Nothing arrives at all → `make hm-compose args='logs alexa-bridge'`; a `403`
  means `ALEXA_SKILL_ID` does not match the skill.

---

## 8. Woolworths bookmarklet

**Browser:** Chrome on the machine you shop from. The trolley is filled by a
bookmarklet that runs on woolworths.com.au and calls the backend's store-tab
API (`docs/features/FEATURE_TROLLEY_HANDOFF.md`).

1. Show the bookmarks bar (**Ctrl/Cmd+Shift+B**).
2. In the web app, choose a Woolworths product for at least one item
   (**Compare prices → Choose**), then press **Send to Woolworths**.
3. Drag **Fill Woolworths trolley** onto the bookmarks bar.
4. In a new tab, sign in at [woolworths.com.au](https://www.woolworths.com.au)
   and set a delivery address if prompted (**Set your delivery address**).

On first use Chrome asks to allow woolworths.com.au to access devices on the
local network (Private Network Access). Allow it; the bookmarklet calls the
backend on the LAN.

**Verify:** the bookmarklet is installed and the Woolworths header shows
your delivery address.

---

## 9. Coles bookmarklet

**Browser:** Chrome, as in §8 (bookmarks bar already shown). Same handoff,
run on coles.com.au. It does not reserve a delivery window; pick one on
Coles after the fill.

1. In the web app, choose a Coles product for at least one item, then press
   **Send to Coles**.
2. Drag **Fill Coles trolley** onto the bookmarks bar.
3. In a new tab, sign in at [coles.com.au](https://www.coles.com.au) (Chrome
   autofills a saved login).
4. Choose **Delivery** to your address if the header asks for a location
   (it may read **Set your location**). The bookmarklet needs a selected
   store and refuses to start without one.

First run: allow Chrome's local-network access prompt for coles.com.au.

**Verify:** the bookmarklet is installed and the Coles header shows your
delivery address.

---

## Optional: PostgreSQL installed on the server

Run the app against a PostgreSQL installed on the home server (bare metal)
instead of the `db` container. Do this before the first `make dev`
(section 6). Developer machines keep the default: `db` in
`COMPOSE_PROFILES` starts PostgreSQL in Docker.

The backend reaches the host through `host.docker.internal`, which
`docker-compose.yml` maps to the host gateway (`extra_hosts`). Container
traffic arrives from a Docker bridge network (inside `172.16.0.0/12`), not
from `localhost`. Migrations run when the backend starts, so the database
only needs to exist. Use PostgreSQL 16 or later, the version the `db`
container runs.

1. **Home server shell:** create the role and database. Put its password in
   the application's `DATABASE_URL`, percent-encoded as required for a URL.

   ```bash
   sudo -u postgres createuser --pwprompt grocery
   sudo -u postgres createdb --owner grocery grocery
   ```

   The application role needs no `CREATEDB` privilege. Tests use a separate
   disposable Docker server and never connect to this application database.

2. **`postgresql.conf`** (`sudo -u postgres psql -c 'SHOW config_file'`):
   listen on the Docker bridge as well as loopback. `172.17.0.1` is the
   `docker0` address that `host-gateway` resolves to; check it with
   `ip -4 addr show docker0`.

   ```
   listen_addresses = 'localhost,172.17.0.1'
   ```

3. **`pg_hba.conf`** (`SHOW hba_file`): allow password logins from Docker
   networks only.

   ```
   host  grocery  grocery  172.16.0.0/12  scram-sha-256
   ```

   Host-side tools (`pg_restore` below) connect over
   `localhost`, which the stock `host all all 127.0.0.1/32 scram-sha-256`
   line already allows.

4. **Home server shell:** `sudo systemctl restart postgresql`. With a host
   firewall (ufw, firewalld), allow `5432/tcp` from `172.16.0.0/12`.

5. **`.env`:** point the backend at the host. Startup removes `db` from the
   active profiles for external PostgreSQL, even in an older configuration.
   Percent-encode any `@ : / # ?` in the password.

| `.env` key | Value |
|---|---|
| `COMPOSE_PROFILES` | Without `db`, e.g. `ollama` |
| `DATABASE_URL` | `postgres://grocery:<POSTGRES_PASSWORD>@host.docker.internal:5432/grocery` |

**Verify:** `make dev` starts no `db` container (`make hm-compose args='ps'`), and
`make hm-compose args='logs backend'` shows the migrations applied with no
connection errors.

**Moving data from the `db` container:** before step 5, with the old stack
still up:

```bash
make hm-compose args='exec -T db pg_dump -U grocery -Fc grocery' > grocery.dump
pg_restore -h localhost -U grocery -d grocery --no-owner grocery.dump
```

| Symptom | Fix |
|---|---|
| Backend logs `connection refused` | `listen_addresses` misses `172.17.0.1`, or the firewall blocks 5432 from Docker. |
| Backend logs `no pg_hba.conf entry` | Add the step 3 line, then restart PostgreSQL. |
| Backend logs `password authentication failed` | The password in `DATABASE_URL` differs from the role's, or has an unencoded special character. |
| `make dev` stops with "COMPOSE_PROFILES does not include db" | `DATABASE_URL` still names `db`; finish step 5. On a developer machine, add `db` instead (a `.env` made before the `db` profile existed lacks it). |
| A `db` container still starts | `db` is still in `COMPOSE_PROFILES`, in `.env` or exported in the shell (the shell wins). |

---

## Optional: web app on your own computer

Run the web app on your own computer while the backend and PostgreSQL stay
on the home server. The browser calls the backend at `VITE_API_BASE_URL`,
so the backend gets its own public hostname on the existing tunnel. Spec:
[`features/FEATURE_REMOTE_BACKEND.md`](features/FEATURE_REMOTE_BACKEND.md).

**Security:** this publishes the whole API. It is safe only with sign-in on:
on the server, `DEV_AUTH_BYPASS=false` and `CLERK_JWKS_URL` set (section 2).
With the bypass on, anyone who finds the hostname can read and change the
list.

### Cloudflare dashboard (dash.cloudflare.com)

In **Networking → Tunnels → grocery-list → Routes → Add route → Published
application**, add a second hostname. Leave **Path** empty so `/api/*` and
the `/ws` WebSocket both reach the backend:

| Field | Value |
|---|---|
| Subdomain | `grocery-api` |
| Domain | your domain |
| Path | *(empty)* |
| Service URL | `http://localhost:8000` (systemd, [`production.md`](production.md)) or `http://backend:8000` (Compose) |

Cloudflare passes WebSockets through without extra settings.

### Home server `.env`

| Key | Value |
|---|---|
| `DEV_AUTH_BYPASS` | `false` |
| `CORS_ORIGINS` | must include the web app's origin, `http://localhost:3000` by default. Comma-separate more than one. |

Restart the backend (`sudo systemctl restart grocery-backend`, or
`make stop-all && make dev` under Compose).

### Your computer

Needs Node 22+, `pnpm`, `git`, `make` and a checkout of this repository. No
Docker and no database. On macOS, with Homebrew ([brew.sh](https://brew.sh)):

```bash
xcode-select --install       # git and make
brew install node pnpm       # Node 22 or newer
```

Create `.env` in the checkout root with only the web app's keys:

| Key | Value |
|---|---|
| `VITE_API_BASE_URL` | `https://grocery-api.<your-domain>` |
| `VITE_WS_URL` | empty (follows `VITE_API_BASE_URL` as `wss://…/ws`) |
| `VITE_CLERK_PUBLISHABLE_KEY`, `CLERK_PUBLISHABLE_KEY` | the publishable key (section 2) |
| `CLERK_SECRET_KEY` | the secret key; the web app's own server reads the sign-in cookie with it |

Clerk development keys (`pk_test_…`) work on `localhost`. Production keys
(`pk_live_…`) work only on the domain set in Clerk, so use the development
instance's keys here.

Then run `make hm-dev-frontend` and open [localhost:3000](http://localhost:3000).
`WEB_PORT=3001 make hm-dev-frontend` picks another port; add that origin to
`CORS_ORIGINS` on the server.

To give it a Dock icon and its own window on macOS:

- **Safari** (macOS 14+): **File → Add to Dock**.
- **Chrome**: **⋮ → Cast, save and share → Install page as app**.

The icon only opens the page; `make web` must be running for it to load.

**Verify:** `curl https://grocery-api.<your-domain>/api/health` answers, and
after signing in the Pending Requests page loads without errors in the
browser console.

| Symptom | Fix |
|---|---|
| Console: `blocked by CORS policy` | The origin in the address bar is missing from the server's `CORS_ORIGINS`, or the backend was not restarted. |
| Every request is `401` | The server's `CLERK_JWKS_URL` is from a different Clerk instance than the keys on your computer. |
| Clerk error about the domain or key | A `pk_live_…` key on `localhost`; use the development instance's keys. |
| Console: `VITE_API_BASE_URL must start with http:// or https://` | Add the scheme to the value, then restart `make hm-dev-frontend`. |
| Requests still go to `localhost:8000` | `VITE_*` values are read at start: restart `make hm-dev-frontend`. |

---

## Optional: Google Tasks

Typed items in a Google Tasks list become intake requests: the backend polls
the list, records each open task, then deletes it. Spec:
[`features/FEATURE_GOOGLE_TASKS.md`](features/FEATURE_GOOGLE_TASKS.md).

- "Hey Google, add milk to my shopping list" does **not** reach Google Tasks;
  Google keeps that list to itself, with no API. This channel is for typed
  items only.
- Use a list for groceries only: **every** open task on it is consumed.
- The OAuth redirect is `http://localhost:3000/settings/help`, so connect
  from a browser on the home server.

1. **Google Tasks** (phone app or [tasks.google.com](https://tasks.google.com)):
   create a list named `Groceries`.
2. **[console.cloud.google.com](https://console.cloud.google.com)**, signed
   in as the account that owns the list:
   1. Create a project `Grocery list` and select it.
   2. **APIs & Services → Library → Google Tasks API → Enable.**
   3. **Google Auth Platform → Get started:** app name `Grocery list`,
      audience **External**, your email as contact. Then **Audience →
      Publish app** (in *Testing*, refresh tokens expire after 7 days; the
      unverified-app warning is fine for a household).
   4. **Clients → Create client:** type **Web application**, authorized
      redirect URI `http://localhost:3000/settings/help`.
3. **`.env`:** copy the new client's values:

| Console field | `.env` key |
|---|---|
| Client ID (`….apps.googleusercontent.com`) | `GOOGLE_CLIENT_ID` |
| Client secret (`GOCSPX-…`) | `GOOGLE_CLIENT_SECRET` |

4. **Home server shell:** `make dev` to restart with the new values.
5. **Web app on the home server** ([localhost:3000](http://localhost:3000)):
   **Intake → Connect Google Tasks**, pick the account, accept the
   unverified-app warning (**Advanced → Go to Grocery list**) and allow Tasks
   access. Back in the app, set **Grocery list** to `Groceries`, turn on
   **Check this list**, **Save**.

**Verify:** the Google Tasks card shows **On**. Add `milk` to the list, press
**Check now**; it appears in **Pending Requests** and leaves Google Tasks.

| Symptom | Fix |
|---|---|
| Card says **Not set up yet** | `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` missing in `.env`; then `make dev`. |
| Google says `redirect_uri_mismatch` | The client's redirect URI must be exactly `http://localhost:3000/settings/help`. |
| Card says "connect Google Tasks again" | **Disconnect**, then repeat step 5. |

---

## Optional: GitHub release protections

Configure these repository rulesets before enabling unattended release updates.
The workflow and installer do not create GitHub rulesets. Confirm they are
**Active**, rather than Evaluate or Disabled; this guide does not establish
that the protections are configured.

1. In **Settings → Rules → Rulesets**, create an Active branch ruleset for
   `main`. Require pull requests and required status checks, require the branch
   to be up to date before merging, and block force pushes and deletion.
   After the native-service workflow has run, select its actual check names:
   `build`, `installation (file)`, `installation (env)`,
   `installation (op-file)` and `installation (op-env)`. Set GitHub Actions
   as their expected source. The publication job is not a required PR check
   because it runs only for production tags.
2. Create an Active tag ruleset targeting `release-*` that restricts tag
   creation. Allow bypass only to trusted maintainers responsible for
   creating production releases.
3. Create a separate Active tag ruleset for the same `release-*` pattern
   that restricts updates and deletion, with no bypass actors. Keeping this
   separate ensures creation bypass cannot rewrite or delete an existing tag.

Review effective rules, required-check sources and bypass lists after setup.
Main must pass required checks before merging; release tags must point to main
commits. Create a new version tag for every release; never move an existing
tag. For the current GitHub controls, see
[creating repository rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/creating-rulesets-for-a-repository)
and [available rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets).

---

## Optional: generic intake webhook

For Home Assistant, IFTTT or `curl`. Not needed if Alexa works.

1. Add a second tunnel route: path `api/voice-requests`, service
   `http://backend:8000`.
2. Configure the client:

| Setting | Value |
|---|---|
| URL | `https://grocery.<your-domain>/api/voice-requests` |
| Method | `POST` |
| `Content-Type` | `application/json` |
| `X-Webhook-Secret` | value of `VOICE_WEBHOOK_SECRET` |
| Body | `{"item": "milk", "quantity": 1}` (IFTTT: `{"item": "{{TextField}}", "quantity": 1}`) |

**Verify:** the item appears in **Pending Requests**.

---

## Desktop client through Tailscale or Cloudflare

The desktop client loads the server's frontend from one origin. It uses that
frontend's API configuration. It does not start an API or database locally.

### On the server

1. Choose how this computer reaches the frontend and API. For Tailscale, connect
   both computers to the tailnet and route each required service with Tailscale
   Serve or an existing reverse proxy. The systemd services listen on loopback,
   so the server's Tailscale IP alone does not publish them. For Cloudflare,
   configure frontend and API HTTPS hostnames in the tunnel. Keep API routes and
   `/ws` pointed at the backend.
2. Set `VITE_API_BASE_URL` to the API origin reachable from the desktop computer.
   If every frontend origin proxies `/api`, `/api/*`, and `/ws` to the backend,
   use `/` instead so browser traffic follows the current frontend origin.
   Leave `VITE_WS_URL` blank to derive its socket URL, or provide the explicit
   `wss://` endpoint. Use HTTPS for the API when filling store trolleys because
   store pages use HTTPS and block insecure remote fetches. Do not use `localhost` for the browser API unless the API
   actually runs on the desktop computer. Use HTTPS for both hostnames.
3. Set `SERVER_API_BASE_URL=http://127.0.0.1:8000` for the loopback backend used
   by systemd. Add the exact frontend origin to `CORS_ORIGINS` while retaining
   existing required origins. Keep Clerk authentication enabled.
4. Rebuild the frontend after changing `VITE_*` values. Restart the frontend
   after changing its SSR configuration. Production Environment changes require
   the owner's approval before applying them.

If Cloudflare Access protects the API hostname, allow the application's existing
browser/store authentication flow through it. The store window's fetches cannot
complete an interactive Access login challenge. Never embed an Access service
token, database credential or Clerk secret in the desktop app.

### On the desktop computer

1. Install Rust and the platform's Tauri build prerequisites, then run `make setup`.
   macOS needs Xcode Command Line Tools. Linux needs WebKitGTK 4.1, GTK 3,
   AppIndicator and librsvg development libraries. Windows needs Microsoft C++
   Build Tools and WebView2.
2. Optionally set `GROCERY_DESKTOP_SERVER_URL` in the development Environment.
   This is the frontend origin, such as `https://groceries.example.com`, rather
   than the API origin. Leave it blank to choose on first run. An installed build
   may embed a public default, and a saved address always takes priority.
3. Run `make hm-dev-desktop` during development, or `make build-desktop` to build
   the platform's installer. On macOS an app-only build is
   `cd desktop && pnpm build --bundles app`. The output is under
   `desktop/src-tauri/target/release/bundle/`. Review unsigned-app prompts through
   the operating system before opening the app.
4. Connect Tailscale first when using a private address. Open the app, enter the
   frontend origin if prompted, then sign in through Clerk. Do not enter a URL
   containing login credentials, a path or a query.
5. Use the tray's **Change server address** to replace the origin. The app
   restarts to replace its command permission. **Reload server** retries the
   current page after reconnecting a tunnel. Development builds have separate
   settings, cookies and instance locks for each worktree.

The configuration and private Service routing guidance are in
[production operations](production.md#private-ingress-with-tailscale).
Keep deployment-specific hostnames and Environment selections in your private
configuration.

Routing references: [Tailscale Serve](https://tailscale.com/docs/features/tailscale-serve)
and [Cloudflare Tunnel protocols](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/routing-to-tunnel/protocols/).
