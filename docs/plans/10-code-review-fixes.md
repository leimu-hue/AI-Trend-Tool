# 代码审查修复计划

> 本文档为 Agent 执行用修复指令，按优先级从高到低排列。每个 Task 包含精确的文件路径、问题描述和修复方案。

---

## Task 1: [P0 Bug] push_records retry_count 硬编码与配置不同步

### 问题

`src/db/push_record.rs` 第 37-47 行的 `list_retry_due_records` 函数中 SQL 写死了 `retry_count < 3`，
但 `config.toml` 中 `pusher.max_retries = 3`，`PusherConfig` 也有 `max_retries` 字段。
如果用户修改了配置中的 `max_retries`，SQL 中的硬编码不会跟着变。

### 修复方案

**文件: `src/db/push_record.rs`**

将 `list_retry_due_records` 函数签名改为接受 `max_retries: u32` 参数：

```rust
pub async fn list_retry_due_records(pool: &SqlitePool, max_retries: u32) -> Result<Vec<PushRecord>, sqlx::Error> {
    sqlx::query_as::<_, PushRecord>(
        "SELECT * FROM push_records \
         WHERE status = 'failed' \
         AND retry_count < ? \
         AND next_retry_at <= datetime('now') \
         ORDER BY next_retry_at ASC",
    )
    .bind(max_retries as i64)
    .fetch_all(pool)
    .await
}
```

**文件: `src/services/pusher.rs`**

第 23 行调用处改为传入 `config.max_retries`：

```rust
let retry_due = match db::push_record::list_retry_due_records(pool, config.max_retries).await {
```

---

## Task 2: [P0 Bug] upsert_hot_event_record 使用 DELETE+INSERT 导致外键断裂

### 问题

`src/services/filter.rs` 第 254-278 行的 `upsert_hot_event_record` 先 DELETE 再 INSERT hot_events 记录。
DELETE 会触发 `push_records` 的 `ON DELETE CASCADE`，导致关联的推送记录被级联删除。
即使没有级联删除，原有的 hot_event.id 也会变化，导致 push_records 中的 hot_event_id 外键指向不存在的记录。

### 修复方案

需要两步：先加数据库迁移添加 UNIQUE 约束，再改代码。

#### 2a. 新建迁移文件

**新建文件: `docs/migrations/20260609000001_hot_events_unique.sql`**

```sql
-- 为 hot_events 添加 (keyword_id, hour_bucket) 唯一约束以支持 UPSERT
-- SQLite 不支持 ALTER TABLE ADD CONSTRAINT，需要重建表

-- 1. 创建新表
CREATE TABLE IF NOT EXISTS hot_events_new (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword_id        INTEGER NOT NULL REFERENCES keywords(id) ON DELETE CASCADE,
    hour_bucket       TEXT    NOT NULL,
    count             INTEGER NOT NULL DEFAULT 0,
    mean_historical   REAL    NOT NULL DEFAULT 0.0,
    stddev_historical REAL    NOT NULL DEFAULT 0.0,
    created_at        DATETIME NOT NULL DEFAULT (datetime('now')),
    UNIQUE(keyword_id, hour_bucket)
);

-- 2. 迁移数据（去重：保留每个 keyword_id+hour_bucket 中 id 最大的记录）
INSERT INTO hot_events_new (keyword_id, hour_bucket, count, mean_historical, stddev_historical, created_at)
SELECT keyword_id, hour_bucket, count, mean_historical, stddev_historical, created_at
FROM hot_events
WHERE id IN (
    SELECT MAX(id) FROM hot_events GROUP BY keyword_id, hour_bucket
);

-- 3. 删除旧表并重命名
DROP TABLE IF EXISTS hot_events;
ALTER TABLE hot_events_new RENAME TO hot_events;

-- 4. 重建索引
CREATE INDEX IF NOT EXISTS idx_hot_events_keyword ON hot_events(keyword_id);
CREATE INDEX IF NOT EXISTS idx_hot_events_bucket  ON hot_events(hour_bucket);
```

#### 2b. 改用 ON CONFLICT 语法

**文件: `src/services/filter.rs`**

