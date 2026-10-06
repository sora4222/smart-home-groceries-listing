# Feature: Store Integration

Status: **product search implemented** — compare a list item's products at
Woolworths and Coles, and choose one per item (`FEATURE_PRODUCT_CHOICE.md`).
Chosen products can be sent to the household's own Woolworths or Coles
trolley (`FEATURE_TROLLEY_HANDOFF.md`). Delivery windows and checkout are not
built; they belong to Order Optimisation and Checkout.

## What this feature does
A household member opens **Compare prices** on any list item. The backend
asks both stores for the item at once and shows each store's products for
it, cheapest per unit first, so the household can see what the item really
costs before anything is ordered.

Nothing is recommended or promoted. Adverts and sponsored placements from
the stores are dropped, products are shown without images or marketing copy,
and the order is price only (reducing marketing-driven buying is a project
goal).

## Endpoint

| Method | Path | Auth | Answer |
|---|---|---|---|
| `GET` | `/api/grocery-items/{id}/products` | Clerk session | 200 with every store's results, 404 unknown item |
| `GET` | `/api/stores/{store}/products/{product_id}` | Clerk session | 200 one product (priced for 1), 404 not sold there, 503 store could not answer, 400 unknown store |

`{store}` is `woolworths` or `coles`; `{product_id}` is the store's own id
(Woolworths stockcode, Coles product id — digits only; anything else is
404 without asking the store). The body is one `ProductResponse`, the same
shape as an entry in `products` below.

The answer is **200 even when a store fails**: that store's entry carries
`status` and a `message`, and the other store's products still show.

```json
{
  "item": { "id": "…", "name": "toilet paper", "quantity": 2, "filter_terms": ["3 ply"] },
  "query": "toilet paper 3 ply",
  "stores": [
    {
      "store": "woolworths", "store_name": "Woolworths", "status": "ok", "message": null,
      "products": [{
        "product_id": "722", "name": "…", "brand": "Kleenex", "package_size": "12 pack",
        "price": "12", "was_price": "13.5", "on_special": true,
        "unit_price": { "amount": "0.0028", "per": "unit", "converted_from": "100 sheets" },
        "unit_price_note": "unit price calculated from 100 sheets",
        "deals": [], "total_price": "24", "deal_applied": false,
        "category": "Toilet Paper", "url": "https://www.woolworths.com.au/shop/productdetails/722/…",
        "available": true
      }]
    },
    { "store": "coles", "store_name": "Coles", "status": "blocked",
      "message": "Coles refused the search (bot protection). Try again later.", "products": [] }
  ]
}
```

`status` is `ok`, `blocked` (401/403/429 or a challenge page), `unreachable`
(network, timeout, 5xx) or `unexpected_response` (the store changed its
website). Money is always a decimal **string**.

## How a search works
1. **Query** — the item's name followed by its chips: `toilet paper 3 ply`.
   Chips go to the store too, so its own ranking favours matching products.
2. **Both stores at once**, each behind a 10-minute cache of identical
   queries (`STORE_SEARCH_CACHE_SECONDS`). Failures are never cached.
3. **Chips filter** — a product stays when every chip appears in its brand,
   name or pack size as whole words, ignoring case, punctuation and spacing:
   `3 ply` matches `3-ply` and `3ply`; `apple` does not match `pineapple`.
4. **Pricing at the item's quantity** — the best multibuy is applied to every
   complete group; the remainder pays shelf price (`total_price`).
5. **Unit prices** are restated per 100 g, per 100 mL or per unit. One basis
   is chosen for the whole search (the most common, across both stores).
   Notes, in priority order: `units differ — compare by hand` (another
   basis), `no unit price from the store`, `unit price calculated from 1kg`
   (a conversion was needed).
6. **Order** — available before unavailable, comparable before not, then by
   unit price after deals, then by `total_price`. The store's own order is
   only the final tie-break.

