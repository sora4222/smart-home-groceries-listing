-- Item rules: persistent filters attached to grocery item names.
--
-- A rule says "whenever an item called <trigger> reaches the list, give it
-- these filter chips". The spec's example: triggers {"toilet paper"}, filter
-- terms {"3 ply"} — saying "add toilet paper" then narrows the product search
-- to 3-ply products without anyone typing the chip.
--
--   triggers         item names or phrases to match, case-insensitively and on
--                    whole words. Several per rule, so "toilet paper" and
--                    "loo roll" can share one set of filters.
--   filter_terms     the chips copied onto a matching item. Bounded exactly as
--                    grocery_items.filter_terms is, since that is where they go.
--   apply_to_manual  voice items always get the rule; items typed into the web
--                    app only when this is on. Off by default, per the spec.
--
-- Chips are copied, not referenced: editing or deleting a rule never changes an
-- item already on the list, and taking a chip off an item never edits the rule.

CREATE TABLE item_rules (
    id              UUID        PRIMARY KEY,
    -- A CHECK cannot contain a subquery, so per-element length is bounded via
    -- the joined form rather than `unnest`, as in migration 0002.
    triggers        TEXT[]      NOT NULL
                                CHECK (coalesce(array_length(triggers, 1), 0) BETWEEN 1 AND 10
                                       AND array_position(triggers, NULL) IS NULL
                                       AND '' <> ALL (triggers)
                                       AND char_length(array_to_string(triggers, ',')) <= 1010),
    filter_terms    TEXT[]      NOT NULL
                                CHECK (coalesce(array_length(filter_terms, 1), 0) BETWEEN 1 AND 10
                                       AND array_position(filter_terms, NULL) IS NULL
                                       AND '' <> ALL (filter_terms)
                                       AND char_length(array_to_string(filter_terms, ',')) <= 600),
    apply_to_manual BOOLEAN     NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Manual additions only consult the rules that opted in.
CREATE INDEX ix_item_rules_apply_to_manual ON item_rules (apply_to_manual)
    WHERE apply_to_manual;
