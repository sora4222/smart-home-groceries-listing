# Woolworths access — what was tried, what works

Date: 2026-10-02. Goal: put the products chosen in the app into the
household's **own** Woolworths trolley. Stop before payment.

## Short answer

- **It works**, using the household's own logged-in browser tab.
- The **server cannot log in to Woolworths by itself.** Login uses passkeys
  and MFA inside Woolworths' JavaScript.
- So the app hands the list to a bookmark ("Fill Woolworths trolley"). You
  press it on woolworths.com.au. It adds the products with Woolworths' own
  trolley call. Then you check the trolley and pay on Woolworths.
- Feature spec: `docs/features/FEATURE_TROLLEY_HANDOFF.md`.

## The bot protection (Akamai)

- Plain `curl` gets **403 Access Denied** from Akamai, even for the home page
  and the script CDN (`cdn1.woolworths.media`).
- A client that copies Chrome's TLS + HTTP/2 fingerprint gets through. The
  first page visit sets the Akamai cookies (`_abck`, `bm_sz`, `ak_bmsc`,
  `bm_sv`) and later calls carry them.
- The app's server-side search already does this with `wreq` + `wreq-util`
  (Chrome 137). No new store dependency was needed.
- A real browser tab (headless or not) also gets through.

## Approaches tried (9 of the 10 allowed)

| # | Approach | Where | Result |
|---|---|---|---|
| 1 | `curl_cffi` copying Chrome's fingerprint: home page, search, trolley read | cloud | ✅ All 200. Search JSON came back. |
| 2 | Headless Chromium + `playwright-stealth`, product page | cloud | ⚠️ Page not blocked, but our own network allowlist blocked `cdn1.woolworths.media`, so scripts did not load. |
| 3 | Trolley calls from public notes: `/apis/ui/Trolley/Update` (404), then `/api/v3/ui/trolley/update` | cloud, guest | ✅ v3 call adds and removes (guest trolley). |
| 4 | Server login: `/shop/securelogin`, `auth.woolworths.com.au`, auth cookies | cloud | ❌ Login is JavaScript only. Session tokens are 1-hour JWTs (`wow-auth-token`, `w-rctx`) with an `mfa` claim. Passkeys cannot be scripted. (Claude also may not type passwords.) |
| 5 | Pass the handoff in the URL after `#` | your browser | ❌ Woolworths removes the `#…` part on load. |
| 6 | Woolworths' trolley call from inside **your logged-in tab** | your browser | ✅ Added 1 milk to your real trolley and removed it. |
| 7 | Approach 2 again after `*.woolworths.media` was allowed; pressed "Add to cart" | cloud | ✅ Full page. The site itself called `POST /api/v3/ui/trolley/update` with the same body as #3. |
| 8 | The app's own Rust client (`wreq`) against the live site | cloud | ⚠️ Not testable here: this workspace only reaches the internet through a proxy, and the client does not use it ("Connect" error). Not a Woolworths refusal. Search worked the same way in #1. |
| 9 | The **built** bookmark script in your logged-in tab (app calls faked) | your browser | ✅ after a fix. The trolley answered `IsAvailable: false` and $0 for a product it **did** add. Now "added" means the trolley quantity went up. (Cause found later: see "Delivery times".) |

Nothing was bought. Your trolley was left as it was (one existing item).

## Woolworths calls (live, 2026-10-02)

