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
| 9 | The **built** bookmark script in your logged-in tab (app calls faked) | your browser | ✅ after a fix. Found that a logged-in trolley says `IsAvailable: false` and $0 for products it **did** add (no delivery time picked yet). Now "added" means the trolley quantity went up; the warning is kept as a note. |

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

## What was built

1. **Backend** (`services/trolley_handoffs`, migration `0005`):
   `POST /api/trolley-handoffs` saves every list item with a chosen
   Woolworths product. The bookmark claims it and reports each product.
2. **Bookmark script** (`frontend/src/lib/store-tab/`): runs on
   woolworths.com.au, adds each product *on top of* what is in the trolley,
   reports back.
3. **"Send to Woolworths" sheet** on the grocery list: drag the bookmark once,
   press "Send and open Woolworths", press the bookmark.
4. **Setting:** `STORE_TAB_SECRET` in `.env`.

## Things to know

- **Chrome may ask** "allow woolworths.com.au to access devices on your local
  network?" the first time the bookmark calls the app at home. Choose Allow.
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
