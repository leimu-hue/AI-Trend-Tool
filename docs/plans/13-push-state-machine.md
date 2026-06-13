# Push State Machine — 完整状态管理设计方案

> **编号**: 13  
> **目标**: 为热点监控推送管道引入双层状态机，安全处理失败重试，避免重复推送  
> **影响范围**: `articles` 表、`push_records` 表、Filter 服务、Pusher 服务、DB 层

---

## 1. 现状分析

### 1.1 当前架构

```
Parser → articles (processed_at IS NULL) → Filter → hot_events + push_records → Pusher → webhook
```

| 组件 | 状态模型 | 并发控制 | 重试机制 |
|------|---------|---------|---------|
| **Articles** | 二元: `processed_at IS NULL / NOT NULL` | **无** (可能被并发处理) | 无 |
| **Push Records** | `pending → processing → success/failed` | 原子 claim (`UPDATE WHERE status='pending'`) | 线性退避 (`n * base_seconds`) |

### 1.2 存在的问题

1. **Articles 无状态区分**: 无法区分 "匹配到关键字" 和 "未匹配" 的文章，也无法识别处理中途失败的文章
2. **Filter 无并发保护**: 两次 `run_filter_once` 可能同时处理同一批文章（无原子 claim）
3. **Pusher 线性退避**: `delay = retry_count * retry_base_seconds`，非指数退避
4. **无 dead letter 状态**: 耗尽重试次数的记录以 `failed` + `next_retry_at = NULL` 存放，语义不清
5. **Pusher 无陈旧恢复**: 如果 pusher 进程在 `processing` 阶段崩溃，记录永久卡死
6. **无错误日志字段**: 推送失败原因无处记录

---

## 2. 双层状态机设计

系统运行在两个层级，保持独立：

### 2.1 Level 1: Article 状态机（新增）

```
发现文章 → pending
              ↓ (Filter 原子 claim)
         processing
              ↓
         匹配关键字
           ↙     ↘
        命中      未命中
          ↓         ↓
       matched    skipped (终态)
```

| 状态 | 含义 | 进入条件 |
|------|------|---------|
| `pending` | 新入库文章，等待 Filter 处理 | Parser 插入时 |
| `processing` | Filter 已 claim，正在匹配关键字 | Filter 原子更新 (防并发) |
| `matched` | 至少命中一个关键字，已生成 hot_event | 匹配完成后 |
| `skipped` | 未命中任何关键字，无需推送 | 匹配完成后 |

**为什么保留 matched/skipped 而不合并为 processed？**
- `matched` 可用于统计命中率和追溯
- `skipped` 的文章可定期清理，节省存储
- 便于前端展示匹配概况

### 2.2 Level 2: Push Record 状态机（增强现有）

```
发现热点 → pending
              ↓ (Pusher 原子 claim)
         processing
              ↓
         发送 webhook
           ↙     ↘
       success    failed (可重试)
        (终态)       ↓
                 retry_count < max?
                  ↙        ↘
                 是          否
                  ↓           ↓
            (下次重试)     dead (终态)
                         → processing
```

| 状态 | 含义 | 变更 |
|------|------|------|
| `pending` | 等待推送 | 已有 |
| `processing` | 已 claim，正在发送 webhook | 已有 |
| `success` | 推送成功 | 已有 |
| `failed` | 推送失败，可重试 (`retry_count < max_retries`) | 已有，增加 `last_error` |
| `dead` | 重试次数耗尽，需人工介入 | **新增** |

---

## 3. 数据库变更

### 3.1 Migration: `20260613000001_article_status.sql`

```sql
-- 为 articles 表添加状态字段，替代 processed_at 二元判断
ALTER TABLE articles ADD COLUMN status TEXT NOT NULL DEFAULT 'pending';

-- 迁移现有数据: processed_at IS NOT NULL → 'matched' (保守估计)
UPDATE articles SET status = 'matched' WHERE processed_at IS NOT NULL;

-- 添加状态索引
CREATE INDEX IF NOT EXISTS idx_articles_status ON articles(status);

-- 删除旧索引 (不再需要)
DROP INDEX IF EXISTS idx_articles_processed;
```

