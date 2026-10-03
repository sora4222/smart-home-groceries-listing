# Backend Agent Guide

## Stack
Rust 1.85+ (edition 2021) · Axum 0.8 (REST + WebSocket) · sqlx 0.9 (async,
compile-time-safe queries, embedded migrations) · PostgreSQL · Clerk JWT via
`jsonwebtoken` · `tower-http` middleware · `aes-gcm` · `tracing` ·
`wreq` (store search, Chrome TLS emulation) · `rust_decimal` (money) ·
`chromiumoxide` (checkout automation, not yet built)

**There is no Python in this crate or anywhere else in `backend/`.** The one
Python process in the repository is `sidecars/alexa-bridge`, which does
nothing but verify Alexa's request signature and forward the item — see the
"Sidecars" section below.

## Toolchain
`cargo` · `rustfmt` (`rustfmt.toml`, 100 columns) · `clippy` (warnings are
errors in CI) · `cargo test` with `#[sqlx::test]`

## Structure
```
src/
├── main.rs              # entrypoint: config, pool, migrate, serve, graceful shutdown
├── lib.rs               # build_app() — state, router, middleware. Thin wiring only.
├── config.rs            # Settings::from_env(); every value comes from the environment
├── error.rs             # ApiError + IntoResponse. Keeps FastAPI's {"detail": ...} shape.
├── logging.rs           # console + daily access-log file (ACCESS_LOG_DIR)
├── middleware/
│   └── access_log/      # logs every request — see backend/skills/access-logs.md
├── state.rs             # AppState: pool, settings, ws hub, auth provider, encryptor
├── routes/              # One module per domain — expose `router()`, merge in routes/mod.rs
│   ├── voice.rs         # POST /api/voice-requests (webhook, shared secret) + queue routes
│   ├── alexa.rs         # POST /api/intake/alexa (bridge sidecar, shared secret)
│   ├── alexa_list_changes.rs # POST /api/intake/alexa/remove, /undo
│   ├── grocery.rs       # /api/grocery-items — the list: add, edit, delete, commit
│   ├── item_rules.rs    # /api/item-rules — list, add, edit, delete rules
│   ├── products.rs      # GET /api/grocery-items/{id}/products — store search
│   ├── selections.rs    # the one product chosen per item: GET/PUT/DELETE
│   ├── order_review.rs  # GET /api/order-review — committed choices re-priced
│   ├── triage.rs        # /api/triage — held/rejected tabs, accept to pending, reject
│   ├── access_logs.rs   # GET /api/access-logs — the /logs page's data
│   ├── dislikes.rs      # /api/product-dislikes + per-item dislike overrides
│   ├── purchases.rs     # /api/purchase-orders (list, Undo) + /api/purchase-history
│   ├── spending.rs      # GET /api/spending (+ /item-prices) — the Spending page
│   ├── health.rs        # /api/health
│   ├── ws.rs            # WebSocket /ws
│   └── extract.rs       # ValidatedJson / OptionalValidatedJson / ValidatedQuery extractors
├── auth/
│   ├── mod.rs           # AuthUser extractor, AuthProvider trait, build_provider()
│   ├── clerk.rs         # Clerk JWKS verification (current implementation)
│   ├── request_user.rs  # slot the extractor fills so the access log knows the user
│   └── secret.rs        # constant-time shared-secret comparison
├── models/
│   ├── db.rs            # row types + status/source enums
│   └── schemas/         # request/response bodies with `validator` constraints,
│                        #   one file per domain (+ common.rs limits), re-exported flat
├── services/            # Business logic — no HTTP types, no pool creation
│   ├── access_log/      # record (file line + background row), page rules, SQL
│   ├── grocery/
│   │   ├── mod.rs       # GroceryService: list rules, commit/release
│   │   ├── duplicates.rs# OnDuplicate + the duplicate-item 409
│   │   ├── annotations.rs# tidying an item's note
│   │   ├── log.rs       # the list's log events
│   │   └── repository.rs# every grocery_items statement, as literals
│   ├── voice/
│   │   ├── mod.rs       # VoiceService: the confirmation-queue rules
│   │   ├── log.rs       # the queue's log events
│   │   ├── counts.rs    # pushes both badge counts (pending, held)
│   │   ├── repository.rs# voice_requests statements, as literals
│   │   └── triage_repository.rs # the triage_* columns of the same table
│   ├── triage/          # LLM triage: TriageModel trait, OpenAI/Ollama client,
│   │                    #   fake, prompt + verdict (pure), background queue,
│   │                    #   Triage view rules — see backend/skills/triage.md
│   ├── voice_changes/   # remove/reduce by voice + Undo: matching.rs (name
│   │                    #   variants, pure), plan.rs (arithmetic, pure),
│   │                    #   undo.rs, repository.rs (voice_list_changes)
│   ├── item_rules/
│   │   ├── mod.rs       # ItemRuleService: list, add, edit, delete
│   │   ├── apply.rs     # filter_terms_for(): the chips a new item gets
│   │   ├── matching.rs  # pure trigger ↔ item-name matcher
│   │   ├── triggers.rs  # tidying trigger phrases
│   │   ├── log.rs       # rule log events
│   │   └── repository.rs# every item_rules statement, as literals
│   ├── stores/          # Woolworths + Coles clients, Product, unit prices, deals,
│   │                    #   fake catalogue, cache — see skills/store-integration.md
│   ├── product_search/  # one item across every store: query, chip filter,
│   │                    #   pricing at quantity, comparability notes, ordering
│   ├── selections/      # choosing one product per item: offer.rs checks it
│   │                    #   against the store's answer, staleness.rs drops it
│   │                    #   on rename/re-chip, repository.rs owns the table
│   ├── dislikes/        # per-member dislikes, per-item overrides; skip.rs is
│   │                    #   the pure rule the optimiser calls — skills/dislikes.md
│   ├── order_review/    # the committed list re-priced: line/ (one choice, pure),
│   │                    #   summary.rs (by store + totals, pure), log.rs
│   ├── purchases/       # a filled trolley saved as bought (background), Undo,
│   │                    #   price/fee/category rules (pure), history, read +
│   │                    #   write repositories — docs/features/FEATURE_PURCHASE_HISTORY.md
│   ├── spending/        # the Spending page: period.rs, range.rs (time zones),
│   │                    #   breakdown.rs (four views, pure)
│   ├── filter_terms.rs  # clean()/merge() for chip lists, shared by list and rules
│   ├── ws_hub.rs        # broadcast fan-out
│   └── encryption.rs    # AES-256-GCM
└── db/mod.rs            # pool construction + MIGRATOR
migrations/*.sql         # sqlx migrations, embedded via sqlx::migrate!
tests/                   # integration tests through the real router
```

