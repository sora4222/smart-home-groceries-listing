-- Undo for an item's chosen products.
--
-- Before a change the web app saves the item's products here; Undo copies
-- them back in one transaction, with the details the store gave then, so no
-- store is searched again. A snapshot outlives its item, so Undo of a
-- removal can put the products on the restored item. Snapshots older than a
-- day are deleted when a new one is taken.

CREATE TABLE selection_snapshots (
    id              UUID PRIMARY KEY,
    grocery_item_id UUID NOT NULL,
    taken_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One row per saved product: a copy of its `item_selections` row.
CREATE TABLE selection_snapshot_rows (
    snapshot_id UUID NOT NULL REFERENCES selection_snapshots (id) ON DELETE CASCADE,
    LIKE item_selections
);

CREATE INDEX ix_selection_snapshot_rows_snapshot ON selection_snapshot_rows (snapshot_id);
CREATE INDEX ix_selection_snapshots_taken_at ON selection_snapshots (taken_at);