将第 254-278 行的 `upsert_hot_event_record` 替换为：

```rust
/// Upsert a hot_event record using ON CONFLICT to preserve id and foreign keys.
async fn upsert_hot_event_record(
    pool: &SqlitePool,
    keyword_id: i64,
    hour_bucket: &str,
    count: i32,
    mean_historical: f64,
    stddev_historical: f64,
) -> Result<crate::models::hot_event::HotEvent, sqlx::Error> {
    sqlx::query_as::<_, crate::models::hot_event::HotEvent>(
        "INSERT INTO hot_events (keyword_id, hour_bucket, count, mean_historical, stddev_historical) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT(keyword_id, hour_bucket) \
         DO UPDATE SET count = excluded.count, \
                       mean_historical = excluded.mean_historical, \
                       stddev_historical = excluded.stddev_historical \
         RETURNING *",
    )
    .bind(keyword_id)
    .bind(hour_bucket)
    .bind(count)
    .bind(mean_historical)
    .bind(stddev_historical)
    .fetch_one(pool)
    .await
}
```

同时删除 `src/db/hot_event.rs` 中的 `insert_hot_event` 函数（因为不再需要，逻辑已内联到 filter.rs）。
如果其他地方没有调用 `insert_hot_event`，直接删除即可。

---

## Task 3: [P1 安全] API Token 哈希存储

### 问题

`api_tokens` 表中 `token` 字段以明文存储。任何有数据库读权限的人都可以获取所有有效 token。

### 修复方案

#### 3a. 添加依赖

**文件: `Cargo.toml`** — 在 `[dependencies]` 中添加：

```toml
sha2 = "0.10"
```

#### 3b. 新建迁移文件

**新建文件: `docs/migrations/20260609000002_token_hash.sql`**

```sql
-- 添加 token_hash 列，保留 token 列用于向后兼容（后续版本可删除）
ALTER TABLE api_tokens ADD COLUMN token_hash TEXT NOT NULL DEFAULT '';

-- 为 token_hash 创建唯一索引（替代原 token 列的唯一约束）
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_tokens_token_hash ON api_tokens(token_hash);
```

#### 3c. 添加哈希工具函数

**文件: `src/db/token.rs`** — 顶部添加：

```rust
use sha2::{Sha256, Digest};

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
```

#### 3d. 修改所有 token 写入和查询逻辑

**文件: `src/db/token.rs`**

1. `create_token`: 同时写入 `token` 和 `token_hash`
2. `get_token_by_value`: 改为按 `token_hash` 查询
3. `insert_initial_token`: 同时写入 `token_hash`

```rust
pub async fn create_token(
    pool: &SqlitePool,
    name: &str,
    token: &str,
    expires_at: Option<NaiveDateTime>,
) -> Result<ApiToken, sqlx::Error> {
    let token_hash = hash_token(token);
    sqlx::query_as::<_, ApiToken>(
        "INSERT INTO api_tokens (name, token, token_hash, expires_at) VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(name)
    .bind(token)
    .bind(&token_hash)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn get_token_by_value(
    pool: &SqlitePool,
    token: &str,
) -> Result<Option<ApiToken>, sqlx::Error> {
    let token_hash = hash_token(token);
    sqlx::query_as::<_, ApiToken>(
        "SELECT * FROM api_tokens WHERE token_hash = ? AND revoked = 0"
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
}

pub async fn insert_initial_token(
    pool: &SqlitePool,
    name: &str,
    token: &str,
) -> Result<(), sqlx::Error> {
    let token_hash = hash_token(token);
    sqlx::query("INSERT INTO api_tokens (name, token, token_hash) VALUES (?, ?, ?)")
        .bind(name)
        .bind(token)
        .bind(&token_hash)
        .execute(pool)
        .await?;
    Ok(())
}
```

#### 3e. ApiToken model 添加 token_hash 字段

**文件: `src/models/token.rs`**

```rust
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ApiToken {
    pub id: i64,
    pub name: String,
    pub token: String,
    pub token_hash: String,
    pub last_used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub revoked: bool,
}
```

