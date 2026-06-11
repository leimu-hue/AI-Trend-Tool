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

## Task 4: ~~[P1 Bug] 前端 401 重定向使用 hash 赋值导致路由不跳转~~ — 无需修复

### 结论

项目使用 `HashRouter`（见 `web/src/renderer/src/main.tsx`），`window.location.hash = '#/auth'` 是正确的跳转方式，
401 后能正常触发路由匹配跳转到认证页。**此条非 Bug，无需修改。**

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

## ─── 以下为第二轮审查新增 ───

---

## Task 13: [P0 数据一致性] Filter 处理流程缺乏事务保护

### 问题

`src/services/filter.rs` 第 186-245 行，`run_filter_once` 的三个关键写操作
（upsert hot_event → insert push_records → mark_articles_processed）没有在一个数据库事务中执行。

如果 upsert hot_event 成功，但 insert_push_records 失败（如磁盘满），
随后第 241 行仍然会把所有文章标记为 `processed`。

**后果**：
- 文章被标记为已处理，永远不会被重新处理
- hot_event 记录存在，但没有对应的 push_records
- 该批次的热点数据永久丢失，无法恢复

### 修复方案

**文件: `src/services/filter.rs`**

将步骤 5-6 的写操作包裹在事务中，只在全部成功后 commit：

```rust
let mut tx = pool.begin().await?;
// ... upsert hot_events, insert push_records 使用 &mut *tx ...
tx.commit().await?;
// 事务提交成功后才标记文章为已处理
db::article::mark_processed_batch(pool, &article_ids).await?;
```

---

## Task 14: [P0 数据一致性] keyword_mentions 表缺少 UNIQUE 约束

### 问题

`docs/migrations/20260607044921_init.sql` 中 `keyword_mentions` 表没有 `(keyword_id, article_id)` 的 UNIQUE 约束。
但 `batch_insert_keyword_mentions` 使用 `INSERT OR IGNORE`，该语句只在违反 UNIQUE/PRIMARY KEY 约束时静默忽略。

由于没有约束，**每次 filter 运行都会为同一 (keyword_id, article_id) 对插入重复记录**，
导致表无限膨胀。

### 修复方案

#### 14a. 新建迁移文件

**新建文件: `docs/migrations/20260610000002_mentions_unique_index.sql`**

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_mentions_unique
    ON keyword_mentions(keyword_id, article_id);
