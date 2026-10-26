# Feature: Product Choice

Status: **implemented.** The household picks the products to buy for each
list item. The order review (`/order`, `FEATURE_ORDER_OPTIMISATION.md`)
reads these choices.

## Behaviour
- **Select item** on a list card opens `/items/$itemId/select`, a full page
  with both stores' products (`FEATURE_STORE_INTEGRATION.md`). Each product
  has **Choose**.
- **Several products per item.** An item with quantity 1 keeps one product
  per store; choosing again swaps it. An item with quantity N can keep up to
  N products at a store. A new product starts at 1, taken from the product
  buying the most. 409 when every product already buys 1.
- **Counts add up to the quantity.** Each product has a count. With two or
  more, the card shows a count dropdown per product. A lone product buys the
  whole quantity. Counts are corrected on read when the item's quantity
  changes (`services/selections/allocation.rs`).
- **One store is bought.** Choosing makes that store the one the order buys
  (`for_order` on every product there). The card shows the other store's
  choice as "Also at …".
- **Undo.** Choose, clear and count changes show a toast with Undo. Undo puts
  the changed store's products and counts back, then the store bought from
  (`lib/choice-undo.ts`, `lib/item-restore.ts`).
- Choosing works on a committed list. Ordered items return 409.
- Nothing is chosen for the household. No defaults.

## Endpoints

| Method | Path | Answer |
|---|---|---|
| `GET` | `/api/item-selections` | 200. For each item, the first product bought, `bought_with` (more products bought at that store), `also_chosen` (other store). Each has `quantity`. |
| `PUT` | `/api/grocery-items/{id}/selection` | 200 saved choice. Body `{store, product_id}`. 404, 409 (renamed while choosing, ordered, no room), 422 (store no longer offers it), 503 (store down) |
| `PUT` | `/api/grocery-items/{id}/selection/quantity` | 204. Body `{store, product_id, quantity}`. Others at the store share the rest. 404, 422 (no room for the others) |
| `DELETE` | `/api/grocery-items/{id}/selection[?store=[&product_id=]]` | 204. One product, one store, or everything. Clearing the bought store hands that role to the other |
| `PUT` | `/api/order-stores` `{picks: [{grocery_item_id, store}]}` | 204; 422 when an item has no choice at that store |

Only the store and product id are sent. Price and other details come from
the store's own search answer. Money is a decimal string. `total_price` is a
snapshot; the order screen re-prices before buying.

## How a choice is checked
1. The item's search runs again (usually from the 10-minute cache).
2. The product must be in that store's results and available (`offer.rs`).
3. The item row is locked. 409 if its name or chips changed. Then
   `room.rs` makes room at the store and the row is upserted on
   `(grocery_item_id, store, product_id)`.

## When a choice goes away
Removing the item deletes its choices (cascade). Renaming it or changing its
chips drops them (`staleness.rs`). Changing the quantity or note keeps them,
and the counts are allocated again on read.

## Data
`item_selections` is unique on `(grocery_item_id, store, product_id)`. It has
`for_order`, `quantity` (NULL for a lone product, added in `20261026100000`)
and the store's details (name, brand, size, price, unit price, total, url,
image_url).
`trolley_handoff_lines` is keyed by `(handoff_id, position)` so one item can
send several products.

## Code
- Backend: `routes/selections.rs`; `services/selections/` (`mod.rs` rules,
  `room.rs`, `allocation.rs`, `grouping.rs`, `parts.rs`, `offer.rs`,
  `staleness.rs`, `repository.rs`). Trolley lines: `trolley_handoffs/split.rs`.
  Planner: `order_plan/combine.rs`.
- Web app: `lib/api/selections.ts`, `components/selections/chosen-product.tsx`,
  `count-select.tsx`, `components/products/product-choice.tsx`,
  `hooks/useProductChoices.ts`, `routes/items.$itemId.select.tsx`.

## Tests
- Backend: `tests/item_selections*.rs`, `tests/item_selection_quantities.rs`,
  unit tests in `allocation.rs`, `grouping.rs`, `parts.rs`, `split.rs`.
- Frontend: `components/selections/__tests__`, `lib/__tests__/choice-undo.test.ts`.
- e2e: `product-choice.spec.ts`, `several-products.spec.ts`.
