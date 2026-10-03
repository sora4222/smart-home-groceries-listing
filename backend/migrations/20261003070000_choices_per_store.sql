-- One product choice per store for each list item, and which one to buy.
--
-- Before this, an item had one chosen product, at one store. To find the
-- cheapest order across Woolworths and Coles the app must know what the
-- household would buy at each store, so an item may now keep one choice per
-- store. The household still picks every product; the order planner only
-- decides which of the item's choices goes in the order.
--
--   for_order   this is the choice the order uses. Exactly one per item that
--               has any choice: choosing a product makes it the one to buy,
--               and the order planner can switch it to the other store.
--
-- Every existing row was the item's only choice, so it is the one to buy.

ALTER TABLE item_selections DROP CONSTRAINT item_selections_grocery_item_id_key;

ALTER TABLE item_selections ADD COLUMN for_order BOOLEAN NOT NULL DEFAULT true;

ALTER TABLE item_selections
    ADD CONSTRAINT item_selections_item_store_key UNIQUE (grocery_item_id, store);

CREATE UNIQUE INDEX ix_item_selections_one_for_order
    ON item_selections (grocery_item_id) WHERE for_order;
