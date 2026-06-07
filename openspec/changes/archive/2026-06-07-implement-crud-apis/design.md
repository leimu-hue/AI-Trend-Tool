## Context

Step 04 of the project plan: implement CRUD API handlers for data sources, keywords, and push channels. All three domain models (`DataSource`, `Keyword`, `PushChannel`) exist in `src/models/`. All db query functions exist in `src/db/{source,keyword,channel}.rs`. Token handlers (`src/handlers/token.rs`) establish the pattern: handlers call db functions, use `ApiResponse` for wrapping, and route via `src/routes.rs` behind auth middleware.

The existing `src/handlers.rs` uses 2018 edition module style (`pub mod token;` in `handlers.rs`, NOT `handlers/mod.rs`). The plan document at `docs/plans/04-crud-apis.md` provides reference implementations but uses `PUT`/`DELETE` methods and a `handlers/mod.rs` file — both conflict with established project conventions.

## Goals / Non-Goals

**Goals:**
- Add 13 handlers across 3 new files (`src/handlers/source.rs`, `keyword.rs`, `channel.rs`)
- Wire routes in `src/routes.rs` under `/api/v1/*` with auth middleware
- Use existing db functions — handlers are thin wrappers
- Follow existing patterns: `ApiResponse` for JSON wrapping, POST-only for mutations, `src/handlers.rs` module style

**Non-Goals:**
- No new db functions (existing ones are sufficient)
- No new models (request/response types already exist)
- No changes to parser/filter/pusher modules
- No changes to auth middleware or error types

## Decisions

### D1: Use POST for update and delete (not PUT/DELETE)

**Decision:** Update → `POST /resource/{id}/update`, Delete → `POST /resource/{id}/delete`

**Rationale:** CLAUDE.md mandates only GET and POST. The existing token API already follows this pattern (`POST /tokens/revoke/{id}` for revocation). The plan document's use of PUT/DELETE predates this project rule.

**Alternatives considered:** Using PUT/DELETE as in the plan document — rejected because it violates CLAUDE.md.

### D2: Use `src/handlers.rs` (not `src/handlers/mod.rs`)

**Decision:** Add `pub mod source; pub mod keyword; pub mod channel;` to the existing `src/handlers.rs`.

**Rationale:** The project uses 2018 edition module style. The `token-api` spec explicitly states "No `src/handlers/mod.rs` file SHALL exist." The plan document's reference to `src/handlers/mod.rs` is incorrect for this codebase.

### D3: Handlers delegate to db module functions, no inline SQL

**Decision:** All handlers call functions in `src/db/{source,keyword,channel}.rs`. Zero inline `sqlx::query*` in handlers.

**Rationale:** CLAUDE.md rule: "All SQL queries MUST live in `src/db/<module>.rs`." Existing db functions already cover all needed operations — handlers are pure delegation.

### D4: Response format follows `ApiResponse` pattern

**Decision:** Use `ApiResponse::ok()`, `ApiResponse::created()`, and `StatusCode::NO_CONTENT` for consistent `{"data": ...}` wrapping.

**Rationale:** The existing token handlers use `ApiResponse`. The plan document sometimes returns raw `Json<Vec<T>>` without wrapping — following the established pattern is more consistent.

### D5: Manual fetch trigger resets `last_fetched_at` to NULL

**Decision:** `POST /sources/{id}/fetch` sets `last_fetched_at = NULL` so the Parser module picks it up on its next cycle.

**Rationale:** The plan document offers two options. Setting to NULL leverages the existing Parser scheduling logic without requiring inter-module communication.

## Risks / Trade-offs

- **Unique constraint on keyword.word not handled in db layer** → The db `create_keyword` function returns a raw `sqlx::Error` on duplicate. The handler must pattern-match on the error message to return 409 Conflict. This is fragile across sqlx versions. Mitigation: add a `db::keyword::exists_by_word()` check before insert, or handle in the handler's error mapping.
- **Dynamic SQL in db update functions uses `format!()`** → SQL injection risk if field names were user-controlled, but they're hardcoded enum values — safe in practice.
- **No request validation beyond type parsing** → `serde` + `Deserialize` handles missing fields. Invalid values (e.g., negative `interval_seconds`) pass through to the database. Lightweight — acceptable for current stage.

## Open Questions

1. **HTTP method convention**: The plan document uses PUT/DELETE. This design follows CLAUDE.md (POST-only). Confirm: use `POST /sources/{id}/update` and `POST /sources/{id}/delete` pattern?
2. **List response wrapping**: Plan shows `Json<Vec<T>>` (no wrapper). This design uses `{"data": [...]}` per existing `ApiResponse` pattern. Confirm?
