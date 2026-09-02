DROP VIEW plant_last_watered;

CREATE VIEW plant_last_watered AS
SELECT plant_id, occurred_at AS last_watered
FROM (
    SELECT
        id,
        plant_id,
        occurred_at,
        ROW_NUMBER() OVER (
            PARTITION BY plant_id
            ORDER BY COALESCE(julianday(occurred_at), -1.0) DESC, id DESC
        ) AS chronological_rank
    FROM care_events
    WHERE event_type = 'watered'
)
WHERE chronological_rank = 1;