### 3.2 Migration: `20260613000002_push_record_enhancements.sql`

```sql
-- 添加 last_error 字段用于记录失败原因
ALTER TABLE push_records ADD COLUMN last_error TEXT;

-- 将已耗尽重试的记录标记为 dead
UPDATE push_records SET status = 'dead'
  WHERE status = 'failed' AND next_retry_at IS NULL AND retry_count > 0;
```

---

## 4. 模型层变更

### 4.1 Article Model (`src/models/article.rs`)

```rust
#[derive(Debug, FromRow, Serialize)]
pub struct Article {
    pub id: i64,
    pub source_id: i64,
    pub link: String,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub published_at: Option<NaiveDateTime>,
    pub fetched_at: NaiveDateTime,
    pub processed_at: Option<NaiveDateTime>,  // 保留向后兼容
    pub status: String,                        // pending | processing | matched | skipped
}

#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub source_id: Option<i64>,
    pub status: Option<String>,  // 替代 processed: Option<bool>
}
```

### 4.2 PushRecord Model (`src/models/push_record.rs`)

```rust
#[derive(Debug, FromRow, Serialize)]
pub struct PushRecord {
    pub id: i64,
    pub hot_event_id: i64,
    pub channel_id: i64,
    pub status: String,              // pending | processing | success | failed | dead
    pub retry_count: i32,
    pub next_retry_at: Option<NaiveDateTime>,
    pub last_error: Option<String>,  // 新增
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

---

## 5. DB 层变更

### 5.1 Article DB (`src/db/article.rs`)

**核心变更: 原子 claim 替代无保护的 SELECT**

```rust
/// 原子 claim: 将 pending 文章标记为 processing 并返回
/// 防止多个 Filter 实例同时处理同一批文章
pub async fn claim_pending_articles(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    // 1. 原子更新 status: pending → processing
    sqlx::query(
        "UPDATE articles SET status = 'processing' \
         WHERE id IN ( \
             SELECT id FROM articles \
             WHERE status = 'pending' \
             ORDER BY fetched_at ASC \
             LIMIT ? \
         )"
    )
    .bind(limit)
    .execute(pool)
    .await?;

    // 2. 返回已 claim 的文章
    sqlx::query_as::<_, Article>(
        "SELECT * FROM articles WHERE status = 'processing' \
         ORDER BY fetched_at ASC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// 批量标记为 matched
pub async fn mark_articles_matched(pool: &SqlitePool, ids: &[i64]) -> Result<(), sqlx::Error> {
    for chunk in ids.chunks(100) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "UPDATE articles SET status = 'matched', processed_at = datetime('now') \
             WHERE id IN ({}) AND status = 'processing'",
            placeholders
        );
        let mut query = sqlx::query(&sql);
        for id in chunk {
            query = query.bind(*id);
        }
        query.execute(pool).await?;
    }
    Ok(())
}

