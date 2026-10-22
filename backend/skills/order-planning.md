# Order planning (delivery fees, modes, store mix)

Spec: `docs/features/FEATURE_ORDER_OPTIMISATION.md`.

## Rules to keep
- **Never pick a product.** The planner only moves an item between the
  products the household chose, one per store (`item_selections`, unique
  `(grocery_item_id, store)`). Exactly one row per item has `for_order`
  (partial unique index); everything that buys reads only that row
  (`selections::repository::list_for_order`, the handoff's
  `chosen_lines_for_store`). A new reader of "the chosen product" must
  filter `for_order` too.
- Switching stores is `SelectionService::buy_at` (all or nothing, 422).
  Clear `for_order` on the other row **before** setting it, or the partial
  index refuses the update.
- Fees come only from Settings › Delivery (`services/delivery_settings`).
  An unset fee is $0 with `fee_known: false`, never a guessed number.

## Where things are
| Piece | File |
|---|---|
| Fee for one store and subtotal (pure) | `services/order_plan/fees.rs` |
| One costed option | `services/order_plan/option.rs` |
| Best mix: exact ≤ 16 flexible items, else local search | `services/order_plan/search.rs` |
| Mode ordering | `services/order_plan/rank.rs` |
| The four options, de-duplicated | `services/order_plan/build.rs` |
| Re-pricing (shared with the review) | `services/order_review::reprice` |

## Adding a mode
1. `OrderMode` in `models/delivery_rows.rs` + the `CHECK` in a new migration.
2. Its ordering in `rank::compare` / `rank::preferred_kind`.
3. `frontend/src/lib/order-modes.ts` (label, description).
4. A case in `services/order_plan/tests_modes.rs`.

## Adding a store
`Store::ALL` drives every loop here; add its `FeeRules` row (no migration
beyond the `CHECK`), and an `OptionKind::Only(store)` label in
`models/schemas/order_plan.rs`.
