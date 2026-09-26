# Frontend Agent Guide

## Stack
React 18 · TanStack Router (file-based) · Shadcn/ui · Radix UI · Tailwind CSS using postcss · TypeScript · pnpm

## Structure
```
src/
├── routes/              # One file per route. Root layout: __root.tsx
│   └── settings/        # Nested routes → /settings/*
├── components/
│   ├── ui/              # Shadcn generated — do not edit directly
│   └── <feature>/       # Compound components per feature area
├── lib/
│   ├── api.ts           # Typed fetch wrappers for all backend endpoints
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
| `routes/order.tsx` | `/order` — Order review |
| `routes/analysis.tsx` | `/analysis` — Spending analysis |
| `routes/settings/index.tsx` | `/settings` |
| `routes/settings/item-rules.tsx` | `/settings/item-rules` |
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
// Events: { type: 'voice_request_added', count: number }
// On event: fire Sonner toast + update pending badge count
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
