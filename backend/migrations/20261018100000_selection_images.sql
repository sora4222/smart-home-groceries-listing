-- The chosen product's picture, so the list and the order screen can show it.
--
-- A link on the store's image server (always https), taken from the store's
-- search answer when the product was chosen. Earlier choices have none and
-- show without a picture until they are chosen again.

ALTER TABLE item_selections ADD COLUMN image_url TEXT;
