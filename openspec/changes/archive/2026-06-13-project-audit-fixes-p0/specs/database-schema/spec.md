## MODIFIED Requirements

### Requirement: api_tokens table

The system SHALL have an `api_tokens` table storing bearer tokens for API authentication.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `name` (TEXT NOT NULL), `token` (TEXT NOT NULL, stores placeholder `***REDACTED***`), `token_hash` (TEXT NOT NULL DEFAULT ''), `last_used_at` (DATETIME), `created_at` (DATETIME NOT NULL DEFAULT current time), `expires_at` (DATETIME), `revoked` (BOOLEAN NOT NULL DEFAULT 0).

**Indexes**: UNIQUE index on `token_hash`.

#### Scenario: Create a new API token

- **WHEN** a row is inserted into `api_tokens` with a name
- **THEN** the row SHALL be persisted with `token = '***REDACTED***'`, `token_hash = SHA256(plaintext)`
- **THEN** `created_at` SHALL be set to the current time
- **THEN** `revoked` SHALL default to `0` (false)
- **THEN** `id` SHALL be auto-assigned

#### Scenario: Duplicate token value

- **WHEN** a row is inserted with a `token_hash` value that already exists
- **THEN** the insert SHALL fail with a UNIQUE constraint violation

#### Scenario: New token gets hash stored

- **WHEN** a new API token is created
- **THEN** `token_hash` SHALL be set to `SHA256(token)`
- **THEN** `token` SHALL be set to `***REDACTED***`
- **THEN** the UNIQUE index on `token_hash` SHALL prevent duplicate hash values

#### Scenario: Existing tokens preserve backward compatibility

- **WHEN** the migration is applied to an existing database
- **THEN** existing rows SHALL have `token` updated to `'***REDACTED***'`
- **THEN** `token_hash` SHALL remain unchanged

### Requirement: keyword_mentions table

The system SHALL have a `keyword_mentions` table recording each keyword-article match event.

**Columns**: `id` (INTEGER PK AUTOINCREMENT), `keyword_id` (INTEGER NOT NULL FK→keywords ON DELETE CASCADE), `article_id` (INTEGER NOT NULL FK→articles ON DELETE CASCADE), `matched_at` (DATETIME NOT NULL DEFAULT current time).

**Indexes**: `idx_mentions_keyword` ON `keyword_id`, `idx_mentions_article` ON `article_id`, `idx_mentions_unique` UNIQUE ON `(keyword_id, article_id)`.

#### Scenario: Record a keyword match

- **WHEN** a keyword match is recorded with keyword_id and article_id
- **THEN** the row SHALL be persisted with `matched_at` set to current time
- **THEN** the indexes SHALL optimize lookups by keyword and by article

#### Scenario: Duplicate (keyword_id, article_id) ignored

- **WHEN** a keyword match is recorded for a (keyword_id, article_id) pair that already exists
- **THEN** `INSERT OR IGNORE` SHALL silently skip the duplicate

#### Scenario: Keyword or article deleted with cascade

- **WHEN** a keyword row is deleted
- **THEN** all mention rows with `keyword_id` referencing that keyword SHALL also be deleted
- **WHEN** an article row is deleted
- **THEN** all mention rows with `article_id` referencing that article SHALL also be deleted
