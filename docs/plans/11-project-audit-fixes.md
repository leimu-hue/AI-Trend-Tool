# 项目审查修复计划

> 本文档为 Agent 执行用修复指令，基于全面项目审查结果，按优先级从高到低排列。每个 Task 包含精确的文件路径、问题描述和修复方案。
> **排除项**：非 RESTful 路由（POST update/delete）不做更改。

---

## Task 1: [P0 安全] 启动日志泄露 Token 明文

### 问题

`src/main.rs` 第 30-33 行，`ensure_initial_token` 函数在**每次启动**时都以 `info` 级别打印活跃 token 明文到日志。
生产环境中日志通常会被收集到 ELK/Loki 等平台，明文 token 泄露风险极高。

```rust
// 当前代码
tracing::info!("  Active token: {}", token.token);
```

### 修复方案

**文件: `src/main.rs`**

仅在首次创建 token 时使用 `warn` 级别打印，已有 token 时只打印数量和掩码信息：

```rust
if count > 0 {
    if let Some(token) = db::token::get_first_active_token(pool).await? {
        let masked = if token.token.len() > 8 {
            format!("{}...{}", &token.token[..4], &token.token[token.token.len()-4..])
        } else {
            "****".to_string()
        };
        tracing::info!("============================================");
        tracing::info!("  Active token: {}", masked);
        tracing::info!("  ({} token(s) total in database)", count);
        tracing::info!("============================================");
    }
    return Ok(());
}
```

首次创建时的 `warn` 级别日志保持不变（仅首次启动触发，合理）。

---

## Task 2: [P0 安全] CORS 配置过于宽松

### 问题

`src/routes.rs` 第 63 行使用 `CorsLayer::permissive()`，允许任意来源、任意方法、任意头部访问 API。
生产环境中应限制为 Electron 的本地请求来源。

### 修复方案

**文件: `src/routes.rs`**

替换 `CorsLayer::permissive()` 为受限配置：

```rust
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

// 在 create_router 函数内
let cors = CorsLayer::new()
    .allow_origin(Any)          // Electron file:// 协议无 origin，开发阶段保持 Any
    .allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ])
    .allow_headers(Any)
    .allow_credentials(false);

Router::new()
    .route("/health", get(health_check))
    .nest("/api/v1", api)
    .with_state(state)
    .layer(cors)
```

> **备注**：由于 Electron `file://` 协议不发送 Origin 头，`allow_origin` 暂时保留 `Any`。
> 如果后续部署为 Web 应用（HTTP 访问），应改为显式白名单。

---

## Task 3: [P0 安全] Token 明文列应逐步废弃

### 问题

`api_tokens` 表同时存储 `token`（明文）和 `token_hash`（SHA-256）。
认证已通过 `token_hash` 查询，但 `token` 列仍保留明文，且 `SELECT *` 会将其加载到内存。
`create_token` 在返回 `ApiToken` 时也会将明文序列化到响应体中（仅创建时一次，合理）。

### 修复方案

#### 3a. 新建迁移文件

**新建文件: `docs/migrations/20260610000001_drop_token_plaintext.sql`**

```sql
-- 将已有 token 明文清空，仅保留 hash 用于认证
-- 注意：此操作不可逆，创建时返回的明文是一次性的
UPDATE api_tokens SET token = '***REDACTED***' WHERE token != '***REDACTED***';
```

> **备注**：不直接 DROP 列，避免 SQLite 的 ALTER TABLE 限制（SQLite 3.35.0+ 才支持 DROP COLUMN）。
> 如需彻底移除列，可使用表重建方式。

#### 3b. 修改 create_token 返回值

**文件: `src/db/token.rs`**

创建后将明文置为占位符再存储，仅通过函数返回值传递明文：

```rust
pub async fn create_token(
    pool: &SqlitePool,
    name: &str,
    token: &str,
    expires_at: Option<NaiveDateTime>,
) -> Result<ApiToken, sqlx::Error> {
    let token_hash = hash_token(token);
    // 插入时 token 列存占位符，明文仅通过返回的内存对象传递
    sqlx::query_as::<_, ApiToken>(
        "INSERT INTO api_tokens (name, token, token_hash, expires_at) VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(name)
    .bind("***REDACTED***")
    .bind(token_hash)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map(|mut t| {
        t.token = token.to_string(); // 将明文回填到返回对象（一次性）
        t
    })
}
```

#### 3c. 修改 insert_initial_token

**文件: `src/db/token.rs`**

同样将初始 token 的明文列设为占位符：

```rust
pub async fn insert_initial_token(
    pool: &SqlitePool,
    name: &str,
    token: &str,
) -> Result<(), sqlx::Error> {
    let token_hash = hash_token(token);
    sqlx::query("INSERT INTO api_tokens (name, token, token_hash) VALUES (?, ?, ?)")
        .bind(name)
        .bind("***REDACTED***")
        .bind(token_hash)
        .execute(pool)
        .await?;
    Ok(())
}
```

