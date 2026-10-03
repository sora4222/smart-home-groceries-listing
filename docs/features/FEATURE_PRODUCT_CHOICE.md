# Feature: Product Choice (one product per store for each item)

Status: **implemented** — the household picks a product for each list item
from the item's price comparison, and may pick one at each store. The order review (`/order`,
`FEATURE_ORDER_OPTIMISATION.md`) reads these choices to know what to buy.

## What this feature does
On any list item, **Compare prices** opens the Woolworths and Coles results
(`FEATURE_STORE_INTEGRATION.md`). Each product now has a **Choose** button.
Choosing saves that product as the item's product; the item's card shows it
("Coles Full Cream Milk · 3L · Coles · $4.95 each") with a **Clear** button,
or "No product chosen yet".

- **One product per store.** Choosing again at the same store replaces that
  store's choice. Choosing at the other store keeps the first choice as
  well, so the order screen can compare the stores
  (`FEATURE_ORDER_OPTIMISATION.md`).
- **One of them is bought.** The newest choice is the one the order buys
  (`for_order`), until the order screen switches the item to its other store.
  The card shows the other store's choice under it as "Also at …".
- **Undo.** Choosing and clearing each show a toast with **Undo**, which puts
  back the store's earlier product and the store the order bought from
  (`lib/choice-undo.ts`).
- **Works on a committed list.** Choosing is what the order needs, so it is
  not locked by "Ready to order". Ordered items cannot be re-chosen (409).
- **Nothing is chosen for the household.** No default, no suggestion — the
  household picks (reducing marketing-driven buying is a project goal).

## Endpoints

| Method | Path | Auth | Answer |
|---|---|---|---|
| `GET` | `/api/item-selections` | Clerk session | 200, each item's choice to buy with `also_chosen` (its other stores' choices) |
| `PUT` | `/api/grocery-items/{id}/selection` | Clerk session | 200 the saved choice, now the one to buy |
| `DELETE` | `/api/grocery-items/{id}/selection[?store=]` | Clerk session | 204 (also when there was none). With `store`, only that store's choice; clearing the one to buy hands that role to the other |
| `PUT` | `/api/order-stores` `{picks: [{grocery_item_id, store}]}` | Clerk session | 204; 422 (nothing changed) when an item has no choice at its store |

`PUT` body: `{ "store": "woolworths" | "coles", "product_id": "<store's id>" }`.
Only these two are sent — **the price and every other detail come from the
store's own search answer**, never from the request.

| `PUT` status | When |
|---|---|
| 200 | Saved (or replaced) |
| 404 | No such item |
| 409 | The item was renamed/re-chipped while choosing, or is already ordered |
| 422 | Unknown store, empty id, the store no longer offers that product for this item and its chips, or it is unavailable |
| 503 | That store could not be searched just now (`detail` says which) |

A saved choice:

```json
{
  "grocery_item_id": "…", "store": "coles", "store_name": "Coles",
  "product_id": "c-milk-3l", "name": "Full Cream Milk", "brand": "Coles",
  "package_size": "3L", "price": "4.95",
  "unit_price": { "amount": "0.165", "per": "100mL" },
  "total_price": "9.90", "priced_quantity": 2,
  "url": "https://www.coles.com.au/…", "selected_by": "user_…",
  "selected_at": "2026-10-01T11:00:00Z"
}
```

Money is a decimal **string**. `total_price` is what `priced_quantity` cost
with the best deal when it was chosen — **the order screen must re-price**
before buying, because prices and the item's quantity can change.

## How a choice is checked
1. The item's search is run again (`ProductSearchService`, usually answered
   from the 10-minute cache), outside any transaction.
2. The product must be in that store's results — so it must still match the
   item's chips — and be available (`services/selections/offer.rs`).
3. The item row is locked; if its name or chips changed since the search,
   409. Then the item's other choice stops being the one to buy, and one
   `INSERT … ON CONFLICT (grocery_item_id, store) DO UPDATE` saves this one
   as the one to buy.

## When a choice goes away
| Change to the item | Choice |
|---|---|
| Removed | deleted (`ON DELETE CASCADE`) |
| Renamed (other than case/spacing) | dropped, in the edit's transaction |
| Chips changed | dropped |
| Quantity or note changed | kept — re-priced by the order screen |

The rule is `services/selections/staleness.rs::choice_still_fits`.

## Data
`item_selections` (migration `0004`; `20261003070000` allows one row per
store, `UNIQUE (grocery_item_id, store)`, and adds `for_order` with a partial
unique index so exactly one row per item is bought). Columns: store, product_id, product_name, brand,
package_size, price, unit_price + unit_price_per, total_price,
priced_quantity, url, selected_by, selected_at. Prices are unscaled `NUMERIC`.

## Code
- Backend: `routes/selections.rs`; `services/selections/` — `mod.rs` (rules),
  `offer.rs` (find the product in a search), `staleness.rs`, `repository.rs`
  (all SQL for the table), `log.rs`; `models/schemas/selections.rs`.
- Web app: `lib/api/selections.ts`; `components/products/product-choice.tsx`
  (context scoped to one `<PriceComparison>`),
  `choose-product-button.tsx`; `components/selections/chosen-product.tsx`;
  `hooks/useProductChoices.ts`. The `/` loader reads the list and the choices
  together.

## Not built yet
- Delivery windows and split-store totals on the order screen
  (`FEATURE_ORDER_OPTIMISATION.md`). Re-pricing is built.
- Nothing left from the spec's "Purchase History and Preferences":
  past purchases (`FEATURE_PURCHASE_HISTORY.md`) and dislike warnings
  (`FEATURE_DISLIKES.md`) are built.
- Live update of choices between open tabs (the list does not sync live yet
  either).

## Tests
- `backend/tests/item_selections.rs`, `item_selections_after_edits.rs`:
  the routes over the fake stores. Unit tests in `offer.rs`, `staleness.rs`.
- `backend/tests/cors.rs`: every method the API serves is allowed
  cross-origin (`PUT` was missing and blocked the browser).
- `frontend/src/components/**/__tests__`: button, chosen panel, sheet, card.
- `frontend/e2e/product-choice.spec.ts`: desktop and phone.
