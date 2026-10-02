# Feature: LLM Triage

Status: **implemented** — every intake item is checked by a classifier
("would a supermarket sell this?") before the household sees it, and the
`/triage` page shows what it held or rejected. **Restore to source** is not
built: no channel that keeps a source list (Google Tasks, Google Keep) exists
yet. Steps for people: `docs/human-setup.md` part 5 and `docs/using-the-app.md`
("Check the Triage page") — keep those in step with this spec.

## What this feature does
Triage filters out what is clearly not a grocery item ("car service"). It
**never accepts an item**. An approved item still waits in Pending Requests
for a person, and so does any item a person lets past triage.

```
Alexa / webhook → recorded as unchecked → 201/200 to the channel at once
                     │ (background task)
                     ▼
            classifier answers ──▶ approved → Pending Requests
                                ├▶ rejected → /triage, Rejected tab
                                └▶ held     → /triage, Held for review tab (badge + toast)
```

| `triage_status` | Meaning | Shown in |
|---|---|---|
| `unchecked` | recorded; the classifier has not answered yet | nowhere |
| `approved` | a confident yes | Pending Requests |
| `rejected` | a confident no | Triage → Rejected |
| `held` | not sure (below `INTAKE_LLM_MIN_CONFIDENCE`), or the classifier failed / timed out | Triage → Held for review |
| `skipped` | triage off, or a person pressed **Accept** in Triage | Pending Requests |

Rows from before this feature were migrated to `skipped`.

### Why the check runs after the item is stored
The spec says "triage, then commit". The Alexa bridge only waits 3 s for the
backend, and a local Ollama model can take longer than that to load; an HTTP
request the bridge gives up on is dropped by the server, item and all. So the
item is stored first (`unchecked`) and checked by a `tokio::spawn` task. At
startup, any request older than the process that is still `unchecked` (a
restart mid-check) is checked again. The check only writes a row that is still
`unchecked`, so a duplicate check can never overwrite a decision. An Alexa
retry (same `external_id`) is never checked twice.

## Endpoints

| Method | Path | Auth | Answer |
|---|---|---|---|
| `GET` | `/api/triage?tab=held\|rejected` | Clerk | 200 undecided requests in that tab, newest first; 400 unknown tab |
| `POST` | `/api/triage/{id}/accept` | Clerk | 200, the request is now `skipped` (in Pending Requests); 404; 409 not held/rejected or already decided |
| `POST` | `/api/triage/{id}/reject` | Clerk | 200, the request is `rejected` for good; 404; 409 already decided |

`GET /api/voice-requests` (Pending Requests) now lists only `approved` and
`skipped` requests. Every intake request in an answer carries
`triage_status`, `triage_reason` (one sentence, ≤ 500 chars) and
`triage_confidence` (0–1, or `null` when the classifier did not answer).

### Push events (`/ws`)
Anything that can change a count sends both:
`{"type": "voice_request_added", "count": n}` (Pending Requests badge) and
`{"type": "triage_held", "count": n}` (Triage badge). The web app toasts
"Item held for review — tap to triage" when the held count rises, and
"New item added — tap to review" when the pending count rises.

## The classifier
`INTAKE_LLM_PROVIDER` picks it (`.env.example` lists every setting):

| Value | Classifier |
|---|---|
| `openai` (default) | OpenAI chat completions, `gpt-4o-mini`. Needs `OPENAI_API_KEY`; without it every item is held with the reason "OPENAI_API_KEY is not set". |
| `ollama` | Ollama's OpenAI-compatible endpoint, `llama3.2`, `http://localhost:11434/v1` (from Docker: `http://host.docker.internal:11434/v1`). Free. |
| `fake` | a keyword list: a service word ("service", "repair", "appointment", "haircut", "plumber", "dentist", "rego", "mechanic") → rejected, anything else → approved, both at 0.9. Development, tests, e2e. |
| `off` | no triage; every item is `skipped`. |

OpenAI and Ollama share one client (`services/triage/openai.rs`) that calls
`POST {base}/chat/completions` with `temperature: 0` and
`response_format: json_object`, and expects
`{"supermarket_item": bool, "confidence": 0..1, "reason": "..."}`. The spec
named `async-openai`; plain `reqwest` was used instead because the call is one
request, the crate was already in the tree, and `wiremock` can test it end to
end.

The item text is untrusted (anyone near the Echo). It travels as a JSON value
in the user message, never inside the instructions, and the worst a crafted
item can do is move itself between queues a person still decides on. Item
names leave the home server for OpenAI when `openai` is chosen; nothing else
does. The key is never logged (`TriageSettings` has a redacting `Debug`).

## Known gaps
- **Restore to source** (spec: recreate the item in Google Tasks / Keep with
  `force_accept`) waits for those channels.
- The spec asks for the OpenAI key to be stored encrypted in PostgreSQL and set
  on `/settings/intake`. It is an environment variable for now, like the other
  secrets; there is no intake settings page yet.
- The spec's `TriageResult` also has `normalised_name` and `quantity`. They are
  not asked for: the channel's own parse is kept, and the household corrects
  names in Pending Requests.

## Code
| Piece | File |
|---|---|
| Settings | `backend/src/config/triage.rs` |
| Classifier trait, OpenAI/Ollama, fake | `backend/src/services/triage/{model,openai,fake}.rs` |
| Prompt + reply parsing, answer → status (pure) | `services/triage/{prompt,verdict}.rs` |
| Time limit, picking the classifier | `services/triage/{assessor,registry}.rs` |
| Background check + startup re-check | `services/triage/queue.rs` |
| Triage view rules | `services/triage/review.rs` |
| SQL (`voice_requests.triage_*`) | `services/voice/triage_repository.rs` |
| Badge counts push | `services/voice/counts.rs` |
| Routes | `backend/src/routes/triage.rs` |
| Migration | `backend/migrations/0007_intake_triage.sql` |
| Page | `frontend/src/routes/triage.tsx`, `components/triage/`, `hooks/useTriageDecisions.ts` |
| Badge + toast | `hooks/useHeldCount.ts`, `hooks/useTriageToasts.ts` (both on `useLiveCount` / `useCountRiseToast`) |

## Tests
- Backend unit: settings, verdict rules, prompt parsing, fake, time limit.
- Backend integration: `tests/triage.rs` (fake classifier: queues, accept,
  reject, 404/409, auth, triage off, startup re-check) and
  `tests/triage_openai.rs` (`wiremock` provider: request shape, low
  confidence, provider error, unreadable reply, Alexa retry not re-checked).
  The check is asynchronous, so tests wait for it with `TestApp::triaged`
  (polls, fails after 5 s). Most other suites run with triage `off`.
- Frontend unit: card, list, confidence label, live-count and toast hooks.
- e2e `frontend/e2e/triage.spec.ts` (desktop + phone). The e2e backend runs
  `INTAKE_LLM_PROVIDER=fake`; `deliverVoiceItem()` waits for triage before a
  test opens a page.