> **注意**：修改后 `ensure_initial_token` 中首次打印的 `token_str` 来自函数局部变量，不受影响。
> 但后续 `get_first_active_token` 返回的 `token.token` 将是 `***REDACTED***`，Task 1 的掩码逻辑已处理此情况。

---

## Task 4: [P1 Bug] 前端 401 重定向使用 hash 赋值导致路由不跳转

### 问题

`web/src/renderer/src/api/client.ts` 第 33 行：

```ts
window.location.hash = '#/auth'
```

项目使用 `react-router-dom` 的 `BrowserRouter`（基于 History API），hash 赋值不会触发 BrowserRouter 的路由匹配，
导致 401 后页面不会跳转到认证页。

### 修复方案

**文件: `web/src/renderer/src/api/client.ts`**

改用 `window.location.href` 或 `window.location.replace` 进行整页跳转到 `/auth`：

```ts
if (error.response.status === 401) {
    localStorage.removeItem('api_token')
    window.location.replace('/auth')
    return Promise.reject(error)
}
```

> **备注**：由于 Electron 使用 `file://` 协议加载页面，`window.location.replace` 可能不适用。
> 需要确认 Electron 的路由模式。如果是 `HashRouter`，则原来的 hash 赋值是正确的；
> 如果是 `MemoryRouter` 或其他，需通过 IPC 通知主进程跳转。
> 建议先确认 `App.tsx` 中使用的 Router 类型再做最终修改。

---

## Task 5: [P1 Bug] list_sources 缺少 article_count 字段

### 问题

前端 `Sources.tsx` 第 187 行显示 `s.article_count`，但后端 `DataSource` 模型和 SQL 查询均无此字段，
导致前端始终显示 `—`。

### 修复方案

#### 5a. 添加数据库视图或修改查询

**文件: `src/db/source.rs`**

新增带 article_count 的查询函数：

```rust
/// 数据源及文章计数
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct SourceWithCount {
    pub id: i64,
    #[sqlx(rename = "type")]
    pub source_type: String,
    pub name: String,
    pub url: String,
    pub config: String,
    pub enabled: bool,
    pub interval_seconds: i64,
    pub last_fetched_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub article_count: i64,
}

pub async fn list_sources_with_count(pool: &SqlitePool) -> Result<Vec<SourceWithCount>, sqlx::Error> {
    sqlx::query_as::<_, SourceWithCount>(
        "SELECT ds.*, COUNT(a.id) as article_count \
         FROM data_sources ds \
         LEFT JOIN articles a ON a.source_id = ds.id \
         GROUP BY ds.id \
         ORDER BY ds.created_at DESC",
    )
    .fetch_all(pool)
    .await
}
```

#### 5b. 修改 handler 使用新查询

**文件: `src/handlers/source.rs`**

`list_sources` 函数改为调用 `list_sources_with_count`：

```rust
pub async fn list_sources(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let sources = db::source::list_sources_with_count(&state.pool).await?;
    Ok(ApiResponse::ok(sources))
}
```

---

## Task 6: [P1 性能] Pusher 每次调用创建新 reqwest::Client

### 问题

`src/services/pusher.rs` 第 41 行，`run_pusher_once` 每次调用都 `reqwest::Client::new()`，
无法复用 TCP 连接池，增加延迟和系统资源消耗。

### 修复方案

**文件: `src/services/pusher.rs`**

在 `start_pusher_loop` 中创建一次 `reqwest::Client`，传递给 `run_pusher_once`：

```rust
pub async fn run_pusher_once(pool: &SqlitePool, config: &PusherConfig, client: &reqwest::Client) {
    // 删除内部的 let client = reqwest::Client::new();
    // ... 其余逻辑不变，使用传入的 client
}

pub async fn start_pusher_loop(
    pool: SqlitePool,
    config: PusherConfig,
    pipeline: Pipeline,
    mut push_rx: mpsc::Receiver<PipelineEvent>,
) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build pusher HTTP client");

    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(config.interval_seconds));

    loop {
        tokio::select! {
            _ = pipeline.cancel.cancelled() => {
                tracing::info!("Pusher: shutting down gracefully");
                break;
            }
            _ = interval.tick() => {
                run_pusher_once(&pool, &config, &client).await;
            }
            Some(_) = push_rx.recv() => {
                run_pusher_once(&pool, &config, &client).await;
            }
        }
    }
}
```

同时更新 `trigger_pusher` handler（`src/handlers/query.rs`）：

