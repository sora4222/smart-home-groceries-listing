-- Purchase history: what the household actually bought, and what it paid.
--
-- The app never sees the household pay — payment happens on the store's own
-- website. So a shop counts as bought once the "Fill trolley" bookmarklet
-- reports that it filled the store's trolley (the household chose this).
-- That report writes one `purchase_orders` row for the handoff and one
-- `purchases` row per product added, and moves those list items to `ordered`.
--
-- The version is a timestamp, not the next small number, because several
-- branches add migrations at once; two files with the same number would stop
-- the backend starting.
--
-- purchase_orders — one shop at one store
--   items_total     the sum of the lines' total_price
--   delivery_fee    the fee of the delivery window the bookmarklet reserved
--                   (0 when it reserved none)
--   recorded_by     the household member who sent the products to the store
--   source          how the shop was learned of: trolley_fill (the bookmarklet's
--                   report). Room for the stores' own order history later.
--   trolley_handoff_id  the handoff it came from; unique, so one report can
--                   never be counted twice
--
-- Undo: deleting a purchase_orders row deletes its purchases and puts each
-- list item back to `item_status_before`, if it is still `ordered`.
--
-- purchases — one product bought for one list item. A snapshot: deleting
-- the list item later keeps the history.
--   item_name       the list item's name when it was bought
--   item_status_before  the list item's status before it became `ordered`,
--                   so Undo can put it back
--   item_key        the same name lowercased with spaces collapsed, so
--                   "Milk" and " milk " are one item in the analysis
--   unit_price      what one cost, deals applied (total_price / quantity)
--   shelf_price     the store's price for one, when it showed one
--   total_price     what `quantity` cost, deals applied
--   delivery_fee_share  this line's part of the order's delivery fee, by cost
--   store_category  the store's own category for the product, when it gave one
--   category        the category shown in the analysis (see category_source)
--   category_source store — mapped from the store's category; name — guessed
--                   from the product name; none — no keyword matched ("Other")

CREATE TABLE purchase_orders (
    id            UUID         PRIMARY KEY,
    store         TEXT         NOT NULL CHECK (store IN ('woolworths', 'coles')),
    items_total   NUMERIC      NOT NULL CHECK (items_total >= 0),
    delivery_fee  NUMERIC      NOT NULL DEFAULT 0 CHECK (delivery_fee >= 0),
    recorded_by   TEXT         NOT NULL,
    source        TEXT         NOT NULL DEFAULT 'trolley_fill'
                               CHECK (source IN ('trolley_fill')),
    trolley_handoff_id UUID    UNIQUE REFERENCES trolley_handoffs (id) ON DELETE SET NULL,
    bought_at     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- The summary card shows the newest order.
CREATE INDEX ix_purchase_orders_bought_at ON purchase_orders (bought_at DESC);

CREATE TABLE purchases (
    id                  UUID         PRIMARY KEY,
    order_id            UUID         NOT NULL REFERENCES purchase_orders (id) ON DELETE CASCADE,
    grocery_item_id     UUID         REFERENCES grocery_items (id) ON DELETE SET NULL,
    item_name           TEXT         NOT NULL CHECK (char_length(item_name) BETWEEN 1 AND 200),
    item_key            TEXT         NOT NULL CHECK (char_length(item_key) BETWEEN 1 AND 200),
    item_status_before  TEXT         NOT NULL CHECK (item_status_before IN ('active', 'committed')),
    store               TEXT         NOT NULL CHECK (store IN ('woolworths', 'coles')),
    product_id          TEXT         NOT NULL CHECK (char_length(product_id) BETWEEN 1 AND 100),
    product_name        TEXT         NOT NULL CHECK (char_length(product_name) BETWEEN 1 AND 300),
    brand               TEXT         CHECK (brand IS NULL OR char_length(brand) <= 200),
    package_size        TEXT         CHECK (package_size IS NULL OR char_length(package_size) <= 100),
    quantity            INTEGER      NOT NULL CHECK (quantity BETWEEN 1 AND 999),
    unit_price          NUMERIC      NOT NULL CHECK (unit_price >= 0),
    shelf_price         NUMERIC      CHECK (shelf_price IS NULL OR shelf_price >= 0),
    total_price         NUMERIC      NOT NULL CHECK (total_price >= 0),
    delivery_fee_share  NUMERIC      NOT NULL DEFAULT 0 CHECK (delivery_fee_share >= 0),
    store_category      TEXT         CHECK (store_category IS NULL
                                            OR char_length(store_category) <= 200),
    category            TEXT         NOT NULL CHECK (char_length(category) BETWEEN 1 AND 100),
    category_source     TEXT         NOT NULL CHECK (category_source IN ('store', 'name', 'none')),
    bought_at           TIMESTAMPTZ  NOT NULL,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Spending analysis filters by date; the product picker looks up one product.
CREATE INDEX ix_purchases_bought_at ON purchases (bought_at);
CREATE INDEX ix_purchases_product ON purchases (store, product_id);
CREATE INDEX ix_purchases_item_key ON purchases (item_key);
CREATE INDEX ix_purchases_order ON purchases (order_id);
