-- Reviewing, annotating and committing the household list.
--
-- Three changes to `grocery_items`, all for the web app's list view:
--
--   note          free text a household member attaches to an item
--                 ("the 3-ply one", "only if it is on special")
--   filter_terms  terms that narrow the later product search, shown in the
--                 UI as removable chips. The spec's Item Rules feature will
--                 populate these automatically; a user can add or remove them
--                 by hand today.
--   'committed'   a fourth status between 'active' and 'ordered': the list has
--                 been reviewed and is locked in for purchase. Committed items
--                 are read-only until they are released back to 'active'.

ALTER TABLE grocery_items
    ADD COLUMN note TEXT CHECK (note IS NULL OR char_length(note) BETWEEN 1 AND 500),
    -- Bounds live here as well as in `validator` so a service bug cannot
    -- write a row the list view is unable to render. A CHECK cannot contain a
    -- subquery, so per-element length is bounded via the joined form rather
    -- than `unnest`.
    ADD COLUMN filter_terms TEXT[] NOT NULL DEFAULT '{}'
        CHECK (coalesce(array_length(filter_terms, 1), 0) <= 10
               AND array_position(filter_terms, NULL) IS NULL
               AND '' <> ALL (filter_terms)
               AND char_length(array_to_string(filter_terms, ',')) <= 600);

-- Widening an enumeration is a constraint change precisely because these
-- columns are TEXT + CHECK rather than a PostgreSQL ENUM.
ALTER TABLE grocery_items DROP CONSTRAINT grocery_items_status_check;
ALTER TABLE grocery_items ADD CONSTRAINT grocery_items_status_check
    CHECK (status IN ('pending', 'active', 'committed', 'ordered'));

-- Removing an item from the list must work even when a voice request points at
-- it. The request is a record of a decision and outlives the item it produced,
-- so the reference is cleared rather than blocking the delete.
ALTER TABLE voice_requests DROP CONSTRAINT voice_requests_grocery_item_id_fkey;
ALTER TABLE voice_requests ADD CONSTRAINT voice_requests_grocery_item_id_fkey
    FOREIGN KEY (grocery_item_id) REFERENCES grocery_items (id) ON DELETE SET NULL;
