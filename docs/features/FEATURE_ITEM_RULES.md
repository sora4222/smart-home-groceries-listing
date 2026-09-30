# Feature: Item Rules

Status: **implemented** — manage rules on `/settings/item-rules`; rules put
filter chips on items as they reach the list.

## What this feature does
A rule attaches persistent filters to grocery item names. The spec's example:
a rule for "toilet paper" with the filter "3 ply" means that when someone says
"Hey Google, add toilet paper", the item lands on the list already chipped
"3 ply" — the later product search is narrowed without anyone typing it.

## A rule

| Field | Meaning |
|---|---|
| `triggers` | Item names or phrases to match. 1–10, each ≤ 100 chars. |
| `filter_terms` | Chips copied onto a matching item. 1–10, each ≤ 60 chars. |
| `apply_to_manual` | Also apply to items typed into the web app. **Off by default.** |

Triggers are stored whitespace-collapsed and de-duplicated
case-insensitively, keeping the first spelling. Filter terms go through
`services::filter_terms::clean`, the same tidying an item's own chips get. A
rule that tidies down to no trigger or no term is a `422`.

## Matching
A trigger matches when its words appear together and in order among the item
name's words, ignoring case and punctuation (`services/item_rules/matching.rs`):

| Trigger | Item name | Match |
|---|---|---|
| `toilet paper` | `Quilton Toilet-Paper` | yes |
| `toilet paper` | `paper towel` | no — the phrase is not there |
| `milk` | `milkshake` | no — part of a word |
| `egg` | `eggs` | no — no stemming; give the rule both triggers |

Every matching rule contributes, in the order the rules were made; repeated
terms keep their first spelling, and the total is capped at the item's 10-chip
limit.

## When rules apply

| How the item arrives | Rules consulted | Order of chips |
|---|---|---|
| Voice request accepted as a new entry | every rule | rule chips |
| Typed into the web app, new entry (incl. `on_duplicate=separate`) | only `apply_to_manual` rules | typed chips, then rule chips |
| Either path merging into an existing item | none | the existing item's chips, untouched |

The name matched is the one filed: for voice, the household's correction if
they made one. Rules are read inside the transaction that inserts the item.

**Chips are copied, not linked.** Removing a chip from an item never edits the
rule (the spec's "remove or override a filter chip directly on the grocery
list"), and editing or deleting a rule never changes an item already on the
list. A merge leaves chips alone so a chip somebody took off does not come
back with the next request.

## Endpoints
All require a Clerk JWT.

- `GET /api/item-rules` — every rule, oldest first.
- `POST /api/item-rules` — `{"triggers": [str], "filter_terms": [str],
  "apply_to_manual": bool = false}`. `201`.
- `PATCH /api/item-rules/{id}` — every field optional; a list that is sent
  replaces the stored one. `404` if missing, `422` if emptied.
- `DELETE /api/item-rules/{id}` — `204`, `404` if missing.

## Web app
`/settings/item-rules`, linked from the nav as "Item Rules": an add form, then
every rule as a card with inline Edit and Delete (Delete asks first). The
manual-additions switch is part of the rule's form, not a global setting.

When an item is added or accepted and rules gave it chips, the success toast
says so: "Item rules added the filter: 3 ply".

## Logging
`item rule saved` (created/updated) and `item rule deleted` from
`item_rules/log.rs`; `item rules matched` from `item_rules/apply.rs` with the
item, the path (`Voice`/`Manual`), the rule ids and the terms. The item-added
and request-accepted events carry the resulting `filter_terms`.

## Where the code is

| Piece | File |
|---|---|
| Migration | `backend/migrations/0003_item_rules.sql` |
| Row type | `backend/src/models/db.rs` (`ItemRule`) |
| Bodies | `backend/src/models/schemas/item_rules.rs` |
| Routes | `backend/src/routes/item_rules.rs` |
| CRUD rules | `backend/src/services/item_rules/mod.rs` |
| Trigger tidying | `backend/src/services/item_rules/triggers.rs` |
| Matcher | `backend/src/services/item_rules/matching.rs` |
| Applying | `backend/src/services/item_rules/apply.rs` |
| SQL | `backend/src/services/item_rules/repository.rs` |
| Backend tests | `backend/tests/item_rules.rs`, `item_rules_applied.rs` |
| Page | `frontend/src/routes/settings/item-rules.tsx` |
| Components | `frontend/src/components/item-rules/`, `components/terms/` |
| API client | `frontend/src/lib/api/item-rules.ts` |
| E2e | `frontend/e2e/item-rules.spec.ts` |

## Known gaps / next steps
- The chips narrow a product search that does not exist yet — that is the
  Store Integration feature.
- No stemming or synonyms: "egg" and "eggs" need two triggers.
- Rules are not re-applied to items already on the list when a rule is added.
- No `/settings` index page yet; the nav links straight to Item Rules.
