# Store Integration

Status: **product search built** (`services/stores/`,
`services/product_search/`). **Woolworths and Coles trolleys built as a browser
handoff** (`services/trolley_handoffs/`, `docs/features/FEATURE_TROLLEY_HANDOFF.md`):
the household's logged-in tab fills the trolley through a bookmarklet, because
the server cannot log in (passkeys/MFA). Server-side delivery windows and
checkout are not built. Behaviour and the API are in
`docs/features/FEATURE_STORE_INTEGRATION.md`.

## Layout
```
services/stores/
├── mod.rs          # re-exports; the only module that calls a store
├── client.rs       # StoreClient trait (search, product lookup) + BoxFuture
├── registry.rs     # build(): live or fake clients, each behind CachedStore
├── cached.rs       # short TTL cache of successful searches and lookups
├── ttl_cache.rs    # TtlCache<V>: bounded map whose entries expire
├── product_id.rs   # is_valid(): store ids are digits (they go in URL paths)
├── http.rs         # wreq client (Chrome emulation, cookies) + error mapping
├── woolworths/     # mod.rs (client) · wire.rs (serde) · mapping.rs (→ Product)
├── coles/          # mod.rs · data_route.rs (build id, 404 kinds) · wire.rs · mapping.rs · build_id.rs
├── fake.rs         # STORE_CLIENTS=fake catalogue
├── product.rs      # Product, the one shape every store maps to
├── measure.rs      # "1KG", "$4.90/ 1kg" → amount of g / mL / count
├── unit_price.rs   # normalise() → per 100 g, per 100 mL, per unit
├── deal.rs         # Deal + total_price()
├── money.rs        # JSON float → Decimal
├── image.rs        # https_only(): picture links kept only when https
├── error.rs        # StoreError: Blocked / Unreachable / UnexpectedResponse
└── store.rs        # Store enum
services/product_search/   # one item across all stores: query, filters,
                           # pricing, comparability notes, ordering, logging
services/product_lookup.rs # one product at one store by id (route:
                           # GET /api/stores/{store}/products/{id})
```
Add a store: a `wire.rs` + `mapping.rs` + client implementing `StoreClient`,
a `Store` variant, and one arm in `registry::live`. Nothing else changes.

Every mapper fills `Product.image_url` with the product's picture and passes
it through `image::https_only` (the web app puts it straight into
`<img src>`). Woolworths: `SmallImageFile`. Coles: the `imageUris` entry with
`type: default` (else the first), joined to
`https://productimages.coles.com.au/productimages`. No picture → `None`; the
web app shows a placeholder.

## Priority order
1. Official retailer API (check first; it may exist or be partly public)
2. Internal XHR/JSON endpoints (read from browser DevTools; the most stable)
3. Headless-browser HTML scraping (last resort; it breaks when the page changes)

## Crates
| Job | Crate | Version |
|---|---|---|
| TLS-fingerprint HTTP client | `wreq` + `wreq-util` | **0.15.3 / 0.1.0** (in use) |
| Money | `rust_decimal` (+ `rust_decimal_macros` for `dec!`) | 1.43 (in use) |
| Mock stores in tests | `wiremock` | 0.6.5 (in use) |

`wreq` 0.16 needs Rust 1.98; the crate's MSRV is 1.85 and the Docker image
builds with 1.95, so stay on 0.15 until both move. Enable `gzip`, `brotli`,
`zstd`, `deflate`: the emulated Chrome advertises them, so responses arrive
compressed. `wreq` builds BoringSSL, so the Docker builder needs `cmake`,
`clang`, `libclang-dev`, `g++`, `perl`, `make`.

Confirm current versions with Context7 MCP before adding anything.

Never use plain `reqwest` against a store URL (see "Anti-scraping" below). `reqwest`
stays in this crate only for Clerk's JWKS endpoint, which has no bot
protection.

## Endpoints in use (search captured 2026-10-01, lookup 2026-10-06)
- Woolworths: `POST /apis/ui/Search/products`, body
  `{Filters:[], IsSpecial:false, Location, PageNumber, PageSize, SearchTerm,
  SortType:"TraderRelevance"}` after a `GET /` for cookies.
- Coles: `GET /_next/data/<buildId>/en/search/products.json?q=`, `buildId`
  from `"buildId":"…"` in the home page's `__NEXT_DATA__`.
- Woolworths lookup: `GET /apis/ui/product/detail/{stockcode}?isMobile=false`
  → `{Product: <search-result shape>}`; 404 when not sold.
