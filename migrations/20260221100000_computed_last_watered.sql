-- Backfill a watered care event for plants that have last_watered set
-- but no corresponding watered care event in care_events.
INSERT INTO care_events (plant_id, event_type, occurred_at)
SELECT id, 'watered', last_watered
FROM plants
WHERE last_watered IS NOT NULL
  AND id NOT IN (
    SELECT DISTINCT plant_id FROM care_events WHERE event_type = 'watered'
  );

-- Drop the last_watered column — now computed from care_events.
ALTER TABLE plants DROP COLUMN last_watered;
