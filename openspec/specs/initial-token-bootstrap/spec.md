# initial-token-bootstrap

## Purpose

On first startup with an empty `api_tokens` table, the system automatically creates an initial admin token so the administrator can immediately use the API without manual database intervention.

## Requirements

### Requirement: Initial token on empty database

When the system starts and `api_tokens` table is empty, the system SHALL automatically create an initial admin token. This ensures the system is usable immediately after first deployment without manual database intervention.

#### Scenario: Table is empty, config has initial_token

- **WHEN** the system starts and `SELECT COUNT(*) FROM api_tokens` returns 0
- **AND** `config.toml` contains `[auth]` with `initial_token = "preconfigured-secret-token"`
- **THEN** the system SHALL insert `INSERT INTO api_tokens (name, token) VALUES ('Initial Admin Token', 'preconfigured-secret-token')`
- **THEN** `tracing::info!` SHALL log "Initial token created successfully"

#### Scenario: Table is empty, config has no initial_token

- **WHEN** the system starts and `SELECT COUNT(*) FROM api_tokens` returns 0
- **AND** `config.toml` has no `initial_token` in `[auth]` (or it is empty string)
- **THEN** the system SHALL generate a 64-character random hex token
- **THEN** `tracing::warn!` SHALL log the generated token with prominent formatting
- **THEN** the system SHALL insert `INSERT INTO api_tokens (name, token) VALUES ('Initial Admin Token', '<generated>')`

#### Scenario: Table already has tokens

- **WHEN** the system starts and `SELECT COUNT(*) FROM api_tokens` returns > 0
- **THEN** the system SHALL skip token creation entirely
- **THEN** no log message about initial token SHALL be emitted

### Requirement: Bootstrap runs after migrations

The initial token bootstrap SHALL execute after `sqlx::migrate!()` has run, ensuring the `api_tokens` table exists before insertion.

#### Scenario: Startup sequence

- **WHEN** `main.rs` executes the startup sequence
- **THEN** `ensure_initial_token(pool, config)` SHALL be called after `sqlx::migrate!().run(&pool).await`

### Requirement: Bootstrap dependency on rand and hex crates

The random token generation SHALL use `rand::thread_rng().gen::<[u8; 32]>()` for 32 random bytes and `hex::encode()` to produce a 64-character hex string.

#### Scenario: Generated token format

- **WHEN** a random token is auto-generated
- **THEN** the token SHALL be exactly 64 characters long
- **THEN** the token SHALL contain only hexadecimal characters (0-9, a-f)