```

#### 14b. 清理已有重复数据（可选）

可在迁移中先清理重复行（保留最早的一条）：

```sql
DELETE FROM keyword_mentions WHERE id NOT IN (
    SELECT MIN(id) FROM keyword_mentions GROUP BY keyword_id, article_id
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mentions_unique
    ON keyword_mentions(keyword_id, article_id);
```

---

## Task 15: [P0 功能] 生产环境 CSP `connect-src` 端口与后端不匹配

### 问题

`web/src/main/index.ts` 第 48 行，生产环境 CSP 设置为 `connect-src 'self' http://localhost:8080`，但：
- 页面通过 `file://` 协议加载，`'self'` 不覆盖 HTTP 请求
- 后端实际监听端口为 **3000**（`config.toml`），而非 8080
- 开发环境 CSP 正确使用了 `http://localhost:*` 通配符

**结果**：生产环境所有 API 调用会被 CSP 拦截，应用完全不可用。

### 修复方案

**文件: `web/src/main/index.ts`**

```ts
// 当前:
"connect-src 'self' http://localhost:8080"

// 修复 — 使用通配符匹配任意端口:
"connect-src 'self' http://localhost:*"
```

---

## Task 16: [P0 功能] axios 客户端默认 baseURL 与后端端口不匹配

### 问题

`web/src/renderer/src/api/client.ts` 第 12 行，`baseURL` fallback 硬编码为 `http://localhost:8080/api/v1`，
但后端 `config.toml` 默认端口为 `3000`。

- `.env.development` 正确指向 `http://localhost:3000/api/v1`
- 生产构建无 `.env.production`，`import.meta.env.VITE_API_BASE_URL` 为 `undefined`，走 fallback

**结果**：生产环境 API 请求指向错误端口。

### 修复方案

**文件: `web/src/renderer/src/api/client.ts`**

```ts
// 当前:
baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api/v1',

// 修复 — 与后端默认端口一致:
baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api/v1',
```

> **备注**：Settings 页面 `web/src/renderer/src/pages/Settings.tsx` 第 47 行
> `port: 8080` 也应同步改为 `port: 3000`。

---

## Task 17: [P1 Bug] Pusher 定时器与事件驱动并发触发 — Webhook 重复发送

### 问题

`src/services/pusher.rs` 第 261-286 行，`start_pusher_loop` 使用 `tokio::select!` 同时监听定时器和事件通道。
当 `run_filter_once` 完成后通过 `try_send` 通知 Pusher，但在 Pusher 处理 webhook 发送（网络 I/O）期间，
定时器 tick 可能触发新一轮 `run_pusher_once`，此时上一轮还未更新状态为 "success"。

**竞态窗口**：

```
run_pusher_once(A): SELECT pending records → [record #1 pending]
run_pusher_once(A): POST webhook (2 seconds)    ← 此时 interval.tick() 触发
run_pusher_once(B): SELECT pending records → [record #1 STILL pending!]
run_pusher_once(B): POST webhook (DUPLICATE!)
run_pusher_once(A): UPDATE status → success
run_pusher_once(B): UPDATE status → optimistic lock fails (warn logged)
```

**结果**：外部服务（钉钉、企业微信、Slack 等）收到重复的热点告警消息。

### 修复方案

**文件: `src/services/pusher.rs`**

在 `run_pusher_once` 开始前，使用原子 UPDATE "领取"待处理记录：

```rust
// 先将 pending → processing（原子领取），只处理标记为 processing 的记录
sqlx::query("UPDATE push_records SET status = 'processing' \
             WHERE status = 'pending' AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))")
    .execute(pool).await?;
// 然后 SELECT ... WHERE status = 'processing'
```

---

## Task 18: [P1 Bug] 分页 per_page 响应值与数据库实际限制不匹配

### 问题

`src/handlers/query.rs` 第 49-71 行（articles）和第 75-96 行（hotspots）。

`list_articles` handler 中，`per_page` 直接取用户传入值（如 200），
而 `db::article::list_articles` 内部 clamp 到 max 100。
但 `PaginatedResponse` 返回的是用户原始值 200，而非实际的 100。

**影响**：假设总共 250 条，用户请求 `per_page=200`：
- DB 返回 100 条（被 clamp），响应 `per_page: 200`
- 客户端计算 `ceil(250/200) = 2` 页，但实际需要 3 页
- **第 201-250 条数据永远无法被访问到**

### 修复方案

**文件: `src/handlers/query.rs`**

在 handler 层面先 clamp，再传入 DB 层和响应：

```rust
// list_articles:
let per_page = query.per_page.unwrap_or(20).min(100);
let query = ArticleQuery { per_page: Some(per_page), ..query };

// list_hotspots 同理:
let per_page = params.per_page.unwrap_or(20).min(100);
```

---

## Task 19: [P1 Bug] `insert_push_records_for_event` 静默吞掉数据库错误

### 问题

`src/db/push_record.rs` 中 `INSERT OR IGNORE` 会忽略**所有**类型的 SQLite 错误（不仅是 UNIQUE 冲突），
包括磁盘满、外键违反等。`if let Ok(Some(r))` 模式把所有错误静默忽略，无任何日志。

**结果**：如果 `channel_id` 引用了已删除的 channel（FK 违反），记录静默丢失，调用方不收到通知。

### 修复方案

**文件: `src/db/push_record.rs`**

区分 `Ok(None)`（UNIQUE 冲突，正常跳过）和 `Err(e)`（真正错误，应记录日志）：

```rust
match sqlx::query_as::<_, PushRecord>(
    "INSERT OR IGNORE INTO push_records ... RETURNING *"
)
.bind(hot_event_id)
.bind(channel_id)
.fetch_optional(pool)
.await
{
    Ok(Some(r)) => records.push(r),
    Ok(None) => {} // duplicate, skip
    Err(e) => {
        tracing::error!("Failed to insert push record for channel {}: {}", channel_id, e);
    }
}
```

---

## Task 20: [P1 Bug] Settings 页面 fallback 默认值与后端配置不匹配

### 问题

`web/src/renderer/src/pages/Settings.tsx` 第 27-48 行，当 API 请求失败时页面显示前端硬编码的 `DEFAULTS`，
但多个值与 `config.toml` 不一致：

| 字段 | DEFAULTS (前端) | config.toml (后端) |
|------|-----------------|-------------------|
| `max_concurrent_fetches` | 5 | **10** |
| `batch_size` | 100 | **1000** |
| `server.port` | 8080 | **3000** |

### 修复方案

**文件: `web/src/renderer/src/pages/Settings.tsx`**

更新 DEFAULTS 对象使其与 `config.toml` 一致：

```ts
const DEFAULTS = {
  server: { host: '0.0.0.0', port: 3000 },
  parser: { max_concurrent_fetches: 10, /* ... */ },
  filter: { batch_size: 1000, /* ... */ },
  // ...
}
```

---

## Task 21: [P1 内存泄漏] Toast 组件嵌套 setTimeout 卸载时无法清理

### 问题

`web/src/renderer/src/components/Toast.tsx` 第 40-43 行，`show` 方法创建嵌套 `setTimeout`
（外层 duration 后触发退出动画，内层 300ms 后移除 DOM）。

当 `ToastProvider` 卸载时（如 HMR 热重载），这些定时器不会被清除，
会对已卸载组件调用 `setToasts`，造成内存泄漏和 React 警告。

### 修复方案

**文件: `web/src/renderer/src/components/Toast.tsx`**

用 `useRef` 收集所有 timer ID，在组件 cleanup 中统一 `clearTimeout`：

```tsx
const timersRef = useRef<Set<ReturnType<typeof setTimeout>>>(new Set())

// show 方法中:
const t1 = setTimeout(() => { /* ... */ }, duration)
const t2 = setTimeout(() => { /* ... */ }, duration + 300)
timersRef.current.add(t1)
timersRef.current.add(t2)

// useEffect cleanup:
useEffect(() => {
  return () => {
    timersRef.current.forEach(clearTimeout)
  }
}, [])
```

---

## Task 22: [P1 误导] Layout 页脚文案声称"每5分钟自动刷新"但无实际逻辑

### 问题

`web/src/renderer/src/components/Layout.tsx` 第 162 行，侧边栏底部固定显示
`"监控中 · 每5分钟自动刷新"`，但代码中没有任何定时数据刷新机制。
各页面仅在 `useEffect` 中加载一次数据，之后不会自动刷新。

### 修复方案

**文件: `web/src/renderer/src/components/Layout.tsx`**

方案 A — 移除误导性文案：

```tsx
// 当前: "监控中 · 每5分钟自动刷新"
// 修复: "监控中" 或 "后端监控运行中"
```

方案 B — 实际实现定时轮询（推荐后续迭代）。

---

## Task 23: [P2 可靠性] Parser spawn 的子任务未被跟踪 — 关闭时可能丢失数据

### 问题

`src/services/parser.rs` 为每个到期数据源 `tokio::spawn` 一个独立任务，但没有收集 `JoinHandle`。
当 Ctrl+C 触发取消时，parser loop 退出，但已 spawn 的 fetch 子任务仍在运行。
`main.rs` 的 `tokio::join!` 只等待 `start_parser_loop` 返回，**不等待其内部 spawn 的子任务**。

**结果**：正在进行的 RSS 抓取和文章插入可能在写入过程中被强制终止。

### 修复方案

**文件: `src/services/parser.rs`**

使用 `JoinSet` 或 `Vec<JoinHandle>` 跟踪子任务，在 cancel 信号后 await 所有进行中的任务：

```rust
let mut tasks = tokio::task::JoinSet::new();
// spawn 时:
tasks.spawn(async move { fetch_source(...).await });
// cancel 后:
while let Some(_) = tasks.join_next().await {}
```

---

## Task 24: [P2 性能] DB 连接池大小 (5) < Parser 并发任务数 (10)

### 问题

`src/db.rs` 中 `max_connections(5)`，但 `config.toml` 中 `max_concurrent_fetches = 10`。
每个 fetch 任务需要多次 DB 操作，当 10 个任务同时运行时只有 5 个能获得连接，
其余排队等待，实际并发度受限于 5。

### 修复方案

**文件: `src/db.rs`**

将 `max_connections` 设为 `>= max_concurrent_fetches + 5`（额外预留给 filter/pusher/API）：

```rust
.max_connections(std::cmp::max(config_max_concurrent_fetches as u32 + 5, 10))
```

或改为从 config 中读取：

```rust
.max_connections(config.database.max_connections.unwrap_or(15))
```

---

## Task 25: [P2 Bug] `hours` 参数 i64→i32 类型截断无校验

### 问题

`src/handlers/query.rs` 第 133-134 行：

```rust
let hours = params.hours.unwrap_or(24);
let rows = db::hot_event::get_hourly_counts(&state.pool, keyword_id, hours as i32).await?;
```

如果用户传入 `hours=2147483648`（超过 i32::MAX），`as i32` 静默截断为负数，
导致 SQL 查询异常。

### 修复方案

**文件: `src/handlers/query.rs`**

```rust
let hours = params.hours.unwrap_or(24).clamp(1, 8760) as i32; // max 1 year
```

---

## Task 26: [P2 Bug] `main.rs` 中 DB 路径 `unwrap()` 可能 panic

### 问题

`src/main.rs` 第 66-68 行：

```rust
let db_dir = std::path::Path::new(&config.database.path)
    .parent()
    .unwrap();
```

当 `database.path` 为根路径 `"/"` 时，`parent()` 返回 `None`，`unwrap()` panic。
虽然 `config.validate()` 检查了 `path.is_empty()`，但 `"/"` 边界情况未覆盖。

### 修复方案

**文件: `src/main.rs`**

```rust
let db_dir = std::path::Path::new(&config.database.path)
    .parent()
    .ok_or("database.path has no valid parent directory")?;
```

---

## Task 27: [P2 安全] ErrorBoundary 向用户暴露原始错误消息

### 问题

`web/src/renderer/src/components/ErrorBoundary.tsx` 第 34 行，
`this.state.error?.message` 直接显示给用户，可能包含内部实现细节（堆栈片段、变量名等）。

### 修复方案

**文件: `web/src/renderer/src/components/ErrorBoundary.tsx`**

使用通用错误提示，详细错误仅输出到 `console.error`：

```tsx
console.error('ErrorBoundary caught:', this.state.error)
// UI 显示: "页面发生了未知错误，请刷新页面重试"
```

---

## Task 28: [P2 代码卫生] Tokens 页面使用已废弃的 `document.execCommand('copy')`

### 问题

`web/src/renderer/src/pages/Tokens.tsx` 第 83-96 行使用 `document.execCommand('copy')`。
虽然注释说明了 contextIsolation 下的兼容考虑，但 preload 脚本已暴露了
`window.electronAPI.clipboard.writeText()` IPC 桥接，应优先使用。

### 修复方案

**文件: `web/src/renderer/src/pages/Tokens.tsx`**

```tsx
// 当前: document.execCommand('copy')
// 修复: 使用 preload 暴露的 IPC 桥接
await window.electronAPI.clipboard.writeText(tokenStr)
```

---

## Task 29: [P2 代码卫生] `useApi` hook 和 `Loading` 组件为死代码

### 问题

- `web/src/renderer/src/hooks/useApi.ts` — 未被任何模块引用
- `web/src/renderer/src/components/Loading.tsx` — 未被任何模块引用
- `useApi` 内部引用的 `useNotify` 也仅被 `useApi` 使用

三者构成完整的死代码链。

### 修复方案

删除以下文件：
- `web/src/renderer/src/hooks/useApi.ts`
- `web/src/renderer/src/components/Loading.tsx`

并从 `lib/notification.ts` 中移除仅被 `useApi` 使用的 `useNotify` hook（如确认无其他引用）。

---

## Task 30: [P2 安全] preload 暴露了未使用的 `clipboard.readText`

### 问题

`web/src/preload/index.ts` 第 11 行暴露了 `clipboard.readText`，
但应用中只有 `clipboard.writeText` 被使用（Tokens 页面）。
根据 Electron 最小权限原则，应移除不必要的 API 暴露。

### 修复方案

**文件: `web/src/preload/index.ts`**

移除 `clipboard.readText` 的 contextBridge 暴露：

```ts
// 移除:
// clipboard: { readText: () => ipcRenderer.invoke('clipboard:read') }
```

同时移除 `web/src/main/index.ts` 中对应的 `ipcMain.handle('clipboard:read', ...)` 处理。

---

## Task 31: [P2 可靠性] `notification.ts` 模块级缓存无清理机制

### 问题

`web/src/renderer/src/lib/notification.ts` 第 30-34 行，`contextApi` 为模块级变量，
由 `useNotificationBridge` 写入但永远不会被清理。
如果 `App` 组件卸载（HMR 或热重载），`contextApi` 仍指向旧的、可能已失效的实例。

### 修复方案

**文件: `web/src/renderer/src/lib/notification.ts`**

在 `useNotificationBridge` 的 `useEffect` 中添加 cleanup：

```tsx
useEffect(() => {
  setNotificationApi(notification)
  return () => setNotificationApi(null) // 需修改 setNotificationApi 允许 null
}, [notification])
```

---

## 执行优先级汇总

| 优先级 | Task | 类型 | 涉及文件 |
|--------|------|------|----------|
| P0 | Task 1 | 安全 | `src/main.rs` |
| P0 | Task 2 | 安全 | `src/routes.rs` |
| P0 | Task 3 | 安全 | `src/db/token.rs`, 新建迁移 |
| P0 | Task 13 | 数据一致性 | `src/services/filter.rs` |
| P0 | Task 14 | 数据一致性 | 新建迁移 |
| P0 | Task 15 | 功能 | `web/src/main/index.ts` |
| P0 | Task 16 | 功能 | `web/.../api/client.ts`, `web/.../pages/Settings.tsx` |
| P1 | Task 4 | Bug | `web/.../api/client.ts` — **无需修复** |
| P1 | Task 5 | Bug | `src/db/source.rs`, `src/handlers/source.rs` |
| P1 | Task 6 | 性能 | `src/services/pusher.rs`, `src/handlers/query.rs` |
| P1 | Task 7 | 配置 | `config.toml`, `.gitignore` |
| P1 | Task 17 | Bug | `src/services/pusher.rs` |
| P1 | Task 18 | Bug | `src/handlers/query.rs` |
| P1 | Task 19 | Bug | `src/db/push_record.rs` |
| P1 | Task 20 | Bug | `web/.../pages/Settings.tsx` |
| P1 | Task 21 | 内存泄漏 | `web/.../components/Toast.tsx` |
| P1 | Task 22 | 误导 | `web/.../components/Layout.tsx` |
| P2 | Task 8 | 改进 | `Cargo.toml`, `src/models/*.rs`, `src/handlers/*.rs`, `src/error.rs` |
| P2 | Task 9 | 改进 | `web/.../pages/Auth.tsx` |
| P2 | Task 10 | 改进 | `src/main.rs` |
| P2 | Task 11 | 改进 | `src/config.rs`, `src/error.rs`, `src/services/filter.rs` |
| P2 | Task 12 | 改进 | `web/.../pages/Articles.tsx` |
| P2 | Task 23 | 可靠性 | `src/services/parser.rs` |
| P2 | Task 24 | 性能 | `src/db.rs` |
| P2 | Task 25 | Bug | `src/handlers/query.rs` |
| P2 | Task 26 | Bug | `src/main.rs` |
| P2 | Task 27 | 安全 | `web/.../components/ErrorBoundary.tsx` |
| P2 | Task 28 | 代码卫生 | `web/.../pages/Tokens.tsx` |
| P2 | Task 29 | 代码卫生 | `web/.../hooks/useApi.ts`, `web/.../components/Loading.tsx` |
| P2 | Task 30 | 安全 | `web/src/preload/index.ts`, `web/src/main/index.ts` |
| P2 | Task 31 | 可靠性 | `web/.../lib/notification.ts` |
