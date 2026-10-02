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
| [5. Item checker](#part-5--item-checker-about-5-minutes) | `.env` | 5 min |
| [6. Start the app](#part-6--start-the-app-about-10-minutes) | Home server terminal, then Cloudflare | 10 min |
| [7. Test with your Echo](#part-7--test-with-your-echo-about-5-minutes) | Echo, then the app | 5 min |
| [8. Woolworths, once](#part-8--woolworths-once-about-10-minutes) | Chrome | 10 min |
| [Optional — Google Tasks](#optional--google-tasks-about-25-minutes) | Google Tasks, Google Cloud, `.env`, the app | 25 min |

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

- [ ] 2. Open `.env` in your text editor. **Keep it open** for parts 2, 3,
  4 and 5. You paste values into it.

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

**Keep this tab open.** You check it again in part 6.

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

## Part 5 — Item checker (about 5 minutes)

**Where:** `.env`, in your text editor.
**You get:** the app's item checker, set up.
**Why:** the app asks a small AI model "would a supermarket sell this?" for
every item Alexa hears. Items that are not groceries wait on the **Triage**
page. The model runs on your home server. It is free, and item names stay at
home.

**Choose one.** Ollama is the easiest. It is already set in a new `.env`.

| Choice | Good for | Put these lines in `.env` |
|---|---|---|
| **Ollama** (recommended) | Any home server | `INTAKE_LLM_PROVIDER=ollama`<br>`COMPOSE_PROFILES=ollama`<br>`INTAKE_LLM_BASE_URL=http://ollama:11434/v1` |
| **llama.cpp** | A small or older computer | `INTAKE_LLM_PROVIDER=llamacpp`<br>`COMPOSE_PROFILES=llamacpp`<br>`INTAKE_LLM_BASE_URL=http://llamacpp:8080/v1` |
| **vLLM** | A computer with an NVIDIA graphics card | `INTAKE_LLM_PROVIDER=vllm`<br>`COMPOSE_PROFILES=vllm`<br>`INTAKE_LLM_BASE_URL=http://vllm:8000/v1` |
| **No checker** | Skipping it | `INTAKE_LLM_PROVIDER=off` |

- [ ] 1. Find the `INTAKE_LLM_PROVIDER=` line in `.env`. If it is not there,
  add the three lines from your choice at the end of `.env`.
- [ ] 2. Make the lines match your choice in the table.
- [ ] 3. Save `.env`.

**It worked if:** `.env` has the lines for your choice.

**OpenAI instead (paid, in the cloud):** set `INTAKE_LLM_PROVIDER=openai`,
`COMPOSE_PROFILES=` empty, `INTAKE_LLM_BASE_URL=` empty, and paste a key from
[platform.openai.com/api-keys](https://platform.openai.com/api-keys) after
`OPENAI_API_KEY=`.

**If you chose vLLM:** the home server needs the NVIDIA driver and the
**NVIDIA Container Toolkit** installed first.

**Already run Ollama, llama.cpp or vLLM yourself, outside Docker?** Leave
`COMPOSE_PROFILES=` empty. Set `INTAKE_LLM_BASE_URL=http://host.docker.internal:`
then the port it uses, then `/v1`. Ollama's port is `11434`.

> You can stop here.

---

## Part 6 — Start the app (about 10 minutes)

**Where:** home server terminal, then the Cloudflare tab from part 3.

**Note on signing in:** Clerk sign-in is not built into the web app yet. Until
it is, the app only works with `DEV_AUTH_BYPASS=true` in `.env`. **Only do
this while the tunnel has only the Alexa route from part 3.** It turns off
signing in for the whole app.

- [ ] 1. In the terminal, start the app:

```bash
make up
```

- [ ] 2. **Only if you chose Ollama in part 5:** download its model. It is
  about 2 GB, so it takes a few minutes:

```bash
docker compose exec ollama ollama pull llama3.2
```

  (llama.cpp and vLLM download their model by themselves the first time.
  Wait a few minutes before step 4.)

- [ ] 3. Start the web app (leave this terminal open while you use the app):

```bash
cd frontend && pnpm install && pnpm dev
```

- [ ] 4. Switch to the **Cloudflare tab** from part 3. Refresh it.

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

## Part 7 — Test with your Echo (about 5 minutes)

**Where:** next to your Echo, then the app.

- [ ] 1. Say: *"Alexa, ask grocery list to add milk."*
- [ ] 2. In the app, open **Pending Requests**.
- [ ] 3. Press **Accept** on "milk".

**It worked if:** milk is now on the **Grocery List** page.

**If it goes wrong:**
- milk is not in Pending Requests → open **Triage**. If it is there, read
  the reason on the card. "could not be reached" → check part 5, and part 6
  step 2. Then press **Accept** on milk.
- Alexa says it can't find the skill → check part 4, step 14 (testing on).
- Nothing in Pending Requests → run `docker compose logs alexa-bridge`. A
  `403` means `ALEXA_SKILL_ID` does not match the skill.

> You can stop here.

---

## Part 8 — Woolworths, once (about 10 minutes)

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

## Optional — Google Tasks (about 25 minutes)

Add groceries by typing them into a Google Tasks list. The app checks the
list, takes each item off it, and puts it in **Pending Requests**.

**Before you start, know this:**
- Saying *"Hey Google, add milk to my shopping list"* does **not** go to
  Google Tasks. Google sends that to its own shopping list, which has no way
  in for this app. Type items into the Tasks app instead.
- **Use a list just for groceries.** The app takes **every** task off the
  list you pick.
- Do step D on the **home server itself**, in its browser.
  Why: Google only sends you back to `localhost` there.

### A. Google Tasks (about 2 minutes)

**Where:** the Google Tasks app on your phone, or
[tasks.google.com](https://tasks.google.com).
**You get:** a list called **Groceries**.

- [ ] 1. Press **Create new list** (on the phone: **+ New list**).
- [ ] 2. Type `Groceries`.
- [ ] 3. Press **Done**.

### B. Google Cloud website (about 15 minutes)

**Where:** [console.cloud.google.com](https://console.cloud.google.com), in
your browser. Log in with the Google account that has the Groceries list.
**You need:** `.env` open in your text editor.
**You get:** a client ID and a client secret.

- [ ] 1. Press the project menu at the top. Press **New project**.
- [ ] 2. Type the name `Grocery list`. Press **Create**.
- [ ] 3. Make sure **Grocery list** is chosen in the project menu.
- [ ] 4. In the search bar at the top, type `Google Tasks API`. Open it.
- [ ] 5. Press **Enable**.
- [ ] 6. In the search bar, type `Google Auth Platform`. Open it.
- [ ] 7. Press **Get started**.
- [ ] 8. Type the app name `Grocery list`. Choose your email. Press **Next**.
- [ ] 9. Choose **External**. Press **Next**.
- [ ] 10. Type your email again. Press **Next**.
- [ ] 11. Tick the box to agree. Press **Continue**, then **Create**.
- [ ] 12. Press **Audience** on the left.
- [ ] 13. Press **Publish app**. Press **Confirm**.
  Why: in "Testing", Google stops the sign-in working after 7 days.
- [ ] 14. Press **Clients** on the left. Press **Create client**.
- [ ] 15. For **Application type**, choose **Web application**.
- [ ] 16. Under **Authorized redirect URIs**, press **Add URI**. Paste:

```
http://localhost:3000/settings/intake
```

- [ ] 17. Press **Create**. A box shows the client ID and secret.
- [ ] 18. In `.env`, look for `GOOGLE_CLIENT_ID=`. If it is not there, add
  these two lines at the end:

```
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
```

| Copy this | Paste it into `.env` after |
|---|---|
| **Client ID** (ends in `.apps.googleusercontent.com`) | `GOOGLE_CLIENT_ID=` |
| **Client secret** (starts with `GOCSPX-`) | `GOOGLE_CLIENT_SECRET=` |

- [ ] 19. Save `.env`.

**It worked if:** both values are in `.env`, with no spaces.

### C. Home server terminal (about 2 minutes)

- [ ] 1. Restart the app so it reads the new values:

```bash
make up
```

### D. The app, on the home server (about 5 minutes)

**Where:** a browser on the home server, at
[localhost:3000](http://localhost:3000).

- [ ] 1. Press **Intake** in the menu.
- [ ] 2. Press **Connect Google Tasks**.
- [ ] 3. Choose the Google account that has the Groceries list.
- [ ] 4. Google says **Google hasn't verified this app**. Press
  **Advanced**, then **Go to Grocery list**.
  Why: the app is yours, so nobody at Google checked it. That is expected.
- [ ] 5. Press **Continue** to allow access to your tasks.
- [ ] 6. Back in the app, open **Grocery list**. Choose **Groceries**.
- [ ] 7. Turn on **Check this list**.
- [ ] 8. Press **Save**.

**It worked if:** the Google Tasks card says **On**. Add `milk` to the
Groceries list, press **Check now**, and milk shows in **Pending Requests**.

**If it goes wrong:**
- The card says **Not set up yet** → check part B's values in `.env`, then
  part C.
- Google says **redirect_uri_mismatch** → check part B, step 16. It must
  match exactly.
- The card shows a red message → read it. "connect Google Tasks again" →
  press **Disconnect**, then do part D again.

> You can stop here.

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