- Coles lookup: `GET /_next/data/<buildId>/en/product/x-{id}.json` → a
  `pageProps.__N_REDIRECT` to `/product/<slug>-{id}`; fetch that once for
  `pageProps.product` (same shape as a search result). Coles' two 404s:
  `{}` = stale build id, `{"notFound":true}` = no such product.

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
- `IsAvailable: false` with `$0` means the product is unavailable at the
  account's store. The trolley still takes it, so judge "added" by
  `QuantityInTrolley`, then put an unavailable product back and report it.
- Delivery times: `GET /apis/ui/Delivery/DeliveryInfo` (address, AreaId,
  reserved window), `GET /api/v3/ui/fulfilment/windows?areaId&fulfilmentMethod=Courier&addressId`,
  reserve with `POST /apis/ui/Fulfilment {addressId, fulfilmentMethod,
  timeslotId, windowDate}`. Times are store-local with no offset. Reserved
  before products are added (`reserve-woolworths-delivery-window.ts`).
- Called from the store's own page (`fill-woolworths-trolley.ts`), so no
  fingerprinting is needed there. Full record: `docs/FEAT_WOOLWORTHS_ACCESS.md`.

## Trolley (Coles, live 2026-10-06)
- Read: `GET /api/bff/trolley/store/{storeId}?sortBy=recentlyAdded` with
  `x-api-version: 3` (404 without it) → `items[]` (`productId` number,
  `quantity`, `orderItemId`), `orderAttributes.slot`.
- Write: `PATCH /api/bff/trolley/store/{storeId}`
  `{ageGateVerified, swapBehaviour:false, [slotCutOffTime, slotId, reservationExpirationTime],
  items:[{actions:[{productId, quantity[, orderItemId]}]}]}`. **No
  `orderItemId` → quantity is added**; with it → quantity is the total,
  `0` deletes. Refusals are HTTP 200 with `results[0].failedItems[].error`.
- Both need `Ocp-Apim-Subscription-Key` (`window.__RUNTIME_CONFIG__.BFF_API_SUBSCRIPTION_KEY`)
  and the `cusp-*` headers from the site's cookies (the website's own
  `prepareHeaders`). Store id: `localStorage.shoppingMethod.currentFulfilmentStoreId`.
- Delivery: the site reserves with `POST /api/bff/slots/{slotId}` (body not
  captured). `fill-coles-trolley.ts` reserves none.

## Adding a store
1. Rust: a `Store` variant, a `StoreClient` for search and product lookup
   (`registry.rs`), a
   `StoreSettings::base_url` arm (the store-tab CORS rule uses it).
2. Web app: a self-contained fill script and a `StoreTab` in
   `frontend/src/lib/store-tab/store-tabs.ts`. `SendToStore` and the
   handoff routes need no change.

## Anti-scraping: Akamai
Both Woolworths and Coles use **Akamai** bot protection. A default `reqwest`
or `hyper` client has a TLS ClientHello and HTTP/2 settings fingerprint that
does not match any real browser, and gets blocked.

The server gets through with `wreq` TLS emulation. Store pages call the
trolley from the household's own tab, so they need nothing extra.

### `wreq` TLS emulation
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

On a 401, 403, or a redirect to a login page, report it. Never retry in a
loop; that is what gets an account flagged.

## `StoreClient` trait
```rust
// src/services/stores/client.rs, as built
pub trait StoreClient: Send + Sync {
    fn store(&self) -> Store;
    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>>;
    /// Ok(None) = not sold there, or not one of its ids (product_id::is_valid).
    fn product<'a>(&'a self, product_id: &'a str) -> BoxFuture<'a, Result<Option<Product>, StoreError>>;
}
```
Held as `Arc<dyn StoreClient>` in `AppState.stores`. Chip filtering happens
after the search (`product_search::filters`), not in the client. Add
`delivery_windows` and `place_order` here when those features are built,
returning `StoreError`, never `ApiError`, because a store failing is not a failed
request.

## Delivery windows for the optimiser (not built)
The windows call needs the household's login, so it only runs in the store
tab today. The optimiser can get windows the same way: have the store tab
report `Days[]` back, rather than a server-side login.

## Unit price normalisation (built: `measure.rs`, `unit_price.rs`)
- Weight: per 100 g · Volume: per 100 mL · Sheets/each/pack: per unit
- `UnitPrice.converted_from` holds the store's measure (`1kg`) when the basis
  changed; `product_search::comparability` turns it into the note
  "unit price calculated from 1kg".
- Products on a different basis from the search's most common one get
  "units differ — compare by hand"; none at all, "no unit price from the
  store". Unknown units (`1 bunch`) are never guessed, so they get no unit price.

Use `rust_decimal`, never `f64`: binary floating point cannot represent a cent
exactly, and these numbers are summed across a whole order.

## Conditional pricing (built: `deal.rs`)
A `Deal` is `{description, min_quantity, unit_price}`, the per-unit price
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
