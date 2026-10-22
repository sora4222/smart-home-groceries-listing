# Remaining work spec

Written 2026-10-06 from the feature docs in `docs/features/`, the repo on `main` (commit `89b8fab`), and the run errors Jesse reported.

Use this as the task list for finishing the project. Each task has an id, what to do, and how to know it is done. Tasks in a group depend on the ones above them unless noted.

## 1. Where the project stands

| Area | State on `main` |
|---|---|
| Grocery list (add, edit, annotate, commit) | Done |
| Intake: webhook, Alexa (add, remove, reduce, undo) | Done. Needs a real Alexa device test |
| Intake: Google Tasks | Done, tested only with fakes. Never run against a real Google account |
| Intake: Google Keep | Paused on purpose (unofficial library, master token risk) |
| AI triage and the Triage page | Done |
| Item Rules | Done, with small gaps (section 6) |
| Store search and price compare (Woolworths, Coles) | Done in tests. Never run from the home server |
| Product choice, order review, delivery fees, order modes, split-store planner | Done |
| Dislikes | Done for manual choice. The optimiser does not use them yet |
| Purchase history and Spending page | Done for Woolworths shops only |
| Fill trolley by bookmark (Woolworths and Coles) and confirm checkout | Done. Coles calls were checked live on 2026-10-06. Full run against the home backend is still to do |
| Access logs, Clerk login | Done. Clerk checked against a real instance only by hand |
| **Desktop app (Tauri)** | **Missing from `main`.** See section 3 |
| Live delivery windows, need-by date, allowed days, perishable timing | Not built. Needs the desktop app |
| One-tap Place order | Not built. Needs the desktop app |

## 2. Make the app run (do this first)

**A1. Fix "Failed to fetch" after sign-in.**
Cause found: the page calls `http://localhost:8000` (`VITE_API_BASE_URL`) and nothing answered. The frontend dev server (`pnpm dev`, port 3000) does not start the backend. `make dev` starts the backend in Docker.
Steps:
1. Run `make dev`, then `docker compose ps` and check `backend` and `db` are healthy.
2. If `backend` is not running, read `docker compose logs backend` and fix the cause (missing `.env` value, database not ready, port 8000 taken).
3. Run `curl -i http://localhost:8000/health` and expect 200.
4. Reload `http://localhost:3000` after sign-in.
Done when: the home page shows the grocery list after sign-in, with no "Failed to fetch" in the console.

**A2. Show a friendly error when the backend is down.**
The console says no route caught the error. Add an `errorComponent` on the root route. It should say "Cannot reach the server at <address>" with a Retry button.
Done when: stopping the backend shows that message instead of a crash, and a test covers it.

**A3. Remove the CSRF warning.**
Add `createCsrfMiddleware({ filter: (ctx) => ctx.handlerType === 'serverFn' })` to `requestMiddleware` in `frontend/src/start.ts`, next to the Clerk middleware. Check sign-in still works.
Done when: the warning no longer prints on `pnpm dev`.

**A4. Run the live store search from the home server.**
The "Compare prices" call fails with the same error today because the backend is down. Once A1 is fixed, search for "milk" from the home server.
If a store answers `blocked`, record it in `FEATURE_STORE_INTEGRATION.md` and use the desktop app route (section 3) instead of cookie reuse on the server.
Done when: both stores return products from the home server, or the `blocked` result is written down.

## 3. Desktop app (Tauri)

**What it is for.** The server cannot log in to Woolworths or Coles because of passkeys, MFA and bot protection. The desktop app runs on the household's Mac or Windows computer, shows the same web app, and keeps its own store logins in a store window. It can then talk to the stores without the problems the server has. The server keeps the database, the backend and the voice channels. The app never stores a store password or card number.

**What went wrong.** The app was built on branch `feat/desktop-app` (commits `864cdb9` to `335d60c`). The merge commit `aa8ebbf` ("Merge branch 'main' ... into feat/desktop-app") dropped the whole `desktop/` folder, the web app bridge files and the Makefile targets. Git then treated the branch as merged, so `main` has no desktop app and no `make desktop`. The code is still in git history at `335d60c`.

