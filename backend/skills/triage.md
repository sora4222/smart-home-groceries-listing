# LLM Triage — working on it

Spec: `docs/features/FEATURE_TRIAGE.md`. Rule that never bends: **triage
never accepts an item.** It only picks the queue a person sees it in.

## Layout (`src/services/triage/`)
| File | Change it when |
|---|---|
| `model.rs` | the classifier contract changes (`TriageModel`, `Classification`, `TriageError`) |
| `prompt.rs` | the instructions or the reply shape change (pure, unit-tested) |
| `verdict.rs` | the answer → `approved`/`rejected`/`held` rule changes (pure) |
| `openai.rs` | the HTTP call changes — the **only** file that talks to a provider |
| `fake.rs` | the dev/test keyword list changes |
| `assessor.rs` | the time limit or "off" behaviour changes |
| `registry.rs` | adding a provider (one match arm) |
| `queue.rs` | when/how checks run (background, startup re-check) |
| `review.rs` | Triage view rules (tabs, accept to pending) |

SQL for the `triage_*` columns is in `services/voice/triage_repository.rs`;
`services/voice/counts.rs` pushes both badge counts.

## Adding a provider
1. Implement `TriageModel` in a new file (return `BoxFuture`, like
   `StoreClient`). Map every failure to `TriageError` with a message a
   person can read — no keys, headers or response bodies.
2. Add a `TriageProvider` variant in `src/config/triage.rs` (+ parse test).
3. Add the arm in `registry.rs`.
4. Test it with `wiremock`, as `tests/triage_openai.rs` does. Never call a
   real LLM from a test.

## Testing
- `TestApp::new` runs with triage **off** (items are `skipped`, in Pending
  Requests at once). Build `TestApp::with_triage(pool, triage_settings(..))`
  to switch it on.
- The check runs in a spawned task. Wait for it with `app.triaged(&id)` or
  `app.webhook_item_triaged(item)` (`tests/common/triage.rs`) — never sleep a
  fixed time.
- The startup re-check only picks rows older than the process, so it never
  races a check started on arrival. Keep that `created_at < started_at`.
- e2e: the backend runs `INTAKE_LLM_PROVIDER=fake`; use `deliverVoiceItem()`
  from `frontend/e2e/fixtures.ts`.

## Running the e2e stack by hand
```bash
DATABASE_URL=postgres://grocery:changeme@localhost:5432/grocery_e2e \
DEV_AUTH_BYPASS=true VOICE_WEBHOOK_SECRET=e2e-webhook-secret \
STORE_TAB_SECRET=e2e-store-tab STORE_CLIENTS=fake INTAKE_LLM_PROVIDER=fake \
BIND_ADDRESS=127.0.0.1:8000 CORS_ORIGINS=http://localhost:3000 \
  ./backend/target/debug/grocery-backend
```
