-- ============================================================
-- Migration: Push record enhancements
-- Add last_error TEXT column and mark exhausted retries as 'dead'
-- ============================================================

-- Step 1: Add last_error column for recording push failure reasons
ALTER TABLE push_records ADD COLUMN last_error TEXT;

-- Step 2: Mark dead records
-- Records that failed, have exhausted retries (next_retry_at IS NULL), and
-- have been retried at least once → final 'dead' state.
UPDATE push_records
SET status = 'dead'
WHERE status = 'failed'
  AND next_retry_at IS NULL
  AND retry_count > 0;
