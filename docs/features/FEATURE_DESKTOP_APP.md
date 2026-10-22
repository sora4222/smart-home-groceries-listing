# Feature: desktop app

Status: **first part built** — the app, its window on the server, the
Woolworths window that keeps you logged in, "Fill trolley in the app",
"Open checkout", and desktop notifications. **Planned:** one tap to place the
order, Coles, start at login. Code: `desktop/`. Steps for people:
`docs/human-setup.md` (part "Desktop app") and `docs/using-the-app.md`
("Desktop connection and shopping").

## Why it exists
The server cannot log in to Woolworths or Coles: passkeys, MFA and Akamai bot
protection stop it (`docs/FEAT_WOOLWORTHS_ACCESS.md`). Today a person logs in
in their own browser and presses a bookmark. The desktop app runs on the
household's Mac or Windows computer, from the home internet, in a real
browser engine, and keeps its own store login. So it can do the store steps
by itself and tell the person when they are done.

## What stays on the server
The voice assistants (Alexa bridge, webhook), PostgreSQL, the backend and the
web app all stay on the home server. The desktop app talks only to:
1. **the server** — it shows the server's own web app;
2. **woolworths.com.au** — in its own store window (Coles later).

It holds no database and no grocery rules. It never stores a store password
or card details.

## Goal features

| # | Goal | Status |
|---|---|---|
| 1 | Shows the server's web app — the same screens, no second copy | built |
| 2 | First run asks for the server address; can be changed from the tray menu | built |
| 3 | Store window with its own saved login (log in once, stays logged in until the store logs you out) | built (Woolworths) |
| 4 | **Fill trolley in the app**: no bookmark — reserve the delivery time and add the products in the store window | built (Woolworths) |
| 5 | Desktop notifications: new item to review, item held for review, trolley filled (with how many not added) | built |
| 6 | Keeps running in the menu bar / system tray when the window is closed, so notifications still arrive | built |
| 7 | **Open checkout**: shows the store's checkout page after the fill, to check and pay | built (Woolworths) |
| 8 | **One tap to buy**: the web app shows items, delivery time and total; the person taps **Place order**; the app places it with the card already saved on the store account | planned — needs the checkout calls captured from the live site |
| 9 | Coles: login window, trolley fill, checkout | planned — needs the Coles trolley calls captured |
| 10 | Start at login (optional setting) | planned |
| 11 | Server pushes "trolley filled" over the WebSocket, so the notice arrives even when the sheet is closed | planned |

### Rules for goal 8 (one tap to buy)
- Never automatic. Only after the person taps **Place order** in the app.
- Shows the total, delivery time and fee from the store's own checkout
  answer, not from our prices.
- Uses the card already saved on the store account. Never types or stores a
  card number or CVV. If the store or bank asks for a code, the store window
  is shown and the person finishes there.
- One order per tap. A second tap on the same handoff is refused.
- Nothing suggests, recommends or adds an item (project goal).

## How the web app changes inside the desktop app
The web app checks `window.__TAURI_INTERNALS__` (`frontend/src/lib/desktop/`).
Outside the desktop app nothing changes.

| Place | In a browser | In the desktop app |
|---|---|---|
| Send to Woolworths sheet | drag bookmark, open Woolworths, press bookmark | pick delivery time, press **Fill trolley in the app**; **Log in to Woolworths** opens the store window |
| After the fill | "pay on Woolworths" | **Open checkout** shows the store window at checkout |
| New item / held item | in-page toast | toast **and** a desktop notification |
| Trolley filled | sheet shows the result | sheet shows it **and** a desktop notification |

Every WebSocket connection starts with a `counts_snapshot` for pending and held
items. Both toast and desktop listeners use it as a silent baseline, including
after reconnects, so the first subsequent increase produces a notice. Existing
items on connection never produce notices. Nav badges also use the snapshot;
a delayed initial HTTP response cannot overwrite a pushed count.

The main webview disables background throttling so macOS 14+ keeps its
WebSocket and notification listeners active while the window is hidden in the
tray. Tauri does not implement this setting on Windows or Linux; those
platforms still need live background-delivery verification.

## How it works
1. The main window (`main`) loads the server address saved in
   `settings.json` in the app's config folder. With none saved it shows the
   bundled setup page (`desktop/setup/`).
2. At start-up (and after a new address is saved) the app adds a capability
   for **that origin only**, letting the main window call the app's own
   commands. No plugin command is opened to the server page.
