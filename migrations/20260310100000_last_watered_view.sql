CREATE VIEW plant_last_watered AS
SELECT plant_id, MAX(occurred_at) AS last_watered
FROM care_events
WHERE event_type = 'watered'
GROUP BY plant_id;
