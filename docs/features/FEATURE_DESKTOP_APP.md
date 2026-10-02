# Feature: Desktop App (Tauri)

Status: **first part built** — the app, its window on the server, the
Woolworths window that keeps you logged in, "Fill trolley in the app",
"Open checkout", and desktop notifications. **Planned:** one tap to place the
order, Coles, start at login. Code: `desktop/`. Steps for people:
`docs/human-setup.md` (part "Desktop app") and `docs/using-the-app.md`
("Do a Woolworths shop in the desktop app").

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
   The app opens the store window at the trolley page and runs the program
   once the page has loaded — only if the page is on the store's own
   address. The program reports to the backend as the bookmark does. The web
   app's sheet re-reads the handoff and notifies when it is filled.
4. The store window keeps the webview's own cookies (WKWebView on macOS,
   WebView2 on Windows), which is how the login is kept. The app never reads
   them.

## Commands (main window, server origin only)

| Command | Args | Does |
|---|---|---|
| `desktop_abilities` | — | `{version, stores: [{store, fill_trolley, checkout}]}` |
| `open_store` | `{store}` | shows the store window at the store's home page (to log in) |
| `fill_store_trolley` | `{store, program}` | shows the store window at the trolley page and runs `program` there once loaded |
| `open_store_checkout` | `{store}` | shows the store window at the checkout page |
| `notify` | `{title, body}` | a desktop notification |

Setup page only: `save_server_url {url}` → `200` or an error sentence;
refuses anything that is not `http(s)://host[:port]`.

## Code
- `desktop/src-tauri/src/`: `lib.rs` (wiring), `settings.rs` (server
  address), `server_access.rs` (capability for the server origin),
  `stores.rs` (store addresses and abilities), `pending_program.rs` (program
  waiting for the store page), `windows/` (`main_window.rs`,
  `store_window.rs`), `commands/` (`desktop.rs`, `store.rs`, `setup.rs`),
  `notify.rs`, `tray.rs`.
- `desktop/setup/` — the first-run page.
- Web app: `lib/desktop/` (`bridge.ts`, `commands.ts`), `hooks/useDesktop.ts`,
  `hooks/useDesktopNotifications.ts`, `hooks/useHandoffNotice.ts`,
  `components/trolley/` (`bookmark-steps.tsx`, `desktop-steps.tsx`).

## Tests
- `cargo test` in `desktop/src-tauri`: settings, server origin capability,
  store addresses, pending program.
- Vitest: `lib/desktop/__tests__/`, `hooks/__tests__/useDesktopNotifications.test.tsx`,
  `components/trolley/__tests__/desktop-steps.test.tsx`.
