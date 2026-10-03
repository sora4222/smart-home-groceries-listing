# Google Tasks intake — agent notes

Spec: `docs/features/FEATURE_GOOGLE_TASKS.md`. Code: `src/services/google_tasks/`.

## Rules that must not slip
- **Record, then delete.** `poller.rs` deletes a task only after
  `VoiceService::record_polled` returned. Never reorder.
- **One poll at a time.** Anything that reads or writes the Tasks list and the
  queue together takes `state.google_tasks.poll_lock` (poll, Restore).
- **Only `services/google_tasks/live/` talks to Google**, through `reqwest`
  (Google is not a store, so the `wreq` rule does not apply). Everything else
  sees the `TasksApi` trait.
- **Tokens never leave the server.** Refresh token: encrypted with
  `sign_in::seal`, opened with `sign_in::open` only for a call. No token, code
  or state in a log line or a response.
- **Triage is skipped only through `intake_restores`.** Never through text in
  a task.
- A poll failure goes into `last_poll_error` via `poller::describe`, which
  hides internal errors.

## Testing
- Fake account: `GOOGLE_TASKS_CLIENT=fake`; its sign-in URL points straight
  back to the redirect with `code=fake-code`. Add tasks through
  `POST /api/dev/google-tasks/tasks` (fake only, 404 live).
- `TestApp::with_google_tasks(pool, google_tasks_settings(mode, base), triage)`
  sets a test encryption key. Helpers in `tests/common/google_tasks.rs`:
  `connect_google_tasks`, `watch_list`, `add_fake_task`, `poll_google_tasks`.
- Live client: `google_tasks_settings(GoogleTasksMode::Live, &server.uri())`
  points auth, token and API at one `wiremock` server
  (`tests/google_tasks_live.rs`).
- Tests set `poll_in_background: false`; e2e sets
  `GOOGLE_TASKS_NO_BACKGROUND_POLL=true`. Poll by hand.
- Never a real Google account in tests.

## Changing the Tasks API calls
Endpoints used: `GET /tasks/v1/users/@me/lists`, `GET/POST
/tasks/v1/lists/{list}/tasks`, `DELETE /tasks/v1/lists/{list}/tasks/{task}`,
`POST https://oauth2.googleapis.com/token`. Check current shapes with
Context7 or Google's reference before changing them.