## Route pattern
```rust
// src/routes/grocery.rs
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::GroceryItemResponse;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/grocery-items", get(list_active_items))
}

async fn list_active_items(
    State(state): State<AppState>,
    _user: AuthUser,                       // ← the auth check; presence is the guard
) -> Result<Json<Vec<GroceryItemResponse>>, ApiError> {
    let items = repository::list_active_items(&state.pool).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}
```

Axum 0.8 path parameters are `{id}`, **not** `:id`. Register the module in
`routes/mod.rs::api_router`.

## CORS
`lib.rs::cors_layer` lists every HTTP method the routes serve. A new method
(`PUT` arrived with product choice) must be added there, or every browser
blocks it while `curl` and the router tests still pass — `tests/cors.rs`
checks the preflight.

## Auth
```rust
// Every route except the intake endpoints: take the extractor.
async fn handler(user: AuthUser) -> ... {}     // 401 if the JWT does not verify

// Intake endpoints cannot carry a session, so they use a shared secret.
verify_shared_secret(&state.settings.voice_webhook_secret, presented, "VOICE_WEBHOOK_SECRET")?;
```

All auth goes through `auth/mod.rs` — never call a provider SDK from a route.
Swapping provider means implementing `AuthProvider` and changing
`build_provider`; nothing else moves.

`DEV_AUTH_BYPASS=true` turns the `AuthUser` extractor into a fixed dev user.
It logs a warning at startup. Never set it on a host reachable through the
Cloudflare Tunnel.

## Service pattern
```rust
pub struct VoiceService<'a> {
    pool: &'a PgPool,
    hub: &'a WsHub,
}

impl<'a> VoiceService<'a> {
    pub fn new(pool: &'a PgPool, hub: &'a WsHub) -> Self { Self { pool, hub } }

    pub async fn list_pending(&self) -> Result<Vec<VoiceRequest>, ApiError> {
        repository::list_pending(self.pool).await
    }
}
```

A service **borrows** the pool a route hands it; it never creates one. No HTTP
calls in a service (that is `services/stores/`), and no SQL either — SQL lives
in the domain's `repository.rs`.

## Triage (LLM)
`AppState.triage` is a `TriageQueue`, built by `services::triage::registry`
from `INTAKE_LLM_PROVIDER` (`ollama` default | `llamacpp` | `vllm` | `openai` |
`fake` | `off`). Only
`services/triage/openai.rs` talks to an LLM provider. An intake service
records the request `unchecked` and calls `triage.start(&request)`, which
classifies it in a spawned task — never inline, because the Alexa bridge only
waits 3 s. Integration tests run with triage `off` unless they build
`TestApp::with_triage`. See `backend/skills/triage.md`.

