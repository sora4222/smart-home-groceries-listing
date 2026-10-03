-- Product dislikes: a household member does not want a store's product again.
--
-- Dislikes are per member (spec "Purchase History and Preferences"). The
-- household view is every member's rows together. A dislike never hides a
-- product: the price comparison still offers it, with a badge saying who
-- disliked it. Only the automated order (the optimiser, not built yet) skips
-- a disliked product.
--
--   user_id             the member's id from the auth provider
--   user_name           how other members see them on the badge, saved when
--                         they disliked it (the token may not carry a name)
--   store, product_id   the store and its own id for the product
--   product_name,       a label for the household list, as the price
--   brand, package_size   comparison showed it
--
-- One row per member per product: disliking again only refreshes the label.
-- Removing the row is the "permanent override" of the spec.

CREATE TABLE product_dislikes (
    id            UUID         PRIMARY KEY,
    user_id       TEXT         NOT NULL CHECK (char_length(user_id) BETWEEN 1 AND 200),
    user_name     TEXT         NOT NULL CHECK (char_length(user_name) BETWEEN 1 AND 200),
    store         TEXT         NOT NULL CHECK (store IN ('woolworths', 'coles')),
    product_id    TEXT         NOT NULL CHECK (char_length(product_id) BETWEEN 1 AND 100),
    product_name  TEXT         NOT NULL CHECK (char_length(product_name) BETWEEN 1 AND 300),
    brand         TEXT         CHECK (brand IS NULL OR char_length(brand) <= 200),
    package_size  TEXT         CHECK (package_size IS NULL OR char_length(package_size) <= 100),
    disliked_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (user_id, store, product_id)
);

CREATE INDEX ix_product_dislikes_product ON product_dislikes (store, product_id);

-- Dislike overrides for one order: "buy it this time anyway".
--
-- An override belongs to one list item, so it lasts for this order only. The
-- next time the item is added it is a new row, and the dislike counts again.
-- Removing the item removes its overrides. The dislikes themselves are kept.

CREATE TABLE dislike_overrides (
    id               UUID         PRIMARY KEY,
    grocery_item_id  UUID         NOT NULL
                                  REFERENCES grocery_items (id) ON DELETE CASCADE,
    store            TEXT         NOT NULL CHECK (store IN ('woolworths', 'coles')),
    product_id       TEXT         NOT NULL CHECK (char_length(product_id) BETWEEN 1 AND 100),
    overridden_by    TEXT         NOT NULL CHECK (char_length(overridden_by) BETWEEN 1 AND 200),
    overridden_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (grocery_item_id, store, product_id)
);
