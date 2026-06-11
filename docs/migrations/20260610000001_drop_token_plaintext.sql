-- Replace persisted token plaintext with redacted placeholder.
-- After this migration, all existing API tokens must be regenerated.
UPDATE api_tokens SET token = '***REDACTED***';
