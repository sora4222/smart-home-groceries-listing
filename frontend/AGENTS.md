# Frontend Agent Guide

<!-- intent-skills:start -->
## Skill Loading

Before editing files for a substantial task:
- Run `pnpm dlx @tanstack/intent@latest list` from the workspace root to see available local skills.
- If a listed skill matches the task, run `pnpm dlx @tanstack/intent@latest load <package>#<skill>` before changing files.
- Use the loaded `SKILL.md` guidance while making the change.
- Monorepos: when working across packages, run the skill check from the workspace root and prefer the local skill for the package being changed.
- Multiple matches: prefer the most specific local skill for the package or concern you are changing; load additional skills only when the task spans multiple packages or concerns.
<!-- intent-skills:end -->

## Stack
React 19 · TanStack Start (SSR) · TanStack Router (file-based) · Shadcn/ui · Radix UI · Tailwind CSS v4 · TypeScript · pnpm

## Backend contract
The backend is **Rust + Axum** (see `backend/AGENTS.md`). It was rewritten from
Python + FastAPI without changing the wire format: same paths, same field
names, same status codes, and errors still arrive as `{ "detail": ... }` —
a string for most failures, an object for the duplicate-item 409. Nothing in
this directory changed because of the rewrite.

Prices from `GET /api/grocery-items/{id}/products` and
`GET /api/item-selections` are decimal **strings** (`"4.95"`); format them
with `lib/money.ts`. Choosing a product sends only `{ store, product_id }`
(`PUT /api/grocery-items/{id}/selection`); the backend takes the price from
the store.

`VoiceRequest` carries a `source` field (`"webhook" | "alexa" | "tasks"`) saying which
intake channel delivered the item, and `triage_status`, `triage_reason` and
`triage_confidence` from the LLM triage step (`docs/features/FEATURE_TRIAGE.md`).

Playwright is a **frontend-only** tool here. The backend's store automation
uses `chromiumoxide`; do not add Playwright to anything outside `frontend/`.

## Structure
```
src/
├── routes/              # One file per route. Root layout: __root.tsx
│   └── settings/        # Nested routes → /settings/*
├── components/
│   ├── ui/              # Shadcn generated — do not edit directly
│   └── <feature>/       # Compound components per feature area
├── lib/
│   ├── api.ts           # Gathers api/* into `api` and re-exports their types
│   ├── api/             # client.ts (request, ApiError) + one module per domain
│   └── ws.ts            # WebSocket client (pending request events)
└── hooks/               # Custom hooks (useWebSocket, usePendingCount, etc.)
```

## Routes (TanStack Router)
File-based routing — one file per route:
| File | Route |
|---|---|
| `routes/__root.tsx` | Root layout (nav, Sonner toaster) |
| `routes/index.tsx` | `/` — Grocery list |
| `routes/pending.tsx` | `/pending` — Confirmation queue |
| `routes/triage.tsx` | `/triage` — Held for review / Rejected tabs (`?tab=rejected`) |
| `routes/order.tsx` | `/order` — Order review |
| `routes/analysis.tsx` | `/analysis` — Spending analysis |
| `routes/settings/index.tsx` | `/settings` |
| `routes/settings/item-rules.tsx` | `/settings/item-rules` |
| `routes/settings/dislikes.tsx` | `/settings/dislikes` — every member's dislikes |
| `routes/settings/intake.tsx` | `/settings/intake` — Google Tasks connect + other channels; Google's sign-in returns here |
| `routes/settings/stores.tsx` | `/settings/stores` |
| `routes/settings/delivery.tsx` | `/settings/delivery` |
| `routes/logs.tsx` | `/logs` |

```typescript
// Navigation
import { Link, useNavigate } from '@tanstack/react-router'

// Params
import { useParams } from '@tanstack/react-router'
```

## Components
- Use Shadcn components from `components/ui/` — run `pnpm dlx shadcn@latest add <component>` to add new ones. Do not modify ui/ files.
  If `ui.shadcn.com` is blocked, copy the component's new-york-v4 source by hand (that is how `ui/select.tsx` arrived).
- Radix Select/Switch need jsdom stand-ins (`hasPointerCapture`, `scrollIntoView`, `ResizeObserver`) — they are in `src/test-setup.ts`.
- Feature UI uses compound components:
  ```tsx
  <GroceryList>
    <GroceryList.Item name="Milk" quantity={2} />
    <GroceryList.Actions onAdd={...} />
  </GroceryList>
  ```
- Context only within a compound component tree — not globally
- Limit props to what the component directly uses

## Tailwind
- Use theme tokens only: `bg-background`, `text-foreground`, `border-border`, `text-muted-foreground`, etc.
- Do not add custom values to `tailwind.config.ts`
- Layer order: base → components → utilities

## Responsive layout
- Desktop: top nav bar
- Mobile (< 768px): bottom tab bar or hamburger menu
- Tables → horizontal scroll or card stack on mobile
- Wide action buttons → full-width or `...` dropdown on mobile
- Test at 390px (iPhone) and 1280px (desktop)

## WebSocket (real-time)
```typescript
// lib/ws.ts — connect on app mount, reconnect on disconnect
// Events: { type: 'voice_request_added', count } — Pending Requests badge
//         { type: 'triage_held', count }         — Triage badge
// Badges: hooks/useLiveCount (usePendingCount, useHeldCount)
// Toasts: hooks/useCountRiseToast — fires only when a count rises
```

## API client pattern
```typescript
// lib/api.ts
const api = {
  grocery: {
    list: () => fetch('/api/grocery-items').then(r => r.json()),
    add: (item: NewItem) => fetch('/api/grocery-items', { method: 'POST', body: JSON.stringify(item) }),
  },
  voice: {
    pending: () => fetch('/api/voice-requests?status=pending').then(r => r.json()),
    accept: (id: string) => fetch(`/api/voice-requests/${id}/accept`, { method: 'POST' }),
    reject: (id: string) => fetch(`/api/voice-requests/${id}/reject`, { method: 'POST' }),
  },
}
```

## Testing
- Unit: `vitest` + `@testing-library/react` in `src/**/*.test.tsx`
- E2e: `@playwright/test` in `e2e/`
- See root `skills/testing.md`

## Skills in this directory
- `skills/routing.md` — TanStack Router patterns
- `skills/components.md` — Shadcn compound component patterns
