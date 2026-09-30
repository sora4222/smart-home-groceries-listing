# Feature: Voice Intake

Status: **implemented** for the generic webhook and Alexa (backend + web UI).
Amazon developer console and Cloudflare Tunnel wiring is a manual setup step —
see `docs/human-setup.md`.

## What this feature does
A household member says *"Alexa, add two oat milk to the shopping list."* The
item lands as **pending** — never directly on the active grocery list — until
a household member reviews it in the web app's Pending Requests view and
accepts or rejects it.

That holds for every channel, however confident it was. Reducing
marketing-driven buying is a project goal, so nothing in this system adds,
suggests or upsells an item on its own.

## Channels

| Channel | Endpoint | Auth | Status |
|---|---|---|---|
| Alexa | `POST /api/intake/alexa` | `X-Bridge-Secret` (from the sidecar) | **implemented** |
| Generic webhook | `POST /api/voice-requests` | `X-Webhook-Secret` | **implemented** |
| Google Tasks | — | Google OAuth | not built |
| Google Keep | — | `gkeepapi` master token | not built |
| Google Home Smart Home Action | — | — | **won't do** |

### Google Home: won't do
Google Home cannot be integrated directly. The Smart Home Action / OAuth
account-linking design was dropped. Despite the repository name, the working
voice channel is Alexa. Google Assistant may still reach the list indirectly
once the Google Tasks or Google Keep sources are built, since Assistant writes
spoken shopping-list items to Keep.

### Alexa
```
"Alexa, add two oat milk to the shopping list"
  → Amazon → Cloudflare Tunnel → sidecars/alexa-bridge :8081/alexa
                                   ├─ ask-sdk verifies signature, cert chain,
                                   │  timestamp and skill id
                                   ├─ reads the item and quantity slots
                                   └─ POST backend:8000/api/intake/alexa
                                        → pending request → web app → accept
```

The sidecar is Python because Amazon's request signing and skill request model
exist only in the `ask-sdk` Python SDK. It holds no database credentials and
no business logic — see `sidecars/alexa-bridge/AGENTS.md`.

Alexa retries an endpoint it believes timed out, reusing the same request id.
The sidecar passes that id through as `external_id`, and a partial unique
index on `(source, external_id)` makes the retry a no-op that returns `200`
with the original request rather than raising a second card.

### Generic webhook
`POST /api/voice-requests` remains for Home Assistant, IFTTT, `curl` and
tests. It carries no `external_id`, so it is not de-duplicated.

## Backend
| Piece | File |
|---|---|
| Webhook + confirmation routes | `backend/src/routes/voice.rs` |
| Alexa intake route | `backend/src/routes/alexa.rs` |
| Business rules | `backend/src/services/voice/mod.rs` |
| SQL | `backend/src/services/voice/repository.rs` |
| WebSocket push | `backend/src/services/ws_hub.rs`, `backend/src/routes/ws.rs` |
| Row types | `backend/src/models/db.rs` (`VoiceRequest`, `GroceryItem`) |
| Request/response bodies | `backend/src/models/schemas.rs` |
| Migration | `backend/migrations/0001_initial.sql` |
| Tests | `backend/tests/voice_requests.rs`, `alexa_intake.rs`, `websocket.rs` |

The table is still called `voice_requests` to limit churn; the concept is an
*intake* request.

### Endpoints
- `POST /api/voice-requests` — generic webhook. Auth: `X-Webhook-Secret`
  matching `VOICE_WEBHOOK_SECRET`, compared in constant time. Body:
  `{"item": str, "quantity": int = 1}`.
- `POST /api/intake/alexa` — from the bridge sidecar. Auth:
  `X-Bridge-Secret` matching `ALEXA_BRIDGE_SECRET` (a **different** secret, so
  leaking one does not grant the other). Body:
  `{"item": str, "quantity": int = 1, "external_id": str?, "raw_text": str?}`.
  `201` for a new item, `200` when `external_id` has already been seen.
- `GET /api/voice-requests` — list pending requests (Clerk JWT).
- `POST /api/voice-requests/{id}/accept` — body optionally overrides
  `{"name", "quantity"}` (the user corrected a misheard item); an absent or
  empty body means "accept as heard". Query `?merge=true` folds the quantity
  into an existing active item with the same normalised name instead of
  creating a duplicate row. Accepting is allowed from `pending` **or
  `rejected`** (the undo path below) but not from an already-`accepted`
  request (`409`). An unknown id is `404`.
