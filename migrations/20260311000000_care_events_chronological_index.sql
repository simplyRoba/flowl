CREATE INDEX idx_care_events_occurred_at_id
ON care_events (COALESCE(julianday(occurred_at), -1.0) DESC, id DESC);