同时在 `ApiTokenInfo` 中也添加 `token_hash` 字段（或排除掉），并更新 `From<ApiToken>` 实现。

---

## Task 4: [P1 性能] reqwest::Client 复用

### 问题

`src/services/parser.rs` 第 54-57 行，`RssParser::fetch_and_parse` 每次调用都创建新的 `reqwest::Client`。
HTTP 客户端的连接池无法复用，且每次创建有额外开销。

### 修复方案

**文件: `src/services/parser.rs`**

将 `reqwest::Client` 作为 `RssParser` 的成员字段：

```rust
pub struct RssParser {
    client: reqwest::Client,
}

impl RssParser {
    pub fn new(config: &ParserConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(&config.default_user_agent)
            .timeout(std::time::Duration::from_secs(config.default_timeout_seconds))
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }
}
```

然后 `fetch_and_parse` 方法体中删除 client 创建代码，直接使用 `&self.client`：

```rust
let response = self.client.get(&source.url).send().await?;
```

---

## Task 5: [P1 安全] 请求输入验证

### 问题

所有 `CreateXxxRequest` 和 `UpdateXxxRequest` 结构体没有做输入校验。
空字符串 `""` 可以作为 name/url/word 提交，无效 URL 格式也不会被拒绝。

### 修复方案

在每个 handler 函数入口处添加手动校验（不引入新依赖），示例：

**文件: `src/handlers/source.rs`** — `create_source` 函数开头添加：

```rust
if req.name.trim().is_empty() {
    return Err(AppError::BadRequest("name must not be empty".into()));
}
if req.url.trim().is_empty() {
    return Err(AppError::BadRequest("url must not be empty".into()));
}
if !req.url.starts_with("http://") && !req.url.starts_with("https://") {
    return Err(AppError::BadRequest("url must start with http:// or https://".into()));
}
```

**文件: `src/handlers/keyword.rs`** — `create_keyword` 函数开头添加：

```rust
if req.word.trim().is_empty() {
    return Err(AppError::BadRequest("word must not be empty".into()));
}
if let Some(m) = req.std_multiplier {
    if m <= 0.0 {
        return Err(AppError::BadRequest("std_multiplier must be positive".into()));
    }
}
if let Some(c) = req.min_hot_count {
    if c < 1 {
        return Err(AppError::BadRequest("min_hot_count must be >= 1".into()));
    }
}
```

**文件: `src/handlers/channel.rs`** — `create_channel` 函数开头添加：

```rust
if req.name.trim().is_empty() {
    return Err(AppError::BadRequest("name must not be empty".into()));
}
if req.config.trim().is_empty() {
    return Err(AppError::BadRequest("config must not be empty".into()));
}
// 验证 config 是合法 JSON
if serde_json::from_str::<serde_json::Value>(&req.config).is_err() {
    return Err(AppError::BadRequest("config must be valid JSON".into()));
}
```

**文件: `src/handlers/token.rs`** — `create_token` 函数开头添加：

```rust
if req.name.trim().is_empty() {
    return Err(AppError::BadRequest("name must not be empty".into()));
}
```

---

## Task 6: [P2 性能] Filter keyword_mention 批量插入

### 问题

`src/services/filter.rs` 第 100-133 行，每个关键词-文章匹配都单独执行一次 INSERT SQL。
当有 1000 篇文章 × 50 个关键词时，最坏情况会产生 50000 次 SQL 调用。

### 修复方案

**文件: `src/db/keyword_mention.rs`**

添加批量插入函数：

```rust
/// Batch insert keyword mentions using a single INSERT with multiple VALUES.
/// Chunks into groups of 100 to stay within SQLite variable limits.
pub async fn batch_insert_keyword_mentions(
    pool: &SqlitePool,
    mentions: &[(i64, i64)], // (keyword_id, article_id)
) -> Result<(), sqlx::Error> {
    for chunk in mentions.chunks(100) {
        let placeholders: Vec<&str> = chunk.iter().map(|_| "(?, ?)").collect();
        let sql = format!(
            "INSERT OR IGNORE INTO keyword_mentions (keyword_id, article_id) VALUES {}",
            placeholders.join(", ")
        );
        let mut query = sqlx::query(&sql);
        for &(kw_id, art_id) in chunk {
            query = query.bind(kw_id).bind(art_id);
        }
        query.execute(pool).await?;
    }
    Ok(())
}
```

