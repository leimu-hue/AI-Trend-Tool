-- Rebuild hot_events with UNIQUE(keyword_id, hour_bucket) to support ON CONFLICT UPSERT
-- Deduplicate: keep only the row with the highest id for each (keyword_id, hour_bucket) pair

CREATE TABLE hot_events_new (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword_id        INTEGER NOT NULL REFERENCES keywords(id) ON DELETE CASCADE,
    hour_bucket       TEXT    NOT NULL,
    count             INTEGER NOT NULL DEFAULT 0,
    mean_historical   REAL    NOT NULL DEFAULT 0.0,
    stddev_historical REAL    NOT NULL DEFAULT 0.0,
    created_at        DATETIME NOT NULL DEFAULT (datetime('now')),
    UNIQUE(keyword_id, hour_bucket)
);

INSERT OR IGNORE INTO hot_events_new (id, keyword_id, hour_bucket, count, mean_historical, stddev_historical, created_at)
SELECT id, keyword_id, hour_bucket, count, mean_historical, stddev_historical, created_at
FROM hot_events
WHERE id IN (
    SELECT MAX(id) FROM hot_events GROUP BY keyword_id, hour_bucket
);

DROP TABLE hot_events;
ALTER TABLE hot_events_new RENAME TO hot_events;

CREATE INDEX IF NOT EXISTS idx_hot_events_keyword ON hot_events(keyword_id);
CREATE INDEX IF NOT EXISTS idx_hot_events_bucket  ON hot_events(hour_bucket);
