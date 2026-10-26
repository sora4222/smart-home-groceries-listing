-- Lets an accepted intake request be undone.
--
--   merged_quantity   how much the accept added to an item already on the
--                     list. NULL when the accept created the item instead.
--                     Undo takes this much back off, or deletes the item it
--                     created, and returns the request to Pending Requests.

ALTER TABLE voice_requests
    ADD COLUMN merged_quantity INTEGER CHECK (merged_quantity IS NULL OR merged_quantity >= 0);