**文件: `src/services/filter.rs`**

修改第 90-134 行的匹配循环，先收集所有 mentions 到 Vec，最后一次性批量插入：

将匹配循环中的 `db::keyword_mention::insert_keyword_mention` 调用替换为收集到 Vec：

```rust
let mut mentions: Vec<(i64, i64)> = Vec::new();

for article in &articles {
    let text = format!("{} {}", article.title, article.summary);
    article_ids.push(article.id);

    if let Some(ref ac) = ci_ac {
        for mat in ac.find_iter(&text) {
            let (_, kw) = ci_kws[mat.pattern()];
            *hourly_counts.entry(kw.id).or_insert(0) += 1;
            mentions.push((kw.id, article.id));
        }
    }

    if let Some(ref ac) = cs_ac {
        for mat in ac.find_iter(&text) {
            let (_, kw) = cs_kws[mat.pattern()];
            *hourly_counts.entry(kw.id).or_insert(0) += 1;
            mentions.push((kw.id, article.id));
        }
    }
}

// 批量插入所有 keyword_mentions
if let Err(e) = db::keyword_mention::batch_insert_keyword_mentions(pool, &mentions).await {
    tracing::error!("Filter: failed to batch insert keyword_mentions: {}", e);
}
```

---

## Task 7: [P2 性能] Pusher 并发推送

### 问题

`src/services/pusher.rs` 第 42-44 行串行处理所有推送记录。
当有多个渠道 × 多个热点时，推送延迟会线性叠加。

### 修复方案

#### 7a. 添加依赖

**文件: `Cargo.toml`** — 在 `[dependencies]` 中添加：

```toml
futures = "0.3"
```

#### 7b. 修改 run_pusher_once

**文件: `src/services/pusher.rs`**

添加顶部 import：

```rust
use futures::stream::{self, StreamExt};
```

将第 39-44 行替换为并发执行（最多 8 个并发）：

```rust
let pool_ref = pool.clone();
let config_ref = config.clone();
stream::iter(pushable.iter())
    .for_each_concurrent(8, |record| {
        let pool = pool_ref.clone();
        let config = config_ref.clone();
        let client = client.clone();
        async move {
            process_one(&pool, &config, &client, record).await;
        }
    })
    .await;
```

注意 `reqwest::Client` 本身就是 `Clone` 且内部共享连接池的，所以 `client.clone()` 很轻量。

---

## Task 8: [P2 性能] Filter compute_historical_stats 批量查询

### 问题

`src/services/filter.rs` 第 147-211 行，对每个关键词都调用一次 `compute_historical_stats`，
每次都执行一条 SQL 查询。当有 100 个关键词时就是 100 次 DB 查询。

### 修复方案

**文件: `src/db/hot_event.rs`**

添加批量查询函数：

```rust
/// Get hourly counts for ALL keywords over recent N hours in a single query.
/// Returns Vec<(keyword_id, hour_bucket, total_count)>.
pub async fn get_all_hourly_counts(
    pool: &SqlitePool,
    hours: i32,
) -> Result<Vec<(i64, String, i32)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, i32)>(
        "SELECT keyword_id, hour_bucket, SUM(count) as total \
         FROM hot_events \
         GROUP BY keyword_id, hour_bucket \
         ORDER BY keyword_id, hour_bucket DESC",
    )
    .fetch_all(pool)
    .await
}
```

**文件: `src/services/filter.rs`**

在关键词循环之前一次性加载所有历史数据，然后在内存中按 keyword_id 分组计算统计量：

