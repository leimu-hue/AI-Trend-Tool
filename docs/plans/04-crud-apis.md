# 步骤 04：CRUD API（数据源 + 关键词 + 推送渠道）

## 前置依赖

- 步骤 01-03 已完成（项目骨架、数据模型、认证中间件）

## 目标

完成后拥有 3 组 CRUD 共 13 个 API 端点：
- 数据源 CRUD + 手动抓取（5 个端点）
- 关键词 CRUD（4 个端点）
- 推送渠道 CRUD（4 个端点）

---

## 1. 数据源 API `src/handlers/source.rs`

### 1.1 GET /api/v1/sources — 列表

**响应体（200）：**
```json
{
  "data": [
    {
      "id": 1,
      "type": "rss",
      "name": "Hacker News",
      "url": "https://hnrss.org/frontpage",
      "config": "{}",
      "enabled": true,
      "interval_seconds": 300,
      "last_fetched_at": null,
      "created_at": "2025-01-01T00:00:00",
      "updated_at": "2025-01-01T00:00:00"
    }
  ]
}
```

**实现：**

```rust
use axum::{extract::State, Json};
use crate::error::AppError;
use crate::models::source::DataSource;
use crate::routes::AppState;

pub async fn list_sources(
    State(state): State<AppState>,
) -> Result<Json<Vec<DataSource>>, AppError> {
    let sources: Vec<DataSource> = sqlx::query_as(
        "SELECT * FROM data_sources ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(sources))
}
```

### 1.2 POST /api/v1/sources — 添加

**请求体：**
```json
{
  "type": "rss",
  "name": "Hacker News",
  "url": "https://hnrss.org/frontpage",
  "interval_seconds": 300,
  "config": "{}"
}
```

**响应体（201）：** 返回创建的 DataSource 对象

**实现：**

```rust
use axum::{extract::State, http::StatusCode, Json};
use crate::error::AppError;
use crate::models::source::{DataSource, CreateSourceRequest};
use crate::routes::AppState;

pub async fn create_source(
    State(state): State<AppState>,
    Json(req): Json<CreateSourceRequest>,
) -> Result<(StatusCode, Json<DataSource>), AppError> {
    let interval = req.interval_seconds.unwrap_or(300);
    let config = req.config.unwrap_or_else(|| "{}".to_string());

    let source: DataSource = sqlx::query_as(
        "INSERT INTO data_sources (type, name, url, config, interval_seconds)
         VALUES (?, ?, ?, ?, ?)
         RETURNING *"
    )
    .bind(&req.source_type)
    .bind(&req.name)
    .bind(&req.url)
    .bind(&config)
    .bind(interval)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(source)))
}
```

### 1.3 PUT /api/v1/sources/{id} — 更新

**请求体（所有字段可选）：**
```json
{
  "name": "Hacker News Updated",
  "url": "https://hnrss.org/newest",
  "enabled": false,
  "interval_seconds": 600
}
```

**响应体（200）：** 返回更新后的 DataSource

**实现：**

```rust
use axum::{extract::{Path, State}, Json};
use crate::error::AppError;
use crate::models::source::{DataSource, UpdateSourceRequest};
use crate::routes::AppState;

pub async fn update_source(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateSourceRequest>,
) -> Result<Json<DataSource>, AppError> {
    // 先检查是否存在
    let existing: DataSource = sqlx::query_as(
        "SELECT * FROM data_sources WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Source {} not found", id)))?;

    // 合并字段
    let name = req.name.unwrap_or(existing.name);
    let url = req.url.unwrap_or(existing.url);
    let enabled = req.enabled.unwrap_or(existing.enabled);
    let interval = req.interval_seconds.unwrap_or(existing.interval_seconds);
    let config = req.config.unwrap_or(existing.config);

    let updated: DataSource = sqlx::query_as(
        "UPDATE data_sources
         SET name = ?, url = ?, enabled = ?, interval_seconds = ?, config = ?,
             updated_at = datetime('now')
         WHERE id = ?
         RETURNING *"
    )
    .bind(&name)
    .bind(&url)
    .bind(enabled)
    .bind(interval)
    .bind(&config)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated))
}
```

### 1.4 DELETE /api/v1/sources/{id} — 删除

**响应：** 204 No Content

**实现：**

```rust
use axum::{extract::{Path, State}, http::StatusCode};
use crate::error::AppError;
use crate::routes::AppState;

pub async fn delete_source(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM data_sources WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Source {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
```

