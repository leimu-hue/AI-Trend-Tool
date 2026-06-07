## ADDED Requirements

### Requirement: Module structure follows modern Rust convention

All model structs SHALL be defined in `src/models/<name>.rs` files, declared in `src/models.rs` using the 2018 edition module style. The `src/models/mod.rs` file SHALL NOT exist.

#### Scenario: Models module file structure

- **WHEN** all 8 model files are created
- **THEN** `src/models.rs` SHALL contain `pub mod token; pub mod source; pub mod article; pub mod keyword; pub mod hot_event; pub mod channel; pub mod push_record;`
- **THEN** each corresponding file SHALL exist at `src/models/<name>.rs`

### Requirement: ApiToken model

The system SHALL have an `ApiToken` struct in `src/models/token.rs` with fields matching the `api_tokens` table. The struct SHALL derive `Debug`, `sqlx::FromRow`, and `serde::Serialize`.

**Fields**: `id: i64`, `name: String`, `token: String`, `last_used_at: Option<NaiveDateTime>`, `created_at: NaiveDateTime`, `expires_at: Option<NaiveDateTime>`, `revoked: bool`.

#### Scenario: Query an API token by id

- **WHEN** `sqlx::query_as::<_, ApiToken>("SELECT * FROM api_tokens WHERE id = ?")` is executed
- **THEN** the result SHALL be `Result<ApiToken, sqlx::Error>` with all fields populated from the row
- **THEN** DATETIME columns SHALL deserialize into `chrono::NaiveDateTime`

#### Scenario: Serialize token for single-use response

- **WHEN** an `ApiToken` is serialized to JSON (e.g., at creation time)
- **THEN** the `token` field SHALL be included in the JSON output

### Requirement: ApiTokenInfo view model

The system SHALL have an `ApiTokenInfo` struct that omits the `token` field for use in list responses. The struct SHALL implement `From<ApiToken>` for conversion. The struct SHALL derive `Debug` and `serde::Serialize`.

#### Scenario: List tokens without exposing secrets

- **WHEN** multiple `ApiToken` rows are queried and converted to `ApiTokenInfo` via `.map(ApiTokenInfo::from)`
- **THEN** each `ApiTokenInfo` SHALL contain `id`, `name`, `last_used_at`, `created_at`, `expires_at`, `revoked`
- **THEN** the `token` field SHALL be absent from the JSON output

### Requirement: CreateTokenRequest

The system SHALL have a `CreateTokenRequest` struct deriving `serde::Deserialize` with fields: `name: String`, `expires_at: Option<NaiveDateTime>`.

#### Scenario: Deserialize token creation request

- **WHEN** a JSON body `{"name": "My Token", "expires_at": "2025-12-31T00:00:00"}` is deserialized
- **THEN** `name` SHALL be "My Token"
- **THEN** `expires_at` SHALL be `Some(NaiveDateTime)` with the parsed value

### Requirement: DataSource model

The system SHALL have a `DataSource` struct in `src/models/source.rs` with `sqlx::FromRow`, `serde::Serialize`, and `serde::Deserialize` derives. The `source_type` field SHALL use `#[serde(rename = "type")]` to serialize/deserialize as `"type"` in JSON while using the Rust field name `source_type`.

#### Scenario: Query all data sources

- **WHEN** `sqlx::query_as::<_, DataSource>("SELECT * FROM data_sources")` is executed
- **THEN** the result SHALL map all columns correctly including the `type` column to `source_type` field

#### Scenario: Serialize data source to JSON

- **WHEN** a `DataSource` is serialized to JSON
- **THEN** the `source_type` field SHALL appear as `"type"` in the JSON output

### Requirement: CreateSourceRequest and UpdateSourceRequest

The system SHALL have request DTOs for creating and updating data sources. `CreateSourceRequest` SHALL require `source_type`, `name`, `url` (all non-optional) and accept optional `interval_seconds` and `config`. `UpdateSourceRequest` SHALL have all optional fields (`name`, `url`, `enabled`, `interval_seconds`, `config`).

