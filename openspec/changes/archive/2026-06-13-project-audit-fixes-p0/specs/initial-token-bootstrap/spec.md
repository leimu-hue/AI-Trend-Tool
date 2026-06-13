## MODIFIED Requirements

### Requirement: Initial token on empty database

When the system starts and `api_tokens` table is empty, the system SHALL automatically create an initial admin token. This ensures the system is usable immediately after first deployment without manual database intervention.

#### Scenario: Table is empty, config has initial_token

- **WHEN** the system starts and `SELECT COUNT(*) FROM api_tokens` returns 0
- **AND** `config.toml` contains `[auth]` with `initial_token = "preconfigured-secret-token"`
- **THEN** the system SHALL insert `INSERT INTO api_tokens (name, token, token_hash) VALUES ('Initial Admin Token', '***REDACTED***', SHA256('preconfigured-secret-token'))`
- **THEN** `tracing::warn!` SHALL log the full plaintext token with prominent formatting

#### Scenario: Table is empty, config has no initial_token

- **WHEN** the system starts and `SELECT COUNT(*) FROM api_tokens` returns 0
- **AND** `config.toml` has no `initial_token` in `[auth]` (or it is empty string)
- **THEN** the system SHALL generate a 64-character random hex token
- **THEN** `tracing::warn!` SHALL log the generated token with prominent formatting
- **THEN** the system SHALL insert `INSERT INTO api_tokens (name, token, token_hash) VALUES ('Initial Admin Token', '***REDACTED***', SHA256('<generated>'))`

#### Scenario: Table already has tokens

- **WHEN** the system starts and `SELECT COUNT(*) FROM api_tokens` returns > 0
- **THEN** the system SHALL skip token creation entirely
- **THEN** the system SHALL log masked active token info: `Active token: abcd...wxyz` with token count
- **THEN** if token length < 8, SHALL display `Active token: ****`
