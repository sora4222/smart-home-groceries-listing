-- Network access logging (docs/features/FEATURE_ACCESS_LOGS.md).
--
-- One row per HTTP request the backend answered. Written in the background
-- after the response, so a slow or missing database never delays a request.
--
--   source_ip      — the address that opened the connection. Behind the
--                    Cloudflare Tunnel this is the cloudflared container.
--   forwarded_for  — what the request's CF-Connecting-IP or X-Forwarded-For
--                    header claimed. Anyone can send these headers, so the
--                    value is shown, never trusted.
--   user_id        — the signed-in household member, or 'unauthenticated'.
--   path           — the path only. The query string is never stored, so a
--                    secret passed in a URL cannot end up in this table.

CREATE TABLE access_logs (
    id            BIGSERIAL PRIMARY KEY,
    occurred_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    source_ip     TEXT CHECK (source_ip IS NULL OR char_length(source_ip) <= 64),
    forwarded_for TEXT CHECK (forwarded_for IS NULL OR char_length(forwarded_for) <= 256),
    user_id       TEXT NOT NULL CHECK (char_length(user_id) <= 256),
    method        TEXT NOT NULL CHECK (char_length(method) <= 16),
    path          TEXT NOT NULL CHECK (char_length(path) <= 2048),
    status_code   SMALLINT NOT NULL,
    duration_ms   INTEGER NOT NULL CHECK (duration_ms >= 0)
);

-- The viewer pages newest first by id, which the primary key already serves.
