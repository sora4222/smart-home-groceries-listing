# Human setup — accounts, signups, and secrets

Everything in this repo is automatable except the steps on this page: they
need a human with an email address, a credit card (free tiers only, but some
services ask), or physical access to a phone/voice assistant. Do these once
per deployment, then `make up` handles the rest.

Checklist:
- [ ] Generate the app's own secrets (no signup — just commands)
- [ ] Choose a PostgreSQL password
- [ ] Clerk account (authentication)
- [ ] Cloudflare account + Tunnel (exposes the Alexa bridge to Amazon)
- [ ] Amazon developer account + Alexa skill
- [ ] Optionally: Home Assistant or IFTTT for the generic webhook

Fill everything into `.env` (copy `.env.example` first) as you go.

---

## 1. Generate the app's own secrets

No account needed — these just need to be random and kept secret.

```bash
# VOICE_WEBHOOK_SECRET — authenticates the generic intake webhook
openssl rand -hex 32

# ALEXA_BRIDGE_SECRET — authenticates the Alexa bridge sidecar to the backend
# Use a DIFFERENT value from VOICE_WEBHOOK_SECRET, so leaking one does not
# grant the other.
openssl rand -hex 32

# CREDENTIAL_ENCRYPTION_KEY — encrypts store account credentials at rest
# (base64-encoded 32 bytes; the backend rejects any other length)
openssl rand -base64 32
```

Paste each into the matching line in `.env`.

## 2. PostgreSQL password

Pick any strong password and set it as `POSTGRES_PASSWORD` in `.env`. Update
`DATABASE_URL`'s password to match (same file, same value).

Note the URL scheme is plain `postgres://` — `sqlx` takes no driver suffix,
unlike the `postgresql+asyncpg://` the previous Python backend needed. If you
are upgrading an existing `.env`, fix that line or the backend will not start.

## 3. Clerk (authentication)

1. Go to [clerk.com](https://clerk.com) and sign up (the free tier is enough
   for a household-sized app).
2. Create a new Application. When asked which sign-in methods to enable, turn
   on **Google** (for Google social login) and **Email + password** — both are
   used per the app spec.
3. In the dashboard, go to **API Keys**. Copy:
   - **Publishable key** → `CLERK_PUBLISHABLE_KEY` in `.env`
   - **Secret key** → `CLERK_SECRET_KEY` in `.env`
4. Still in **API Keys**, find the **JWKS URL** (sometimes listed under
   "Advanced" or shown as
   `https://<your-instance>.clerk.accounts.dev/.well-known/jwks.json` — copy
   it exactly as Clerk shows it, the subdomain is unique to your instance).
   Set it as `CLERK_JWKS_URL` in `.env`.
