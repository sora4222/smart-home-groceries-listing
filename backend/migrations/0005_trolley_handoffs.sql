-- Trolley handoffs: chosen products waiting to be put into a store's online
-- trolley by the household's own, logged-in browser tab.
--
-- Why a handoff and not a server-side login: Woolworths login is a
-- JavaScript flow with passkeys and MFA, so the server cannot log in on its
-- own. Instead the web app writes the lines here, the household member opens
-- the store's website where they are already logged in, and the "Fill
-- trolley" bookmarklet claims the handoff, adds each line through the
-- website's own trolley call, and reports what happened.
-- See docs/FEAT_WOOLWORTHS_ACCESS.md.
--
-- Lifecycle (status):
--   waiting_for_store_tab   created by the web app; claimable until expires_at
--   claimed_by_store_tab    the bookmarklet took it and is filling the trolley
--   filled                  every line was added
--   filled_with_problems    at least one line could not be added
--   replaced                a newer handoff for the same store superseded it

CREATE TABLE trolley_handoffs (
    id           UUID         PRIMARY KEY,
    store        TEXT         NOT NULL CHECK (store IN ('woolworths', 'coles')),
    status       TEXT         NOT NULL CHECK (status IN (
                                  'waiting_for_store_tab', 'claimed_by_store_tab',
                                  'filled', 'filled_with_problems', 'replaced')),
    created_by   TEXT         NOT NULL,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ  NOT NULL,
    claimed_at   TIMESTAMPTZ,
    reported_at  TIMESTAMPTZ
);

-- At most one claimable handoff per store is looked up at a time.
CREATE INDEX trolley_handoffs_waiting
    ON trolley_handoffs (store, created_at DESC)
    WHERE status = 'waiting_for_store_tab';

-- One line per list item: the chosen product and how many to put in the
-- trolley, in list order (`position`). `outcome` and `problem` are written by the bookmarklet's report.
-- Lines are a snapshot: removing the list item later keeps the history.
CREATE TABLE trolley_handoff_lines (
    handoff_id       UUID     NOT NULL REFERENCES trolley_handoffs (id) ON DELETE CASCADE,
    position         INTEGER  NOT NULL CHECK (position >= 0),
    grocery_item_id  UUID     NOT NULL,
    product_id       TEXT     NOT NULL CHECK (char_length(product_id) BETWEEN 1 AND 100),
    product_name     TEXT     NOT NULL CHECK (char_length(product_name) BETWEEN 1 AND 300),
    quantity         INTEGER  NOT NULL CHECK (quantity BETWEEN 1 AND 999),
    outcome          TEXT     CHECK (outcome IN ('added', 'failed')),
    problem          TEXT     CHECK (problem IS NULL OR char_length(problem) <= 300),
    PRIMARY KEY (handoff_id, grocery_item_id)
);
