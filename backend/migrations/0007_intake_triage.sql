-- LLM triage of intake requests (docs/features/FEATURE_TRIAGE.md).
--
-- Every intake request is classified as a plausible supermarket purchase or
-- not before the household sees it. Triage never accepts an item: an
-- `approved` request still waits in the Pending Requests queue for a person.
--
--   unchecked — recorded, the classifier has not answered yet
--   approved  — looks like a supermarket item; shown in Pending Requests
--   rejected  — does not; shown in the Triage view's Rejected tab
--   held      — low confidence or the classifier failed; Held for review tab
--   skipped   — triage bypassed (switched off, or a person moved it on)
--
-- Rows recorded before triage existed are `skipped`: they were already
-- in front of the household, and must stay there.

ALTER TABLE voice_requests
    ADD COLUMN triage_status TEXT NOT NULL DEFAULT 'skipped'
        CHECK (triage_status IN ('unchecked', 'approved', 'rejected', 'held', 'skipped')),
    ADD COLUMN triage_reason TEXT
        CHECK (triage_reason IS NULL OR char_length(triage_reason) <= 500),
    ADD COLUMN triage_confidence REAL
        CHECK (triage_confidence IS NULL OR triage_confidence BETWEEN 0 AND 1);

-- New rows must say where they stand; only the backfill above uses a default.
ALTER TABLE voice_requests ALTER COLUMN triage_status DROP DEFAULT;

-- The queues filter on both columns together.
CREATE INDEX ix_voice_requests_status_triage ON voice_requests (status, triage_status);
