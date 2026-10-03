# Access logs — working on it

Spec: `docs/features/FEATURE_ACCESS_LOGS.md`. Rules that never bend: **every
answered request is logged**, the **query string is never stored**, and the
database write **never delays a response**.

## Layout
| File | Change it when |
|---|---|
| `middleware/access_log/mod.rs` | what is gathered per request, or where the layer sits |
| `middleware/access_log/request_info.rs` | how the IP, forwarded header or path is read (pure, unit-tested) |
| `auth/request_user.rs` | how the signed-in user reaches the middleware |
| `services/access_log/mod.rs` | `record()` (file now, row in background), page size, health-check hiding |
| `services/access_log/entry.rs` | the entry's fields, `unauthenticated` |
| `services/access_log/file.rs` | the file line's fields (target `access_log`) |
| `services/access_log/repository.rs` | the SQL |
| `logging.rs` | file rotation, files kept, console filter |
| `routes/access_logs.rs` | `GET /api/access-logs` |

## Adding a field
1. A migration adding the column (never edit `0008`).
2. `AccessLogEntry`, `AccessLogRow`, `AccessLogResponse`, the INSERT and the
   SELECT, and `file::write_line`.
3. Gather it in the middleware (pure helper in `request_info.rs` + unit test).
4. `frontend/src/lib/api/access-logs.ts` and `access-log-row.tsx`.

## The user id
The middleware never verifies a token. It inserts an empty `RequestUser` into
the request's extensions; `AuthUser::from_request_parts` fills it after a
successful check. A route that does not take `AuthUser` (intake, health) is
logged as `unauthenticated`. Swapping the auth provider needs no change here.

## Testing
Rows are written after the response, on a background task. Integration tests
wait for them with `tests/common/access_logs.rs` (`wait_for_path`,
`wait_for_logged`, 5 s deadline) — never read the table straight after a
request. `oneshot` has no connection info; insert
`ConnectInfo<SocketAddr>` into the request's extensions to test `source_ip`.
The file layer is unit-tested in `logging.rs` with a local subscriber, because
the global one can only be set once per process.
