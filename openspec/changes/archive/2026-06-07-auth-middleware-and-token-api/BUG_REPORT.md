# Bug Report: axum 0.7.9 Path extractor + middleware routing failure

## Summary

In axum 0.7.9, any route handler that extracts `axum::extract::Path<T>` fails to match at runtime when the router has a middleware layer applied via `middleware::from_fn` or `middleware::from_fn_with_state`. The route compiles successfully but never matches — requests receive axum's default 404 (empty body, CORS headers from CorsLayer).

## Environment

- axum = 0.7.9
- No custom tower layers beyond `middleware::from_fn_with_state` and `tower_http::CorsLayer`
- Project: TrendAITool (Rust backend)

## Reproduction

### Minimal failing example

```rust
use axum::{extract::Path, middleware, Router, routing::get};

let app = Router::new()
    .route("/items/{id}", get(|Path(id): Path<i64>| async move { format!("id={}", id) }))
    .layer(middleware::from_fn(|req, next| async move { next.run(req).await }));
```

`GET /items/42` → 404 Not Found (route never matches)

### Working: same route WITHOUT middleware

```rust
let app = Router::new()
    .route("/items/{id}", get(|Path(id): Path<i64>| async move { format!("id={}", id) }));
```

`GET /items/42` → 200 "id=42"

### Working: WITH middleware but WITHOUT Path extractor

```rust
let app = Router::new()
    .route("/items/{id}", get(|| async { "ok" }))
    .layer(middleware::from_fn(|req, next| async move { next.run(req).await }));
```

`GET /items/42` → 200 "ok" (handler matches even though {id} param is unused)

### Working: no-param route WITH State extractor + middleware

```rust
let app = Router::new()
    .route("/items", get(|State(state): State<AppState>| async move { "ok" }))
    .layer(middleware::from_fn_with_state(state, auth_mw));
```

`GET /items` → 200

### Failing combinations

| Path param in route | Path extractor | State extractor | middleware | Works? |
|---------------------|----------------|-----------------|------------|--------|
| Yes | Yes | No | `from_fn` | **NO** |
| Yes | Yes | Yes | `from_fn_with_state` | **NO** |
| Yes | No | No | `from_fn` | Yes |
| Yes | No | Yes | `from_fn_with_state` | Yes |
| No | N/A | Yes | `from_fn_with_state` | Yes |
| Yes | Yes | No | None | Yes |

## Method Filter Specifics

All HTTP methods are affected equally when `Path` extractor is used:
- `get()`, `post()`, `delete()`, `put()`, `patch()` — all fail with Path+middleware
- `on(MethodFilter::DELETE, ...)` — same behavior as `delete()`

Additionally, `on(MethodFilter::DELETE, ...)` and `delete()` showed independent issues with path-param routes (even without Path extractor in some configurations), but this was not exhaustively characterized.

## Impact on This Project

The `revoke_token` handler extracts `Path<i64>` for the `{id}` path parameter. This prevents the `DELETE /api/v1/tokens/{id}` route from matching when auth middleware is applied.

**Workaround applied**: Changed revoke endpoint to `POST /api/v1/tokens/revoke` with token ID passed via JSON body (`{"id": <token_id>}`), avoiding `Path` extractor entirely. Auth middleware uses a closure capturing `SqlitePool` directly instead of `from_fn_with_state`.

## Possible Root Cause

The middleware layer likely modifies how axum's internal `Router` stores or resolves path-parameterized routes. When a handler declares `Path<T>` extraction, axum's routing layer may register the route differently, and the middleware layer wrapper corrupts or hides the registration for routes that require path parameter parsing.

## Recommended Fixes

1. **Upgrade axum to 0.8.x** — the likely simplest fix; 0.8 may have resolved this
2. **Avoid Path extractor with middleware** — parse path parameters manually from `request.uri().path()` or use query parameters / request body instead
3. **Use tower middleware trait implementation** instead of `from_fn` — might not trigger the same code path
