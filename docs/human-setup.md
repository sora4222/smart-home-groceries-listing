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

## Prerequisites

- Shell access to the home server, in a checkout of this repository, with
  Docker (Compose v2), `make`, `openssl`, Node 22 and `pnpm`.
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

**Verify:** the script exits printing `Done.`
**Troubleshooting:** `openssl: not found` → install OpenSSL and re-run.

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
| Publishable key (`pk_…`) | `VITE_CLERK_PUBLISHABLE_KEY` |
| Secret key (`sk_…`) | `CLERK_SECRET_KEY` |
| JWKS URL (ends `/.well-known/jwks.json`; may be under **Advanced**) | `CLERK_JWKS_URL` |

The publishable key is `VITE_`-prefixed because the web app reads it in the
browser; setting it turns sign-in on (`FEATURE_AUTH.md`).

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

**Security:** expose only the sidecar. Never route to `backend:8000`; that
would publish the whole API, which trusts the sidecar's shared secret and,
with `DEV_AUTH_BYPASS=true`, nothing else.

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

**Updating an existing skill:** if the skill already exists with only
`AddItemIntent`, do steps 3–4 only: add the three new intents, save,
rebuild. The endpoint and Skill ID do not change.

---

## 5. Intake triage model

**File:** `.env`. Every intake item is classified by an LLM ("would a
supermarket sell this?") before it reaches Pending Requests; low-confidence
or non-grocery items are held on `/triage`. Spec:
[`features/FEATURE_TRIAGE.md`](features/FEATURE_TRIAGE.md). The local
providers run as Compose profiles, so item names never leave the server.

| Provider | Use when | `.env` |
|---|---|---|
| **Ollama** (default) | Any host | `INTAKE_LLM_PROVIDER=ollama`<br>`COMPOSE_PROFILES=ollama`<br>`INTAKE_LLM_BASE_URL=http://ollama:11434/v1` |
| **llama.cpp** | Low-spec host | `INTAKE_LLM_PROVIDER=llamacpp`<br>`COMPOSE_PROFILES=llamacpp`<br>`INTAKE_LLM_BASE_URL=http://llamacpp:8080/v1` |
| **vLLM** | NVIDIA GPU host (needs the driver and NVIDIA Container Toolkit) | `INTAKE_LLM_PROVIDER=vllm`<br>`COMPOSE_PROFILES=vllm`<br>`INTAKE_LLM_BASE_URL=http://vllm:8000/v1` |
| **Off** | No triage; everything goes straight to Pending Requests | `INTAKE_LLM_PROVIDER=off` |

- **OpenAI** (hosted, paid): `INTAKE_LLM_PROVIDER=openai`, empty
  `COMPOSE_PROFILES` and `INTAKE_LLM_BASE_URL`, and an API key from
  [platform.openai.com/api-keys](https://platform.openai.com/api-keys) in
  `OPENAI_API_KEY`.
- **A model server already running on the host:** leave `COMPOSE_PROFILES`
  empty and set `INTAKE_LLM_BASE_URL=http://host.docker.internal:<port>/v1`
  (Ollama's default port is `11434`).

---

## 6. Start the stack

Keep `DEV_AUTH_BYPASS=false`. The web app signs people in with Clerk
(section 2); the bypass is for local development and e2e only.

```bash
make up
```

With the Ollama profile, pull the model once (~2 GB). llama.cpp and vLLM
download theirs on first start.

```bash
docker compose exec ollama ollama pull llama3.2
```

Start the frontend dev server (foreground):

```bash
cd frontend && pnpm install && pnpm dev
```

**Verify:** the tunnel reports **Healthy** in Cloudflare, and
[localhost:3000](http://localhost:3000) redirects to `/sign-in`. Sign in
with **Continue with Google** or email and password; the Grocery List loads
with your avatar top right.

**Troubleshooting:**
- Tunnel not Healthy → check `CLOUDFLARE_TUNNEL_TOKEN` for stray whitespace,
  then `make down && make up`.
- No sign-in page, and a **Sign-in is off** banner → `VITE_CLERK_PUBLISHABLE_KEY`
  is empty; set it and restart `pnpm dev`.
- "Not allowed to sign up" → add the address to the Clerk allowlist
  (section 2, step 3).
- Anything else → `docker compose logs <service>`.

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
- Nothing arrives at all → `docker compose logs alexa-bridge`; a `403`
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

## Optional: Google Tasks

Typed items in a Google Tasks list become intake requests: the backend polls
the list, records each open task, then deletes it. Spec:
[`features/FEATURE_GOOGLE_TASKS.md`](features/FEATURE_GOOGLE_TASKS.md).

- "Hey Google, add milk to my shopping list" does **not** reach Google Tasks;
  Google keeps that list to itself, with no API. This channel is for typed
  items only.
- Use a list for groceries only: **every** open task on it is consumed.
- The OAuth redirect is `http://localhost:3000/settings/intake`, so connect
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
      redirect URI `http://localhost:3000/settings/intake`.
3. **`.env`:** copy the new client's values:

| Console field | `.env` key |
|---|---|
| Client ID (`….apps.googleusercontent.com`) | `GOOGLE_CLIENT_ID` |
| Client secret (`GOCSPX-…`) | `GOOGLE_CLIENT_SECRET` |

4. **Home server shell:** `make up` to restart with the new values.
5. **Web app on the home server** ([localhost:3000](http://localhost:3000)):
   **Intake → Connect Google Tasks**, pick the account, accept the
   unverified-app warning (**Advanced → Go to Grocery list**) and allow Tasks
   access. Back in the app, set **Grocery list** to `Groceries`, turn on
   **Check this list**, **Save**.

**Verify:** the Google Tasks card shows **On**. Add `milk` to the list, press
**Check now**; it appears in **Pending Requests** and leaves Google Tasks.

| Symptom | Fix |
|---|---|
| Card says **Not set up yet** | `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` missing in `.env`; then `make up`. |
| Google says `redirect_uri_mismatch` | The client's redirect URI must be exactly `http://localhost:3000/settings/intake`. |
| Card says "connect Google Tasks again" | **Disconnect**, then repeat step 5. |

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

## Optional: network allowlist for Claude's cloud workspace

The agent workspace only reaches allowlisted hosts. If an agent reports a
host as blocked, add it next to `*.woolworths.media`:

| Host | Needed for |
|---|---|
| `*.woolworths.media` | Woolworths page scripts (already added) |
| `ui.shadcn.com` | `shadcn` CLI component installs |
