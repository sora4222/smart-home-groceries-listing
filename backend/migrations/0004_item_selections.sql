-- Item selections: the one product the household chose for a list item.
--
-- The order screen reads these to know what to buy for each item. One row per
-- item at most (`grocery_item_id` is unique): choosing again replaces the row.
--
-- The product's details are a snapshot taken from the store's own search
-- answer at the moment of choosing — never from the request — so the order
-- screen can show what was chosen even when a store cannot be reached later.
-- Prices change, so the order screen re-prices before anything is bought.
--
--   store, product_id   the store and its own id for the product
--   price               shelf price for one, when the store showed one
--   unit_price,         the normalised unit price (per 100g, per 100mL or per
--   unit_price_per        unit) — both present or both absent
--   total_price         what `priced_quantity` cost with the best deal applied
--   priced_quantity     the item's quantity when the choice was made
--   selected_by         the household member who chose it
--
-- Removing an item removes its choice. Renaming an item or changing its chips
-- clears the choice in the service, since the product was chosen for what the
-- item used to be.

CREATE TABLE item_selections (
    id               UUID          PRIMARY KEY,
    grocery_item_id  UUID          NOT NULL UNIQUE
                                   REFERENCES grocery_items (id) ON DELETE CASCADE,
    store            TEXT          NOT NULL CHECK (store IN ('woolworths', 'coles')),
    product_id       TEXT          NOT NULL CHECK (char_length(product_id) BETWEEN 1 AND 100),
    product_name     TEXT          NOT NULL CHECK (char_length(product_name) BETWEEN 1 AND 300),
    brand            TEXT          CHECK (brand IS NULL OR char_length(brand) <= 200),
    package_size     TEXT          CHECK (package_size IS NULL OR char_length(package_size) <= 100),
    price            NUMERIC       CHECK (price IS NULL OR price >= 0),
    unit_price       NUMERIC       CHECK (unit_price IS NULL OR unit_price >= 0),
    unit_price_per   TEXT          CHECK (unit_price_per IN ('100g', '100mL', 'unit')),
    total_price      NUMERIC       CHECK (total_price IS NULL OR total_price >= 0),
    priced_quantity  INTEGER       NOT NULL CHECK (priced_quantity BETWEEN 1 AND 999),
    url              TEXT          NOT NULL CHECK (char_length(url) <= 2000),
    selected_by      TEXT          NOT NULL,
    selected_at      TIMESTAMPTZ   NOT NULL DEFAULT now(),
    CHECK ((unit_price IS NULL) = (unit_price_per IS NULL))
);
