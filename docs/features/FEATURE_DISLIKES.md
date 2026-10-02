# Feature: Disliked Products

Status: **implemented** for manual choice. The automated order (the
optimiser) is not built yet; it has a ready function to call
(`services/dislikes/skip.rs`). Spec: "Purchase History and Preferences" ›
"Disliked items" and "Per-user vs household preferences".

How a person uses it: [`using-the-app.md` › Dislike a product](../using-the-app.md#dislike-a-product).

## What this feature does
- **Per member.** Each household member has their own dislikes. A dislike
  is one store product (`store` + the store's `product_id`).
- **Household view.** Everyone sees everyone's dislikes:
  - in the price comparison, a disliked product shows *"Phu disliked this
    item previously."* (*"You …"* for your own);
  - **Dislikes** (`/settings/dislikes`) lists every member's dislikes, yours
    first.
- **Manual choice is never blocked.** A disliked product keeps its
  **Choose** button. The warning is all that changes.
- **Override for this order.** **Buy it this time** on the warning sets the
  dislike aside for that one list item. The dislike itself stays. **Undo**
  brings it back.
- **Override for good.** **Remove my dislike** (in the comparison or on the
  Dislikes page) deletes your own dislike. Nobody can remove another
  member's dislike.
- **Nothing is suggested.** A dislike only warns or skips; it never offers
  another product instead (project goal: less marketing-driven buying).

## Rules for the automated order (for the optimiser)

| Scope | Skips |
|---|---|
| `DislikeScope::Member(user_id)` | only that member's dislikes |
| `DislikeScope::Household` | any member's dislike |
| an override on the item | nothing — the dislike is set aside |

If every candidate is disliked, the best-ranked one is bought anyway and the
order must show who disliked it before it is confirmed
(`AutomatedPick::OnlyDisliked`).

```rust
let book = DislikeService::new(&pool).book_for_item(item_id).await?;
match skip::pick_for_automated_order(&ranked, &book, DislikeScope::Household) {
    AutomatedPick::Allowed(product) => { /* buy it */ }
    AutomatedPick::OnlyDisliked { candidate, disliked_by } => { /* buy, warn first */ }
    AutomatedPick::NoCandidates => { /* nothing to buy */ }
}
```

Candidates implement `skip::CandidateProduct` (`store()`, `product_id()`).

## Endpoints

| Method | Path | Answer |
|---|---|---|
| `GET` | `/api/product-dislikes` | 200, every member's dislikes, newest first |
| `PUT` | `/api/product-dislikes` | 200 the saved dislike (also when it existed) |
| `DELETE` | `/api/product-dislikes/{store}/{product_id}` | 204 your own dislike removed (also when none) |
| `GET` | `/api/dislike-overrides` | 200, every override on the list |
| `PUT` | `/api/grocery-items/{id}/dislike-override` | 200 the override |
| `DELETE` | `/api/grocery-items/{id}/dislike-override/{store}/{product_id}` | 204 (also when none) |

All need a Clerk session (401 without).

`PUT /api/product-dislikes` body:
`{ "store", "product_id", "name", "brand"?, "package_size"? }`. The name,
brand and size are only a label for the Dislikes page, as the comparison
showed them. 422: unknown store, empty id or name, or too long.

A dislike:

```json
{
  "store": "coles", "store_name": "Coles", "product_id": "c-milk-3l",
  "name": "Full Cream Milk", "brand": "Coles", "package_size": "3L",
  "user_id": "user_…", "user_name": "phu", "mine": false,
  "disliked_at": "2026-10-02T05:00:00Z"
}
```

`PUT …/dislike-override` body: `{ "store", "product_id" }`. 404 unknown
item; 409 the item is ordered; 422 nobody dislikes that product.

## The member's name
The session token has an id and maybe an email, not a display name.
`user_name` is the part of the email before `@`, or *"A household member"*
when there is no email (`services/dislikes/member_name.rs`). It is saved
with the dislike. When Clerk sends a real name, change only that file.

## Data
Migration `0009_product_dislikes.sql`:
- `product_dislikes` — one row per member per product
  (`UNIQUE (user_id, store, product_id)`). Disliking again keeps the date
  and refreshes the label.
- `dislike_overrides` — one row per item per product, `ON DELETE CASCADE`
  from `grocery_items`. An override lives as long as the item: the next
  time the item is added it is a new row, so the dislike counts again.

## Code
- Backend: `routes/dislikes.rs`; `services/dislikes/` — `mod.rs` (rules),
  `skip.rs` (pure, for the optimiser; tests in `skip_tests.rs`),
  `member_name.rs`, `repository.rs`, `overrides_repository.rs`, `log.rs`;
  `models/schemas/dislikes.rs`; `models/dislike_rows.rs`.
- Web app: `lib/api/dislikes.ts`, `lib/dislikes.ts` (pure: who dislikes it,
  the warning sentence, grouping); `hooks/useItemDislikes.ts`;
  `components/dislikes/` — `product-dislikes.tsx` (context scoped to one
  `<PriceComparison>`), `dislike-button.tsx`, `dislike-notice.tsx`,
  `household-dislikes.tsx`; `routes/settings/dislikes.tsx`.

## Logs
`product disliked`, `product dislike removed`, `dislike overridden for this
order`, `dislike override cleared`, `dislike override refused` (warn).

## Not built yet
- The optimiser itself, and so the "warn before the order is confirmed"
  screen (`FEATURE_ORDER_OPTIMISATION.md`).
- Dislike badges on the order review and on the chosen product on the list.
- Purchase history in the comparison (spec "Purchase History").

## Tests
- `backend/src/services/dislikes/skip_tests.rs` — every scope and override
  rule, and the all-disliked fallback.
- `backend/tests/product_dislikes.rs`, `backend/tests/dislike_overrides.rs` —
  the routes, the household view with another member, and the book.
- `frontend/src/lib/__tests__/dislikes.test.ts`,
  `frontend/src/components/dislikes/__tests__/`.
- `frontend/e2e/dislikes.spec.ts` — desktop and phone.
