-- Initial schema: the household grocery list and the intake confirmation queue.
--
-- Status and source columns are TEXT with CHECK constraints rather than
-- PostgreSQL ENUM types: adding a value later is a one-line constraint change
-- instead of an ALTER TYPE that cannot run inside a transaction.

CREATE TABLE grocery_items (
    id               UUID        PRIMARY KEY,
    name             TEXT        NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    quantity         INTEGER     NOT NULL DEFAULT 1 CHECK (quantity BETWEEN 1 AND 999),
    status           TEXT        NOT NULL DEFAULT 'active'
                                 CHECK (status IN ('pending', 'active', 'ordered')),
    source           TEXT        NOT NULL CHECK (source IN ('voice', 'manual')),
    added_by_user_id TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ix_grocery_items_status ON grocery_items (status);

-- Duplicate detection compares names case- and whitespace-insensitively.
-- The expression is repeated verbatim in VoiceService's duplicate lookup so
-- the query can use this index; keep the two in step.
CREATE INDEX ix_grocery_items_normalised_name
    ON grocery_items (lower(btrim(regexp_replace(name, '\s+', ' ', 'g'))))
    WHERE status = 'active';

CREATE TABLE voice_requests (
    id              UUID        PRIMARY KEY,
    -- Which intake channel delivered this item.
    source          TEXT        NOT NULL DEFAULT 'webhook'
                                CHECK (source IN ('webhook', 'alexa')),
    -- The channel's own id for the item, when it has one. NULL for channels
    -- that do not supply one (the generic webhook).
    external_id     TEXT        CHECK (external_id IS NULL
                                       OR char_length(external_id) BETWEEN 1 AND 255),
    raw_text        TEXT        NOT NULL CHECK (char_length(raw_text) <= 2000),
    parsed_name     TEXT        NOT NULL CHECK (char_length(parsed_name) BETWEEN 1 AND 200),
    parsed_quantity INTEGER     NOT NULL DEFAULT 1
                                CHECK (parsed_quantity BETWEEN 1 AND 999),
    status          TEXT        NOT NULL DEFAULT 'pending'
                                CHECK (status IN ('pending', 'accepted', 'rejected')),
    grocery_item_id UUID        REFERENCES grocery_items (id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ix_voice_requests_status ON voice_requests (status);

-- Re-delivery of the same source item (an Alexa retry, a re-poll) must not
-- create a second card. Partial so the generic webhook's NULL external_id
-- rows are unconstrained.
CREATE UNIQUE INDEX ux_voice_requests_source_external_id
    ON voice_requests (source, external_id)
    WHERE external_id IS NOT NULL;