/// 批量标记为 skipped
pub async fn mark_articles_skipped(pool: &SqlitePool, ids: &[i64]) -> Result<(), sqlx::Error> {
    for chunk in ids.chunks(100) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "UPDATE articles SET status = 'skipped', processed_at = datetime('now') \
             WHERE id IN ({}) AND status = 'processing'",
            placeholders
        );
        let mut query = sqlx::query(&sql);
        for id in chunk {
            query = query.bind(*id);
        }
        query.execute(pool).await?;
    }
    Ok(())
}
```

**查询过滤器更新:**

```rust
fn build_article_filter(query: &ArticleQuery) -> (String, Vec<String>) {
    let mut conditions = vec![];
    let mut params: Vec<String> = vec![];
    if let Some(source_id) = query.source_id {
        conditions.push("source_id = ?".to_string());
        params.push(source_id.to_string());
    }
    if let Some(ref status) = query.status {
        conditions.push("status = ?".to_string());
        params.push(status.clone());
    }
    // ...
}
```

### 5.2 PushRecord DB (`src/db/push_record.rs`)

**新增: 陈旧 processing 恢复**

```rust
/// 将卡在 processing 超过 timeout_minutes 的记录重置为 pending
pub async fn recover_stale_processing_records(
    pool: &SqlitePool,
    timeout_minutes: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE push_records SET status = 'pending', updated_at = datetime('now') \
         WHERE status = 'processing' \
         AND updated_at < datetime('now', ? || ' minutes')"
    )
    .bind(format!("-{}", timeout_minutes))
    .execute(pool)
    .await?;
    if result.rows_affected() > 0 {
        tracing::warn!(
            "Pusher: recovered {} stale 'processing' records",
            result.rows_affected()
        );
    }
    Ok(result.rows_affected())
}
```

**更新: update_push_status 增加 last_error**

```rust
pub async fn update_push_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
    retry_count: i32,
    next_retry_at: Option<NaiveDateTime>,
    last_error: Option<&str>,     // 新增
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE push_records \
         SET status = ?, retry_count = ?, next_retry_at = ?, \
             last_error = ?, updated_at = datetime('now') \
         WHERE id = ?"
    )
    .bind(status)
    .bind(retry_count)
    .bind(next_retry_at)
    .bind(last_error)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
```

**更新: update_push_status_optimistic 增加 last_error**

```rust
pub async fn update_push_status_optimistic(
    pool: &SqlitePool,
    id: i64,
    expected_status: &str,
    new_status: &str,
    retry_count: i32,
    next_retry_at: Option<NaiveDateTime>,
    last_error: Option<&str>,     // 新增
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE push_records \
         SET status = ?, retry_count = ?, next_retry_at = ?, \
             last_error = ?, updated_at = datetime('now') \
         WHERE id = ? AND status = ?"
    )
    .bind(new_status)
    .bind(retry_count)
    .bind(next_retry_at)
    .bind(last_error)
    .bind(id)
    .bind(expected_status)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