### 1.5 POST /api/v1/sources/{id}/fetch — 手动触发抓取

**响应体（200）：**
```json
{ "data": { "message": "Fetch triggered for source 1" } }
```

**实现要点：**
- 仅标记该源的 `last_fetched_at` 为 NULL（使其在下次 Parser 循环时立即被拉取）
- 或立即异步触发一次抓取（需要 Parser 模块支持，此处先标记为 TODO）

```rust
use axum::{extract::{Path, State}, Json};
use serde_json::{json, Value};
use crate::error::AppError;
use crate::routes::AppState;

pub async fn trigger_fetch(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // 检查源是否存在
    let exists: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM data_sources WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    if !exists {
        return Err(AppError::NotFound(format!("Source {} not found", id)));
    }

    // 重置 last_fetched_at，使 Parser 下次循环时立即拉取
    sqlx::query("UPDATE data_sources SET last_fetched_at = NULL WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": format!("Fetch triggered for source {}", id) })))
}
```

---

## 2. 关键词 API `src/handlers/keyword.rs`

### 2.1 GET /api/v1/keywords — 列表

```rust
pub async fn list_keywords(
    State(state): State<AppState>,
) -> Result<Json<Vec<Keyword>>, AppError> {
    let keywords: Vec<Keyword> = sqlx::query_as(
        "SELECT * FROM keywords ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(keywords))
}
```

### 2.2 POST /api/v1/keywords — 添加

**请求体：**
```json
{
  "word": "GPT-5",
  "case_sensitive": false,
  "std_multiplier": 2.0,
  "min_hot_count": 3
}
```

```rust
pub async fn create_keyword(
    State(state): State<AppState>,
    Json(req): Json<CreateKeywordRequest>,
) -> Result<(StatusCode, Json<Keyword>), AppError> {
    let case_sensitive = req.case_sensitive.unwrap_or(false);
    let std_multiplier = req.std_multiplier.unwrap_or(2.0);
    let min_hot_count = req.min_hot_count.unwrap_or(3);

    let keyword: Keyword = sqlx::query_as(
        "INSERT INTO keywords (word, case_sensitive, std_multiplier, min_hot_count)
         VALUES (?, ?, ?, ?)
         RETURNING *"
    )
    .bind(&req.word)
    .bind(case_sensitive)
    .bind(std_multiplier)
    .bind(min_hot_count)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err)
            if db_err.message().contains("UNIQUE") => {
            AppError::Conflict(format!("Keyword '{}' already exists", req.word))
        }
        _ => AppError::from(e),
    })?;

    Ok((StatusCode::CREATED, Json(keyword)))
}
```

### 2.3 PUT /api/v1/keywords/{id} — 更新

```rust
pub async fn update_keyword(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateKeywordRequest>,
) -> Result<Json<Keyword>, AppError> {
    let existing: Keyword = sqlx::query_as("SELECT * FROM keywords WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Keyword {} not found", id)))?;

    let word = req.word.unwrap_or(existing.word);
    let case_sensitive = req.case_sensitive.unwrap_or(existing.case_sensitive);
    let enabled = req.enabled.unwrap_or(existing.enabled);
    let std_multiplier = req.std_multiplier.unwrap_or(existing.std_multiplier);
    let min_hot_count = req.min_hot_count.unwrap_or(existing.min_hot_count);

    let updated: Keyword = sqlx::query_as(
        "UPDATE keywords
         SET word = ?, case_sensitive = ?, enabled = ?, std_multiplier = ?, min_hot_count = ?
         WHERE id = ?
         RETURNING *"
    )
    .bind(&word)
    .bind(case_sensitive)
    .bind(enabled)
    .bind(std_multiplier)
    .bind(min_hot_count)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated))
}
```

### 2.4 DELETE /api/v1/keywords/{id} — 删除

```rust
pub async fn delete_keyword(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM keywords WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Keyword {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
```

---

## 3. 推送渠道 API `src/handlers/channel.rs`

### 3.1 GET /api/v1/channels — 列表

```rust
pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<Json<Vec<PushChannel>>, AppError> {
    let channels: Vec<PushChannel> = sqlx::query_as(
        "SELECT * FROM push_channels ORDER BY id"
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(channels))
}
```

### 3.2 POST /api/v1/channels — 添加

**请求体：**
```json
{
  "name": "钉钉告警群",
  "channel_type": "webhook",
  "config": "{\"url\": \"https://oapi.dingtalk.com/robot/send?access_token=xxx\"}"
}
```

