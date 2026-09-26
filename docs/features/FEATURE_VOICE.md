# Feature: Google Home Voice Intake

Status: **implemented** (backend + web UI). Google Actions Console / Cloudflare
Tunnel wiring is a manual setup step — see `docs/human-setup.md`.

## What this feature does
A household member says *"Hey Google, add [quantity] [item] to the shopping
list."* Google Home (via a Smart Home/Conversational Action) posts the parsed
item and quantity to the FastAPI backend. The request lands as **pending** —
never directly on the active grocery list — until a household member reviews
it in the web app's Pending Requests view and accepts or rejects it.

## Backend
| Piece | File |
|---|---|
| Webhook + confirmation routes | `backend/app/routes/voice.py` |
| Business logic | `backend/app/services/voice.py` |
| WebSocket push | `backend/app/services/ws_manager.py`, `backend/app/routes/ws.py` |
| Models | `backend/app/models/db.py` (`VoiceRequest`, `GroceryItem`) |
| Migration | `backend/alembic/versions/0001_initial.py` |

### Endpoints
- `POST /api/voice-requests` — the Google Home webhook. Auth: `X-Webhook-Secret`
  header matching `VOICE_WEBHOOK_SECRET` (never a Clerk JWT — Google Home can't
  carry one). Body: `{"item": str, "quantity": int = 1}`.
- `GET /api/voice-requests` — list pending requests (Clerk JWT required).
- `POST /api/voice-requests/{id}/accept` — body optionally overrides
  `{"name", "quantity"}` (the user corrected a misheard item). Query
  `?merge=true` folds the quantity into an existing active item with the
  same normalised name instead of creating a duplicate row. Accepting is
  allowed from `pending` **or `rejected`** (an intentional undo path — see
  below) but not from an already-`accepted` request (409).
- `POST /api/voice-requests/{id}/reject` — marks rejected; only allowed from
  `pending`.

### Duplicate handling
`VoiceService._find_active_duplicate` compares normalised names
(lowercased, whitespace-collapsed) against active `grocery_items`. If a
match exists and the caller didn't pass `merge=true`, accept returns
`409` with a `DuplicateItemWarning` body so the frontend can ask the user
"add another or update the existing quantity?" per the spec.

### Real-time push
Every create/accept/reject broadcasts `{"type": "voice_request_added",
"count": <pending count>}` over `/ws` to every open browser session. The
frontend uses this for the nav badge and the Sonner toast — no polling.

## Frontend
| Piece | File |
|---|---|
| Pending Requests page | `frontend/src/routes/pending.tsx` |
| Request card (compound) | `frontend/src/components/pending/pending-request-card.tsx` |
| Nav badge | `frontend/src/components/nav.tsx`, `hooks/usePendingCount.ts` |
| Toast on new request | `hooks/useVoiceRequestToasts.ts` |
| WebSocket client | `frontend/src/lib/ws.ts` |
| API client | `frontend/src/lib/api.ts` |

### The "dulled reject" UX
Per spec: "Rejecting an item dulls the item but on mouse over the user can
click accept, this stops accidental rejections from being a problem."
Implemented as: reject still calls the backend immediately (so the pending
count is accurate everywhere else), but the frontend keeps the card
mounted at `opacity-40` with only an **Accept** button showing. Hovering
(or focusing, for keyboard users) restores full opacity. Clicking Accept
calls the backend's accept-from-rejected undo path described above. There
is no time limit — the card stays until the page is reloaded, at which
point a truly rejected item no longer appears in `GET /api/voice-requests`
(it only returns `pending`).

## Auth note
All routes except the webhook require a Clerk JWT (`app/auth/clerk.py`,
verified against Clerk's JWKS). Clerk is not yet wired into the frontend —
see `frontend/src/lib/auth.ts`'s TODO. For local development without Clerk
configured, set `DEV_AUTH_BYPASS=true` in the backend's environment; never
enable this in a deployment reachable through the Cloudflare Tunnel.

## Known gaps / next steps
- Frontend Clerk provider + `getAuthToken()` wiring (auth section of the
  spec, not part of this goal).
- Item Rules (auto-filter chips) are not applied yet — `GroceryItem` has no
  `filter_terms` column; that's the Item Rules feature.
- `backend/tests/integration/test_voice_requests.py` needs a Docker daemon
  (testcontainers spins up real Postgres) — run via `make test-backend`,
  not directly on a host without Docker.
