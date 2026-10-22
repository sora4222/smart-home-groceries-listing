# Home Grocery List

A grocery list for one household.

1. You say an item to Alexa, or type it into Google Tasks.
2. The item waits in the web app until a person accepts it.
3. The web app shows the price of each item at Woolworths and Coles.
4. The web app puts the products you chose into your Woolworths or Coles
   trolley. You pay on the store's website. This app never pays.

The app has three parts:

| Part | What it does | Where it runs |
|---|---|---|
| Backend | Stores the list and gets prices from Woolworths and Coles | The home server |
| Database | PostgreSQL. The backend saves everything here | The home server |
| Web app | The pages you use in a browser | The home server, or your own computer |

Google Home is not connected. Google does not allow it. See
[`docs/features/FEATURE_VOICE.md`](docs/features/FEATURE_VOICE.md).

## Which guide to read

| You want to | Read |
|---|---|
| Set up the app for the first time | [`docs/human-setup.md`](docs/human-setup.md) |
| Install the app on the home server | [`docs/production.md`](docs/production.md) |
| Use the app from your Mac | [Use the web app on a Mac](#use-the-web-app-on-a-mac) below |
| Do a shop, undo a mistake, or fix a problem | [`docs/using-the-app.md`](docs/using-the-app.md) |
| Know how one feature works | [`docs/features/`](docs/features/) |

## Use the web app on a Mac

There is no Mac app to download. You run the web app on your Mac with one
command. The web app gets the list and the prices from the backend on the
home server.

Before you start, the home server must be reachable from the internet.
[`docs/human-setup.md`, "Optional: web app on your own computer"](docs/human-setup.md#optional-web-app-on-your-own-computer)
shows how to set that up.

1. Install the tools. Open Terminal and run:

   ```bash
   xcode-select --install
   brew install node pnpm
   ```

   `brew` comes from [brew.sh](https://brew.sh). You need Node 22 or newer.

2. Download the code:

   ```bash
   git clone https://github.com/sora4222/smart-home-groceries-listing.git
   cd smart-home-groceries-listing
   ```

3. Make a file named `.env` in that folder. Put the backend address and the
   Clerk keys in it. The table in
   [`docs/human-setup.md`, "Your computer"](docs/human-setup.md#your-computer)
   lists each key and where to find its value.

4. Start the web app:

   ```bash
   make web
   ```

5. Open [http://localhost:3000](http://localhost:3000) and sign in.

6. Optional: add the web app to your Dock.
   - Safari: **File → Add to Dock**.
   - Chrome: **⋮ → Cast, save and share → Install page as app**.

   The Dock icon works only while `make web` is running. To stop the web app,
   press `Ctrl+C` in Terminal.

## Run everything on your own computer (development)

You need Docker, `make`, Node 22 and `pnpm`.

```bash
make setup-env    # makes .env and fills in the random passwords
make up           # starts the database, backend, Alexa bridge and tunnel
cd frontend && pnpm install && pnpm dev    # web app on http://localhost:3000
```

Fill in the rest of `.env` with [`docs/human-setup.md`](docs/human-setup.md).

Two `.env` settings help while you develop:

| Setting | What it does |
|---|---|
| `STORE_CLIENTS=fake` | Uses a small fake shop. The backend does not call Woolworths or Coles |
| `DEV_AUTH_BYPASS=true` | Turns sign-in off. Never use it on a server that the internet can reach |

`make down` stops everything.

## Run the tests

You need Docker running. The tests start their own database and never touch
yours.

```bash
pnpm install
cd frontend && pnpm install && pnpm exec playwright install chromium && cd ..
make test
```

`make hm-test` runs the same tests with longer output. `make check` runs the
linters.

## Folders

| Folder | What is in it |
|---|---|
| `backend/` | The backend, in Rust |
| `frontend/` | The web app, in React |
| `sidecars/alexa-bridge/` | A small Python program that passes Alexa requests to the backend |
| `deploy/` | Files that install the app on the home server |
| `docs/` | Guides and feature notes |

## Rules for changing the code

Read [`AGENTS.md`](AGENTS.md) first. `backend/`, `frontend/` and
`sidecars/alexa-bridge/` each have their own `AGENTS.md`.