```

**更新: PushRecordWithChannel 增加 last_error**

```rust
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct PushRecordWithChannel {
    pub id: i64,
    pub hot_event_id: i64,
    pub channel_id: i64,
    pub channel_name: String,
    pub status: String,
    pub retry_count: i32,
    pub next_retry_at: Option<NaiveDateTime>,
    pub last_error: Option<String>,    // 新增
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

对应 SQL 查询也需增加 `pr.last_error`。

---

## 6. 服务层变更

### 6.1 Filter 服务 (`src/services/filter.rs`)

**核心变更流程:**

```rust
pub async fn run_filter_once(pool: &SqlitePool, config: &FilterConfig) -> bool {
    // 1. 原子 claim pending 文章 (替代 get_unprocessed_articles)
    let articles = db::article::claim_pending_articles(pool, config.batch_size as i64).await?;
    if articles.is_empty() { return false; }

    // 2. 加载关键字 + 构建 Aho-Corasick (不变)
    let keywords = db::keyword::list_enabled_keywords(pool).await?;

    // 3. 匹配文章
    let mut matched_ids: Vec<i64> = Vec::new();
    let mut skipped_ids: Vec<i64> = Vec::new();
    let mut mentions: Vec<(i64, i64)> = Vec::new();

    for article in &articles {
        let text = format!("{} {}", article.title, article.summary);
        let mut article_matched = false;

        // ... Aho-Corasick 匹配逻辑 ...
        // 对每个命中: mentions.push((kw_id, article.id)); article_matched = true;

        if article_matched {
            matched_ids.push(article.id);
        } else {
            skipped_ids.push(article.id);
        }
    }

    // 4. Burst detection + hot_event + push_records (不变，但仅针对 matched 文章)
    // ... 现有逻辑 ...

    // 5. Commit transaction (不变)
    tx.commit().await?;

    // 6. 分别标记 matched 和 skipped
    if !matched_ids.is_empty() {
        db::article::mark_articles_matched(pool, &matched_ids).await?;
    }
    if !skipped_ids.is_empty() {
        db::article::mark_articles_skipped(pool, &skipped_ids).await?;
    }

    created_push
}
```

### 6.2 Pusher 服务 (`src/services/pusher.rs`)

**6.2.1 指数退避 + dead 状态**

```rust
async fn mark_failed(
    pool: &SqlitePool,
    config: &PusherConfig,
    record_id: i64,
    current_retry_count: i32,
    error_msg: &str,           // 新增: 错误信息
) {
    let new_retry_count = current_retry_count + 1;

    if new_retry_count >= config.max_retries as i32 {
        tracing::warn!(
            "Pusher: record {} reached max retries ({}), marking as dead",
            record_id,
            config.max_retries
        );
        // 终态: dead
        let _ = db::push_record::update_push_status(
            pool, record_id, "dead", new_retry_count, None, Some(error_msg),
        ).await;
    } else {
        // 指数退避: base * 2^(n-1), 如 base=60s → 60s, 120s, 240s, 480s
        let base = config.retry_base_seconds as i64;
        let delay = base * 2_i64.pow((new_retry_count as u32).saturating_sub(1));
        // 上限: retry_max_seconds
        let delay = delay.min(config.retry_max_seconds as i64);
        let at = Utc::now().naive_utc() + chrono::Duration::seconds(delay);

        let _ = db::push_record::update_push_status(
            pool, record_id, "failed", new_retry_count, Some(at), Some(error_msg),
        ).await;
    }
}
```

**6.2.2 陈旧处理恢复**

在 `run_pusher_once` 开头添加恢复步骤:

```rust
pub async fn run_pusher_once(pool: &SqlitePool, config: &PusherConfig, client: &reqwest::Client) {
    // 0. 恢复卡死的 processing 记录 (> 10 分钟)
    if let Err(e) = db::push_record::recover_stale_processing_records(pool, 10).await {
        tracing::error!("Pusher: failed to recover stale records: {}", e);
    }

    // 1. 原子 claim pending 记录 (不变)
    // 2. 获取 processing + retry_due 记录 (不变)
    // 3. 并发推送 (不变)
}
```

**6.2.3 成功时记录 last_error = None**

```rust
// 推送成功
let updated = db::push_record::update_push_status_optimistic(
    pool,
    record.id,
    &record.status,
    "success",
    record.retry_count,
    None,
    None,  // last_error = None
).await;
```

**6.2.4 失败时传递错误信息**

```rust
// HTTP 错误
let error_msg = format!("HTTP {}", response.status());
mark_failed(pool, config, record.id, record.retry_count, &error_msg).await;

// 网络错误
let error_msg = format!("Network error: {}", e);
mark_failed(pool, config, record.id, record.retry_count, &error_msg).await;

// Channel 配置错误
let error_msg = format!("Channel {} has no valid webhook URL", channel.id);
mark_failed(pool, config, record.id, record.retry_count, &error_msg).await;
```

---

## 7. 配置变更

### 7.1 PusherConfig (`src/config.rs`)

```rust
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PusherConfig {
    pub interval_seconds: u64,
    pub max_retries: u32,
    pub retry_base_seconds: u64,
    #[serde(default = "default_retry_max_seconds")]
    pub retry_max_seconds: u64,    // 新增: 退避上限
    #[serde(default = "default_stale_timeout_minutes")]
    pub stale_timeout_minutes: u64, // 新增: 陈旧 processing 超时
}

fn default_retry_max_seconds() -> u64 { 3600 }        // 1 小时
fn default_stale_timeout_minutes() -> u64 { 10 }       // 10 分钟
```

### 7.2 config.toml

```toml
[pusher]
interval_seconds = 10
max_retries = 5
retry_base_seconds = 60
retry_max_seconds = 3600       # 退避上限 1 小时
stale_timeout_minutes = 10     # processing 超时 10 分钟
```

### 7.3 退避时间表示例

| 重试次数 | 指数退避 (base=60s) | 加上限 (max=3600s) |
|---------|--------------------|--------------------|
| 1 | 60s (1 min) | 60s |
| 2 | 120s (2 min) | 120s |
| 3 | 240s (4 min) | 240s |
| 4 | 480s (8 min) | 480s |
| 5 | 960s (16 min) | 960s |
| 6 | 1920s (32 min) | 1920s |
| 7 | 3840s (64 min) | **3600s (capped)** |
| 8 | → **dead** | → **dead** |

---

## 8. Handler/API 变更

### 8.1 Query Handler (`src/handlers/query.rs`)

更新文章查询 API，`processed` 参数改为 `status`:

```rust
// Before: ?processed=true
// After:  ?status=matched
```

### 8.2 PushRecordWithChannel 查询 SQL

```sql
SELECT pr.id, pr.hot_event_id, pr.channel_id, pc.name as channel_name,
       pr.status, pr.retry_count, pr.next_retry_at, pr.last_error,
       pr.created_at, pr.updated_at
FROM push_records pr
JOIN push_channels pc ON pc.id = pr.channel_id
WHERE pr.hot_event_id = ?
ORDER BY pr.channel_id
```

---

## 9. 并发安全分析

| 场景 | 方案 | 机制 |
|------|------|------|
| 多个 Filter 实例同时 claim 文章 | `UPDATE ... WHERE status = 'pending'` 原子操作 | SQLite 写锁保证串行 |
| 多个 Pusher 实例同时 claim push_record | 现有 `claim_pending_records` 原子 UPDATE | 同上 |
| Pusher 崩溃后 processing 记录卡死 | `recover_stale_processing_records` 定时恢复 | 超时重置为 pending |
| 推送结果更新时记录已被其他进程修改 | `update_push_status_optimistic` (WHERE status = expected) | 乐观锁 |

---

## 10. 实施步骤

| # | 任务 | 涉及文件 |
|---|------|---------|
| 1 | 创建 migration `20260613000001_article_status.sql` | `docs/migrations/` |
| 2 | 创建 migration `20260613000002_push_record_enhancements.sql` | `docs/migrations/` |
| 3 | 更新 Article model (添加 `status` 字段) | `src/models/article.rs` |
| 4 | 更新 PushRecord model (添加 `last_error` 字段) | `src/models/push_record.rs` |
| 5 | 更新 article DB 层 (claim + mark_matched + mark_skipped) | `src/db/article.rs` |
| 6 | 更新 push_record DB 层 (recover_stale + last_error) | `src/db/push_record.rs` |
| 7 | 更新 Filter 服务 (原子 claim + 分类标记) | `src/services/filter.rs` |
| 8 | 更新 Pusher 服务 (指数退避 + dead + 错误记录) | `src/services/pusher.rs` |
| 9 | 更新 PusherConfig (retry_max_seconds + stale_timeout_minutes) | `src/config.rs`, `config.toml` |
| 10 | 更新 handlers/query 和 article 列表 API | `src/handlers/query.rs` |
| 11 | 更新 PushRecordWithChannel 查询 SQL | `src/db/push_record.rs` |
| 12 | 更新单元测试 | `src/services/filter.rs` |
| 13 | `cargo build` + `cargo test` 验证 | — |

---

## 11. 向后兼容注意事项

- `processed_at` 字段保留，与 `status` 同步更新（matched/skipped 时设置 `processed_at`）
- 前端查询 API 暂时可同时支持 `processed` 和 `status` 参数，后续废弃 `processed`
- `build.rs` 的 `rerun-if-changed` 需包含新 migration 文件
