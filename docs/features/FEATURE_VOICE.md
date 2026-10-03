# Feature: Voice Intake

Status: **implemented** for the generic webhook and Alexa (backend + web UI),
including removing, reducing and undoing items by Alexa. Amazon developer
console and Cloudflare Tunnel wiring is a manual setup step — see
`docs/human-setup.md` §3–4.

## What this feature does
A household member says *"Alexa, add two oat milk to the shopping list."* The
item lands as **pending** — never directly on the active grocery list. An LLM
first checks it is something a supermarket sells (`FEATURE_TRIAGE.md`); items
it rejects or is unsure about wait on the `/triage` page instead. A household
member then reviews it in the web app's Pending Requests view and accepts or
rejects it.

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

## Removing, reducing and undo by voice

```
"Alexa, ask grocery list to remove two milk"
  → sidecar: RemoveItemIntent {item: "milk", quantity: 2}
  → POST /api/intake/alexa/remove {"item": "milk", "quantity": 2, "external_id": <request id>}
  → milk 3 → 1, change recorded in voice_list_changes
  ← "milk is down to 1 on the list. Say undo to change it back."   (session left open)
"Undo"
  → POST /api/intake/alexa/undo {"external_id": <request id>}
  ← "Added 2 milk back to the list."
```

**Why no confirmation queue.** Adds wait in Pending Requests because nothing
may reach the list unasked. A remove or reduce can only shrink the order, so
it applies at once; the safety net is Undo instead of a second step. Every
change is recorded in `voice_list_changes` with a snapshot of the item.

### Intents (sidecar)
| Intent | Slots | Forwarded `quantity` |
|---|---|---|
| `RemoveItemIntent` | `item`, `quantity` (optional) | absent → whole item; said → that many |
| `ReduceItemIntent` | `item`, `quantity` (optional) | said, else `1` |
| `UndoIntent` | — | — |

Sample utterances: `docs/human-setup.md` §4. The sidecar keeps the session
open after a change (`shouldEndSession: false`) so "undo" needs no
re-invocation. Handlers: `sidecars/alexa-bridge/src/alexa_bridge/list_change_handlers.py`;
HTTP client: `list_changes.py`; spoken text: `speech.py`.

### Rules (backend)
- **Matching** (`services/voice_changes/matching.rs`): the normalised name
  (`lower`, whitespace collapsed, same expression as duplicate detection), or
  its plain plural/singular (`-s`, `-es`). An exact match beats a plural one,
  then `active` beats `committed`, then the oldest row wins. No substring or
  fuzzy matching: "milk" never matches "oat milk". No match → `404`, nothing
  changes.
- **Locked list**: a matching `committed` item → `409`, same as web edits.
  Pending intake requests are not list items and are never matched.
- **Arithmetic** (`plan.rs`): no quantity → delete; otherwise
  `max(current − n, 0)`; `0` → delete.
- **Undo** (`undo.rs`): reverts the newest change from the same source that
  is not yet undone and is younger than `UNDO_WINDOW` (30 min). Repeat to go
  further back. A removed item is re-inserted with its original id,
  `created_at`, quantity, note and chips, as `active`; its chosen product
  (`item_selections`, deleted by `ON DELETE CASCADE`) is not restored. A
  reduced item gets the decrement added back to its *current* quantity
  (capped at 999), so web edits made since are kept; if it was deleted or
  committed since, `409`. Nothing to undo → `404`.
