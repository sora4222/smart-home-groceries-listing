# Human setup — accounts, signups, and secrets

Everything in this repo is automatable except the steps on this page: they
need a human with an email address, a credit card (free tiers only, but
some services ask), or physical access to a phone/voice assistant. Do these
once per deployment, then `make up` handles the rest.

Checklist:
- [ ] Generate the app's own secrets (no signup — just commands)
- [ ] Choose a PostgreSQL password
- [ ] Clerk account (authentication)
- [ ] Cloudflare account + Tunnel (exposes the backend to Google Home)
- [ ] Wire up "Hey Google, add ... to the shopping list"

Fill everything into `.env` (copy `.env.example` first) as you go.

---

## 1. Generate the app's own secrets

No account needed — these just need to be random and kept secret.

```bash
# VOICE_WEBHOOK_SECRET — authenticates the Google Home webhook call
openssl rand -hex 32

# CREDENTIAL_ENCRYPTION_KEY — encrypts store account credentials at rest
openssl rand -base64 32
```

Paste each into the matching line in `.env`.

## 2. PostgreSQL password

Pick any strong password and set it as `POSTGRES_PASSWORD` in `.env`. Update
`DATABASE_URL`'s password to match (same file, same value).

## 3. Clerk (authentication)

1. Go to [clerk.com](https://clerk.com) and sign up (free tier is enough
   for a household-sized app).
2. Create a new Application. When asked which sign-in methods to enable,
   turn on **Google** (for Google social login) and **Email + password**
   — both are used per the app spec.
3. In the dashboard, go to **API Keys**. Copy:
   - **Publishable key** → `CLERK_PUBLISHABLE_KEY` in `.env`
   - **Secret key** → `CLERK_SECRET_KEY` in `.env`
4. Still in **API Keys**, find the **JWKS URL** (sometimes listed under
   "Advanced" or shown as `https://<your-instance>.clerk.accounts.dev/.well-known/jwks.json`
   — copy it exactly as Clerk shows it, the subdomain is unique to your
   instance). Set it as `CLERK_JWKS_URL` in `.env`.
5. Until this is wired into the frontend (see `FEATURE_VOICE.md`'s "known
   gaps" — Clerk's React SDK isn't installed yet), you can leave the
   frontend running without sign-in and set `DEV_AUTH_BYPASS=true` on the
   backend for local testing. **Never set that to `true` on a machine
   reachable through the Cloudflare Tunnel** — it turns off authentication
   entirely.

## 4. Cloudflare Tunnel

This is what lets Google's servers reach your home server without opening
any ports on your router.

1. Go to [dash.cloudflare.com](https://dash.cloudflare.com) and sign up
   (free tier). You'll need a domain added to Cloudflare — if you don't
   have one, Cloudflare sells them, or you can use a free subdomain from a
   provider that supports custom nameservers.
2. In the dashboard, go to **Zero Trust → Networks → Tunnels** (Zero Trust
   has its own free tier; you'll be prompted to set it up the first time).
3. **Create a tunnel** → choose **Cloudflared** as the connector. Give it a
   name (e.g. `grocery-list`).
4. On the install step, copy the **tunnel token** shown (a long string
   starting with `eyJ...`). Set it as `CLOUDFLARE_TUNNEL_TOKEN` in `.env` —
   this repo's `docker-compose.yml` already runs `cloudflared` with that
   token, so you don't need to install anything yourself.
5. Still in the tunnel's settings, go to **Public Hostnames** → **Add a
   public hostname**:
   - Hostname: something like `grocery.yourdomain.com`
   - Service: `HTTP` → `host.docker.internal:8000` (the FastAPI backend).
     If `host.docker.internal` doesn't resolve on your OS, use your home
     server's LAN IP instead, e.g. `192.168.1.50:8000`.
6. Start the stack (`make up`) and confirm the tunnel shows **Healthy** in
   the Cloudflare dashboard.
7. Your webhook URL for the next step is:
   `https://grocery.yourdomain.com/api/voice-requests`

## 5. Wiring up "Hey Google, add ... to the shopping list"

The backend already implements the webhook the spec describes
(`POST /api/voice-requests` with `{"item": ..., "quantity": ...}`, header
`X-Webhook-Secret: <your VOICE_WEBHOOK_SECRET>`). What's *not* built yet is
a published, Google-reviewed Smart Home Action — that requires an OAuth
"account linking" flow the backend doesn't implement (tracked as a known
gap in `FEATURE_VOICE.md`). Until that exists, use one of these two
options to actually get a voice command to hit the webhook:

### Option A — Home Assistant (the spec's own documented fallback, no
third party)
If you already run, or are willing to run, Home Assistant on your home
server:
1. Install the **Google Assistant** integration in Home Assistant and link
   it to your Google account (Home Assistant's docs walk through this —
   search "Home Assistant Google Assistant integration").
2. Create a Home Assistant **automation**: trigger on a voice command
   phrase or an `input_text` helper, action = **RESTful command** that
   POSTs to `https://grocery.yourdomain.com/api/voice-requests` with the
   `X-Webhook-Secret` header and a JSON body containing the item text.
3. Because Home Assistant, not Google, does the NLU here, the wording
   users say is whatever phrase you configure in Home Assistant, not
   exactly the spec's "Hey Google, add ... to the shopping list" — close
   enough for the POC; tightening this is future work.

### Option B — IFTTT (fastest to get working today)
1. Sign up at [ifttt.com](https://ifttt.com) (free tier is enough for one
   applet).
2. Create an applet: **If** "Google Assistant" → "Say a phrase with a
   number" or "Say a phrase with a text ingredient" (IFTTT's Google
   Assistant trigger lets you say things like *"Add $ to the shopping
   list"*). **Then** "Webhooks" → "Make a web request":
   - URL: `https://grocery.yourdomain.com/api/voice-requests`
   - Method: `POST`
   - Content Type: `application/json`
   - Headers: `X-Webhook-Secret: <your VOICE_WEBHOOK_SECRET>`
   - Body: `{"item": "{{TextField}}", "quantity": 1}` (map IFTTT's captured
     phrase into `item`)
3. Say the configured phrase to any Google Assistant device (a Google Home
   speaker, or the Assistant app on a phone) and check the **Pending
   Requests** page in the web app.

Either option lets you validate the whole pipeline (webhook → pending
queue → web app accept/reject → active grocery list → real-time badge and
toast) today. Swap in a real Smart Home Action later without touching the
backend's webhook contract.

## Verifying everything works end to end

Set `DEV_AUTH_BYPASS=true` in `.env` first if Clerk isn't wired into the frontend yet (see step 3.5) — otherwise the web app's requests to list/accept/reject will 401.

```bash
make up
make migrate
cd backend && uv run uvicorn app.main:app --reload --port 8000 &
cd frontend && pnpm dev &

curl -X POST http://localhost:8000/api/voice-requests \
  -H "Content-Type: application/json" \
  -H "X-Webhook-Secret: $VOICE_WEBHOOK_SECRET" \
  -d '{"item": "milk", "quantity": 2}'
```
Then open `http://localhost:3000/pending` — the "milk" request should be
there, and accepting it should move it to `http://localhost:3000/` (the
active grocery list).
