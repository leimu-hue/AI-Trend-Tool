-- ============================================================
-- Migration: Article status column
-- Add status TEXT column to articles table for state machine:
--   pending → processing → matched | skipped
-- ============================================================

-- Step 1: Add status column (SQLite ADD COLUMN always appends at end)
ALTER TABLE articles ADD COLUMN status TEXT NOT NULL DEFAULT 'pending';

-- Step 2: Migrate existing data
-- Articles that have been processed (processed_at IS NOT NULL) are marked as 'matched'
-- because early versions only set processed_at on successfully matched articles.
UPDATE articles SET status = 'matched' WHERE processed_at IS NOT NULL;

-- Step 3: Create index on status for efficient filtering of pending/processing/matched/skipped
CREATE INDEX IF NOT EXISTS idx_articles_status ON articles(status);

-- Step 4: Drop the old processed_at index (no longer the primary query filter)
DROP INDEX IF EXISTS idx_articles_processed;