```rust
pub async fn trigger_pusher(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let client = reqwest::Client::new(); // 手动触发用临时 client 可接受
    crate::services::pusher::run_pusher_once(&state.pool, &state.config.pusher, &client).await;
    Ok(ApiResponse::ok(json!({"message": "Pusher executed"})))
}
```

---

## Task 7: [P1 安全] 数据库路径放在 docs 目录不合理

### 问题

`config.toml` 中 `path = "./docs/data/hotspot.db"`，数据库文件存放在文档目录中。
`docs` 目录通常用于文档和版本控制，数据库文件不应放在此处。

### 修复方案

**文件: `config.toml`**

```toml
[database]
path = "./data/hotspot.db"
```

**文件: `.gitignore`**

确保 `./data/` 被忽略：

```
/data/
```

> **注意**：如果已有 `docs/data/` 目录被 git 跟踪，需要迁移或更新 `.gitignore`。
> 检查当前 `.gitignore` 中 `docs/data/` 的忽略规则。

---

## Task 8: [P2 改进] 输入验证逻辑重复

### 问题

各 handler（`source.rs`、`keyword.rs`、`channel.rs`、`token.rs`）中大量重复手动验证：
- `trim().is_empty()` 检查
- URL 格式检查
- JSON 格式检查
- 数值范围检查

### 修复方案

引入 `validator` crate 统一验证。

#### 8a. 添加依赖

**文件: `Cargo.toml`**

```toml
validator = { version = "0.19", features = ["derive"] }
```

#### 8b. 为 Request 结构体添加 derive 验证

**文件: `src/models/source.rs`**

```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSourceRequest {
    #[serde(rename = "type")]
    pub source_type: String,
    #[validate(length(min = 1, message = "name must not be empty"))]
    pub name: String,
    #[validate(url(message = "url must be a valid URL"))]
    pub url: String,
    pub interval_seconds: Option<i64>,
    pub config: Option<String>,
}
```

**文件: `src/models/keyword.rs`**

```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateKeywordRequest {
    #[validate(length(min = 1, message = "word must not be empty"))]
    pub word: String,
    pub case_sensitive: Option<bool>,
    #[validate(range(min = 0.0, exclusive_min = true, message = "std_multiplier must be positive"))]
    pub std_multiplier: Option<f64>,
    #[validate(range(min = 1, message = "min_hot_count must be >= 1"))]
    pub min_hot_count: Option<i32>,
}
```

#### 8c. 添加验证中间件或提取器

**文件: `src/error.rs`**

添加 `validator::ValidationErrors` 到 `AppError` 的 From 实现：

```rust
impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let msg = errors.to_string();
        AppError::BadRequest(msg)
    }
}
```

**文件: `src/handlers/source.rs` 等**

在 handler 开头添加验证调用：

```rust
use validator::Validate;

pub async fn create_source(
    State(state): State<AppState>,
    Json(req): Json<CreateSourceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    req.validate()?;
    // ... 移除手动验证代码
}
```

---

## Task 9: [P2 改进] Auth 页面混用 Ant Design 组件

### 问题

`web/src/renderer/src/pages/Auth.tsx` 使用了 `Card, Input, Button, Alert, Typography` 等 antd 组件，
而其他 5 个管理页面（Sources/Keywords/Channels/Tokens/Settings）遵循项目规范使用自定义内联样式组件，风格不统一。

### 修复方案

**文件: `web/src/renderer/src/pages/Auth.tsx`**

将所有 antd 组件替换为项目自定义组件 + Tailwind/CSS 类，与其他页面保持一致：

- `Card` → 自定义 `div` + `panel` 样式类
- `Input.Password` → 原生 `<input type="password">`
- `Button` → 项目统一的 `<button className="btn btn-primary">`
- `Alert` → 自定义错误提示 `div`
- `Typography.Title/Text` → 原生 `h2/p` + 项目字体类

```tsx
// 替换后的骨架结构
export default function AuthPage() {
    // ... 状态逻辑不变
    return (
        <div className="min-h-screen flex items-center justify-center bg-bg">
            <div className="panel w-[420px] max-w-[90vw] p-8">
                {/* 自定义品牌 Logo */}
                {/* 原生 input + button */}
                {/* 自定义错误提示 */}
            </div>
        </div>
    )
}
```

---

## Task 10: [P2 改进] 日志级别硬编码

### 问题

`src/main.rs` 第 57 行：

```rust
tracing_subscriber::fmt().with_env_filter("info").init();
```

硬编码为 `info` 级别，忽略了 `RUST_LOG` 环境变量。调试时无法动态调整日志级别。

### 修复方案

**文件: `src/main.rs`**

使用 `EnvFilter` 优先读取 `RUST_LOG`，默认回退到 `info`：

```rust
use tracing_subscriber::EnvFilter;

tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"))
    )
    .init();
```

