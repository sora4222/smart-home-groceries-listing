# Store Integration

## Priority order
1. Official retailer API (check first — may exist or be partially public)
2. Internal XHR/JSON endpoints (reverse-engineered from browser DevTools — most stable)
3. Playwright HTML scraping (last resort — fragile, breaks on UI changes)

## Finding internal endpoints
1. Open Chrome DevTools → Network tab → filter to `Fetch/XHR`
2. Browse the store site (search, product page, cart, delivery windows)
3. Look for JSON responses containing product data
4. Key endpoints to find:
   - Product search: `?query=<term>&pageSize=24`
   - Product detail + pricing (includes conditional/multipack pricing)
   - Delivery windows with fees
   - Cart add / cart view

## Anti-scraping: Akamai
Both Woolworths and Coles use **Akamai** bot protection. Standard `requests`/`httpx` produce a detectable TLS fingerprint and will be blocked.

### Strategy 1 — Human login + Playwright cookie reuse (preferred)
```python
# User logs in once via their own browser.
# Export cookies (e.g. EditThisCookie extension → JSON).
# Store encrypted in PostgreSQL alongside credentials.
# Playwright loads the cookie jar at session start.

async def load_session(page: Page, cookies: list[dict]) -> None:
    await page.context.add_cookies(cookies)

# When a 401/403 or login redirect is detected → mark cookies expired,
# prompt user to re-authenticate.
```

### Strategy 2 — curl_cffi TLS spoofing (API calls outside Playwright)
```python
from curl_cffi import requests as cffi_requests

# Impersonate a real browser's TLS ClientHello
session = cffi_requests.Session(impersonate="chrome120")
resp = session.get(
    "https://www.woolworths.com.au/api/2/pages/...",
    headers={"User-Agent": "Mozilla/5.0 ..."},
)
data = resp.json()
```

### Strategy 3 — tls-client (alternative if curl_cffi insufficient)
```python
import tlsclient.requests as tls_requests
session = tls_requests.Session(client_identifier="chrome_120")
```

### Session cookie storage
Cookies stored AES-256 encrypted in PostgreSQL alongside store credentials.
Refresh trigger: any store request returning 401, 403, or redirect to login page.

## StoreClient protocol
```python
# app/services/stores/base.py
from typing import Protocol
from app.models.schemas import ProductResult, DeliveryWindow

class StoreClient(Protocol):
    async def search(self, query: str, filters: list[str]) -> list[ProductResult]: ...
    async def get_delivery_windows(self) -> list[DeliveryWindow]: ...
    async def place_order(self, items: list, window_id: str) -> str: ...  # returns order ref or cart URL
```

Woolworths and Coles each implement this protocol. The optimiser only knows the protocol.

## Playwright checkout flow
```python
async def checkout(page: Page, items: list, window_id: str) -> CheckoutResult:
    await load_session(page, cookies)          # load stored cookies
    await navigate_to_cart(page)
    for item in items:
        await add_to_cart(page, item)
    await select_delivery_window(page, window_id)
    await proceed_to_checkout(page)
    saved_card = await detect_saved_payment(page)
    # STOP HERE — never enter payment details
    return CheckoutResult(url=page.url, has_saved_card=saved_card)
```

## Unit price normalisation
All prices normalised to a common unit for comparison:
- Weight: per 100 g (convert kg → g)
- Volume: per 100 mL (convert L → mL)
- Sheets/units: per unit

When conversion is performed, tag the result:
```python
ProductResult(
    unit_price=0.45,
    unit="per 100g",
    unit_price_note="calculated from kg price"  # shown in UI
)
```
When units can't be normalised (e.g. "1 pack" vs "200 g"), set `unit_price_note="units differ — manual comparison needed"`.

## Conditional pricing
Parse multipack / "2 for $5" / member pricing from product data.
Calculate effective per-unit price at the user's intended quantity:
```python
def effective_unit_price(product: ProductResult, quantity: int) -> Decimal:
    for condition in product.conditional_pricing:
        if quantity >= condition.min_qty:
            return condition.price / condition.unit_count
    return product.unit_price
```
The optimiser always uses `effective_unit_price`, not `unit_price`.