3. **Fill trolley:** the web app creates the handoff as usual
   (`POST /api/trolley-handoffs`), builds the same fill program the bookmark
   carries (`lib/store-tab/bookmarklet.ts`), and calls `fill_store_trolley`.
   One operation per store owns the fill (`fill_operation.rs`): it waits for
   the trolley page (a login page on the way does not use it up), starts the
   program once there — only on the store's own address — and then runs until
   the program reports. A second fill, **Log in** or **Open checkout** for the
   same store is refused while the program runs, so nothing moves the window
   mid-fill. The program keeps its progress in `window.__groceryFill`; the
   store page gets no app commands, so the app reads that value
   (`fill_watch.rs`, `fill_status.rs`). The command resolves with the
   program's message, or rejects with a sentence when the program fails, the
   page reloads, the window closes, or 15 minutes pass. Any of these ends the
   operation, so no program is left waiting. The program also reports to the
   backend as the bookmark does, and the sheet re-reads the handoff.
4. The store window keeps the webview's own cookies (WKWebView on macOS,
   WebView2 on Windows), which is how the login is kept. The app never reads
   them.

## Commands (main window, server origin only)

| Command | Args | Does |
|---|---|---|
| `desktop_abilities` | — | `{version, stores: [{store, fill_trolley, checkout}]}` |
| `open_store` | `{store}` | shows the store window at the store's home page (to log in); refused while a fill runs there |
| `fill_store_trolley` | `{store, program}` | shows the store window at the trolley page, runs `program` there once loaded, and answers `{message}` when it finishes; refused while a fill runs at that store |
| `open_store_checkout` | `{store}` | shows the store window at the checkout page; refused while a fill runs there |
| `notify` | `{title, body}` | a desktop notification |

Setup page only: `save_server_url {url}` → `200` or an error sentence;
refuses anything that is not `http(s)://host[:port]`.

## Code
- `desktop/src-tauri/src/`: `lib.rs` (wiring), `settings.rs` (server
  address), `server_access.rs` (capability for the server origin),
  `stores.rs` (store addresses, abilities and which page is the trolley),
  `fill_operation.rs` (one fill
  per store, waiting to finished), `fill_watch.rs` and `fill_status.rs`
  (reading the program's progress), `windows/` (`main_window.rs`,
  `store_window.rs`), `commands/` (`desktop.rs`, `store.rs`, `setup.rs`),
  `notify.rs`, `tray.rs`.
- `desktop/setup/` — the first-run page.
- Web app: `lib/desktop/` (`bridge.ts`, `commands.ts`, `notify.ts`: the one
  `notifyDesktop`), `hooks/useDesktop.ts`,
  `hooks/useDesktopNotifications.ts`, `hooks/useHandoffNotice.ts`,
  `components/trolley/` (`bookmark-steps.tsx`, `desktop-steps.tsx`).

## Tests
- `cargo test` in `desktop/src-tauri`: settings, server origin capability,
  store addresses, fill operations and progress, async
  store commands, tray actions.
- Vitest: `lib/desktop/__tests__/`, `lib/__tests__/desktop-fill-program.test.ts`, `hooks/__tests__/useDesktopNotifications.test.tsx`,
  `components/trolley/__tests__/desktop-steps.test.tsx`.

## Remote server contract

The app uses one frontend origin through Tailscale or Cloudflare. A saved
`settings.json` address wins over `GROCERY_DESKTOP_SERVER_URL` at runtime, then
its optional build default. Blank defaults show setup. Invalid addresses show
setup without granting remote commands. Normalize saved origins before granting
access and reject wildcard hosts, credentials, paths, queries and fragments.
The tray can reload the current page after reconnecting the network. Changing
the origin restarts the app to replace the remote capability.

The server's built `VITE_API_BASE_URL` and optional `VITE_WS_URL` must be reachable
from the desktop. `SERVER_API_BASE_URL` is private SSR configuration and can use
loopback. HTTPS API routing is required for fetches from the stores' HTTPS pages.
Tailscale Serve can publish loopback services with HTTPS inside the tailnet.
Cloudflare routing must preserve both HTTP API requests and `/ws`. Keep the
frontend origin in the backend CORS allowlist and retain Clerk authentication.
Manual routing and login steps live in `docs/human-setup.md`.

The native launcher resolves only the public desktop URL and passes an explicit
toolchain environment allowlist. It never passes database, Clerk-secret, store
or 1Password credentials. Build defaults contain no secrets. Development
identifiers derive from the real worktree path to isolate settings, cookies and
single-instance locks. Production packages retain the stable application ID.

`SendToStore` reads native abilities per store. Woolworths uses the same fill
program and delivery helpers as its bookmark. Desktop completion is reported
to the app without a checkout dialog or navigation. The checkout button remains an
explicit action. Coles retains its browser bookmark flow while native abilities
are off. Browser bookmark checkout confirmation remains intact.

Validation includes launcher isolation and credential-boundary tests, native
origin/default tests and frontend trolley tests. The macOS native workflow runs
formatting, Clippy and unit tests. Local unit tests and a built application do
not establish live Tailscale, Cloudflare, Clerk or authenticated store behavior.