- `POST /api/voice-requests/{id}/reject` — marks rejected; only from
  `pending`, otherwise `409`. Unknown id is `404`.

### Duplicate handling
`repository::lock_active_duplicate` compares normalised names (lowercased,
whitespace-collapsed) against active `grocery_items`, using the expression
index `ix_grocery_items_normalised_name`. If a match exists and the caller did
not pass `merge=true`, accept returns `409` with a `DuplicateItemWarning` in
the `detail` field so the frontend can ask "add another or update the existing
quantity?" per the spec. The transaction is rolled back, so the request stays
pending and the user still gets to choose.

`merge=true` adds to the existing quantity, capped at 999 to stay inside the
column's `CHECK` constraint.

Accept runs in one transaction and takes `SELECT ... FOR UPDATE` on the
request row and on any duplicate it finds, so two browser tabs racing to
accept the same card cannot both succeed.

### Real-time push
Every create/accept/reject broadcasts
`{"type": "voice_request_added", "count": <pending count>}` over `/ws` to
every open browser session. The frontend uses this for the nav badge and the
toast — no polling. A `tokio::sync::broadcast` channel does the fan-out, so a
slow client cannot block a publisher. Events are process-local; fronting
several backend processes would need Postgres `LISTEN/NOTIFY`.

## Frontend
| Piece | File |
|---|---|
| Pending Requests page | `frontend/src/routes/pending.tsx` |
| Request card (compound) | `frontend/src/components/pending/pending-request-card.tsx` |
| Nav badge | `frontend/src/components/nav.tsx`, `hooks/usePendingCount.ts` |
| Toast on new request | `hooks/useVoiceRequestToasts.ts` |
| WebSocket client | `frontend/src/lib/ws.ts` |
| API client | `frontend/src/lib/api.ts` |

**The frontend was not changed by the move to Rust.** Paths, field names,
status codes and the `{"detail": ...}` error envelope are identical to the
FastAPI implementation. `VoiceRequestResponse` gained a `source` field, which
is additive.

### The "dulled reject" UX
Per spec: "Rejecting an item dulls the item but on mouse over the user can
click accept, this stops accidental rejections from being a problem."
Implemented as: reject calls the backend immediately (so the pending count is
accurate everywhere else), but the frontend keeps the card mounted at
`opacity-40` with only an **Accept** button showing. Hovering (or focusing,
for keyboard users) restores full opacity. Clicking Accept uses the
accept-from-rejected undo path above. There is no time limit — the card stays
until the page is reloaded, at which point a truly rejected item no longer
appears in `GET /api/voice-requests` (it only returns `pending`).

## Auth note
All routes except the two intake endpoints require a Clerk JWT, verified
against Clerk's JWKS (`backend/src/auth/clerk.rs`). The signing algorithm is
taken from the JWK, never from the token's own header, so a token claiming a
different algorithm cannot be verified against the RSA public key. Clerk is
not yet wired into the frontend — see `frontend/src/lib/auth.ts`'s TODO. For
local development set `DEV_AUTH_BYPASS=true`; the backend logs a warning at
startup, and it must never be enabled on a deployment reachable through the
Cloudflare Tunnel.

## Known gaps / next steps
- Frontend Clerk provider + `getAuthToken()` wiring (auth section of the
  spec, not part of this goal).
- **LLM triage is not built.** The intake brief plans a "is this something a
  supermarket sells?" classifier with `approved`/`rejected`/`held` states and
  a `/triage` view. The schema has `source` and `external_id` but none of the
  triage columns.
- **Google Tasks and Google Keep sources are not built.** Tasks is a plain
  OAuth REST API and belongs in Rust; Keep would need a Python sidecar
  (`gkeepapi` is unofficial and authenticates with a master token — treat that
  risk explicitly before building it).
- Alexa uses a custom `AddItemIntent`. Amazon's household list events
  (`AlexaHouseholdListEvent.ItemsCreated`) would remove the custom invocation
  but need the List API and a permissions grant.
- Item Rules (auto-filter chips) are not applied yet — `GroceryItem` has no
  `filter_terms` column; that is the Item Rules feature.
- Verify the intake brief's assumptions before building on them.
