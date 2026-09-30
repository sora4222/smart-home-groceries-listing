# Store Integration

Status: **not built.** `services/stores/` does not exist yet. This file
records the approach and the crates to use, so whoever builds it does not have
to re-derive the decisions. Add the dependencies when you start — they are
deliberately not in `Cargo.toml` yet.

## Priority order
1. Official retailer API (check first — may exist or be partially public)
2. Internal XHR/JSON endpoints (reverse-engineered from browser DevTools — most stable)
3. Headless-browser HTML scraping (last resort — fragile, breaks on UI changes)

## Crates
| Job | Crate | Version at time of writing |
|---|---|---|
| Headless Chrome via CDP | `chromiumoxide` | 0.9.1 (features: `tokio`, `rustls`) |
| TLS-fingerprint HTTP client | `wreq` + `wreq-util` | 0.16.1 / 0.2.0 |
| HTML parsing | `scraper` | 0.27.0 |
| Money | `rust_decimal` | check for current |

Confirm current versions with Context7 MCP before adding anything.

`chromiumoxide` replaces Playwright; `wreq` replaces `curl_cffi`. Never use
plain `reqwest` against a store URL — see "Anti-scraping" below. `reqwest`
stays in this crate only for Clerk's JWKS endpoint, which has no bot
protection.

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
    .emulation(Emulation::Chrome140)   // matches a real ClientHello + H2 settings
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
// src/services/stores/mod.rs
use crate::error::ApiError;
use crate::models::schemas::{DeliveryWindow, ProductResult};

/// One retailer. The optimiser only ever sees this trait.
#[allow(async_fn_in_trait)]
pub trait StoreClient: Send + Sync {
    async fn search(&self, query: &str, filters: &[String]) -> Result<Vec<ProductResult>, ApiError>;
    async fn delivery_windows(&self) -> Result<Vec<DeliveryWindow>, ApiError>;
    /// Returns an order reference or a cart URL.
    async fn place_order(&self, items: &[OrderLine], window_id: &str) -> Result<String, ApiError>;
}
```

Woolworths and Coles each implement it. If it needs to be stored as
`Box<dyn StoreClient>`, return `BoxFuture` from the methods the way
`auth::AuthProvider` does.

## Checkout flow
Automate to the payment page and stop.

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

## Unit price normalisation
All prices normalised to a common unit for comparison:
- Weight: per 100 g (convert kg → g)
- Volume: per 100 mL (convert L → mL)
- Sheets/units: per unit

Tag a converted result so the UI can say so:

```rust
ProductResult {
    unit_price: dec!(0.45),
    unit: "per 100g".into(),
    unit_price_note: Some("calculated from kg price".into()),
    ..
}
```

When units cannot be normalised ("1 pack" vs "200 g"), set
`unit_price_note: Some("units differ — manual comparison needed".into())`.

Use `rust_decimal`, never `f64`: binary floating point cannot represent a cent
exactly, and these numbers are summed across a whole order.

## Conditional pricing
Parse multipack / "2 for $5" / member pricing from product data and compute
the effective per-unit price at the user's intended quantity:

```rust
/// Price per unit at `quantity`, honouring multipack and member deals.
fn effective_unit_price(product: &ProductResult, quantity: u32) -> Decimal {
    product
        .conditional_pricing
        .iter()
        .filter(|c| quantity >= c.min_qty)
        .map(|c| c.price / Decimal::from(c.unit_count))
        .min()
        .unwrap_or(product.unit_price)
}
```

The optimiser always uses `effective_unit_price`, never `unit_price`.

## Testing
Never hit a real store in tests. Mock the HTTP layer (`wiremock` = 0.6) and
keep captured JSON fixtures under `tests/fixtures/`. See `skills/testing.md`.
