-- Google Tasks intake (docs/features/FEATURE_GOOGLE_TASKS.md).
--
-- The version is a timestamp, like main's other new migrations, so branches
-- adding migrations at the same time never pick the same number.
--
-- The backend polls one Google Tasks list the household picks, records each
-- open task as an intake request (source 'tasks'), then deletes the task.
-- Triage's "Restore to source" puts a task back, marked so the next poll
-- lets it past triage.

-- 1. A new intake channel. The constraint is the auto-named one from 0001.
ALTER TABLE voice_requests DROP CONSTRAINT voice_requests_source_check;
ALTER TABLE voice_requests
    ADD CONSTRAINT voice_requests_source_check
        CHECK (source IN ('webhook', 'alexa', 'tasks'));

-- 2. The household's one link to Google Tasks. At most one row: `singleton`
--    is always true and unique.
CREATE TABLE google_tasks_link (
    id                      UUID        PRIMARY KEY,
    singleton               BOOLEAN     NOT NULL DEFAULT true UNIQUE CHECK (singleton),
    -- AES-256-GCM (services/encryption.rs). NULL when not connected.
    refresh_token_encrypted TEXT        CHECK (refresh_token_encrypted IS NULL
                                               OR char_length(refresh_token_encrypted) <= 4000),
    connected_at            TIMESTAMPTZ,
    -- The Tasks list that is the grocery inbox.
    task_list_id            TEXT        CHECK (task_list_id IS NULL
                                               OR char_length(task_list_id) BETWEEN 1 AND 255),
    task_list_title         TEXT        CHECK (task_list_title IS NULL
                                               OR char_length(task_list_title) <= 255),
    -- Polling only runs when enabled AND connected AND a list is chosen.
    enabled                 BOOLEAN     NOT NULL DEFAULT false,
    poll_seconds            INTEGER     NOT NULL DEFAULT 60
                                        CHECK (poll_seconds BETWEEN 30 AND 3600),
    last_polled_at          TIMESTAMPTZ,
    -- Why the last poll failed, in words for a person. NULL after a good one.
    last_poll_error         TEXT        CHECK (last_poll_error IS NULL
                                               OR char_length(last_poll_error) <= 500),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. One-use `state` values for the Google sign-in round trip. Only a
--    SHA-256 hash is kept; a row is deleted when used.
CREATE TABLE google_oauth_states (
    id         UUID        PRIMARY KEY,
    state_hash TEXT        NOT NULL UNIQUE CHECK (char_length(state_hash) = 64),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 4. Items a person put back in their source list from the Triage view. The
--    next poll that sees `external_id` records it with triage skipped.
CREATE TABLE intake_restores (
    id            UUID        PRIMARY KEY,
    source        TEXT        NOT NULL CHECK (source IN ('tasks')),
    external_id   TEXT        NOT NULL CHECK (char_length(external_id) BETWEEN 1 AND 255),
    restored_from UUID        NOT NULL REFERENCES voice_requests (id),
    restored_by   TEXT        NOT NULL CHECK (char_length(restored_by) BETWEEN 1 AND 255),
    used_at       TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (source, external_id)
);