---

## Task 11: [P2 改进] 缺少单元测试

### 问题

项目没有任何测试文件。核心逻辑（filter 突发检测、pusher 重试逻辑、config 验证）应该有测试覆盖。

### 修复方案

#### 11a. 为 config 验证添加单元测试

**文件: `src/config.rs`** 底部添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port_zero() {
        let config = AppConfig {
            server: ServerConfig { host: "0.0.0.0".into(), port: 0 },
            database: DatabaseConfig { path: "./test.db".into() },
            auth: AuthConfig { initial_token: None },
            parser: ParserConfig {
                max_concurrent_fetches: 10,
                default_user_agent: "test".into(),
                default_timeout_seconds: 30,
                interval_seconds: 30,
            },
            filter: FilterConfig {
                batch_size: 100, interval_seconds: 60, history_hours: 24, min_history_hours: 6,
            },
            pusher: PusherConfig {
                interval_seconds: 10, max_retries: 3, retry_base_seconds: 60,
            },
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_config() {
        let content = std::fs::read_to_string("config.toml").unwrap();
        let config: AppConfig = toml::from_str(&content).unwrap();
        assert!(config.validate().is_ok());
    }
}
```

#### 11b. 为 error 模块添加单元测试

**文件: `src/error.rs`** 底部添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn test_not_found_response() {
        let err = AppError::NotFound("test".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_database_error_masks_details() {
        let err = AppError::Database(sqlx::Error::PoolTimedOut);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
```

#### 11c. 为突发检测核心逻辑提取可测试函数

**文件: `src/services/filter.rs`**

将均值/标准差计算提取为独立函数：

```rust
/// 计算均值和标准差（纯函数，可独立测试）
pub fn compute_stats(counts: &[i32]) -> (f64, f64) {
    if counts.is_empty() {
        return (0.0, 0.0);
    }
    let n = counts.len() as f64;
    let mean = counts.iter().map(|c| *c as f64).sum::<f64>() / n;
    let variance = counts.iter().map(|c| (*c as f64 - mean).powi(2)).sum::<f64>() / n;
    (mean, variance.sqrt())
}
```

底部添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_stats_empty() {
        assert_eq!(compute_stats(&[]), (0.0, 0.0));
    }

    #[test]
    fn test_compute_stats_single() {
        let (mean, std) = compute_stats(&[5]);
        assert!((mean - 5.0).abs() < f64::EPSILON);
        assert!((std - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_stats_normal() {
        let (mean, std) = compute_stats(&[2, 4, 4, 4, 5, 5, 7, 9]);
        assert!((mean - 5.0).abs() < 0.01);
        assert!((std - 2.0).abs() < 0.01);
    }
}
```

---

## Task 12: [P2 改进] 前端缺少分页控件

### 问题

后端 `list_articles` 和 `list_hotspots` 支持分页参数（page/per_page），返回 `PaginatedResponse` 结构，
但前端 Articles 和 Hotspots 页面没有分页 UI，只能看到第一页数据。

### 修复方案

为 Articles 页面添加简单的分页控件。

**文件: `web/src/renderer/src/pages/Articles.tsx`**

添加分页状态和 UI：

```tsx
const [page, setPage] = useState(1)
const [total, setTotal] = useState(0)
const perPage = 20

// API 调用传入 page 和 per_page
// 底部添加分页控件
```

分页控件使用项目统一的 `btn btn-ghost btn-sm` 按钮样式，与现有管理页面一致。

> **备注**：具体的 Articles.tsx 当前内容需要进一步确认后编写完整方案。

---

## 执行优先级汇总

| 优先级 | Task | 类型 | 涉及文件 |
|--------|------|------|----------|
| P0 | Task 1 | 安全 | `src/main.rs` |
| P0 | Task 2 | 安全 | `src/routes.rs` |
| P0 | Task 3 | 安全 | `src/db/token.rs`, 新建迁移 |
| P1 | Task 4 | Bug | `web/.../api/client.ts` |
| P1 | Task 5 | Bug | `src/db/source.rs`, `src/handlers/source.rs` |
| P1 | Task 6 | 性能 | `src/services/pusher.rs`, `src/handlers/query.rs` |
| P1 | Task 7 | 配置 | `config.toml`, `.gitignore` |
| P2 | Task 8 | 改进 | `Cargo.toml`, `src/models/*.rs`, `src/handlers/*.rs`, `src/error.rs` |
| P2 | Task 9 | 改进 | `web/.../pages/Auth.tsx` |
| P2 | Task 10 | 改进 | `src/main.rs` |
| P2 | Task 11 | 改进 | `src/config.rs`, `src/error.rs`, `src/services/filter.rs` |
| P2 | Task 12 | 改进 | `web/.../pages/Articles.tsx` |
