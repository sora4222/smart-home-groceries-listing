# Dislikes (backend)

Spec and endpoints: `docs/features/FEATURE_DISLIKES.md`.

## The three ideas
- **Dislike** — one member, one store product (`store` + store `product_id`).
  Table `product_dislikes`, repository `services/dislikes/repository.rs`.
- **Override for this order** — one list item, one product. Table
  `dislike_overrides`, `ON DELETE CASCADE` from the item, repository
  `overrides_repository.rs`. It never deletes a dislike.
- **Override for good** — the member deletes their own dislike
  (`DislikeService::remove_mine`). Never another member's.

## Asking "may the automated order buy this?"
Use the pure rule in `services/dislikes/skip.rs`; do not write SQL for it.

1. `DislikeService::book_for_item(item_id)` → `DislikeBook` (all dislikes +
   that item's overrides).
2. Rank your candidates best first; implement `CandidateProduct` for them.
3. `pick_for_automated_order(&candidates, &book, scope)`:
   - `DislikeScope::Household` — any member's dislike skips;
   - `DislikeScope::Member(user_id)` — only theirs;
   - `OnlyDisliked { candidate, disliked_by }` — every candidate was
     disliked: buy `candidate`, and the order screen must warn first.

Manual choice (`SelectionService::choose`) never consults dislikes: the spec
lets a person choose a disliked product.

## Names on the badge
`member_name.rs` turns the token's email into the name others see. It is
saved on the row (`user_name`) because the token has no display name. If
the auth provider later sends a name, change only that file (and the
`Member` the route builds).

## Tests
- Pure rule: `skip_tests.rs` (`#[path]` from `skip.rs`).
- Routes: `tests/product_dislikes.rs`, `tests/dislike_overrides.rs`. The
  router is always the dev user, so another member's dislike is written with
  `repository::upsert` straight on the pool.