- **Idempotency**: `external_id` (Alexa's request id) is unique per source
  on both the change and the undo (`undo_external_id`). A retry returns the
  original change with `200` instead of applying again; a retried undo
  never reverts a second change. All changes and undos take one transaction
  advisory lock, so retries racing the original serialise.

### Endpoints
Both authenticate with `X-Bridge-Secret`, like `POST /api/intake/alexa`.
- `POST /api/intake/alexa/remove` — body
  `{"item": str, "quantity": int?, "external_id": str?}`. `201` with a
  `VoiceListChangeResponse`
  (`{id, kind: "removed"|"reduced", item_name, quantity_before, quantity_after, undone}`),
  `200` on a retry, `404` no match, `409` committed, `422` bad body.
- `POST /api/intake/alexa/undo` — body `{"external_id": str?}` (may be
  empty). `201` with the reverted change (`undone: true`), `200` on a retry,
  `404` nothing to undo, `409` cannot be reverted.

### Known gaps
- No WebSocket event for list changes, so an open Grocery List page shows a
  voice change only after reload.
- Undo is voice-only; the web app has no view of `voice_list_changes`.

## Backend
| Piece | File |
|---|---|
| Webhook + confirmation routes | `backend/src/routes/voice.rs` |
| Alexa intake route | `backend/src/routes/alexa.rs` |
| Alexa remove/undo routes | `backend/src/routes/alexa_list_changes.rs` |
| Remove/reduce/undo rules | `backend/src/services/voice_changes/` |
| Business rules | `backend/src/services/voice/mod.rs` |
| Log events | `backend/src/services/voice/log.rs` |
| SQL (`voice_requests`) | `backend/src/services/voice/repository.rs` |
| SQL (`grocery_items`) | `backend/src/services/grocery/repository.rs` |
| WebSocket push | `backend/src/services/ws_hub.rs`, `backend/src/routes/ws.rs` |
| Row types | `backend/src/models/db.rs` (`VoiceRequest`, `GroceryItem`) |
| Request/response bodies | `backend/src/models/schemas/voice.rs` |
| Migrations | `backend/migrations/0001_initial.sql`, `20261002200000_voice_list_changes.sql` |
| Tests | `backend/tests/voice_requests.rs`, `alexa_intake.rs`, `alexa_list_changes.rs`, `alexa_undo.rs`, `websocket.rs` |

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
- `GET /api/voice-requests` — list pending requests that triage approved or
  skipped (Clerk JWT). Held and rejected ones are on `/api/triage`.
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
`services::grocery::repository::lock_active_duplicate` compares normalised
names (lowercased,
whitespace-collapsed) against active `grocery_items`, using the expression
index `ix_grocery_items_normalised_name`. If a match exists and the caller did
not pass `merge=true`, accept returns `409` with a `DuplicateItemWarning` in
the `detail` field so the frontend can ask "add another or update the existing
quantity?" per the spec. The transaction is rolled back, so the request stays
pending and the user still gets to choose.

`merge=true` adds to the existing quantity, capped at 999 to stay inside the
column's `CHECK` constraint.

### Item rules
An accepted request that becomes a new list entry gets the chips of every item
rule whose trigger matches its name — the corrected name, when there is one.
A merge leaves the existing item's chips alone. See `FEATURE_ITEM_RULES.md`.

Accept runs in one transaction and takes `SELECT ... FOR UPDATE` on the
request row and on any duplicate it finds, so two browser tabs racing to
accept the same card cannot both succeed.

### Real-time push
Every create/accept/reject (and every stored triage check) broadcasts
`{"type": "voice_request_added", "count": <pending count>}` and
`{"type": "triage_held", "count": <held count>}` over `/ws` to every open
browser session. The frontend uses this for the nav badge and the
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
- LLM triage is built — see `FEATURE_TRIAGE.md`.
- **Google Tasks and Google Keep sources are not built.** Tasks is a plain
  OAuth REST API and belongs in Rust; Keep would need a Python sidecar
  (`gkeepapi` is unofficial and authenticates with a master token — treat that
  risk explicitly before building it).
- Alexa uses custom intents (`AddItemIntent`, `RemoveItemIntent`,
  `ReduceItemIntent`, `UndoIntent`). Amazon's household list events
  (`AlexaHouseholdListEvent.ItemsCreated`) would remove the custom invocation
  but need the List API and a permissions grant.
- Verify the intake brief's assumptions before building on them.