**B1. Recover the lost code on a new branch.**
Make branch `feat/restore-desktop` from `main`. Restore these paths from `335d60c`:
- `desktop/` (Tauri project, `setup/` first-run page, `src-tauri/src/` with `commands/`, `windows/`, `settings.rs`, `server_access.rs`, `stores.rs`, `fill_operation.rs`, `fill_watch.rs`, `fill_status.rs`, `notify.rs`, `tray.rs`)
- `frontend/src/lib/desktop/`, `frontend/src/hooks/useDesktop.ts`, `useDesktopNotifications.ts`, `frontend/src/components/trolley/desktop-steps.tsx` and their tests
- `docs/features/FEATURE_DESKTOP_APP.md`, `skills/desktop.md`
- the desktop lines in `AGENTS.md`
- the desktop Makefile targets from `5662049`
`main` has moved on since (Coles trolley fill, product lookup), so merge by hand where the trolley sheet and `send-to-store` changed.
Done when: the restored tests pass (`cargo test` in `desktop/src-tauri`, the desktop Vitest files) and the rest of the suite still passes.

**B2. Add `make desktop`.**
- `make desktop` starts the app in development and opens its window.
- `make desktop-test` runs the Rust and Vitest tests.
- `make desktop-build` makes an installer for the current computer.
- Follow the `makefile` skill: short output for agents, `hm-desktop` for people.
First run shows the setup page and asks for the server address (default `http://localhost:3000`). The address can be changed from the tray menu.
Done when: on Jesse's Mac, `make desktop` opens a window showing the sign-in page of the web app.

**B3. Desktop-only screens in the web app.**
The web app checks `window.__TAURI_INTERNALS__`. Outside the app nothing changes. Inside it:
- Send to store sheet shows **Fill trolley in the app** instead of the bookmark steps.
- **Log in to Woolworths** and **Log in to Coles** open the store window.
- **Open checkout** shows the store checkout page.
- New items, held items and a filled trolley also raise a desktop notification.
Done when: each of those screens has a Vitest test, and the browser build is unchanged (a test checks the bookmark steps still show).

**B4. Coles in the desktop app.**
Store window with its own saved Coles login, trolley fill and open checkout, the same as Woolworths. The Coles trolley calls are captured. The slot reservation call `POST /api/bff/slots/{slotId}` is not. Capture its body in a real browser first.
Done when: a Coles handoff fills the trolley from the app and reserves a delivery time.

**B5. Read real delivery windows and fees through the app.**
The app has the store login, so it can read the windows the server cannot. Send them to the backend and show them on `/order` with their fees. Reserved windows last about 15 minutes, so fetch again when the hold runs out.
Done when: `/order` lists real windows from both stores with fees, and the planner uses them.

**B6. Delivery constraints that need windows.**
Build need-by date and time, allowed days, and perishable timing (items that must arrive within N hours of each other) in the planner and on `/settings/delivery`. Depends on B5.
Done when: each rule has planner tests, and the recommended option never breaks one.

**B7. Import order history through the app.**
Read past orders from Woolworths and Coles in the store window and save them as purchase history. This also fills the Coles gap, since Coles shops are not saved today.
Done when: the Spending page shows a Coles order after an import.

**B8. One tap to buy.**
The web app shows items, delivery time and total. The person taps **Place order** and the app places it with the card already saved on the store account. Rules:
- Never automatic. Only after the tap.
- Total, delivery time and fee come from the store's own checkout answer, not our prices.
- Never type or store a card number or CVV. If the store or bank asks for a code, show the store window and let the person finish there.
- One order per tap. A second tap on the same handoff is refused.
- Nothing suggests or adds an item.
Needs the checkout calls captured from each live site.
Done when: a test order on a real account places once, and a second tap is refused.

**B9. Smaller app features.**
- Server pushes "trolley filled" over the WebSocket, so the notice arrives with the sheet closed.
- Optional start at login.
Done when: both have tests and a line in `using-the-app.md`.

**B10. Package and document.**
Build installers for macOS and Windows. Write the human steps (install, first run, log in to each store) in `docs/human-setup.md` and `docs/using-the-app.md`, following the `human-instructions` skill. Update `FEATURE_DESKTOP_APP.md`, `AGENTS.md`, `README.md`.
Done when: someone who has not seen the project can install the app and do a Woolworths shop from the steps alone.

