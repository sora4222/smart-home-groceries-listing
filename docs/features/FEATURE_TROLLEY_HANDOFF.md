# Feature: Trolley Handoff (Send to Woolworths)

Status: **implemented for Woolworths.** Puts every chosen Woolworths product
into the household's own online trolley. Payment stays on woolworths.com.au.
Why it works this way: `docs/FEAT_WOOLWORTHS_ACCESS.md`.
Steps for people: `docs/using-the-app.md` ("Do a Woolworths shop") and
`docs/human-setup.md` part 8 — keep those in step with this spec.

## What this feature does
1. On the grocery list, or in the Woolworths card on `/order`,
   **Send to Woolworths (N items)** opens a sheet.
   N = items still to buy (active or committed) whose chosen product is at
   Woolworths (`lib/chosen-at-store.ts`). Disabled when N is 0.
2. Once only: drag **Fill Woolworths trolley** to the bookmarks bar.
3. Pick a **delivery day** (default **Tomorrow**, then the next 6 days) and a
   **time of day** (Any · Morning < 12pm · Afternoon 12–5pm · Evening ≥ 5pm).
4. **Send and open Woolworths** creates a handoff and opens
   `woolworths.com.au/shop/mytrolley`. Log in there if asked.
5. Press the bookmark on Woolworths. It **reserves the delivery time first**,
   then adds each product **on top of** what is already in the trolley, then
   reports back. The sheet shows each
   product as Added / Not added (it re-reads every 3 s).

Nothing is bought and no Woolworths login or cookie is ever stored.

## Endpoints

| Method | Path | Auth | Answer |
|---|---|---|---|
| `POST` | `/api/trolley-handoffs` `{store}` | Clerk | 201 handoff; 422 nothing chosen at that store |
| `GET` | `/api/trolley-handoffs/{id}` | Clerk | 200 handoff with each line's outcome; 404 |
| `GET` | `/api/trolley-handoffs/store-tab-secret` | Clerk | 200 `{secret}`; 500 not configured |
| `POST` | `/api/store-tab/trolley-handoffs/claim` `{secret, store}` | secret in body | 200 claimed handoff (one line per product); 204 nothing waiting |
| `POST` | `/api/store-tab/trolley-handoffs/{id}/report` `{secret, lines}` | secret in body | 200; 404; 409 not claimed / already reported; 422 report misses or repeats a product |

Store-tab requests come from the **store's** origin. They send JSON as
`text/plain` (a CORS simple request, no preflight), and only these two routes
answer with `Access-Control-Allow-Origin` for the stores' websites
(`WOOLWORTHS_BASE_URL` / `COLES_BASE_URL`).

Report line: `{product_id, outcome: "added" | "failed", problem?}`. An added
line may carry a `problem` (a Woolworths warning, e.g. "not available yet").

## Delivery time
- **Default:** tomorrow = the day after the **store's** today
  (`CurrentDateAtFulfilmentStore`), sent as `date: null` so a device's time
  zone cannot pick the wrong day. Any time of day.
- **Choice:** first day on/after the wanted day with a matching available,
  non-express window; on that day the cheapest, then earliest start, then
  shortest. Another day used → `problem` says so.
- **Kept:** a window already reserved on the wanted day and part of the day is
  not changed (`outcome: kept`).
- **Changeable later** on the store's website; the app never locks it.
- Rule code: `lib/store-tab/choose-woolworths-window.ts`; reserving:
  `lib/store-tab/reserve-woolworths-delivery-window.ts`.
- Create body: `{store, delivery?: {date?: "YYYY-MM-DD", time_of_day?}}` —
  422 for a day more than 1 day in the past or 15 days ahead (UTC slack).
- Report body: `delivery?: {outcome: reserved|kept|failed, window_label,
  window_start, window_end (store-local, no offset), fee, problem}`; a
  reserved/kept window needs label, start < end; a failure needs `problem`.
- Response: `delivery: {requested: {date, time_of_day}, outcome, window_label,
  window_start, window_end, fee, problem}`.

## Rules
- **Lifetime:** a handoff can be claimed for 30 minutes. Creating a new one
  marks any still waiting for that store `replaced`.
- **Claimed once:** the newest waiting, unexpired handoff; `SKIP LOCKED`.
- **Same product on two items:** sent to the store tab as one product with
  the quantities summed; the report's outcome is written to both lines.
- **Status:** `waiting_for_store_tab` → `claimed_by_store_tab` → `filled` or
  `filled_with_problems` (any failed line). Or `replaced`.
- **Quantity:** the list item's current quantity, not the priced quantity.
- **"Added"** means Woolworths' `QuantityInTrolley` reached the new quantity
  **and** the product is available at the household's store. An unavailable
  product (`IsAvailable: false`, $0) is put back to its old quantity and
  reported as failed: "Not available at your Woolworths store right now".

## Data
Migration `0005`: `trolley_handoffs` (id, store, status, created_by,
created_at, expires_at, claimed_at, reported_at; `0006` adds delivery_date,
delivery_time_of_day, delivery_outcome, delivery_window_label,
delivery_window_start, delivery_window_end, delivery_fee, delivery_problem) and `trolley_handoff_lines`
(handoff_id, position, grocery_item_id, product_id, product_name, quantity,
outcome, problem). Lines are a snapshot; deleting the list item keeps them.

## Code
- Backend: `services/trolley_handoffs/` (`mod.rs` service, `repository.rs`
  handoff SQL, `line_repository.rs` line SQL, `store_tab.rs` and
  `delivery.rs` pure rules, `log.rs`), `models/trolley_handoff_rows.rs`, `routes/trolley_handoffs.rs`
  (web app), `routes/store_tab.rs` (bookmark), `routes/extract.rs`
  `AnyContentTypeJson`.
- Frontend: `lib/store-tab/fill-woolworths-trolley.ts` (the script that runs
  on Woolworths — must stay self-contained; its helpers arrive as its second
  argument), `choose-woolworths-window.ts`, `reserve-woolworths-delivery-window.ts`,
  `lib/store-tab/bookmarklet.ts`, `lib/delivery-days.ts`,
  `lib/api/trolley-handoffs.ts`, `hooks/useTrolleyHandoff.ts`,
  `components/trolley/` (`SendToWoolworths` compound, `DeliveryTimeChooser`,
  `FillTrolleyBookmark`, `HandoffStatus`).
- Setting: `STORE_TAB_SECRET`.

## Not built
- Coles (same handoff; needs a Coles script and its trolley call).
- Showing the store's real windows and fees in the app before sending (the
  app cannot see them without the household's Woolworths login).
- A saved default in Settings › Delivery; checkout steps (`FEATURE_CHECKOUT.md`).
- A server-only route with imported cookies (see the access doc).

## Tests
- `backend/tests/trolley_handoffs.rs` — create, claim once, replace, merged
  products, report, refusals, CORS header, auth.
- `backend/tests/trolley_handoff_delivery.rs` — default and chosen delivery,
  refused days, reserved/failed reports, nonsense windows.
- `frontend/src/lib/__tests__/choose-woolworths-window.test.ts`,
  `reserve-woolworths-delivery-window.test.ts`, `delivery-days.test.ts`
  (fixtures: `woolworths-fixtures.ts`, trimmed live answers).
- `services/trolley_handoffs/store_tab.rs` unit tests.
- `frontend/src/lib/__tests__/fill-woolworths-trolley.test.ts` — the script
  against Woolworths' live answer shapes, and the built bookmarklet run on
  its own.
- `frontend/src/components/trolley/__tests__/send-to-woolworths.test.tsx`.
