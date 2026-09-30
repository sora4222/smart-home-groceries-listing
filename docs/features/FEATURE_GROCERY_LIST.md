# Feature: Grocery List

Status: **implemented** — read, add, review, annotate, remove and commit.
Product search, pricing and the order itself are the Store Integration and
Order Optimisation features.

## What this feature does
The household's shared list, and everything done to it before anybody spends
money. A member opens the main page, sees what is on the list, adds what is
missing, corrects a name or a quantity, leaves a note for whoever does the
shopping, narrows an item with filter chips, and finally commits the list for
purchase.

Committing is the line between the two halves of the application. Up to that
point the list is a draft anybody can change; after it, the items are locked
until somebody releases them.

## The list

| Status | Meaning |
|---|---|
| `active` | on the list and open to changes |
| `committed` | reviewed and locked in for purchase |
| `ordered` | bought — purchase history, not this feature |
| `pending` | unused on `grocery_items`; intake requests hold that state |

`GET /api/grocery-items` returns `active` and `committed` together, newest
first; the web app groups them.

## Endpoints
All require a Clerk JWT. Bodies are validated before they reach a service, so
a value outside its range is `422` and never touches a row.

- `GET /api/grocery-items` — the list.
- `POST /api/grocery-items` — add by hand. Body
  `{"name": str, "quantity": int = 1, "note": str?, "filter_terms": [str]?}`.
  `201` with the item. Source is `manual`; items added by accepting an intake
  request are `voice` (see `FEATURE_VOICE.md`).
- `PATCH /api/grocery-items/{id}` — every field optional; an absent field is
  left as it was. `409` if the item is committed, `404` if it does not exist.
- `DELETE /api/grocery-items/{id}` — `204`. `409` if committed.
- `POST /api/grocery-items/commit` — every `active` item becomes `committed`.
- `POST /api/grocery-items/release` — every `committed` item becomes `active`.

### Duplicate handling
Adding a name that an **active** item already covers answers `409` with a
`DuplicateItemWarning` in `detail`, and the web app asks the spec's question:
*"[Item] is already on the list. Add another or update the existing
quantity?"*. `?on_duplicate=` says what to do instead of asking:

| Value | Behaviour |
|---|---|
| `ask` (default) | `409` with the clashing item |
| `merge` | add to the existing quantity, capped at 999 |
| `separate` | keep both entries |

`separate` is what lets one name carry two annotations — "full cream" and
"oat, for Sam" — rather than one ambiguous line.

Names match case- and whitespace-insensitively through
`lower(btrim(regexp_replace(name, '\s+', ' ', 'g')))`, which is also the
expression behind `ix_grocery_items_normalised_name`. The lookup takes
`SELECT ... FOR UPDATE`, so two tabs adding the same item cannot both win.

Detection is scoped to `active` on purpose: a committed item is on its way
into an order and must not absorb a new request, so the same name can start
the next list.

## Annotation

**Note** — free text, up to 500 characters, for whatever the list cannot say
on its own ("the recycled one, not the bamboo"). Sending `""` clears it;
leaving the field out keeps it.

**Filter chips** (`filter_terms`) — terms that will narrow the product search:
`3 ply`, `organic`. At most 10 per item, 60 characters each. The service trims
them, drops blanks and collapses case-insensitive repeats, so `["3 Ply", " 3
ply "]` stores one chip.

A chip is replaced wholesale: to remove one, send the ones that remain. That
is what the × on a chip does, from the list itself — the spec requires a chip
to be overridable here without editing the rule that put it there. The Item
Rules feature, which will apply chips automatically, is not built.

## Commit and release
`commit` and `release` each move every matching row in one statement, so two
tabs pressing the button together cannot half-commit the list. Both are
idempotent: committing an empty list, or committing twice, changes nothing
rather than failing.

While a list is committed, new items can still be added — they start the next
list as `active`, and the page shows both the locked banner and the commit
button.

## Where the code is

| Piece | File |
|---|---|
| Routes | `backend/src/routes/grocery.rs` |
| Rules | `backend/src/services/grocery/mod.rs` |
| SQL | `backend/src/services/grocery/repository.rs` |
| Row type | `backend/src/models/db.rs` (`GroceryItem`) |
| Bodies | `backend/src/models/schemas.rs` |
| Migration | `backend/migrations/0002_grocery_item_notes_filters_and_commit.sql` |
| Backend tests | `backend/tests/grocery_items.rs`, `grocery_commit.rs` |
| Page | `frontend/src/routes/index.tsx` |
| Components | `frontend/src/components/grocery/` |
| API client | `frontend/src/lib/api.ts` |
| E2e | `frontend/e2e/grocery-list.spec.ts`, `grocery-list-usability.spec.ts` |

All `grocery_items` SQL lives in the grocery repository, including the
statements the intake queue uses when it accepts a request: the table belongs
to the list, not to the channel that fed it, and the normalising expression
duplicate detection depends on then has one home.

## Frontend notes
The page reads from its loader and calls `router.invalidate()` after every
mutation, so the loader stays the single source of truth and two open tabs
cannot drift apart. Drafts — a half-typed item, an open editor — are local
state, so cancelling changes nothing.

Quantity fields hold what was typed rather than a number; coercing on every
keystroke snapped an emptied field back to `1` and made the next digit append
to it. `frontend/src/lib/quantity.ts` turns that string into a bounded number
on blur and on submit.

## Known gaps / next steps
- **Item Rules are not built.** Chips are added by hand; the rules that would
  apply them on their own, and the `/settings/item-rules` page, are a feature
  of their own.
- Committing does not yet hand anything to an order — `/order` does not exist.
  `committed` is the state that screen will read.
- No per-item ownership: any household member can change or remove any item,
  which matches the spec's "no admin roles in this POC".
- The list does not update live between tabs. Intake events already have a
  WebSocket; list changes could reuse it.
