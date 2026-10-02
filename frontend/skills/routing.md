# TanStack Router / Start patterns

## This is TanStack Start (SSR), not a static SPA
- No `index.html`, no `src/main.tsx`. Vite's `tanstackStart()` + `nitro()` plugins
  (see `vite.config.ts`) generate the client and server entries and a real
  Node HTTP server.
- `src/router.tsx` exports `getRouter()` — called per-request on the server
  and once on the client for hydration. Never treat it as a singleton.
- `src/routes/__root.tsx` is the `shellComponent`: it owns `<html>`, `<head>`
  (via `HeadContent`) and `<body>` (ending with `Scripts`). Every other
  route only returns its own content — never its own `<html>`.
- Dev: `pnpm dev` (Vite dev server, SSR on). Build: `pnpm build` → `.output/`.
  Run the built server with `pnpm start` (`node .output/server/index.mjs`).
  `dist/` is the raw Vite build output — not what you run in production.

## Adding a route
File-based routing under `src/routes/`. After adding/removing a route file,
run `pnpm generate-routes` (or just `pnpm dev`, which does it automatically)
to refresh `src/routeTree.gen.ts` — never hand-edit that file.

```tsx
export const Route = createFileRoute('/pending')({
  loader: () => api.voice.pending(),   // runs on the server for the initial
                                        // request, then client-side on nav
  component: PendingPage,
})
```

## Search params
Validate them on the route and give the page a default:
`validateSearch: (s) => (s.tab === "rejected" ? { tab: "rejected" } : {})`,
then `const { tab = "held" } = Route.useSearch()` (see `routes/triage.tsx`).
A toast or link can then open a tab directly.

## Mutations
A route's loader is the single source of truth. Mutate through `lib/api`, then
`await router.invalidate()` — inside the same `try`, so an invalidation still
in flight when the user navigates away cannot reject unobserved (that shows up
as an unhandled "Failed to fetch"). Do not mirror loader data into `useState`
and patch it by hand; local state is for drafts only.

## Hydration
`#app` carries `data-hydrated` once the root has mounted (`hooks/useHydrated`).
SSR means the markup is interactive-looking before React attaches a handler,
and anything typed or clicked in that window is lost. The e2e helper
`gotoList()` waits on this attribute; in dev tools it tells you whether a dead
click was a hydration race or a real bug.

## Navigation
```tsx
import { Link, useNavigate } from '@tanstack/react-router'
```
`Link`'s `[&.active]` Tailwind selector (see `components/nav.tsx`) is how
the current route gets nav-highlighted — no manual `usePathname` check.

## Real-time (WebSocket) + toasts
`lib/ws.ts` is a client-only singleton socket (guarded on `typeof window`).
Each event carries a fresh count: `voice_request_added` (Pending Requests) and
`triage_held` (held for review). Two generic hooks `subscribe()` to it:
- `useLiveCount(type, load)` — a nav badge: loads once, then follows pushes
  (`usePendingCount`, `useHeldCount`). Give it a module-level `load`.
- `useCountRiseToast(type, onRise)` — fires only when the count *rises* (a
  new item), not on every accept/reject (`useVoiceRequestToasts`,
  `useTriageToasts`). Mount the toast hooks once, at the root.

A new badge or toast is a two-line hook on top of these, plus a new event
type in `lib/ws.ts` and `ServerEvent` in `backend/src/services/ws_hub.rs`.
