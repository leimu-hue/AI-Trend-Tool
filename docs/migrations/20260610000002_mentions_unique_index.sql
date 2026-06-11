-- Remove duplicate keyword_mentions rows before creating the unique index.
DELETE FROM keyword_mentions
WHERE rowid NOT IN (
    SELECT MIN(rowid)
    FROM keyword_mentions
    GROUP BY keyword_id, article_id
);

-- Prevent duplicate (keyword_id, article_id) records from INSERT OR IGNORE expansion.
CREATE UNIQUE INDEX IF NOT EXISTS idx_mentions_unique
    ON keyword_mentions(keyword_id, article_id);
