-- NOEMA schema v2. Studio BI: latency, project attribution, thinking mode.
ALTER TABLE usage_events ADD COLUMN latency_ms INTEGER;
ALTER TABLE usage_events ADD COLUMN project_id TEXT;
ALTER TABLE usage_events ADD COLUMN thinking TEXT;