```rust
pub async fn create_channel(
    State(state): State<AppState>,
    Json(req): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<PushChannel>), AppError> {
    let channel_type = req.channel_type.unwrap_or_else(|| "webhook".to_string());

    let channel: PushChannel = sqlx::query_as(
        "INSERT INTO push_channels (name, channel_type, config)
         VALUES (?, ?, ?)
         RETURNING *"
    )
    .bind(&req.name)
    .bind(&channel_type)
    .bind(&req.config)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(channel)))
}
```

### 3.3 PUT /api/v1/channels/{id} — 更新

```rust
pub async fn update_channel(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateChannelRequest>,
) -> Result<Json<PushChannel>, AppError> {
    let existing: PushChannel = sqlx::query_as(
        "SELECT * FROM push_channels WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Channel {} not found", id)))?;

    let name = req.name.unwrap_or(existing.name);
    let config = req.config.unwrap_or(existing.config);
    let enabled = req.enabled.unwrap_or(existing.enabled);

    let updated: PushChannel = sqlx::query_as(
        "UPDATE push_channels SET name = ?, config = ?, enabled = ? WHERE id = ? RETURNING *"
    )
    .bind(&name)
    .bind(&config)
    .bind(enabled)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated))
}
```

### 3.4 DELETE /api/v1/channels/{id} — 删除

```rust
pub async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM push_channels WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Channel {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
```

---

## 4. 路由注册

更新 `src/routes.rs` 中的 `api_routes()` 函数：

```rust
fn api_routes() -> Router<AppState> {
    Router::new()
        // Token API（步骤03已添加）
        .route("/tokens", post(handlers::token::create_token))
        .route("/tokens", get(handlers::token::list_tokens))
        .route("/tokens/{id}", delete(handlers::token::revoke_token))
        // 数据源 API
        .route("/sources", get(handlers::source::list_sources))
        .route("/sources", post(handlers::source::create_source))
        .route("/sources/{id}", put(handlers::source::update_source))
        .route("/sources/{id}", delete(handlers::source::delete_source))
        .route("/sources/{id}/fetch", post(handlers::source::trigger_fetch))
        // 关键词 API
        .route("/keywords", get(handlers::keyword::list_keywords))
        .route("/keywords", post(handlers::keyword::create_keyword))
        .route("/keywords/{id}", put(handlers::keyword::update_keyword))
        .route("/keywords/{id}", delete(handlers::keyword::delete_keyword))
        // 推送渠道 API
        .route("/channels", get(handlers::channel::list_channels))
        .route("/channels", post(handlers::channel::create_channel))
        .route("/channels/{id}", put(handlers::channel::update_channel))
        .route("/channels/{id}", delete(handlers::channel::delete_channel))
        // 认证中间件
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
```

同时更新 `src/handlers/mod.rs`：

```rust
pub mod token;
pub mod source;
pub mod keyword;
pub mod channel;
```

---

## 预期文件清单

| 文件路径 | 说明 |
|---------|------|
| `src/handlers/source.rs` | 数据源 CRUD handler（5个端点） |
| `src/handlers/keyword.rs` | 关键词 CRUD handler（4个端点） |
| `src/handlers/channel.rs` | 推送渠道 CRUD handler（4个端点） |
| `src/handlers/mod.rs` | 更新：添加新模块声明 |
| `src/routes.rs` | 更新：注册所有 CRUD 路由 |

---

## 验证节点

```bash
# 编译检查
cargo check

# 启动服务器
cargo run -- --config config.toml all

TOKEN="<your-token>"

# --- 数据源 ---
# 添加
curl -X POST http://localhost:8080/api/v1/sources \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"type":"rss","name":"HN","url":"https://hnrss.org/frontpage"}'

# 列表
curl http://localhost:8080/api/v1/sources -H "Authorization: Bearer $TOKEN"

# 更新
curl -X PUT http://localhost:8080/api/v1/sources/1 \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"interval_seconds":600}'

# --- 关键词 ---
curl -X POST http://localhost:8080/api/v1/keywords \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"word":"GPT-5","std_multiplier":2.0,"min_hot_count":3}'

# --- 推送渠道 ---
curl -X POST http://localhost:8080/api/v1/channels \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"name":"钉钉群","config":"{\"url\":\"https://oapi.dingtalk.com/...\"}"}'
```
