-- Voice list changes: removing an item, or lowering its quantity, by voice.
--
-- Adding by voice goes through the confirmation queue (`voice_requests`).
-- Taking away does not: "Alexa, remove milk" changes the list at once, since
-- it can only ever shrink what the household buys. To keep it reversible,
-- every change is recorded here with enough of the item to put it back, and
-- "Alexa, undo" reverses the newest one.
--
-- voice_list_changes — one spoken remove or reduce
--   source            the intake channel (only 'alexa' today)
--   external_id       the channel's request id; unique per source, so a
--                     retried delivery is not applied twice
--   kind              removed — the item left the list (quantity_after = 0)
--                     reduced — its quantity went down
--   grocery_item_id   the item changed. No foreign key: a removed item's row
--                     is deleted, and Undo re-inserts it with this same id.
--   item_*            a snapshot of the item before the change, so Undo can
--                     re-insert a removed item as it was. Its chosen product
--                     (item_selections) is not kept; the household picks again.
--   undone_at         when Undo reversed it; NULL while it stands
--   undo_external_id  the request id of the Undo; unique per source, so a
--                     retried Undo does not also reverse the change before it

CREATE TABLE voice_list_changes (
    id                    UUID        PRIMARY KEY,
    source                TEXT        NOT NULL CHECK (source IN ('webhook', 'alexa')),
    external_id           TEXT        CHECK (external_id IS NULL
                                             OR char_length(external_id) BETWEEN 1 AND 255),
    kind                  TEXT        NOT NULL CHECK (kind IN ('reduced', 'removed')),
    grocery_item_id       UUID        NOT NULL,
    item_name             TEXT        NOT NULL CHECK (char_length(item_name) BETWEEN 1 AND 200),
    quantity_before       INTEGER     NOT NULL CHECK (quantity_before BETWEEN 1 AND 999),
    quantity_after        INTEGER     NOT NULL CHECK (quantity_after BETWEEN 0 AND 998),
    item_source           TEXT        NOT NULL CHECK (item_source IN ('voice', 'manual')),
    item_note             TEXT,
    item_filter_terms     TEXT[]      NOT NULL DEFAULT '{}',
    item_added_by_user_id TEXT,
    item_created_at       TIMESTAMPTZ NOT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    undone_at             TIMESTAMPTZ,
    undo_external_id      TEXT        CHECK (undo_external_id IS NULL
                                             OR char_length(undo_external_id) BETWEEN 1 AND 255),
    CHECK ((kind = 'removed') = (quantity_after = 0)),
    CHECK (quantity_after < quantity_before)
);

CREATE UNIQUE INDEX ux_voice_list_changes_source_external_id
    ON voice_list_changes (source, external_id)
    WHERE external_id IS NOT NULL;

CREATE UNIQUE INDEX ux_voice_list_changes_source_undo_external_id
    ON voice_list_changes (source, undo_external_id)
    WHERE undo_external_id IS NOT NULL;

-- Undo looks for the newest change still standing.
CREATE INDEX ix_voice_list_changes_standing
    ON voice_list_changes (source, created_at DESC)
    WHERE undone_at IS NULL;
