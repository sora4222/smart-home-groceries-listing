# Backend Agent Guide

## Stack
Rust 1.85+ (edition 2021) · Axum 0.8 (REST + WebSocket) · sqlx 0.9 (async,
compile-time-safe queries, embedded migrations) · PostgreSQL · Clerk JWT via
`jsonwebtoken` · `tower-http` middleware · `aes-gcm` · `tracing` ·
`wreq` (store search, Chrome TLS emulation) · `rust_decimal` (money) ·
`chromiumoxide` (checkout automation, not yet built).

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
│   ├── grocery.rs       # /api/grocery-items — the list: add, restore, edit, delete, commit
│   ├── item_rules.rs    # /api/item-rules — list, add, edit, delete rules
│   ├── products.rs      # GET /api/grocery-items/{id}/products — store search
│   ├── selections.rs    # products chosen per item and store; PUT /api/order-stores
│   ├── order_options.rs # GET /api/order-options — the planner
│   ├── delivery_settings.rs # GET/PUT /api/delivery-settings
│   ├── order_review.rs  # GET /api/order-review — committed choices re-priced
│   ├── triage.rs        # /api/triage — held/rejected tabs, accept, reject, restore
│   ├── google_tasks.rs  # /api/intake/google-tasks — sign-in, lists, choices, poll
│   ├── intake_settings.rs # GET /api/intake/settings — every channel's status
│   ├── access_logs.rs   # GET /api/access-logs — Settings › Logs data
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
│   ├── socket.rs        # SocketUser: the /ws check, token from header or ?token=
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
│   ├── google_tasks/    # Google Tasks: TasksApi trait, live client, fake, poller,
│   │                    #   schedule, restore, sign-in — see skills/google-tasks.md
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
│   ├── selections/      # choosing an item's products: offer.rs checks each
│   │                    #   against the store's answer, staleness.rs drops them
│   │                    #   on rename/re-chip, repository.rs owns the table
│   │                    #   and the Undo snapshots
│   ├── dislikes/        # per-member dislikes, per-item overrides; skip.rs is
│   │                    #   the pure rule the optimiser calls — skills/dislikes.md
│   ├── order_review/    # the committed list re-priced: line/ (one choice, pure),
│   │                    #   summary.rs (by store + totals, pure), log.rs
│   ├── order_plan/      # ways to buy with delivery: fees, option, search
│   │                    #   (best store mix), rank (modes), build — skills/order-planning.md
│   ├── delivery_settings/ # Settings › Delivery: fee rules per store, mode, cap
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
