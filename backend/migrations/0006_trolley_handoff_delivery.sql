-- Delivery time for a trolley handoff.
--
-- Before adding products, the "Fill trolley" bookmarklet reserves a delivery
-- window on the store's website, so prices and availability are for a real
-- delivery. The household can change the window on the store's website later.
--
-- What the web app asked for:
--   delivery_date          the day wanted; NULL = the day after the store's
--                          own "today" (the default)
--   delivery_time_of_day   any | morning (starts before 12pm) |
--                          afternoon (12pm–5pm) | evening (5pm or later)
--
-- What the store tab reported:
--   delivery_outcome       reserved (a window was chosen) | kept (the
--                          reserved window already fitted) | failed
--   delivery_window_label  the store's own words, e.g. "7:00am - 10:00am"
--   delivery_window_start, the window, in the store's local time
--   delivery_window_end
--   delivery_fee           the window's fee
--   delivery_problem       why no window was reserved, or why another day was used

ALTER TABLE trolley_handoffs
    ADD COLUMN delivery_date          DATE,
    ADD COLUMN delivery_time_of_day   TEXT NOT NULL DEFAULT 'any'
        CHECK (delivery_time_of_day IN ('any', 'morning', 'afternoon', 'evening')),
    ADD COLUMN delivery_outcome       TEXT
        CHECK (delivery_outcome IN ('reserved', 'kept', 'failed')),
    ADD COLUMN delivery_window_label  TEXT
        CHECK (delivery_window_label IS NULL OR char_length(delivery_window_label) <= 100),
    ADD COLUMN delivery_window_start  TIMESTAMP,
    ADD COLUMN delivery_window_end    TIMESTAMP,
    ADD COLUMN delivery_fee           NUMERIC CHECK (delivery_fee IS NULL OR delivery_fee >= 0),
    ADD COLUMN delivery_problem       TEXT
        CHECK (delivery_problem IS NULL OR char_length(delivery_problem) <= 300);
