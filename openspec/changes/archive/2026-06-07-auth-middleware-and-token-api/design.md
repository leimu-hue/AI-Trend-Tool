## Context

TrendAITool backend currently has no authentication. All endpoints under `/api/v1/*` are open. Step 01 (project scaffold) and Step 02 (database models) are complete — `AppError::Unauthorized` exists, `ApiToken` model exists, `AuthConfig` with optional `initial_token` exists. The `routes.rs` already has a skeleton with `api_routes()` ready for middleware attachment.

This design follows the plan document `docs/plans/03-auth-and-token-api.md`, adapted to the existing module organization convention (2018 edition, no `mod.rs`).

## Goals / Non-Goals

**Goals:**
- Protect all `/api/v1/*` endpoints with Bearer Token authentication
- Provide Token CRUD API for administrators (create, list, revoke)
- Auto-bootstrap initial token on first startup so the system is immediately usable
- Use existing infrastructure: `AppState`, `AppError::Unauthorized`, `ApiToken` model

**Non-Goals:**
- Role-based access control (RBAC) — all valid tokens have equal access
- Token refresh or rotation mechanism
- Rate limiting per token
- OAuth or third-party identity providers
- Password-based user accounts — this is a token-only auth system

## Decisions

### 1. Axum `from_fn_with_state` for middleware

**Decision**: Use `middleware::from_fn_with_state(state.clone(), auth_middleware)` applied as a layer on the nested API router.

**Alternatives considered**:
- `from_fn` (no state): Cannot access `SqlitePool`. Rejected.
- Tower middleware trait implementation: More boilerplate. `from_fn_with_state` is simpler for a single-function middleware. Rejected.
- Per-route `.layer()`: Repetitive. Nested router with one layer is cleaner. Chosen.

**Rationale**: The auth function needs `SqlitePool` to query tokens. `from_fn_with_state` passes the router's state into the middleware function. The nested router pattern (`/api/v1` as a sub-router with auth layer) ensures `/health` remains unprotected.

### 2. Nested router with auth layer

**Decision**: Create an inner `Router` for `/api/v1`, apply auth middleware to it, then nest it into the main router.

```
Router::new()
  .route("/health", ...)           // no auth
  .nest("/api/v1", api_router)     // auth applied to api_router
```

**Rationale**: This is the standard axum pattern for scoped middleware. The auth layer only fires for `/api/v1/*`, not `/health` or other top-level routes.

### 3. Soft delete for token revocation

**Decision**: `DELETE /api/v1/tokens/{id}` sets `revoked = 1` (soft delete) rather than deleting the row.

**Rationale**:
- Preserves audit trail of created/revoked tokens
- Foreign key references (if any in future) remain intact
- Consistent with the existing data model (`revoked` column already exists)

### 4. Token generation: 32 random bytes → 64 hex chars

**Decision**: Generate tokens with `rand::thread_rng().gen::<[u8; 32]>()` encoded via `hex::encode()`.

**Alternatives considered**:
- UUID v4: Shorter (36 chars) but less entropy (122 bits vs 256 bits). Acceptable but plan doc specifies hex. Follow plan.
- Base64: More compact but `+`, `/`, `=` are URL-unfriendly without encoding. Hex is pure alphanumeric.
- JWT: Adds complexity (key management, signing). Overkill for simple Bearer token auth. Rejected.

### 5. Background `tokio::spawn` for `last_used_at` update

**Decision**: Update `last_used_at` via `tokio::spawn` with a cloned pool, not `await` inline.

**Rationale**: The update is a side effect that should not add latency to the request. If the update fails, the request still succeeds — losing a `last_used_at` update is acceptable. The pool clone is cheap (SqlitePool wraps `Arc`).

**Risk**: In very high-throughput scenarios, many spawned tasks could accumulate. Mitigation: SQLite serializes writes anyway; the actual concurrency is bounded by SQLite's write lock.

### 6. 2018 edition module style

**Decision**: Use `src/middleware.rs` (entry) + `src/middleware/auth.rs` (impl) and `src/handlers.rs` (entry) + `src/handlers/token.rs` (impl). No `mod.rs` files.

**Rationale**: The `module-organization` spec mandates this. The entry files (`src/middleware.rs`, `src/handlers.rs`) already exist. The plan document's suggestion to create `mod.rs` files is overridden by project convention.

### 7. `ensure_initial_token` placement in `main.rs`

**Decision**: Call `ensure_initial_token(&pool, &config).await?` in `main.rs` immediately after `sqlx::migrate!().run(&pool).await?`, before building the router.

**Rationale**: Ensures the token exists before the server starts accepting requests. If the server starts without any token, the first admin would be locked out (cannot call `/api/v1/tokens` to create one because auth is required).

## Risks / Trade-offs

- **[Risk] Token in plaintext in database**: The `token` column stores plaintext, not a hash. A database breach exposes all tokens. → **Mitigation**: SQLite database file permissions. Future step could add hashing (bcrypt/sha256) with only the creation response showing plaintext.
- **[Risk] No token rotation**: Tokens are long-lived. A leaked token is valid until manually revoked. → **Mitigation**: Expiry support (`expires_at` column). Admins should create tokens with expiry for non-permanent use.
- **[Risk] All tokens have equal access**: No scope or permission differentiation. Any valid token can create/revoke other tokens. → **Mitigation**: Acceptable for current scope. RBAC can be added in future steps if needed.
- **[Trade-off] `last_used_at` is best-effort**: Fire-and-forget via `tokio::spawn`. Under extreme load, some updates may be missed. → Acceptable: `last_used_at` is informational only, not security-critical.

## Open Questions

None. All design decisions are resolved by the plan document plus existing project conventions.
