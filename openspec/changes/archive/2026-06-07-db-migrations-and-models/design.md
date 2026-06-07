## Context

Step 01 (backend-project-setup, archived) created the compilable project skeleton: axum server, SQLite connection pool, config parsing, error handling. The `src/models/` directory exists but only contains `.gitkeep`. No database tables, no migration system, no Rust structs for querying data.

This design specifies the database schema (8 tables) and their corresponding Rust data models with `sqlx::FromRow` support. All subsequent steps — auth tokens (03), CRUD APIs (04), background modules (05) — depend on this data layer.

## Goals / Non-Goals

**Goals:**
- Define complete SQLite DDL for 8 tables with indexes, constraints, foreign keys
- Auto-run migrations on server startup via `sqlx migrate`
- Create typed Rust structs for all tables (`sqlx::FromRow` + `serde::Serialize`)
- Create request/response DTOs where needed (create, update, query params)
- Match the file structure and naming conventions from the plan (`docs/plans/02-database-migrations.md`)
- Compile clean with `cargo check` after creation

**Non-Goals:**
- Business logic in models (services layer handles that in step 04)
- API endpoints (step 04)
- Auth middleware integration (step 03)
- Background module logic (step 05)
- Tests (added incrementally)
- Database seeding (config-driven initial token belongs to step 03)

## Decisions

### 1. Single migration file for all tables

**Choice**: One `docs/migrations/<timestamp>_init.sql` containing all 8 `CREATE TABLE IF NOT EXISTS` statements.

**Rationale**: All tables form a coherent initial schema — splitting into per-table migrations adds complexity without benefit. `sqlx migrate` tracks a single `_migrations` table; subsequent changes add new migration files. The `IF NOT EXISTS` guard makes the migration idempotent — safe to run repeatedly.

**Alternatives considered**:
- Per-table migration files: More granular but overkill for seed schema with no existing data. Creates unnecessary migration history.
- `sqlx migrate add` for each table: 8 separate migration files for a first deploy is noise.

### 2. sqlx migrate for migration management

**Choice**: Use `sqlx::migrate!()` compile-time macro that embeds migration SQL into the binary and auto-runs on startup.

**Rationale**: No runtime dependency on filesystem for migrations. Single binary contains everything needed to bootstrap a fresh database. Migration state tracked in `_sqlx_migrations` table — sqlx handles the "already applied" check internally.

**Alternatives considered**:
- Manual `CREATE TABLE IF NOT EXISTS` without sqlx migrate: Simpler but loses version tracking. When schema changes in later steps, no migration history to diff against.
- External migration tool: Adds tooling dependency. `sqlx migrate` is already in the dependency tree.

### 3. JSON config fields stored as `TEXT`

**Choice**: `data_sources.config` and `push_channels.config` are `TEXT NOT NULL DEFAULT '{}'` columns. Rust models represent them as `String`, not structured types.

**Rationale**: Different source types (RSS, Atom, JSON Feed) and channel types (DingTalk, Feishu) have different config shapes. Storing as JSON string avoids schema changes when adding new types. Application layer parses JSON when needed.

**Alternatives considered**:
- Separate tables per type: Normalized but creates many sparse tables. New types require migrations.
- `serde_json::Value` in Rust model: Tight coupling to serde_json. String gives callers choice of parser.

### 4. Separate request structs from entity structs

**Choice**: `Article` is the database entity (`FromRow`). `ArticleQuery` is a separate struct for query parameters (`Deserialize` only, no `FromRow`). Same pattern for `Create*Request` / `Update*Request`.

**Rationale**: Database entities may have fields that shouldn't come from user input (auto-generated IDs, timestamps, computed columns). Separate DTOs prevent mass-assignment bugs and make the API contract explicit. `FromRow` + `Serialize` for reads, `Deserialize` for writes — clean separation.

**Alternatives considered**:
- Single struct with `#[serde(skip)]` annotations: Works but mixes concerns. Harder to know which fields a request modifies vs supplements.
- JSON Patch style: Overly complex for CRUD operations.

### 5. View model for ApiToken (hide token secret)

**Choice**: `ApiToken` has all fields including `token`. `ApiTokenInfo` omits the `token` field and is used for list responses. `From<ApiToken> for ApiTokenInfo` provides implicit conversion.

**Rationale**: Never expose bearer tokens in API responses after creation. The token is shown once at creation time, then only the hash/prefix is visible.

### 6. Modern module style for models

**Choice**: `src/models.rs` declares submodules; each model gets `src/models/<name>.rs`. No `mod.rs`.

**Rationale**: Follows the module-organization spec established in step 01. Matches how `handlers`, `middleware`, and `services` modules are structured.

## Risks / Trade-offs

- **SQLite foreign key defaults**: SQLite does not enforce foreign keys by default. `PRAGMA foreign_keys = ON` must be set per connection. Already handled in step 01's `db.rs` connection pool initialization. Mitigation: verify the PRAGMA is set in the pool factory.
- **`ON DELETE CASCADE` with SQLite**: CASCADE works when foreign keys are enabled, but if the PRAGMA is off, orphaned rows accumulate. Mitigation: same as above — the pool setup in step 01 handles this.
- **`datetime('now')` defaults**: SQLite uses UTC. Chrono `NaiveDateTime` has no timezone awareness. For a single-machine monitoring tool, this is acceptable. If multi-timezone becomes needed, switch to storing epoch seconds or `DATETIME` with explicit UTC offset.
- **Large migration file**: Single file is ~80 lines of DDL — manageable now. If schema grows significantly, future changes add new migration files rather than editing this one.
- **Compile-time migration embedding**: `sqlx::migrate!()` embeds SQL at compile time. If migration files are malformed, compilation fails. This is a feature (catch errors early), but requires migration files to exist when compiling the binary.

## Open Questions

<!-- All design decisions resolved by the plan. No outstanding questions. -->