#### Scenario: Partial update of a data source

- **WHEN** a JSON body `{"enabled": false}` is deserialized into `UpdateSourceRequest`
- **THEN** `enabled` SHALL be `Some(false)`
- **THEN** all other fields SHALL be `None` (partial update)

### Requirement: Article model and ArticleQuery

The system SHALL have an `Article` struct matching the `articles` table with `sqlx::FromRow` and `serde::Serialize`. The system SHALL have an `ArticleQuery` struct for query parameters: `page: Option<u32>`, `per_page: Option<u32>`, `source_id: Option<i64>`, `processed: Option<bool>`. `ArticleQuery` SHALL derive `serde::Deserialize` only (not `FromRow`).

#### Scenario: Query articles with pagination and filter

- **WHEN** `ArticleQuery { page: Some(2), per_page: Some(20), source_id: Some(1), processed: Some(false) }` is constructed
- **THEN** the query parameters SHALL represent page 2, 20 per page, filtered by source_id=1 and unprocessed only

### Requirement: Keyword model and request DTOs

The system SHALL have a `Keyword` struct with `sqlx::FromRow` and `serde::Serialize`. The system SHALL have `CreateKeywordRequest` (required: `word`) and `UpdateKeywordRequest` (all optional: `word`, `case_sensitive`, `enabled`, `std_multiplier`, `min_hot_count`).

#### Scenario: Create keyword with custom sensitivity

- **WHEN** `CreateKeywordRequest { word: "AI", case_sensitive: Some(false), std_multiplier: Some(3.0), min_hot_count: Some(5) }` is constructed
- **THEN** `std_multiplier` SHALL be 3.0 and `min_hot_count` SHALL be 5

### Requirement: HotEvent model

The system SHALL have a `HotEvent` struct matching the `hot_events` table. The `hour_bucket` field SHALL be `String` (format YYYYMMDDHH). The struct SHALL derive `Debug`, `sqlx::FromRow`, `serde::Serialize`.

#### Scenario: Query hotspot events for a keyword

- **WHEN** hotspot events are queried by `keyword_id` ordered by `hour_bucket DESC`
- **THEN** each row SHALL deserialize into `HotEvent` with correct `mean_historical` and `stddev_historical` as `f64`

### Requirement: PushChannel model and request DTOs

The system SHALL have a `PushChannel` struct with `sqlx::FromRow` and `serde::Serialize`. The `config` field SHALL be `String` (JSON). The system SHALL have `CreateChannelRequest` (required: `name`, `config`) and `UpdateChannelRequest` (optional: `name`, `config`, `enabled`).

#### Scenario: Create a webhook channel

- **WHEN** `CreateChannelRequest { name: "DingTalk", channel_type: None, config: "{\"url\":\"https://...\"}" }` is deserialized
- **THEN** `channel_type` SHALL be `None` (server defaults to "webhook")

### Requirement: PushRecord model

The system SHALL have a `PushRecord` struct matching the `push_records` table with `sqlx::FromRow` and `serde::Serialize`. The `status` field SHALL be `String` (values: pending, success, failed).

#### Scenario: Query pending push records

- **WHEN** `sqlx::query_as::<_, PushRecord>("SELECT * FROM push_records WHERE status = 'pending'")` is executed
- **THEN** the result SHALL contain all records with `status = "pending"`

### Requirement: All models compile with cargo check

The project SHALL compile successfully with `cargo check` after all model files are created. There SHALL be no type mismatch between Rust struct fields and SQLite column types. The `sqlx::migrate!()` macro SHALL embed the migration file at compile time.

#### Scenario: cargo check passes

- **WHEN** `cargo check` is run after all models and migration files are created
- **THEN** the command SHALL exit with code 0
- **THEN** there SHALL be no compilation errors or warnings related to models or migrations
