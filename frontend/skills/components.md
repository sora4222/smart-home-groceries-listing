# Shadcn-pattern compound components

## UI primitives (`components/ui/`)
Hand-built to match shadcn/ui's own generated output (cva variants + `cn()`
from `lib/utils.ts`) rather than run via the shadcn CLI, since this repo has
no network access to `ui.shadcn.com` guaranteed at agent-run time. If you
add a new primitive, follow the same shape: `cva` for variants, `cn()` for
class merging, `data-slot` attributes, no local component state.
Do not edit `components/ui/*` files ad hoc — extend the `*Variants` export.

## Feature components are compound, not context-heavy
`components/pending/pending-request-card.tsx` is the current example: all
inputs (edited name/quantity) are local `useState`, and the card reports
decisions up via `onAccept`/`onReject` callback props — no context provider
for a single card. Reach for React context only when three or more levels
need the same value; prefer prop drilling one or two levels first.

## Theme tokens
`src/styles.css` defines the full shadcn CSS-variable palette (light +
`.dark`) mapped into Tailwind v4 via `@theme inline`. Always use the token
classes (`bg-background`, `text-muted-foreground`, `border-border`, …) —
never a raw hex/oklch value or an ad-hoc Tailwind color in a component.
