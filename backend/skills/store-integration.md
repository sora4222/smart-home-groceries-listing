# Store Integration

Status: **product search built** (`services/stores/`,
`services/product_search/`). **Woolworths trolley built as a browser
handoff** (`services/trolley_handoffs/`, `docs/features/FEATURE_TROLLEY_HANDOFF.md`):
the household's logged-in tab fills the trolley through a bookmarklet, because
the server cannot log in (passkeys/MFA). Delivery windows and checkout are
not built — the sections on them below are the plan. Behaviour and the API are in
`docs/features/FEATURE_STORE_INTEGRATION.md`.

## Layout
```
services/stores/
├── mod.rs          # re-exports; the only module that calls a store
├── client.rs       # StoreClient trait (search) + BoxFuture
├── registry.rs     # build(): live or fake clients, each behind CachedStore
├── cached.rs       # short TTL cache of successful searches
├── http.rs         # wreq client (Chrome emulation, cookies) + error mapping
├── woolworths/     # mod.rs (client) · wire.rs (serde) · mapping.rs (→ Product)
├── coles/          # mod.rs · wire.rs · mapping.rs · build_id.rs
├── fake.rs         # STORE_CLIENTS=fake catalogue
├── product.rs      # Product — the one shape every store maps to
├── measure.rs      # "1KG", "$4.90/ 1kg" → amount of g / mL / count
├── unit_price.rs   # normalise() → per 100 g, per 100 mL, per unit
├── deal.rs         # Deal + total_price()
├── money.rs        # JSON float → Decimal
├── error.rs        # StoreError: Blocked / Unreachable / UnexpectedResponse
└── store.rs        # Store enum
services/product_search/   # one item across all stores: query, filters,
                           # pricing, comparability notes, ordering, logging
```
Add a store: a `wire.rs` + `mapping.rs` + client implementing `StoreClient`,
a `Store` variant, and one arm in `registry::live`. Nothing else changes.

## Priority order
1. Official retailer API (check first — may exist or be partially public)
2. Internal XHR/JSON endpoints (reverse-engineered from browser DevTools — most stable)
3. Headless-browser HTML scraping (last resort — fragile, breaks on UI changes)

## Crates
| Job | Crate | Version |
|---|---|---|
| TLS-fingerprint HTTP client | `wreq` + `wreq-util` | **0.15.3 / 0.1.0** (in use) |
| Money | `rust_decimal` (+ `rust_decimal_macros` for `dec!`) | 1.43 (in use) |
| Mock stores in tests | `wiremock` | 0.6.5 (in use) |
| Headless Chrome via CDP | `chromiumoxide` | 0.9.1 — for checkout, not added yet |
| HTML parsing | `scraper` | 0.27.0 — not needed so far |

`wreq` 0.16 needs Rust 1.98; the crate's MSRV is 1.85 and the Docker image
builds with 1.95, so stay on 0.15 until both move. Enable `gzip`, `brotli`,
`zstd`, `deflate`: the emulated Chrome advertises them, so responses arrive
compressed. `wreq` builds BoringSSL — the Docker builder needs `cmake`,
`clang`, `libclang-dev`, `g++`, `perl`, `make`.

Confirm current versions with Context7 MCP before adding anything.

`chromiumoxide` replaces Playwright; `wreq` replaces `curl_cffi`. Never use
plain `reqwest` against a store URL — see "Anti-scraping" below. `reqwest`
stays in this crate only for Clerk's JWKS endpoint, which has no bot
protection.

## Endpoints in use (captured 2026-10-01)
- Woolworths: `POST /apis/ui/Search/products`, body
  `{Filters:[], IsSpecial:false, Location, PageNumber, PageSize, SearchTerm,
  SortType:"TraderRelevance"}` after a `GET /` for cookies.
- Coles: `GET /_next/data/<buildId>/en/search/products.json?q=`, `buildId`
  from `"buildId":"…"` in the home page's `__NEXT_DATA__`.