```rust
// 在循环之前，一次性查询所有关键词的历史数据
let all_history = db::hot_event::get_all_hourly_counts(pool, config.history_hours as i32)
    .await
    .unwrap_or_default();

let mut history_by_kw: std::collections::HashMap<i64, Vec<f64>> = std::collections::HashMap::new();
for (kw_id, _bucket, count) in &all_history {
    history_by_kw.entry(*kw_id).or_default().push(*count as f64);
}

// 预计算每个关键词的 (mean, stddev)
let mut stats_by_kw: std::collections::HashMap<i64, (f64, f64)> = std::collections::HashMap::new();
for (&kw_id, counts) in &history_by_kw {
    let n = counts.len() as f64;
    if n == 0.0 { continue; }
    let mean = counts.iter().sum::<f64>() / n;
    let variance = counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / n;
    stats_by_kw.insert(kw_id, (mean, variance.sqrt()));
}
```

然后在关键词循环中直接从 `stats_by_kw` 获取统计量，替换 `compute_historical_stats` 调用。

---

## Task 9: [P2 可靠性] 优雅关闭等待后台任务完成

### 问题

`src/main.rs` 第 92-108 行，`tokio::spawn` 启动了三个后台任务但没有保存 `JoinHandle`。
当 Ctrl+C 触发后，server 关闭但后台任务可能正在执行中途（如正在写入数据库），主进程直接退出导致数据不一致。

### 修复方案

**文件: `src/main.rs`**

保存 JoinHandle 并在 server 退出后 await：

```rust
let parser_handle = tokio::spawn(services::parser::start_parser_loop(
    pool.clone(),
    config.parser.clone(),
    pipeline.clone(),
));
let filter_handle = tokio::spawn(services::filter::start_filter_loop(
    pool.clone(),
    config.filter.clone(),
    pipeline.clone(),
    articles_rx,
));
let pusher_handle = tokio::spawn(services::pusher::start_pusher_loop(
    pool.clone(),
    config.pusher.clone(),
    pipeline.clone(),
    push_rx,
));
```

在 `axum::serve(...)` 之后添加：

```rust
tracing::info!("Waiting for background tasks to finish...");
let _ = tokio::join!(parser_handle, filter_handle, pusher_handle);
tracing::info!("All background tasks finished");
```

---

## Task 10: [P2 可观测性] Health check 增加数据库探活

### 问题

`src/routes.rs` 第 63-65 行的 `/health` 端点只返回固定 `{"status": "ok"}`，
无法反映数据库连接是否正常。

### 修复方案

**文件: `src/routes.rs`**

将 health_check 改为接受 State 并执行 `SELECT 1`：

```rust
async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db_status = match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => "ok",
        Err(e) => {
            tracing::error!("Health check: database error: {}", e);
            "error"
        }
    };
    Json(json!({
        "status": if db_status == "ok" { "ok" } else { "degraded" },
        "database": db_status
    }))
}
```

同时需要把 health_check 路由从外层 Router 移到带 state 的 Router 上，但保持在 auth middleware 之外：

```rust
let api = Router::new()
    // ... 所有 API 路由
    .with_state(state.clone())
    .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

Router::new()
    .route("/health", get(health_check))
    .with_state(state.clone())
    .nest("/api/v1", api)
    .layer(CorsLayer::permissive())
```

需要在 use 中添加 `use axum::extract::State;`。

---

## Task 11: [P2 健壮性] 配置加载后校验

### 问题

`src/config.rs` 的 `AppConfig::load` 只做了反序列化，没有校验字段合法性。
无效配置（如 port=0, interval_seconds=0）会在运行时才暴露。

### 修复方案

**文件: `src/config.rs`**

为 `AppConfig` 添加 `validate` 方法：

```rust
impl AppConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server.port == 0 {
            return Err("server.port must be > 0".into());
        }
        if self.database.path.is_empty() {
            return Err("database.path must not be empty".into());
        }
        if self.parser.interval_seconds == 0 {
            return Err("parser.interval_seconds must be > 0".into());
        }
        if self.parser.max_concurrent_fetches == 0 {
            return Err("parser.max_concurrent_fetches must be > 0".into());
        }
        if self.filter.batch_size == 0 {
            return Err("filter.batch_size must be > 0".into());
        }
        if self.filter.interval_seconds == 0 {
            return Err("filter.interval_seconds must be > 0".into());
        }
        if self.pusher.interval_seconds == 0 {
            return Err("pusher.interval_seconds must be > 0".into());
        }
        if self.pusher.max_retries == 0 {
            return Err("pusher.max_retries must be > 0".into());
        }
        Ok(())
    }
}
```

