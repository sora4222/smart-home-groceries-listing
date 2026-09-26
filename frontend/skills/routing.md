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

## Navigation
```tsx
import { Link, useNavigate } from '@tanstack/react-router'
```
`Link`'s `[&.active]` Tailwind selector (see `components/nav.tsx`) is how
the current route gets nav-highlighted — no manual `usePathname` check.

## Real-time (WebSocket) + toasts
`lib/ws.ts` is a client-only singleton socket (guarded on `typeof window`).
`hooks/usePendingCount.ts` and `hooks/useVoiceRequestToasts.ts` both
`subscribe()` to it — one drives the nav badge, the other fires the Sonner
toast only when the pending count *increases* (a new voice request), not on
every accept/reject. Mount `useVoiceRequestToasts` once, at the root.
