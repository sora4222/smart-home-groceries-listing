-- Several products for one list item, with how many of each.
--
-- An item wanting more than one may now have several chosen products at a
-- store, as long as there are no more products than the item's quantity.
--
--   quantity    how many of this product to buy. NULL while it is the only
--               product at its store: it then buys the item's whole quantity.
--               The app keeps the counts at a store adding up to the item's
--               quantity, also after the quantity is edited.
--   for_order   now true on every product at the store the order buys from.
--
-- A trolley handoff can then hold several lines for one item, so its lines
-- are keyed by position.

ALTER TABLE item_selections
    ADD COLUMN quantity INTEGER CHECK (quantity IS NULL OR quantity BETWEEN 1 AND 999);

ALTER TABLE item_selections DROP CONSTRAINT item_selections_item_store_key;

ALTER TABLE item_selections
    ADD CONSTRAINT item_selections_item_store_product_key
        UNIQUE (grocery_item_id, store, product_id);

DROP INDEX ix_item_selections_one_for_order;

ALTER TABLE trolley_handoff_lines DROP CONSTRAINT trolley_handoff_lines_pkey;

ALTER TABLE trolley_handoff_lines ADD PRIMARY KEY (handoff_id, position);
