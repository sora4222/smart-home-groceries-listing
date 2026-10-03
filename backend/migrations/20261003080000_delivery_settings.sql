-- Delivery fees and the order planner's preferences (Settings › Delivery).
--
-- The app cannot read a store's delivery fees without the household's own
-- logged-in browser (docs/FEAT_WOOLWORTHS_ACCESS.md), so the household types
-- each store's fee rules once. The order planner adds them to every option.
--
-- store_delivery_fees: one row per store, written the first time it is saved.
--   delivery_fee        what one delivery costs; NULL = not set yet
--   free_delivery_over  an order this big or bigger delivers free; NULL = never
--   minimum_order       the store refuses an order smaller than this; NULL = none
--
-- order_preferences: a single row (id = 1) once saved.
--   mode                the planner's default way of ranking options
--   max_delivery_spend  options whose delivery fees add up to more are not
--                       recommended; NULL = no cap

CREATE TABLE store_delivery_fees (
    store               TEXT          PRIMARY KEY CHECK (store IN ('woolworths', 'coles')),
    delivery_fee        NUMERIC       CHECK (delivery_fee BETWEEN 0 AND 1000),
    free_delivery_over  NUMERIC       CHECK (free_delivery_over BETWEEN 0 AND 1000),
    minimum_order       NUMERIC       CHECK (minimum_order BETWEEN 0 AND 1000),
    updated_by          TEXT          NOT NULL,
    updated_at          TIMESTAMPTZ   NOT NULL DEFAULT now()
);

CREATE TABLE order_preferences (
    id                  SMALLINT      PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    mode                TEXT          NOT NULL CHECK (mode IN (
                            'minimise_total', 'minimise_delivery',
                            'woolworths_only', 'coles_only', 'manual')),
    max_delivery_spend  NUMERIC       CHECK (max_delivery_spend BETWEEN 0 AND 1000),
    updated_by          TEXT          NOT NULL,
    updated_at          TIMESTAMPTZ   NOT NULL DEFAULT now()
);