## 4. Store and ordering gaps

**C1. Use dislikes in the optimiser.**
Call `services/dislikes/skip.rs` when the planner picks a product. Skip disliked products, and if there is no other product, keep it and show a warning before the order is confirmed. Add dislike badges on the order review and on the chosen product.
Done when: planner tests cover every dislike scope and the all-disliked fallback, and the order screen shows the warning.

**C2. Prefer specials mode.**
An order setting that makes the planner favour products on special.
Done when: the setting is saved per household and tests show it changes the pick.

**C3. Member and rewards pricing.**
`MemberPriceData` is ignored today. Use member prices in the search results and the planner, and handle "pick any N" multibuys across different products.
Done when: a fixture with a member price changes the unit price and the total.

**C4. Screen for the product lookup route.**
The lookup route exists in the backend only. Add a web screen for it.
Done when: a person can look up a product by name or link and choose it for an item.

**C5. Checkout extras (low priority).**
Detect a saved payment method before checkout. Order tracking.
Done when: decided, built or moved to "out of scope" in the spec.

## 5. Intake gaps

**D1. Run Google Tasks against a real Google account.**
Follow `human-setup.md` ("Optional: Google Tasks"). Check the consent screen is "In production", add an item by voice, and see it reach Pending Requests and be deleted from Tasks.
Done when: the real run works, and `FEATURE_GOOGLE_TASKS.md` says so.

**D2. Test Alexa on a real device.**
Check add, remove, reduce and undo by voice, and the request signature check, through the Cloudflare Tunnel.
Done when: each intent works on the Echo and results are written in `FEATURE_VOICE.md`.

**D3. Store the OpenAI key encrypted and set it on `/settings/intake`.**
The spec asks for this. Today it is an environment variable.
Done when: the key is saved encrypted, never returned to the browser, and triage uses it.

**D4. Decide on Google Keep.**
It is paused. Either build it with the risk accepted (dedicated Google account, encrypted master token), or move it to "out of scope" in the spec and remove it from the docs.
Done when: the spec and `FEATURE_VOICE.md` agree with the decision.

## 6. Small gaps

- **E1.** Item Rules: re-apply a new rule to items already on the list, handle plurals ("egg" and "eggs"), add a `/settings` index page.
- **E2.** Update the list live between open tabs, reusing the intake WebSocket.
- **E3.** Access logs: delete old rows after a set time, and filter by user or path.
- **E4.** Google Tasks: handle subtasks and task notes, and revoke the token at Google on Disconnect.
- **E5.** Clerk: add the `azp` check, and plan a production instance (own Google OAuth app and a domain) if the app leaves the home network.
- **E6.** Local HTTPS for the web app and API (mkcert), and `cargo audit` plus `pnpm audit` in CI. Check which are already done before starting.

## 7. Update the spec (do alongside section 3)

The spec still says store orders use chromiumoxide browser automation on the server. The project now uses the desktop app for logins, delivery windows, trolley fill and checkout. Update these parts of the specification doc:
- Architecture table and infrastructure diagram: add the desktop app, remove or demote chromiumoxide.
- Checkout and Payment: describe the one-tap rules from B8.
- Intake Channels: mark Google Keep as paused or removed (D4).
- Order Optimisation: note that live windows come from the desktop app.
- Route table: confirm every route listed exists.

## 8. Suggested order

1. A1, A2, A3 (the app runs and fails clearly).
2. A4 (know whether the server can reach the stores).
3. B1, B2, B3 (the desktop app is back and starts with `make desktop`).
4. B4, B5, B6 (Coles and real delivery windows, then the constraints).
5. C1, B7 (dislikes in orders, order history import).
6. B8 (one tap to buy), then B9, B10.
7. D1 to D4 (real accounts and devices), C2 to C5, then sections 6 and 7.

## 9. The project is finished when

- A household member can say an item to Alexa or add it to Google Tasks, accept it in the web app, and pick products.
- The desktop app shows real delivery windows and fees from both stores, and the planner recommends the cheapest valid order.
- One tap places that order with the saved card.
- The order appears in purchase history and on the Spending page.
- `make dev` and `make desktop` start everything, and the human setup steps work for a new person.
