# Feature: Google Tasks intake

Status: **implemented** (backend, `/settings/intake`, Triage "Restore to
source"). Tested against `wiremock` and an in-memory fake — **not yet against
a real Google account**. Steps for people:
`docs/human-setup.md` ("Optional: Google Tasks") and
`docs/using-the-app.md` ("Add items with Google Tasks").

## What this feature does
A household member types `2 oat milk` into a Google Tasks list they chose.
The backend polls that list, records each open task as an intake request
(`source = 'tasks'`), and deletes the task from Google. The request goes
through triage like every other channel and waits in Pending Requests for a
person. **Nothing reaches the list unasked.**

```
Google Tasks list ──poll (default 60 s)──▶ record request (unchecked)
                                            │ committed
                                            ▼
                                     DELETE the task ──▶ triage ──▶ Pending / Triage
```

## Checked assumptions (2026-10-02)
The spec's brief guessed at Google; checked before building:

| Spec says | Found |
|---|---|
| "Hey Google, add X to my grocery list" lands in Google Tasks | **No.** Assistant / Gemini voice shopping items go to Google's own shopping list (Google Home / Express); Gemini can add to *Keep* lists. Neither has a public API. Tasks is for typed items. |
| Scope `https://www.googleapis.com/auth/tasks` | Right. It is a sensitive scope: an unverified app shows "Google hasn't verified this app" (fine for one household, under 100 users). |
| Publish the consent screen "In production" to avoid the 7-day refresh-token expiry | Right — "Testing" refresh tokens expire after 7 days. |
| Delete after commit | Built that way, and tested (a failed delete is retried, no second card). |

## Polling rules
- One list, chosen on `/settings/intake`; only open tasks
  (`showCompleted=false`, `showHidden=false`), every page up to 1000 tasks.
- Polling runs only when **switched on, signed in, and a list is chosen**.
  Interval 30 s – 1 h (`poll_seconds`), re-read every round, so a change needs
  no restart.
- One poll at a time (`GoogleTasks.poll_lock`): "Check now", the background
  loop and Restore never interleave.
- Per task: blank title → left on the list (`blank`). Otherwise
  `services/google_tasks/title.rs` reads `2 oat milk`, `2x milk`,
  `milk x2`, `milk × 3`; the raw title is kept as `raw_text`.
- Record, **then** delete. The task id is the request's `external_id`; the
  unique index `(source, external_id)` makes a re-poll a no-op
  (`already_seen`), whose delete is retried.
- A failed poll stores a sentence for people in `last_poll_error` (never a
  database error: those say "see its log"). Never retried early.
- The background loop is off when `GOOGLE_TASKS_NO_BACKGROUND_POLL=true`
  (integration tests, e2e) — tests poll by hand.

## Sign-in
OAuth 2.0 web flow, `access_type=offline`, `prompt=consent`, scope `tasks`.
The redirect URI is the **web app's** `/settings/intake` page, which posts
`code` + `state` to the backend with the member's session — no
unauthenticated callback exists. `state` is 244 random bits, stored only as
SHA-256, valid 10 minutes, single use (`google_oauth_states`).
The refresh token is AES-256-GCM encrypted (`CREDENTIAL_ENCRYPTION_KEY`) in
`google_tasks_link`, decrypted only for a call, never logged or returned.
Access tokens are cached in memory until a minute before expiry. A 401 from
the API or `invalid_grant` from the token endpoint → "connect Google Tasks
again" (409). Disconnect forgets the token and stops polling; it does not
revoke it at Google (the person can remove access in their Google account).

Google only accepts `http` redirect URIs on `localhost`, so with the default
`GOOGLE_REDIRECT_URI` the household connects from a browser on the home
server. Another address needs HTTPS on a real domain.

## Restore to source
`POST /api/triage/{id}/restore` (Triage → **Put back in Google Tasks**, only
on Tasks items). Holding the poll lock and the request row lock, it inserts a
task with the original `raw_text`, records `intake_restores (tasks, new task
id)`, and closes the request as `rejected`. The next poll that sees that id
records it with triage `skipped` → Pending Requests. If Google refuses,
nothing changes. This is the spec's `force_accept`, kept server-side so a
typed marker can never skip triage.

## Endpoints (all Clerk)
| Method | Path | Answer |
|---|---|---|
| `GET` | `/api/intake/settings` | every channel's status, the triage provider; never a secret |
| `POST` | `/api/intake/google-tasks/sign-in` | `{authorize_url}`; 500 when the client or encryption key is missing |
| `POST` | `/api/intake/google-tasks/sign-in/finish` | `{code, state}` → status; 409 bad/expired/reused state or code |
| `DELETE` | `/api/intake/google-tasks/sign-in` | 204 |
| `GET` | `/api/intake/google-tasks/lists` | lists on the account; 409 not connected |
| `PUT` | `/api/intake/google-tasks` | `{task_list_id, enabled, poll_seconds}`; 404 list not on the account, 409 on without a list, 422 interval |
| `POST` | `/api/intake/google-tasks/poll` | "Check now" → `{recorded, already_seen, blank, not_deleted}`; 409 not set up; 503 Google unavailable |
| `POST` | `/api/dev/google-tasks/tasks` | fake only: add a task to "Groceries"; 404 against real Google |

## Settings
`GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URI`
(default `http://localhost:3000/settings/intake`), `GOOGLE_TASKS_CLIENT`
(`live` | `fake`), `GOOGLE_TIMEOUT_SECONDS` (15). Test-only overrides:
`GOOGLE_OAUTH_AUTH_URL`, `GOOGLE_OAUTH_TOKEN_URL`, `GOOGLE_TASKS_API_BASE`,
`GOOGLE_TASKS_NO_BACKGROUND_POLL`. The three addresses must be `https`
(plain `http` only to this machine), because the client secret and tokens
travel to them; the backend refuses to start otherwise.

## Code
| Piece | File |
|---|---|
| Settings | `backend/src/config/google_tasks.rs` |
| Client trait, live client, fake | `backend/src/services/google_tasks/{api,live/,fake}.rs` |
| Poll, schedule, restore, connect | `services/google_tasks/{poller,schedule,restore,connection}.rs` |
| SQL | `services/google_tasks/{link_repository,oauth_states}.rs`, `services/voice/restore_repository.rs` |
| Recording a polled item | `services/voice/polled.rs` |
| Routes | `backend/src/routes/{google_tasks,intake_settings,triage}.rs` |
| Migration | `backend/migrations/20261002130000_google_tasks_intake.sql` |
| Page | `frontend/src/routes/settings/intake.tsx`, `components/intake/`, `hooks/useGoogleTasks.ts`, `hooks/useGoogleSignInReturn.ts` |
| Restore button | `components/triage/triage-request-card.tsx`, `hooks/useTriageDecisions.ts` |

## Tests
- Backend unit: title parsing, sign-in URL, state hashing, fake, settings.
- Backend integration: `tests/google_tasks.rs` (settings, lists, choices,
  polling, auth), `google_tasks_sign_in.rs` (state once, bad code, no key,
  disconnect), `google_tasks_restore.rs`, and `google_tasks_live.rs`
  (`wiremock` as Google: token form, delete after record, failed delete
  retried, Google down, revoked sign-in).
- Frontend unit: the settings card, other channels, Restore button, poll
  wording. e2e `frontend/e2e/google-tasks.spec.ts` (desktop + phone): fake
  sign-in round trip, Check now → Pending, Restore → Pending.

## Known gaps
- Never run against a real Google account (no account in CI, by rule).
- Subtasks are treated like tasks. Task notes are ignored.
- Disconnect does not revoke the token at Google.