| Call | Use | Notes |
|---|---|---|
| `GET /` | Akamai cookies | Needs a Chrome-like fingerprint |
| `POST /apis/ui/Search/products` | search | Already used by the app |
| `GET /api/v3/ui/trolley` | read trolley | `Products[]` with `Stockcode`, `QuantityInTrolley`, `IsAvailable`; `Errors[]` (e.g. "$50 minimum spend") |
| `POST /api/v3/ui/trolley/update` | set quantity | Body `{"items":[{"stockcode":88436,"quantity":2,"source":"ProductDetail","diagnostics":"0","searchTerm":null,"evaluateRewardPoints":false,"offerId":null,"profileId":null,"priceLevel":null}]}`. Sets the quantity (not adds). `0` removes. Unknown stockcode → `UpdatedItems: []` |
| `GET /apis/ui/Trolley` | old trolley read | Did not show items in the logged-in trolley. Do not use |
| `GET /auth/heartbeat` | session keep-alive | Seen in the page; not used |
| `GET /apis/ui/Delivery/DeliveryInfo` | address + reserved time | `Address.AddressId`, `Address.AreaId`, `DeliveryMethod` (`Courier` = delivery), `CurrentDateAtFulfilmentStore`, `FulfilmentStoreId`, `ReservedTime` (`Id` 0 = none) |
| `GET /api/v3/ui/fulfilment/windows?areaId=&fulfilmentMethod=Courier&addressId=` | delivery times | `Days[]` (about 7) → `Times[]`: `Id`, `TimeWindow`, `StartDateTime`, `EndDateTime` (store-local, no offset), `SalePrice`, `Available`, `IsExpress`, `IsCrowdSourced` ("Partner Driver") |
| `POST /apis/ui/Fulfilment` | reserve a time | Body `{"addressId":N,"fulfilmentMethod":"Courier","timeslotId":N,"windowDate":"YYYY-MM-DD"}` → `{"IsSuccessful":true}`. Changeable on the website later |

## Delivery times (added later on 2026-10-02)

- Found from the site's own scripts (`*.woolworths.media`), then read live in
  your logged-in tab. Calls are in the table above.
- Reserving tomorrow 7–10am on your account worked first time. **That
  reservation is still on your account** — change it on Woolworths any time.
- **Correction:** `IsAvailable: false` / $0 is **not** caused by having no
  delivery time. Those products are really unavailable at your store (store
  3800). Woolworths' own product page says so, and other products (e.g.
  tortillas, garlic bread) show prices and are available. The bookmark now
  takes such a product back out and reports "Not available at your
  Woolworths store right now".
- Search on the server runs as a guest at a default store, so a product can
  look available in "Compare prices" and still be unavailable at your store.
- A live run of the finished bookmark program on your account was **blocked
  by Claude's safety check** (it would add a real, priced product and could
  change your reservation). Each call it makes was checked live on its own
  first. The first real run is when you press the bookmark.

## What was built

1. **Backend** (`services/trolley_handoffs`, migration `0005`):
   `POST /api/trolley-handoffs` saves every list item with a chosen
   Woolworths product. The bookmark claims it and reports each product.
2. **Bookmark script** (`frontend/src/lib/store-tab/`): runs on
   woolworths.com.au, adds each product *on top of* what is in the trolley,
   reports back.
3. **"Send to Woolworths" sheet** on the grocery list: drag the bookmark once,
   pick a delivery day and time of day (default: tomorrow, any time), press
   "Send and open Woolworths", press the bookmark. The bookmark reserves the
   time first, then adds the products.
4. **Setting:** `STORE_TAB_SECRET` in `.env`.

## Things to know

- Steps for people (setup, a weekly shop, fixes) are in
  [`human-setup.md` part 7](human-setup.md#part-7--woolworths-once-about-10-minutes)
  and [`using-the-app.md`](using-the-app.md). Do not repeat them here.
- The bookmark calls the app at `VITE_API_BASE_URL`. It must be reachable
  from the browser (it already is, if the app works).
- Woolworths can change these calls at any time. If the bookmark starts
  failing, open DevTools → Network on Woolworths, press "Add to cart", and
  compare with the table above.
- Automating a store's website may be against its terms of use. This only
  does what you could do by hand, in your own logged-in tab, at human speed.

## If you want it fully on the server later (no bookmark)

Not possible with what is here, because of login. Things to search for:

- **"Woolworths API partner" / "Woolworths Group developer portal"** — check
  whether Woolworths offers an ordering API to partners (none found public).
- **Delivery apps that sell Woolworths** (e.g. Uber Eats, DoorDash, Woolworths
  Metro) — some have partner APIs.
- **Scraping/automation services** with real browsers and residential IPs
  (search "Woolworths scraper API", "Apify Woolworths", "residential proxy
  browser API"). They help with bot blocking, **not** with your login.
- **Open-source projects**, e.g. the "Woolworths MCP server" (elijah-g on
  glama.ai): it also needs cookies copied from a logged-in browser.
- **Cookie hand-off:** you copy your Woolworths cookies into the app, and the
  server uses them with `wreq`. Tokens last about 1 hour, so it needs
  refreshing (`/auth/heartbeat`) and was not built.
