# 步骤 03：认证中间件 + Token API

## 前置依赖

- 步骤 01 已完成（项目骨架、统一错误处理）
- 步骤 02 已完成（api_tokens 表、ApiToken 模型）

## 目标

完成后拥有：
- Bearer Token 认证中间件（保护所有 /api/v1/* 端点）
- 初始 Token 自动引导机制
- Token CRUD API（3 个端点）

---

## 1. 认证中间件 `src/middleware/auth.rs`

### 1.1 中间件实现

```rust
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::SqlitePool;
use chrono::Utc;

use crate::error::AppError;
use crate::models::token::ApiToken;

/// 从 Authorization header 提取 Bearer token 并验证
pub async fn auth_middleware(
    State(pool): State<SqlitePool>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. 提取 Bearer token
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let token_str = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization format, expected Bearer".to_string()))?;

    // 2. 查询数据库验证 token
    let token: ApiToken = sqlx::query_as(
        "SELECT * FROM api_tokens WHERE token = ? AND revoked = 0"
    )
    .bind(token_str)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid or revoked token".to_string()))?;

    // 3. 检查过期时间
    if let Some(expires_at) = token.expires_at {
        if expires_at < Utc::now().naive_utc() {
            return Err(AppError::Unauthorized("Token has expired".to_string()));
        }
    }

    // 4. 更新 last_used_at（异步，不阻塞响应）
    let pool_clone = pool.clone();
    let token_id = token.id;
    tokio::spawn(async move {
        let _ = sqlx::query("UPDATE api_tokens SET last_used_at = datetime('now') WHERE id = ?")
            .bind(token_id)
            .execute(&pool_clone)
            .await;
    });

    // 5. 将 token 信息存入请求扩展，供 handler 使用
    request.extensions_mut().insert(token);

    Ok(next.run(request).await)
}
```

### 1.2 在路由中应用中间件

更新 `src/routes.rs`，对 `/api/v1` 路由组应用认证中间件：

```rust
use axum::middleware;
// ...

fn api_routes() -> Router<AppState> {
    Router::new()
        // Token API
        .route("/tokens", post(handlers::token::create_token))
        .route("/tokens", get(handlers::token::list_tokens))
        .route("/tokens/{id}", delete(handlers::token::revoke_token))
        // ... 后续步骤添加更多路由
        .layer(middleware::from_fn_with_state(
            // 需要从 AppState 提取 pool
            // 使用 layer 时需要传递 state
        ))
}
```

> **注意**：axum 中间件需要访问 state 时，使用 `from_fn_with_state`。具体实现方式见下方完整路由整合。

### 1.3 完整路由整合（替换 routes.rs）

```rust
use axum::{
    middleware,
    routing::{get, post, delete},
    Router,
};
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;

use crate::config::AppConfig;
use crate::middleware::auth::auth_middleware;

pub fn create_router(pool: SqlitePool, config: AppConfig) -> Router {
    let state = AppState { pool: pool.clone(), config };

    // 需认证的 API 路由
    let api = Router::new()
        // Token API
        .route("/tokens", post(handlers::token::create_token))
        .route("/tokens", get(handlers::token::list_tokens))
        .route("/tokens/{id}", delete(handlers::token::revoke_token))
        // 步骤04、05 在此继续添加路由
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api)
        .with_state(state)
        .layer(CorsLayer::permissive())
}

async fn health_check() -> &'static str {
    "ok"
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: AppConfig,
}
```

---

## 2. 初始 Token 引导

### 2.1 逻辑说明

系统首次启动时，若 `api_tokens` 表为空：
- 若 `config.toml` 中配置了 `auth.initial_token`，则自动插入该 token
- 若未配置，则自动生成一个随机 token 并在日志中打印（一次性引导）

### 2.2 实现代码（在 main.rs 中调用）

```rust
/// 在 main.rs 的 sqlx::migrate! 之后调用
pub async fn ensure_initial_token(pool: &SqlitePool, config: &AppConfig) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api_tokens")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        return Ok(());
    }

    let token_str = match &config.auth.initial_token {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            // 生成 64 字符随机 hex token
            use rand::Rng;
            let bytes: [u8; 32] = rand::thread_rng().gen();
            let generated = hex::encode(bytes);
            tracing::warn!("============================================");
            tracing::warn!("  INITIAL TOKEN (save this!): {}", generated);
            tracing::warn!("============================================");
            generated
        }
    };

    sqlx::query(
        "INSERT INTO api_tokens (name, token) VALUES (?, ?)"
    )
    .bind("Initial Admin Token")
    .bind(&token_str)
    .execute(pool)
    .await?;

    tracing::info!("Initial token created successfully");
    Ok(())
}
```

在 `main.rs` 中添加调用：

```rust
// 在 sqlx::migrate! 之后
ensure_initial_token(&pool, &config).await?;
```

---

## 3. Token API Handler `src/handlers/token.rs`

### 3.1 POST /api/v1/tokens — 创建新 Token

**请求体：**
```json
{
  "name": "前端UI",
  "expires_at": "2025-12-31T23:59:59"   // 可选，null 表示永不过期
}
```

**响应体（201）：**
```json
{
  "data": {
    "id": 2,
    "name": "前端UI",
    "token": "a1b2c3d4...64字符hex...",
    "created_at": "2025-01-01T00:00:00",
    "expires_at": "2025-12-31T23:59:59",
    "revoked": false
  }
}
```

**实现：**

```rust
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use sqlx::SqlitePool;
use rand::Rng;

use crate::error::AppError;
use crate::models::token::{ApiToken, CreateTokenRequest};
use crate::routes::AppState;

pub async fn create_token(
    State(state): State<AppState>,
    Json(req): Json<CreateTokenRequest>,
) -> Result<(StatusCode, Json<ApiToken>), AppError> {
    // 生成 64 字符随机 hex token
    let bytes: [u8; 32] = rand::thread_rng().gen();
    let token_str = hex::encode(bytes);

    let token: ApiToken = sqlx::query_as(
        "INSERT INTO api_tokens (name, token, expires_at)
         VALUES (?, ?, ?)
         RETURNING *"
    )
    .bind(&req.name)
    .bind(&token_str)
    .bind(req.expires_at)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(token)))
}
```

### 3.2 GET /api/v1/tokens — 列出所有 Token（隐藏明文）

**响应体（200）：**
```json
{
  "data": [
    {
      "id": 1,
      "name": "Initial Admin Token",
      "last_used_at": "2025-01-01T12:00:00",
      "created_at": "2025-01-01T00:00:00",
      "expires_at": null,
      "revoked": false
    }
  ]
}
```

**实现：**

```rust
use axum::{extract::State, Json};
use crate::models::token::{ApiToken, ApiTokenInfo};
use crate::routes::AppState;
use crate::error::AppError;

pub async fn list_tokens(
    State(state): State<AppState>,
) -> Result<Json<Vec<ApiTokenInfo>>, AppError> {
    let tokens: Vec<ApiToken> = sqlx::query_as(
        "SELECT * FROM api_tokens ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await?;

    let infos: Vec<ApiTokenInfo> = tokens.into_iter().map(|t| t.into()).collect();
    Ok(Json(infos))
}
```

### 3.3 DELETE /api/v1/tokens/{id} — 吊销指定 Token

**响应：** 204 No Content

**实现：**

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use crate::error::AppError;
use crate::routes::AppState;

pub async fn revoke_token(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query(
        "UPDATE api_tokens SET revoked = 1 WHERE id = ?"
    )
    .bind(id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Token with id {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
```

---

## 4. 注册模块

在 `src/main.rs` 顶部添加模块声明：

```rust
mod middleware;
mod handlers;
```

创建文件：
- `src/middleware/mod.rs` → `pub mod auth;`
- `src/handlers/mod.rs` → `pub mod token;`

---

## 预期文件清单

| 文件路径 | 说明 |
|---------|------|
| `src/middleware/mod.rs` | middleware 模块声明 |
| `src/middleware/auth.rs` | Bearer Token 认证中间件 |
| `src/handlers/mod.rs` | handlers 模块声明 |
| `src/handlers/token.rs` | Token CRUD handler（3个端点） |
| `src/routes.rs` | 更新：应用认证中间件 + 注册 Token 路由 |
| `src/main.rs` | 更新：添加 initial token 引导逻辑 |

---

## 验证节点

```bash
# 编译检查
cargo check

# 启动服务器（观察初始 token 日志输出）
cargo run -- --config config.toml all

# 使用初始 token 创建新 token
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <initial-token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "测试Token"}'

# 列出所有 token
curl http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <initial-token>"

# 吊销 token
curl -X DELETE http://localhost:8080/api/v1/tokens/2 \
  -H "Authorization: Bearer <initial-token>"

# 无 token 访问应返回 401
curl http://localhost:8080/api/v1/tokens
# 预期: {"error":{"code":"UNAUTHORIZED","message":"Missing Authorization header"}}
```