---

## Task 12: [P3 代码质量] list_articles / count_articles 消除重复分支

### 问题

`src/db/article.rs` 第 45-78 行和第 135-146 行，`list_articles` 和 `count_articles` 各有 4 个 match 分支，
区别仅在于 bind 参数数量。代码重复度高，维护成本大。

### 修复方案

**文件: `src/db/article.rs`**

重构 `list_articles` 和 `count_articles`，用动态绑定替代 match 分支：

```rust
enum BindValue {
    Int(i64),
    Bool(bool),
}

fn build_article_filter(query: &ArticleQuery) -> (String, Vec<BindValue>) {
    let mut conditions = vec![];
    let mut params = vec![];
    if let Some(source_id) = query.source_id {
        conditions.push("source_id = ?");
        params.push(BindValue::Int(source_id));
    }
    if let Some(processed) = query.processed {
        if processed {
            conditions.push("processed_at IS NOT NULL");
        } else {
            conditions.push("processed_at IS NULL");
        }
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    (where_clause, params)
}

pub async fn list_articles(
    pool: &SqlitePool,
    query: &ArticleQuery,
) -> Result<Vec<Article>, sqlx::Error> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (where_clause, bind_values) = build_article_filter(query);
    let sql = format!(
        "SELECT * FROM articles{} ORDER BY fetched_at DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut q = sqlx::query_as::<_, Article>(&sql);
    for val in &bind_values {
        match val {
            BindValue::Int(v) => q = q.bind(*v),
            BindValue::Bool(v) => q = q.bind(*v),
        }
    }
    q.bind(per_page as i64).bind(offset as i64).fetch_all(pool).await
}

pub async fn count_articles(pool: &SqlitePool, query: &ArticleQuery) -> Result<i64, sqlx::Error> {
    let (where_clause, bind_values) = build_article_filter(query);
    let sql = format!("SELECT COUNT(*) as count FROM articles{}", where_clause);

    let mut q = sqlx::query_as::<_, (i64,)>(&sql);
    for val in &bind_values {
        match val {
            BindValue::Int(v) => q = q.bind(*v),
            BindValue::Bool(v) => q = q.bind(*v),
        }
    }
    let count = q.fetch_one(pool).await?;
    Ok(count.0)
}
```

---

## Task 13: [P3 逻辑] revoke_token 先检查后操作

### 问题

`src/handlers/token.rs` 第 49-65 行，先执行 UPDATE 再检查存在性，逻辑顺序倒置。

### 修复方案

**文件: `src/handlers/token.rs`** — `revoke_token` 函数：

```rust
pub async fn revoke_token(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // 先检查存在性
    let exists = db::token::get_token_by_id(&state.pool, id).await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("Token with id {} not found", id)));
    }

    // 再执行 revoke
    db::token::revoke_token(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

---

## 执行顺序建议

按以下顺序执行，可最小化冲突和编译错误：

1. **Task 11** — 配置校验（无外部依赖，纯新增）
2. **Task 13** — revoke_token 顺序修复（纯逻辑调整）
3. **Task 4** — reqwest::Client 复用（纯内部重构）
4. **Task 1** — retry_count 硬编码（函数签名变更，影响面小）
5. **Task 5** — 输入验证（纯新增校验代码）
6. **Task 10** — health check 探活（路由结构调整）
7. **Task 12** — list_articles 重构（纯内部重构）
8. **Task 2** — hot_events UPSERT（涉及迁移，需先执行 SQL）
9. **Task 3** — Token 哈希存储（涉及迁移 + 依赖 + model 变更）
10. **Task 6** — keyword_mention 批量插入（新增函数 + 重构循环）
11. **Task 7** — Pusher 并发（新增依赖 + 重构循环）
12. **Task 8** — 历史统计批量查询（新增函数 + 重构循环）
13. **Task 9** — 优雅关闭（依赖 Task 7/8 完成后的代码结构）
