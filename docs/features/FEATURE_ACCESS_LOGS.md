# Feature: Network access logging

Status: **implemented** — every request the backend answers is written to a
daily log file and to the `access_logs` table, and the `/logs` page shows the
table. Steps for people: `docs/using-the-app.md` ("See who used the app") —
keep it in step with this spec.

## What is recorded
One entry per answered request, including refusals (401, 404, a body that is
too big, a timeout) and CORS preflights:

| Field | Where it comes from |
|---|---|
| `occurred_at` | when the row was stored (UTC) |
| `source_ip` | the address that opened the connection. Behind the Cloudflare Tunnel this is the `cloudflared` container, not the visitor. `null` when unknown (in-process tests). |
| `forwarded_for` | what `CF-Connecting-IP` (first) or `X-Forwarded-For` claimed. **Anyone can send these headers**, so this is shown, never trusted or used to decide anything. |
| `user_id` | the signed-in household member's id, or `unauthenticated` (intake endpoints, the health check, a missing or bad token) |
| `method`, `path` | the request line. **Only the path**: the query string is never stored, so a secret in a URL cannot reach the log. |
| `status_code`, `duration_ms` | the answer, and how long until it was ready |

## How it works
```
request ─▶ access-log middleware (outermost layer, lib.rs)
             │ puts an empty RequestUser slot in the request's extensions
             ▼
           route ─▶ AuthUser extractor verifies the token, fills the slot
             ▼
           response ready ─▶ file line now (tracing target "access_log")
                           └▶ table row on a background task
```

- **No second token check.** The middleware cannot know the user until a
  route's `AuthUser` extractor has verified the token, so the extractor
  records the id in a `RequestUser` slot (`backend/src/auth/request_user.rs`)
  and the middleware reads it once the response is ready. A future auth
  provider gets this for free: it lives in the extractor, not in Clerk code.
- **The database never delays a request.** The row is written by a
  `tokio::spawn` task after the response. If the database is slow or down,
  the row is dropped with a warning; the file line is already written.
- **The file** is `access.YYYY-MM-DD.log` in `ACCESS_LOG_DIR`, one per day,
  the newest 30 kept (`backend/src/logging.rs`). Only access-log lines go
  there; the console keeps following `RUST_LOG`. Compose sets
  `ACCESS_LOG_DIR=/var/log/grocery` and keeps it in the `access_logs` volume.
  Unset means no file. A folder that cannot be used is a warning, not a
  failure to start.

  ```
  2026-10-02T04:52:58.626233Z  INFO request ip=127.0.0.1 forwarded_for=- user=dev-user method=GET path=/api/grocery-items status=200 duration_ms=3
  ```

## Endpoint

| Method | Path | Auth | Answer |
|---|---|---|---|
| `GET` | `/api/access-logs?before=&limit=&hide_health_checks=` | Clerk | 200 `{ entries, next_before }`, newest first; 400 bad parameter |

- `limit` defaults to 100, clamped to 1–500.
- `before` is the previous page's `next_before`; `next_before` is `null` when
  the page was not full.
- `hide_health_checks` defaults to `true`: the container asks `/api/health`
  every 10 seconds (8,640 rows a day), which would bury everything else.

There is no admin role in this POC: every signed-in member may read the log.

## The `/logs` page
Newest first. Each row: status badge (red for 5xx, outlined for 4xx), method,
path, time, who ("Not signed in" for `unauthenticated`), source (with any
forwarded claim in brackets) and duration. **Show health checks** reloads from
the newest row; **Load older** appends the next page. Linked from the nav as
**Logs**.

## Not built
- **Retention for the table.** Rows are never deleted. At ~10,000 rows a day
  (mostly health checks) PostgreSQL copes for years, but a clean-up job would
  be the next step if the table grows.
- **Filtering by user or path** on the page.

## Code
| Part | File |
|---|---|
| Table | `backend/migrations/0008_access_logs.sql` |
| Middleware | `backend/src/middleware/access_log/` (`request_info.rs`: IP, headers, path) |
| User slot | `backend/src/auth/request_user.rs` |
| Record + page rules | `backend/src/services/access_log/` |
| File set-up | `backend/src/logging.rs`, `backend/src/config/access_log.rs` |
| Route | `backend/src/routes/access_logs.rs` |
| Page | `frontend/src/routes/logs.tsx`, `components/access-logs/`, `hooks/useAccessLogPages.ts` |
| Tests | `backend/tests/access_logs.rs`, `frontend/e2e/access-logs.spec.ts` |
