# Feature: Order Optimisation

Status: **first part built — the order review (`/order`).** It shows what the
committed list costs today, store by store. Optimisation modes, split-store
suggestions, delivery fees and delivery constraints are **not built**.

## What the order review does
1. On the grocery list, **Ready to order** commits the list.
2. **Order** in the nav opens `/order`. For every committed item:
   - with a chosen product → the store is asked again, **only the chosen
     store**, at the item's **current** quantity (deals applied);
   - with no product → listed under **No product chosen yet**, with a link
     back to the list.
3. Lines are grouped by store (Woolworths first), each store with a
   **Subtotal**, then a **Total for items**. Delivery fees are not included.
4. A line shows **Up from $X** / **Down from $X** when the shelf price for one
   moved since it was chosen, and **Deal applied** when a multibuy lowers it.
5. A line that cannot be bought as it is shows **No price** and why. It is not
   counted, and the subtotal and total read **so far**.
6. **Check prices again** re-runs the review. **Send to Woolworths** sits in
   the Woolworths card (`FEATURE_TROLLEY_HANDOFF.md`).

Nothing here picks a product or a store for the household. It only reports
what was chosen and what it costs now (project goal: less marketing-driven
buying).

## Endpoint

| Method | Path | Auth | Answer |
|---|---|---|---|
| `GET` | `/api/order-review` | Clerk session | 200 the review; 401 |

A store that cannot be asked does **not** fail the request: its lines carry
`status: "store_failed"` and a `problem` sentence.

```json
{
  "stores": [{
    "store": "coles", "store_name": "Coles", "subtotal": "8.85", "complete": true,
    "lines": [{
      "grocery_item_id": "…", "item_name": "oat milk", "quantity": 2,
      "product_id": "c-oat-1l", "product_name": "Barista Oat Milk",
      "brand": "Minor Figures", "package_size": "1L", "url": "…",
      "status": "priced", "price": "3.90", "total_price": "6.50",
      "deal_applied": true, "price_change": "same",
      "chosen_price": "3.90", "chosen_total_price": "3.90",
      "chosen_priced_quantity": 1, "problem": null
    }]
  }],
  "unchosen": [{ "grocery_item_id": "…", "name": "bread", "quantity": 2 }],
  "total": "8.85",
  "complete": false
}
```

| `status` | Meaning | Counted |
|---|---|---|
| `priced` | still sold; today's price | yes (if the store shows a price) |
| `unavailable` | listed but cannot be delivered now | no |
| `not_offered` | no longer in the store's results for the item and its chips | no |
| `store_failed` | the store could not be asked | no |

`price_change`: `same` · `up` · `down` · `null` (a price unknown).
`complete`: every item has a product **and** every line has a price.
Money is a decimal string.

## Rules
- **Only committed items.** Active items are still being reviewed on the list.
- **Current quantity.** The choice's `total_price` was for
  `priced_quantity`; the review prices the item's quantity now.
- **One store per item, one item at a time.** Items are re-priced in turn,
  each only at its chosen store, so the stores see a household's pace. Most
  answers come from the 10-minute search cache.
- **Send count is honest.** The Send button counts active and committed items
  chosen at Woolworths, because that is what a handoff sends
  (`lib/chosen-at-store.ts`, shared with the list page).

## Code
- Backend: `routes/order_review.rs`; `services/order_review/` — `mod.rs`
  (service), `line/` (pure re-pricing of one choice; `tests.rs`,
  `fixtures.rs`), `summary.rs` (pure grouping and totals), `log.rs`;
  `models/schemas/order_review.rs`; `ProductSearchService::search_at` (one
  store for one item).
- Frontend: `routes/order.tsx`; `components/order/` — `OrderReview`
  compound (`.Empty`, `.Unchosen`, `.Store`, `.Total`), `StoreOrderCard`,
  `OrderLineRow`, `PriceChangeBadge`; `lib/api/order-review.ts`,
  `lib/price-change.ts`, `lib/chosen-at-store.ts`.

## Logs
`order reviewed` (stores, lines, unchosen, problems, total, complete) on every
review; `order line needs attention` (warn) for a line with a problem;
`chosen product's price changed` when a shelf price moved.

## Not built (spec "Order Optimisation")
- Optimisation modes (minimise total, minimise delivery, single store, manual)
  and the saved default.
- Split-store comparison and ranked options.
- Delivery windows and fees in the app, and the 15-minute hold. The app cannot
  see them without the household's store login (`FEAT_WOOLWORTHS_ACCESS.md`).
- Delivery constraints from Settings › Delivery (need-by, max delivery spend,
  allowed days, perishable timing).
- Dislike warnings (spec "Purchase History and Preferences").

## Tests
- `backend/src/services/order_review/line/tests.rs`, `summary.rs` — every
  line outcome, grouping, totals.
- `backend/tests/order_review.rs` — through the real router over the fake
  stores: empty, grouped, current quantity, unchosen, price change, product
  gone, store down, auth.
- `frontend/src/components/order/__tests__/`, `lib/__tests__/price-change.test.ts`,
  `lib/__tests__/chosen-at-store.test.ts`.
- `frontend/e2e/order-review.spec.ts` — desktop and phone.