## Store clients
`AppState.stores` holds every `StoreClient`, built once by
`services::stores::registry::build` from `STORE_CLIENTS` (`live` | `fake`).
Only `services/stores/` talks to a store, only through `wreq` (never
`reqwest`), and a store failure is a `StoreError` reported inside a 200 —
never an `ApiError`. Money is `rust_decimal::Decimal`, sent as a string.

## SQL rules
- **Every query is a literal `&'static str` with bind parameters.** sqlx 0.9
  refuses a runtime-built string unless it is wrapped in `AssertSqlSafe`;
  never reach for that wrapper. Repeating a column list is the cheaper price.
- Multi-row changes go in one transaction, and lock the row they decide on
  (`SELECT ... FOR UPDATE`) before reading it — two browser tabs accepting the
  same card must not both succeed.
- Enums are `TEXT` with `CHECK` constraints, not PostgreSQL `ENUM` types:
  adding a value is a constraint change rather than an `ALTER TYPE` that
  cannot run in a transaction.
- The duplicate-name expression
  `lower(btrim(regexp_replace(name, '\s+', ' ', 'g')))` appears in
  `services::grocery::repository::lock_active_duplicate`, in
  `ix_grocery_items_normalised_name`, and as
  `services::grocery::repository::normalise` in Rust. Change one, change all
  three.
- **A table has one repository** (a module directory; `voice/` splits its
  statements over `repository.rs` and `triage_repository.rs` for length). Every `grocery_items` statement lives in
  `services/grocery/repository.rs`, including the ones the intake queue uses
  when it accepts a request — `services/voice/` calls into it rather than
  writing item SQL of its own.

## Migrations
`sqlx` migrations in `migrations/`, embedded into the binary by
`sqlx::migrate!` and applied at startup. See `skills/../../skills/migrations.md`.

## Logging
Each domain's events live in its own `log.rs` — one function per thing that
happened (`log::added(&item, how, user_id)`), so a service reads as rules and
every event carries the same field names. Log at `info` for a change,
`warn` for a refused one, `debug` for a no-op. Log item names; never log
notes, credentials or tokens.

## WebSocket (real-time push)
`services/ws_hub.rs` holds a `tokio::sync::broadcast` channel. Publishers call
`hub.broadcast(ServerEvent::VoiceRequestAdded { count })`; each socket task
holds a receiver, so a slow client cannot block a publisher. Events are
process-local — fronting several backend processes would need Postgres
`LISTEN/NOTIFY`.

## Validation
Request bodies derive `validator::Validate` and are extracted with
`ValidatedJson<T>` (or `OptionalValidatedJson<T>` where the body is optional).
A body outside its limits is a 422 that never reaches a service, so a service
may assume its inputs are in range. Keep the limits in step with the
migration's `CHECK` constraints.

## Errors
Return `ApiError`. `Internal` and `Misconfigured` log their cause and answer
with a generic message — a database error, a connection string or an upstream
message must never reach a response body.

## File length
Hard limit: **300 lines**. Extract to a service first; if the service is still
over, split by responsibility (that is why `services/voice/` is a directory
with `mod.rs` for rules and `repository.rs` for SQL).

## Credentials (AES-256-GCM)
```rust
let encryptor = state.encryptor.as_ref().ok_or_else(|| {
    ApiError::Misconfigured("CREDENTIAL_ENCRYPTION_KEY is not set".into())
})?;
let stored = encryptor.encrypt(plaintext)?;
let plaintext = encryptor.decrypt(&stored)?;
```

The key is base64-encoded 32 bytes from `CREDENTIAL_ENCRYPTION_KEY`. Never
hardcoded, never logged, never returned to the frontend. `Encryptor`'s `Debug`
is redacted on purpose.

## Sidecars
`sidecars/alexa-bridge` (Python) verifies Alexa's request signature with
Amazon's `ask-sdk` and forwards the item to `POST /api/intake/alexa` over the
Compose network, authenticated by `ALEXA_BRIDGE_SECRET`. It holds no database
credentials and no business logic.

A Google Keep source would take the same shape — `gkeepapi` is Python-only and
unofficial — but is unbuilt. Any new sidecar must: verify or authenticate at
its own boundary, forward to a backend intake endpoint with its **own** shared
secret, and contain no rules about the grocery list.

## Skills in this directory
- `skills/store-integration.md` — store endpoints, `wreq`, bot-protection rules,
  unit prices and deals
- `skills/dislikes.md` — dislike scopes, overrides, and how the optimiser
  asks which product it may buy