## The stores
| Store | Call | Notes |
|---|---|---|
| Woolworths | `POST /apis/ui/Search/products` (the website's own call) | Visits the home page once for bot-protection cookies; a refusal makes the *next* search visit again |
| Coles | `GET /_next/data/<buildId>/en/search/products.json?q=` | `buildId` read from the home page; a 404 with body `{}` means Coles redeployed — re-read once |

Looking up one product by id (`StoreClient::product`):

| Store | Call | Notes |
|---|---|---|
| Woolworths | `GET /apis/ui/product/detail/{stockcode}?isMobile=false` | `Product` has the search result's shape; 404 = not sold |
| Coles | `GET /_next/data/<buildId>/en/product/x-{id}.json` | Answers `pageProps.__N_REDIRECT` to the real slug (`/product/coles-full-cream-milk-3l-8150288`), followed once; 404 `{"notFound":true}` = not sold |

Lookups are cached like searches (same `STORE_SEARCH_CACHE_SECONDS`).

Both are reached through `wreq` emulating Chrome 137 (TLS + HTTP/2
fingerprint); plain `reqwest` is refused by the stores. No search retries in
a loop. Response shapes are from real responses captured 2026-10-01 (search)
and 2026-10-06 (lookup) —
`backend/tests/fixtures/` holds trimmed copies.

### Mapping notes
- Woolworths `CupPrice`/`CupMeasure` → unit price; `CentreTag.MultibuyData`
  (`Quantity` for `$Price` total) → deal; `IsSponsoredAd` → dropped;
  category from `AdditionalAttributes.piescategorynamesjson` (last), else
  `sapcategoryname`.
- Coles unit price is read from `pricing.comparable` (`$4.90/ 1kg`), not
  `pricing.unit` — for weighed produce `unit.ofMeasure*` says "per 1 g"
  beside a per-kilogram price. `multiBuyPromotion.reward` is already a
  per-unit price. Entries with an `adId` or `featured: true` are dropped.
- `was` / `WasPrice` only count when higher than the current price.

## Configuration
| Variable | Default | Meaning |
|---|---|---|
| `STORE_CLIENTS` | `live` | `fake` uses the built-in catalogue (`services/stores/fake.rs`) — development, e2e, CI |
| `WOOLWORTHS_BASE_URL` / `COLES_BASE_URL` | the real sites | Point a client at a mock |
| `STORE_TIMEOUT_SECONDS` | `12` | Per-store limit for one search |
| `STORE_SEARCH_CACHE_SECONDS` | `600` | How long an identical query is reused |

The fake catalogue covers milk, toilet paper and bananas. A query containing
`outage` makes the fake Coles fail as unreachable.

## Web app
`components/products/`: `PriceComparison` (compound: `.Trigger`,
`.Content`, a Shadcn `Sheet`), `StoreResults`, `ProductRow`, `ProductPrice`,
`UnitPriceLine`, `SpecialsOnlySwitch`. `hooks/useItemProducts.ts` searches
only while the sheet is open, and again when the item's name, quantity or
chips change. "Specials only" is a display filter (special or multibuy) — it
never reorders.

## Not built yet
- **Live check against the real stores from the home server.** The calls
  were checked in a real browser (search 2026-10-01, lookup 2026-10-06),
  but not yet from the server's `wreq` client. If a store answers `blocked` there, the next step is the
  human-login + cookie-reuse route in `backend/skills/store-integration.md`.
- A web app screen for the lookup route (backend only so far).
- Delivery windows, cart and checkout, member/rewards pricing (`MemberPriceData` is ignored),
  "Pick any N" multibuys across different products (treated per product).
- Storing products and categories for Spending Analysis.

## Tests
- Unit: measures, unit prices, deals, both mappings (against the fixtures),
  chip matching, comparability, ordering, the cache.
- `tests/woolworths_client.rs`, `tests/coles_client.rs`: the real clients
  against `wiremock` — cookies carried, refusals reported not retried, the
  Coles redeploy path, challenge pages.
- `tests/woolworths_product_lookup.rs`, `tests/coles_product_lookup.rs`:
  lookup against `wiremock` — redirect followed, not-found vs. redeploy,
  ids that are not digits never sent.
- `tests/item_products.rs`, `tests/store_products.rs`: the routes over the
  fake stores.
- `frontend/e2e/price-comparison.spec.ts`: desktop and phone.