To capture a fresh shape, open the store's search page in a real browser and
read the call from DevTools (or the page's `__NEXT_DATA__` for Coles), then
trim it into `tests/fixtures/<store>/`.

## Trolley (Woolworths, live 2026-10-02)
- Read: `GET /api/v3/ui/trolley` → `Products[]` (`Stockcode`,
  `QuantityInTrolley`, `IsAvailable`). The old `GET /apis/ui/Trolley` did not
  show a logged-in trolley's items.
- Write: `POST /api/v3/ui/trolley/update` with
  `{"items":[{"stockcode":N,"quantity":Q,"source":"ProductDetail","diagnostics":"0",
  "searchTerm":null,"evaluateRewardPoints":false,"offerId":null,"profileId":null,"priceLevel":null}]}`.
  `quantity` is the new total, `0` removes. Unknown stockcode → `UpdatedItems: []`.
- With no delivery time picked, a logged-in trolley answers `IsAvailable:
  false` and `$0` for products it did add — judge success by
  `QuantityInTrolley`, never by `IsAvailable`.
- Called from the store's own page (`fill-woolworths-trolley.ts`), so no
  fingerprinting is needed there. Full record: `docs/FEAT_WOOLWORTHS_ACCESS.md`.

## Finding internal endpoints
1. Chrome DevTools → Network → filter to `Fetch/XHR`
2. Browse the store site (search, product page, cart, delivery windows)
3. Look for JSON responses containing product data
4. Endpoints to find:
   - Product search: `?query=<term>&pageSize=24`
   - Product detail + pricing (includes conditional/multipack pricing)
   - Delivery windows with fees
   - Cart add / cart view

## Anti-scraping: Akamai
Both Woolworths and Coles use **Akamai** bot protection. A default `reqwest`
or `hyper` client has a TLS ClientHello and HTTP/2 settings fingerprint that
does not match any real browser, and gets blocked.

### Strategy 1 — human login + cookie reuse (preferred)
```rust
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::CookieParam;

/// Loads a previously captured session into a fresh page.
///
/// The user logs in once in their own browser; the cookie jar is stored
/// AES-256-GCM encrypted in PostgreSQL (see `services::encryption`) and
/// replayed here until it expires.
async fn load_session(page: &chromiumoxide::Page, cookies: Vec<CookieParam>)
    -> Result<(), chromiumoxide::error::CdpError>
{
    for cookie in cookies {
        page.set_cookie(cookie).await?;
    }
    Ok(())
}
```

`chromiumoxide` needs the handler polled or nothing happens — this is the
mistake to expect:

```rust
let (mut browser, mut handler) = Browser::launch(
    BrowserConfig::builder().no_sandbox().build()?
).await?;

// Required. Without this task the browser never makes progress.
let handler_task = tokio::spawn(async move {
    while let Some(event) = handler.next().await {
        if event.is_err() { break; }
    }
});

// ... work ...
browser.close().await?;
handler_task.await?;
```

On a 401, 403, or a redirect to the login page → mark the cookies expired and
prompt the user to re-authenticate. Never retry in a loop; that is what gets
an account flagged.

### Strategy 2 — `wreq` TLS emulation (API calls outside the browser)
```rust
use wreq::Client;
use wreq_util::Emulation;

let client = Client::builder()
    .emulation(Emulation::Chrome137)   // newest in wreq-util 0.1; see http.rs
    .cookie_store(true)
    .build()?;

let products: ProductSearch = client
    .get("https://www.woolworths.com.au/api/v3/ui/...")
    .send()
    .await?
    .json()
    .await?;
```

Pick an emulation target close to the Chromium `chromiumoxide` drives, so the
two paths do not present as two different browsers on one session.

### Session cookie storage
Cookies and store credentials are AES-256-GCM encrypted in PostgreSQL via
`services::encryption::Encryptor`. Refresh trigger: any store request
returning 401, 403, or a redirect to a login page.

## `StoreClient` trait
```rust
// src/services/stores/client.rs — as built
pub trait StoreClient: Send + Sync {
    fn store(&self) -> Store;
    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>>;
}
```
Held as `Arc<dyn StoreClient>` in `AppState.stores`. Chip filtering happens
after the search (`product_search::filters`), not in the client. Add
`delivery_windows` and `place_order` here when those features are built,
returning `StoreError`, never `ApiError` — a store failing is not a failed
request.

## Checkout flow
Automate to the payment page and stop. **Login cannot be automated** (see
above), so any server-side `chromiumoxide` checkout needs an imported,
refreshed session; prefer extending the browser handoff instead.

```rust
/// Drives the cart up to the payment step. Never enters card details.
async fn checkout(page: &Page, items: &[OrderLine], window_id: &str)
    -> Result<CheckoutResult, ApiError>
{
    load_session(page, cookies).await?;
    navigate_to_cart(page).await?;
    for item in items {
        add_to_cart(page, item).await?;
    }
    select_delivery_window(page, window_id).await?;
    proceed_to_checkout(page).await?;
    let has_saved_card = detect_saved_payment(page).await?;
    // STOP HERE — never enter or store payment details.
    Ok(CheckoutResult { url: page.url().await?, has_saved_card })
}
```

## Unit price normalisation (built: `measure.rs`, `unit_price.rs`)
- Weight: per 100 g · Volume: per 100 mL · Sheets/each/pack: per unit
- `UnitPrice.converted_from` holds the store's measure (`1kg`) when the basis
  changed; `product_search::comparability` turns it into the note
  "unit price calculated from 1kg".
- Products on a different basis from the search's most common one get
  "units differ — compare by hand"; none at all, "no unit price from the
  store". Unknown units (`1 bunch`) are never guessed — no unit price.

Use `rust_decimal`, never `f64`: binary floating point cannot represent a cent
exactly, and these numbers are summed across a whole order.

## Conditional pricing (built: `deal.rs`)
A `Deal` is `{description, min_quantity, unit_price}` — the per-unit price
inside each complete group. `deal::total_price(shelf, deals, qty)` applies
the best deal to complete groups and shelf price to the remainder, ignoring
deals dearer than shelf. `PricedProduct::effective_unit_price()` scales the
unit price by that saving. The optimiser must use these, never shelf price.
Member pricing (`MemberPriceData`) is not parsed yet.

## Testing
Never hit a real store in tests. Mapping tests read `tests/fixtures/`
(trimmed real responses); client tests (`tests/<store>_client.rs`) serve
them from `wiremock`; route tests use `STORE_CLIENTS=fake` via
`common::fake_store_settings()`. See `skills/testing.md`.
