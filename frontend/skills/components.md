# Shadcn-pattern compound components

## UI primitives (`components/ui/`)
Hand-built to match shadcn/ui's own generated output (cva variants + `cn()`
from `lib/utils.ts`) rather than run via the shadcn CLI, since this repo has
no network access to `ui.shadcn.com` guaranteed at agent-run time. If you
add a new primitive, follow the same shape: `cva` for variants, `cn()` for
class merging, `data-slot` attributes, no local component state.
Do not edit `components/ui/*` files ad hoc — extend the `*Variants` export.

## Feature components are compound, not context-heavy
`components/grocery/` is the fullest example. `GroceryList` is the container
and carries `GroceryList.Item` and `GroceryList.Empty`; the page composes them,
so a second section (the committed items) was composition at the call site
rather than another prop. Everything below it — `AddItemForm`,
`GroceryItemCard`, `GroceryItemEditor`, `CommitBar`,
`DuplicatePrompt` — keeps its drafts in local `useState` and reports decisions
up through callback props. No context, no store.

A card is display-or-editor: `GroceryItemCard` owns the mode and hands the
whole item to `GroceryItemEditor`, which means cancelling an edit is just
throwing local state away.

`components/pending/pending-request-card.tsx` is the smaller example: all
inputs (edited name/quantity) are local `useState`, and the card reports
decisions up via `onAccept`/`onReject` callback props — no context provider
for a single card. Reach for React context only when three or more levels
need the same value; prefer prop drilling one or two levels first.

`components/item-rules/` follows the same shape: `ItemRuleList` is the
compound container, `ItemRuleCard` is display-or-editor like
`GroceryItemCard`, and `ItemRuleForm` serves both the add form (no `initial`,
clears itself after saving) and the in-place edit (`initial` + `onCancel`).

## Shared chip lists
`components/terms/` is used by more than one feature. `TermChips` renders a
list of removable chips; pass `label` for the list's accessible name and
`noun` for its remove buttons ("Remove filter 3 ply", "Remove trigger milk").
`TermListField` adds the text input, the Add button and Enter-to-add; the
chips belong to the caller and only the half-typed term is local. The rule it
applies — trim, ignore repeats in any casing, stop at 10 — is `addTerm` in
`lib/terms.ts`, matching what the backend stores.

## Shared field helpers
`lib/quantity.ts` holds the 1..999 bounds and `clampQuantity`. Quantity inputs
keep what the user typed as a string and coerce on blur and on submit —
coercing on every keystroke snaps an emptied field back to `1`, so the next
digit appends to it.

## Accessible names
Icon-only controls carry an `aria-label` that says what they act on:
`Remove filter 3 ply`, `Edit milk`, `Confirm removing milk`. The e2e suite
asserts these, so they are part of the contract rather than decoration.

`components/ui/switch.tsx` is a hand-built `role="switch"` button rather than
Radix's Switch, to avoid a dependency for one toggle. Label it with
`<label htmlFor>`.

## Theme tokens
`src/styles.css` defines the full shadcn CSS-variable palette (light +
`.dark`) mapped into Tailwind v4 via `@theme inline`. Always use the token
classes (`bg-background`, `text-muted-foreground`, `border-border`, …) —
never a raw hex/oklch value or an ad-hoc Tailwind color in a component.
