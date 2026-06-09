-- Add token_hash column for SHA-256 token storage
-- Existing token column is retained for backward compatibility

ALTER TABLE api_tokens ADD COLUMN token_hash TEXT NOT NULL DEFAULT '';
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_tokens_hash ON api_tokens(token_hash);
