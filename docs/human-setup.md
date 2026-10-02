# Set up the grocery app

Do this once. It takes about **1½ hours**. You can stop after any part.
Your progress is saved in the `.env` file.

The parts are in this order so you start the app only once, at the end.

| Part | Where | Time |
|---|---|---|
| [1. Make the settings file](#part-1--make-the-settings-file-about-5-minutes) | Home server terminal | 5 min |
| [2. Clerk](#part-2--clerk-website-about-10-minutes) | clerk.com | 10 min |
| [3. Cloudflare](#part-3--cloudflare-website-about-15-minutes) | cloudflare.com | 15 min |
| [4. Amazon Alexa](#part-4--amazon-developer-website-about-20-minutes) | developer.amazon.com | 20 min |
| [5. Start the app](#part-5--start-the-app-about-10-minutes) | Home server terminal, then Cloudflare | 10 min |
| [6. Test with your Echo](#part-6--test-with-your-echo-about-5-minutes) | Echo, then the app | 5 min |
| [7. Woolworths, once](#part-7--woolworths-once-about-10-minutes) | Chrome | 10 min |

Doing a shop after setup: [`using-the-app.md`](using-the-app.md).

## Before you start

You need:
- The home server (the computer that runs the app), with a terminal open in
  the project folder.
- An email address for new accounts.
- A domain name you own (like `yourname.com`). Cloudflare can sell you one.
- The Amazon account your Echo uses.
- Your Woolworths login (passkey or 1Password).

**Words**
- **Terminal:** the window where you type commands.
- **`.env`:** the app's settings file, in the project folder. Open it in a
  text editor (like VS Code or TextEdit).
- **Paste after `NAME=`:** put the value straight after the `=`, with no
  spaces.
- **Tab:** one page in your browser.

---

## Part 1 — Make the settings file (about 5 minutes)

**Where:** home server terminal, in the project folder.
**You get:** a `.env` file with every random password already filled in.

- [ ] 1. Type this and press Enter:

```bash
make setup-env
```

- [ ] 2. Open `.env` in your text editor. **Keep it open** for parts 2, 3
  and 4. You paste values into it.

**It worked if:** the terminal ends with `Done.`

**If it goes wrong:** `openssl: not found` → install OpenSSL, then try again.
Running it twice is safe. It never changes a value that is already set.

> You can stop here.

---

## Part 2 — Clerk website (about 10 minutes)

**Where:** [dashboard.clerk.com](https://dashboard.clerk.com), in your browser.
**You need:** `.env` open.
**You get:** 3 values for `.env`.
**Why:** Clerk handles signing in to the app.

- [ ] 1. Sign up. The free plan is enough.
- [ ] 2. Create an application.
- [ ] 3. When it asks how people sign in, turn on **Google** and **Email**
  (with password).
- [ ] 4. Open **API Keys**.
- [ ] 5. Copy each value into `.env`:

| Copy this from Clerk | Paste it into `.env` after |
|---|---|
| Publishable key | `CLERK_PUBLISHABLE_KEY=` |
| Secret key | `CLERK_SECRET_KEY=` |
| JWKS URL (it ends in `/.well-known/jwks.json`; it may be under **Advanced**) | `CLERK_JWKS_URL=` |

- [ ] 6. Save `.env`.

**It worked if:** all 3 lines in `.env` have a value.

> You can stop here.

---

## Part 3 — Cloudflare website (about 15 minutes)

**Where:** [dash.cloudflare.com](https://dash.cloudflare.com), in your browser.
**You need:** `.env` open. Your domain added to Cloudflare.
**You get:** the tunnel token, and your **Alexa address** for part 4.
**Why:** so Amazon can reach your home server without opening your router.

- [ ] 1. Sign up or log in. The free plan is enough.
- [ ] 2. If your domain is not in Cloudflare yet, press **Add a domain** and
  follow the steps.
- [ ] 3. Press **Networking**, then **Tunnels**. (Older screens: **Zero
  Trust → Networks → Tunnels**.)
- [ ] 4. Press **Create Tunnel**. Choose **Cloudflared**.
- [ ] 5. Type the name `grocery-list`. Press **Save**.
- [ ] 6. On the install page, find the long text that starts with `eyJ`.
  Copy only that text. Do **not** run the install command — the app already
  runs Cloudflare for you.
- [ ] 7. Paste it into `.env` after `CLOUDFLARE_TUNNEL_TOKEN=`. Save `.env`.
- [ ] 8. Go to the tunnel's **Routes**. Press **Add route**, then
  **Published application**. (Older screens: **Public Hostnames → Add a
  public hostname**.)
- [ ] 9. Fill it in like this, then press **Add route** (or **Save**):

| Box | Type this |
|---|---|
| Subdomain | `grocery` |
| Domain | your domain |
| Path (if there is one) | `alexa` |
| Service URL | `http://alexa-bridge:8081` |

- [ ] 10. Write down your **Alexa address**:
  `https://grocery.yourdomain.com/alexa` (use your domain).

**Warning:** only add the route above. Do not point a route at the whole app
(`backend:8000`). That would put the whole app on the internet.

**It worked if:** the tunnel is listed. It says **Down** or **Inactive** for
now. That is fine — the app is not started yet.

**Keep this tab open.** You check it again in part 5.

> You can stop here.

---

## Part 4 — Amazon developer website (about 20 minutes)

**Where:** [developer.amazon.com/alexa/console/ask](https://developer.amazon.com/alexa/console/ask), in your browser.
**You need:** `.env` open. Your **Alexa address** from part 3.
**You get:** the Skill ID for `.env`.

**Warning:** log in with the **same Amazon account your Echo uses**.
Otherwise the skill will not work on your Echo.

- [ ] 1. Press **Create Skill**.
- [ ] 2. Fill it in, then press **Create skill**:

| Box | Choose or type |
|---|---|
| Skill name | `Grocery List` |
| Primary locale | **English (AU)** |
| Type of experience / model | **Custom** |
| Hosting | **Provision your own** |

- [ ] 3. If it asks for a template, choose **Start from scratch**.
- [ ] 4. Open **Build → Invocation**. Type `grocery list` as the invocation
  name. Press **Save**.
- [ ] 5. Open **Build → Interaction Model → JSON Editor**.
- [ ] 6. Find the list called `"intents"`. Paste this inside it, after the
  last `}` in that list, with a comma before it:

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

- [ ] 7. Press **Save**, then **Build skill** (or **Build Model**). Wait for
  it to finish.
- [ ] 8. Open **Build → Endpoint**. Choose **HTTPS**.
- [ ] 9. In **Default Region**, paste your **Alexa address** from part 3.
- [ ] 10. For the certificate, choose: **My development endpoint is a
  sub-domain of a domain that has a wildcard certificate from a certificate
  authority**.
- [ ] 11. Press **Save Endpoints**.
- [ ] 12. Copy the **Skill ID** (it starts `amzn1.ask.skill.`). It is at the
  top of the skill, or under **View Skill ID**.

| Copy this | Paste it into `.env` after |
|---|---|
| Skill ID | `ALEXA_SKILL_ID=` |

- [ ] 13. Save `.env`. You can close `.env` now.
- [ ] 14. Open the **Test** tab. Set testing to **Development**. This turns
  the skill on for your Echo.

**It worked if:** the endpoint is saved and `.env` has the Skill ID.

> You can stop here.

---

## Part 5 — Start the app (about 10 minutes)

**Where:** home server terminal, then the Cloudflare tab from part 3.

**Note on signing in:** Clerk sign-in is not built into the web app yet. Until
it is, the app only works with `DEV_AUTH_BYPASS=true` in `.env`. **Only do
this while the tunnel has only the Alexa route from part 3.** It turns off
signing in for the whole app.

- [ ] 1. In the terminal, start the app:

```bash
make up
```

- [ ] 2. Start the web app (leave this terminal open while you use the app):

```bash
cd frontend && pnpm install && pnpm dev
```

- [ ] 3. Switch to the **Cloudflare tab** from part 3. Refresh it.

**It worked if:**
- the tunnel says **Healthy**, and
- [localhost:3000](http://localhost:3000) shows the Grocery List.

**If it goes wrong:**
- Tunnel not Healthy → check `CLOUDFLARE_TUNNEL_TOKEN` in `.env` has no
  spaces. Then run `make down` and `make up`.
- Something else → run `docker compose logs` and share the last lines with
  Claude.

> You can stop here.

---

## Part 6 — Test with your Echo (about 5 minutes)

**Where:** next to your Echo, then the app.

- [ ] 1. Say: *"Alexa, ask grocery list to add milk."*
- [ ] 2. In the app, open **Pending Requests**.
- [ ] 3. Press **Accept** on "milk".

**It worked if:** milk is now on the **Grocery List** page.

**If it goes wrong:**
- Alexa says it can't find the skill → check part 4, step 14 (testing on).
- Nothing in Pending Requests → run `docker compose logs alexa-bridge`. A
  `403` means `ALEXA_SKILL_ID` does not match the skill.

> You can stop here.

---

## Part 7 — Woolworths, once (about 10 minutes)

**Where:** Chrome on the computer you shop from. Do all of it in Chrome.
**Why:** the app fills your Woolworths trolley through a bookmark you press on
the Woolworths website.

- [ ] 1. Show the bookmarks bar: press **Ctrl+Shift+B** (Mac: **Cmd+Shift+B**).
- [ ] 2. Open the app's **Grocery List** page.
- [ ] 3. Press **Send to Woolworths**. (It needs at least one item with a
  Woolworths product chosen. Use **Compare prices → Choose** first.)
- [ ] 4. Drag the **Fill Woolworths trolley** button up onto the bookmarks bar.
- [ ] 5. In a new tab, open [woolworths.com.au](https://www.woolworths.com.au)
  and log in (passkey or 1Password).
- [ ] 6. Look at the top of the Woolworths page. If it says **Set your
  delivery address**, press it and choose your address.

**It worked if:** the bookmark is on the bookmarks bar, and Woolworths shows
your address at the top.

**The first time you press the bookmark,** Chrome may ask to let
woolworths.com.au reach devices on your local network. Press **Allow**.

Setup is done. To shop, follow [`using-the-app.md`](using-the-app.md).

---

## Optional — Home Assistant or IFTTT

Only if you want to add items from something other than Alexa. You do not
need this if Alexa works.

**Where:** Cloudflare tab, then Home Assistant or IFTTT.

- [ ] 1. In Cloudflare, add a second route to the tunnel (like part 3, step
  9): Path `api/voice-requests`, Service URL `http://backend:8000`.
- [ ] 2. Set up the other app to send this:

| Setting | Value |
|---|---|
| URL | `https://grocery.yourdomain.com/api/voice-requests` |
| Method | `POST` |
| Content type | `application/json` |
| Header | `X-Webhook-Secret:` then the value of `VOICE_WEBHOOK_SECRET` in `.env` |
| Body | `{"item": "milk", "quantity": 1}` (IFTTT: `{"item": "{{TextField}}", "quantity": 1}`) |

**It worked if:** the item shows in **Pending Requests**.

---

## Optional — Let Claude reach the websites it needs

Claude's workspace can only reach websites you allow. If Claude says a site
is **blocked**, add it where you added `*.woolworths.media`:

| Website | Why Claude needs it |
|---|---|
| `*.woolworths.media` | Woolworths' page scripts (added) |
| `ui.shadcn.com` | Adding new screen parts (buttons, lists) to the app |