5. Until this is wired into the frontend (see `FEATURE_VOICE.md`'s "known
   gaps" — Clerk's React SDK isn't installed yet), you can leave the frontend
   running without sign-in and set `DEV_AUTH_BYPASS=true` on the backend for
   local testing. The backend logs a warning at startup when it is on.
   **Never set that to `true` on a machine reachable through the Cloudflare
   Tunnel** — it turns off authentication entirely.

## 4. Cloudflare Tunnel

This is what lets Amazon's servers reach your home server without opening any
ports on your router.

1. Go to [dash.cloudflare.com](https://dash.cloudflare.com) and sign up (free
   tier). You'll need a domain added to Cloudflare — if you don't have one,
   Cloudflare sells them, or you can use a free subdomain from a provider that
   supports custom nameservers.
2. In the dashboard, go to **Zero Trust → Networks → Tunnels** (Zero Trust has
   its own free tier; you'll be prompted to set it up the first time).
3. **Create a tunnel** → choose **Cloudflared** as the connector. Give it a
   name (e.g. `grocery-list`).
4. On the install step, copy the **tunnel token** shown (a long string
   starting with `eyJ...`). Set it as `CLOUDFLARE_TUNNEL_TOKEN` in `.env` —
   this repo's `docker-compose.yml` already runs `cloudflared` with that
   token, so you don't need to install anything yourself.
5. Still in the tunnel's settings, go to **Public Hostnames** → **Add a public
   hostname**:
   - Hostname: something like `grocery.yourdomain.com`
   - Path: `/alexa`
   - Service: `HTTP` → `alexa-bridge:8081`

   Point the tunnel at the **Alexa bridge**, not the backend. The backend is
   not published to the host at all — only the bridge and, if you want it, the
   webhook path are exposed. Expose the whole API and you would be publishing
   every authenticated endpoint to the internet.
6. If you also want the generic webhook (step 6 below), add a second public
   hostname entry:
   - Path: `/api/voice-requests`
   - Service: `HTTP` → `backend:8000`
7. Start the stack (`make up`) and confirm the tunnel shows **Healthy** in the
   Cloudflare dashboard.
8. Your Alexa endpoint URL for the next step is
   `https://grocery.yourdomain.com/alexa`.

## 5. Amazon developer account + Alexa skill

This is the working voice channel. (Google Home cannot be integrated directly
— see `docs/features/FEATURE_VOICE.md`.)

1. Sign up at
   [developer.amazon.com](https://developer.amazon.com/alexa/console/ask)
   with the **same Amazon account your Echo device is registered to**. A skill
   under a different account will not be available on your device.
2. **Create Skill**:
   - Name: e.g. `Grocery List`
   - Primary locale: `English (AU)` (or your own)
   - Model: **Custom**
   - Hosting: **Provision your own** — the skill runs on your home server, not
     on Lambda.
3. In **Build → Invocation**, set the skill invocation name, e.g.
   `grocery list`.
4. In **Build → Interaction Model → JSON Editor**, add the intent the bridge
   expects. Paste this into the `intents` array:

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
   }
   ```

   `AMAZON.Food` covers groceries reasonably well. If it mishears items you
   buy often, replace it with a custom slot type listing them — the household
   member can still correct any item on the confirmation card.
5. In **Build → Endpoint**, choose **HTTPS** and set:
   - Default region: `https://grocery.yourdomain.com/alexa`
   - Certificate type: **My development endpoint is a sub-domain of a domain
     that has a wildcard certificate from a certificate authority** (Cloudflare
     terminates TLS for you).
6. Copy the **Skill ID** shown at the top of the skill's page
   (`amzn1.ask.skill.<uuid>`) into `ALEXA_SKILL_ID` in `.env`. The bridge
   verifies every request against it, so a different skill pointed at your
   endpoint is rejected.
7. **Save** and **Build** the model.
8. Restart the stack so the bridge picks up the new environment
   (`make down && make up`), then say:
   *"Alexa, ask grocery list to add milk"* — or, once Amazon has enabled the
   skill on your device, *"Alexa, add milk to the shopping list"*.
9. Check the **Pending Requests** page in the web app. The item should be
   there, waiting for you to accept it.

If nothing arrives, the bridge's logs say why —
`docker compose logs alexa-bridge`. A `400`/`403` there means signature or
skill-id verification failed, which is the bridge doing its job: check
`ALEXA_SKILL_ID` matches the skill you actually spoke to.

## 6. Optional: the generic webhook

`POST /api/voice-requests` takes `{"item": ..., "quantity": ...}` with header
`X-Webhook-Secret: <your VOICE_WEBHOOK_SECRET>`. It exists for Home
Assistant, IFTTT, `curl` and tests. You do not need it if Alexa is working.

### Home Assistant
1. Install the **Google Assistant** integration in Home Assistant and link it
   to your Google account.
2. Create an automation: trigger on a voice command phrase or an `input_text`
   helper, action = **RESTful command** POSTing to
   `https://grocery.yourdomain.com/api/voice-requests` with the
   `X-Webhook-Secret` header and a JSON body containing the item text.
3. Home Assistant, not Google, does the NLU here, so the phrase users say is
   whatever you configure.

### IFTTT
1. Sign up at [ifttt.com](https://ifttt.com) (free tier is enough for one
   applet).
2. Create an applet: **If** "Google Assistant" → "Say a phrase with a text
   ingredient". **Then** "Webhooks" → "Make a web request":
   - URL: `https://grocery.yourdomain.com/api/voice-requests`
   - Method: `POST`
   - Content Type: `application/json`
   - Headers: `X-Webhook-Secret: <your VOICE_WEBHOOK_SECRET>`
   - Body: `{"item": "{{TextField}}", "quantity": 1}`

## Verifying everything works end to end

Set `DEV_AUTH_BYPASS=true` in `.env` first if Clerk isn't wired into the
frontend yet (see step 3.5) — otherwise the web app's requests to
list/accept/reject will 401.

```bash
make up                     # Postgres, backend, Alexa bridge, tunnel
cd frontend && pnpm dev &

# The backend migrates itself at startup. Confirm it is up:
curl http://localhost:8000/api/health          # if you published the port
docker compose exec backend grocery-backend --health-check && echo ok

# Simulate an intake item without speaking to anything:
curl -X POST http://localhost:8000/api/voice-requests \
  -H "Content-Type: application/json" \
  -H "X-Webhook-Secret: $VOICE_WEBHOOK_SECRET" \
  -d '{"item": "milk", "quantity": 2}'
```

Then open `http://localhost:3000/pending` — the "milk" request should be
there, and accepting it should move it to `http://localhost:3000/` (the active
grocery list). The badge and toast should update without a reload, which
confirms the WebSocket is connected.

To run the backend on the host rather than in its container (handy while
developing):

```bash
make up                                   # Postgres is enough
cd backend && cargo run
```
