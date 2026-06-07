# 步骤 05：查询 API + 系统控制 + 后台模块（Parser/Filter/Pusher）

## 前置依赖

- 步骤 01-04 已完成（项目骨架、数据模型、认证、CRUD API）

## 目标

完成后拥有：
- 数据查询 API（4 个端点）
- 系统控制 API（3 个端点）
- Parser 后台模块（RSS 定时拉取）
- Filter 后台模块（关键词匹配 + 热点检测）
- Pusher 后台模块（webhook 推送 + 重试）

---

## 1. 数据查询 API

### 1.1 GET /api/v1/articles — 文章列表（分页 + 过滤）

**查询参数：**
| 参数 | 类型 | 默认 | 说明 |
|------|------|------|------|
| page | u32 | 1 | 页码 |
| per_page | u32 | 20 | 每页条数，最大 100 |
| source_id | i64 | - | 按数据源过滤 |
| processed | bool | - | true=已处理, false=未处理 |

**响应体（200）：**
```json
{
  "data": {
    "items": [...],
    "total": 150,
    "page": 1,
    "per_page": 20
  }
}
```

**实现 `src/handlers/query.rs`：**

```rust
use axum::{extract::{Query, State}, Json};
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use crate::models::article::{Article, ArticleQuery};
use crate::routes::AppState;

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

pub async fn list_articles(
    State(state): State<AppState>,
    Query(query): Query<ArticleQuery>,
) -> Result<Json<PaginatedResponse<Article>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // 动态构建查询
    let mut sql = "SELECT * FROM articles WHERE 1=1".to_string();
    let mut count_sql = "SELECT COUNT(*) FROM articles WHERE 1=1".to_string();

    if query.source_id.is_some() {
        sql.push_str(" AND source_id = ?");
        count_sql.push_str(" AND source_id = ?");
    }
    if let Some(processed) = query.processed {
        if processed {
            sql.push_str(" AND processed_at IS NOT NULL");
            count_sql.push_str(" AND processed_at IS NOT NULL");
        } else {
            sql.push_str(" AND processed_at IS NULL");
            count_sql.push_str(" AND processed_at IS NULL");
        }
    }

    sql.push_str(" ORDER BY fetched_at DESC LIMIT ? OFFSET ?");

    // 使用 sqlx::query_as 执行
    // 注意：动态 SQL 需根据参数数量绑定
    let articles: Vec<Article> = sqlx::query_as(&sql)
        // 按条件绑定参数（具体实现根据参数组合）
        .fetch_all(&state.pool)
        .await?;

    let total: i64 = sqlx::query_scalar(&count_sql)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(PaginatedResponse { items: articles, total, page, per_page }))
}
```

> **注意**：动态 SQL 绑定需要仔细处理参数顺序。可使用 `sqlx::QueryBuilder` 简化。

### 1.2 GET /api/v1/hotspots — 热点事件列表

**查询参数：**
| 参数 | 类型 | 默认 | 说明 |
|------|------|------|------|
| page | u32 | 1 | 页码 |
| per_page | u32 | 20 | 每页条数 |
| keyword_id | i64 | - | 按关键词过滤 |

**响应体（200）：**
```json
{
  "data": {
    "items": [
      {
        "id": 1,
        "keyword_id": 3,
        "hour_bucket": "2025010114",
        "count": 15,
        "mean_historical": 3.2,
        "stddev_historical": 1.5,
        "created_at": "2025-01-01T14:30:00"
      }
    ],
    "total": 42,
    "page": 1,
    "per_page": 20
  }
}
```

**实现：**

```rust
pub async fn list_hotspots(
    State(state): State<AppState>,
    Query(params): Query<HotspotQuery>,
) -> Result<Json<PaginatedResponse<HotEvent>>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (items, total) = if let Some(kid) = params.keyword_id {
        let items: Vec<HotEvent> = sqlx::query_as(
            "SELECT * FROM hot_events WHERE keyword_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(kid).bind(per_page).bind(offset)
        .fetch_all(&state.pool).await?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM hot_events WHERE keyword_id = ?"
        )
        .bind(kid).fetch_one(&state.pool).await?;

        (items, total)
    } else {
        let items: Vec<HotEvent> = sqlx::query_as(
            "SELECT * FROM hot_events ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(per_page).bind(offset)
        .fetch_all(&state.pool).await?;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hot_events")
            .fetch_one(&state.pool).await?;

        (items, total)
    };

    Ok(Json(PaginatedResponse { items, total, page, per_page }))
}

#[derive(Deserialize)]
pub struct HotspotQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub keyword_id: Option<i64>,
}
```

### 1.3 GET /api/v1/hotspots/{id}/push-records — 某热点的推送记录

```rust
pub async fn get_push_records(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<PushRecord>>, AppError> {
    let records: Vec<PushRecord> = sqlx::query_as(
        "SELECT * FROM push_records WHERE hot_event_id = ? ORDER BY created_at"
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(records))
}
```

### 1.4 GET /api/v1/trend/{keyword_id} — 近 N 小时计数曲线

**查询参数：**
| 参数 | 类型 | 默认 | 说明 |
|------|------|------|------|
| hours | u32 | 24 | 查看最近多少小时 |

**响应体（200）：**
```json
{
  "data": {
    "keyword_id": 3,
    "keyword": "GPT-5",
    "points": [
      { "hour_bucket": "2025010100", "count": 2 },
      { "hour_bucket": "2025010101", "count": 5 },
      ...
    ]
  }
}
```

**实现：**

```rust
use axum::extract::Path;

#[derive(Deserialize)]
pub struct TrendQuery {
    pub hours: Option<u32>,
}

#[derive(Serialize)]
pub struct TrendPoint {
    pub hour_bucket: String,
    pub count: i32,
}

#[derive(Serialize)]
pub struct TrendResponse {
    pub keyword_id: i64,
    pub keyword: String,
    pub points: Vec<TrendPoint>,
}

pub async fn get_trend(
    State(state): State<AppState>,
    Path(keyword_id): Path<i64>,
    Query(params): Query<TrendQuery>,
) -> Result<Json<TrendResponse>, AppError> {
    let hours = params.hours.unwrap_or(24);

    // 获取关键词信息
    let keyword: crate::models::keyword::Keyword = sqlx::query_as(
        "SELECT * FROM keywords WHERE id = ?"
    )
    .bind(keyword_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Keyword {} not found", keyword_id)))?;

    // 查询该关键词在 articles 中按小时桶的计数
    // 使用 articles.fetched_at 按小时分桶
    let rows: Vec<(String, i32)> = sqlx::query_as(
        "SELECT strftime('%Y%m%d%H', fetched_at) as hour_bucket, COUNT(*) as count
         FROM articles a
         JOIN keyword_mentions km ON km.article_id = a.id
         WHERE km.keyword_id = ?
           AND a.fetched_at >= datetime('now', ?)
         GROUP BY hour_bucket
         ORDER BY hour_bucket"
    )
    .bind(keyword_id)
    .bind(format!("-{} hours", hours))
    .fetch_all(&state.pool)
    .await?;

    let points: Vec<TrendPoint> = rows
        .into_iter()
        .map(|(bucket, count)| TrendPoint { hour_bucket: bucket, count })
        .collect();

    Ok(Json(TrendResponse {
        keyword_id,
        keyword: keyword.word,
        points,
    }))
}
```

---

## 2. 系统控制 API

### 2.1 POST /api/v1/trigger/filter — 手动运行过滤器

```rust
pub async fn trigger_filter(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    // 直接调用 filter 处理函数
    crate::services::filter::run_filter_once(&state.pool, &state.config.filter).await?;
    Ok(Json(json!({ "message": "Filter executed" })))
}
```

### 2.2 POST /api/v1/trigger/pusher — 手动运行推送器

```rust
pub async fn trigger_pusher(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    crate::services::pusher::run_pusher_once(&state.pool, &state.config.pusher).await?;
    Ok(Json(json!({ "message": "Pusher executed" })))
}
```

### 2.3 GET /health — 健康检查（免认证）

已在步骤 01 的 routes.rs 中实现。

---

## 3. Parser 模块 `src/services/parser.rs`

### 3.1 设计概要

- 后台异步任务，每 30 秒扫描一次所有启用的数据源
- 对每个数据源，检查 `NOW() - last_fetched_at >= interval_seconds`
- 满足条件的源，根据 `type` 字段选择对应解析器（当前仅 `rss`）
- 使用 `feed-rs` 库解析 RSS，提取文章写入 `articles` 表（忽略重复 link）
- 最大并发抓取数由 `config.parser.max_concurrent_fetches` 控制

### 3.2 Parser trait（可扩展设计）

```rust
use async_trait::async_trait;
use sqlx::SqlitePool;
use crate::models::source::DataSource;

/// 解析后的文章条目
pub struct ParsedArticle {
    pub link: String,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub published_at: Option<chrono::NaiveDateTime>,
}

#[async_trait]
pub trait Parser: Send + Sync {
    /// 抓取并解析数据源，返回文章列表
    async fn fetch_and_parse(&self, source: &DataSource) -> Result<Vec<ParsedArticle>, Box<dyn std::error::Error + Send + Sync>>;
}
```

### 3.3 RssParser 实现

```rust
use feed_rs::parser;
use reqwest::Client;
use crate::config::ParserConfig;

pub struct RssParser {
    client: Client,
}

impl RssParser {
    pub fn new(config: &ParserConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.default_timeout_seconds))
            .user_agent(&config.default_user_agent)
            .build()
            .expect("Failed to build HTTP client");

        RssParser { client }
    }
}

#[async_trait]
impl Parser for RssParser {
    async fn fetch_and_parse(&self, source: &DataSource) -> Result<Vec<ParsedArticle>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.get(&source.url).send().await?;
        let bytes = response.bytes().await?;

        let feed = parser::parse(&bytes[..])?;

        let articles: Vec<ParsedArticle> = feed.entries.into_iter().map(|entry| {
            let link = entry.links.first()
                .map(|l| l.href.clone())
                .unwrap_or_default();

            let title = entry.title
                .map(|t| t.content)
                .unwrap_or_default();

            let summary = entry.summary
                .map(|s| s.content)
                .unwrap_or_default();

            let published_at = entry.published
                .or(entry.updated)
                .map(|dt| dt.naive_utc());

            ParsedArticle {
                link,
                title,
                summary,
                content: String::new(),
                published_at,
            }
        }).collect();

        Ok(articles)
    }
}
```

### 3.4 后台调度循环

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{interval, Duration};
use sqlx::SqlitePool;
use crate::config::ParserConfig;
use crate::models::source::DataSource;

pub async fn start_parser_loop(
    pool: SqlitePool,
    config: ParserConfig,
) {
    let rss_parser = Arc::new(RssParser::new(&config));
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_fetches));
    let mut ticker = interval(Duration::from_secs(30));

    loop {
        ticker.tick().await;
        tracing::info!("Parser: scanning data sources...");

        // 查询需要抓取的源
        let sources: Vec<DataSource> = sqlx::query_as(
            "SELECT * FROM data_sources
             WHERE enabled = 1
               AND (last_fetched_at IS NULL
                    OR (julianday('now') - julianday(last_fetched_at)) * 86400 >= interval_seconds)"
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        for source in sources {
            let pool = pool.clone();
            let parser = rss_parser.clone();
            let permit = semaphore.clone();

            tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();

                tracing::info!("Parser: fetching source '{}' ({})", source.name, source.url);

                match parser.fetch_and_parse(&source).await {
                    Ok(articles) => {
                        let mut inserted = 0;
                        for article in articles {
                            // INSERT OR IGNORE 跳过重复 link
                            let result = sqlx::query(
                                "INSERT OR IGNORE INTO articles (source_id, link, title, summary, content, published_at)
                                 VALUES (?, ?, ?, ?, ?, ?)"
                            )
                            .bind(source.id)
                            .bind(&article.link)
                            .bind(&article.title)
                            .bind(&article.summary)
                            .bind(&article.content)
                            .bind(article.published_at)
                            .execute(&pool)
                            .await;

                            if let Ok(r) = result {
                                inserted += r.rows_affected();
                            }
                        }

                        // 更新 last_fetched_at
                        let _ = sqlx::query(
                            "UPDATE data_sources SET last_fetched_at = datetime('now') WHERE id = ?"
                        )
                        .bind(source.id)
                        .execute(&pool)
                        .await;

                        tracing::info!("Parser: source '{}' inserted {} new articles", source.name, inserted);
                    }
                    Err(e) => {
                        tracing::error!("Parser: failed to fetch source '{}': {:?}", source.name, e);
                    }
                }
            });
        }
    }
}
```

---

## 4. Filter 模块 `src/services/filter.rs`

### 4.1 设计概要

- 后台异步任务，每 `config.filter.interval_seconds`（默认 300 秒 = 5 分钟）运行一次
- 也可通过 API 手动触发

### 4.2 处理流程

```
1. 批量获取 processed_at IS NULL 的文章（LIMIT batch_size）
2. 加载所有 enabled=1 的关键词
3. 用 Aho-Corasick 自动机对每篇文章的 (title + summary) 进行多关键词匹配
4. 记录命中明细到 keyword_mentions 表
5. 按关键词 + 小时桶（fetched_at 的 UTC 小时）累加计数
6. 对每个有命中的关键词执行热点检测：
   a. 查询过去 history_hours（默认24）个完整小时的计数
   b. 计算均值 mean 和标准差 stddev
   c. 若 当前小时计数 > mean + std_multiplier * stddev 且 计数 >= min_hot_count
   d. 则创建 hot_events 记录
7. 对新热点，为每个 enabled 的 push_channels 插入 push_records (status='pending')
8. 更新已处理文章的 processed_at
```

### 4.3 完整实现

```rust
use aho_corasick::AhoCorasick;
use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use crate::config::FilterConfig;
use crate::models::article::Article;
use crate::models::keyword::Keyword;

/// 供后台循环和手动触发共用
pub async fn run_filter_once(pool: &SqlitePool, config: &FilterConfig) -> Result<(), sqlx::Error> {
    // 1. 获取未处理文章
    let articles: Vec<Article> = sqlx::query_as(
        "SELECT * FROM articles WHERE processed_at IS NULL ORDER BY id LIMIT ?"
    )
    .bind(config.batch_size)
    .fetch_all(pool)
    .await?;

    if articles.is_empty() {
        tracing::debug!("Filter: no unprocessed articles");
        return Ok(());
    }

    // 2. 加载关键词
    let keywords: Vec<Keyword> = sqlx::query_as(
        "SELECT * FROM keywords WHERE enabled = 1"
    )
    .fetch_all(pool)
    .await?;

    if keywords.is_empty() {
        // 没有关键词，直接标记所有文章为已处理
        let ids: Vec<i64> = articles.iter().map(|a| a.id).collect();
        // ... 批量更新 processed_at
        return Ok(());
    }

    // 3. 构建 Aho-Corasick 自动机
    let patterns: Vec<String> = keywords.iter().map(|k| {
        if k.case_sensitive {
            k.word.clone()
        } else {
            k.word.to_lowercase()
        }
    }).collect();

    let ac = AhoCorasick::builder()
        .ascii_case_insensitive(true)   // 对不区分大小写的关键词
        .build(&patterns)
        .expect("Failed to build Aho-Corasick automaton");

    // 用于累加每个关键词在当前小时的命中计数
    let current_hour = Utc::now().format("%Y%m%d%H").to_string();
    let mut keyword_hourly_counts: HashMap<i64, i32> = HashMap::new();
    let mut processed_ids: Vec<i64> = Vec::new();

    // 4. 匹配每篇文章
    for article in &articles {
        let text = if true {  // 假设大部分关键词不区分大小写
            format!("{} {}", article.title, article.summary).to_lowercase()
        } else {
            format!("{} {}", article.title, article.summary)
        };

        for mat in ac.find_iter(&text) {
            let keyword = &keywords[mat.pattern().as_usize()];

            // 记录命中明细
            let _ = sqlx::query(
                "INSERT INTO keyword_mentions (keyword_id, article_id) VALUES (?, ?)"
            )
            .bind(keyword.id)
            .bind(article.id)
            .execute(pool)
            .await;

            // 累加计数
            *keyword_hourly_counts.entry(keyword.id).or_insert(0) += 1;
        }

        processed_ids.push(article.id);
    }

    // 5. 热点检测
    for (keyword_id, current_count) in &keyword_hourly_counts {
        let keyword = keywords.iter().find(|k| k.id == *keyword_id).unwrap();

        // 获取历史小时桶计数（过去 history_hours 个完整小时）
        let history: Vec<(String, i32)> = sqlx::query_as(
            "SELECT hour_bucket, count FROM hot_events
             WHERE keyword_id = ?
             ORDER BY hour_bucket DESC
             LIMIT ?"
        )
        .bind(keyword_id)
        .bind(config.history_hours)
        .fetch_all(pool)
        .await?;

        // 需要至少 min_history_hours 的历史数据才做检测
        if history.len() < config.min_history_hours as usize {
            // 历史不足，直接写入当前计数但不告警
            let _ = sqlx::query(
                "INSERT OR REPLACE INTO hot_events (keyword_id, hour_bucket, count, mean_historical, stddev_historical)
                 VALUES (?, ?, ?, 0, 0)"
            )
            .bind(keyword_id)
            .bind(&current_hour)
            .bind(*current_count)
            .execute(pool)
            .await;
            continue;
        }

        // 计算均值和标准差
        let counts: Vec<f64> = history.iter().map(|(_, c)| *c as f64).collect();
        let n = counts.len() as f64;
        let mean = counts.iter().sum::<f64>() / n;
        let variance = counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();

        let threshold = mean + keyword.std_multiplier * stddev;

        // 记录当前小时桶
        let _ = sqlx::query(
            "INSERT OR REPLACE INTO hot_events (keyword_id, hour_bucket, count, mean_historical, stddev_historical)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(keyword_id)
        .bind(&current_hour)
        .bind(*current_count)
        .bind(mean)
        .bind(stddev)
        .execute(pool)
        .await;

        // 判断是否触发热点
        if (*current_count as f64) > threshold && *current_count >= keyword.min_hot_count {
            tracing::info!(
                "Filter: HOT EVENT detected for keyword '{}' (count={}, threshold={:.1})",
                keyword.word, current_count, threshold
            );

            // 获取最新创建的 hot_event
            let hot_event_id: i64 = sqlx::query_scalar(
                "SELECT id FROM hot_events WHERE keyword_id = ? AND hour_bucket = ?"
            )
            .bind(keyword_id)
            .bind(&current_hour)
            .fetch_one(pool)
            .await?;

            // 为所有启用的推送渠道创建推送记录
            let channels: Vec<(i64,)> = sqlx::query_as(
                "SELECT id FROM push_channels WHERE enabled = 1"
            )
            .fetch_all(pool)
            .await?;

            for (channel_id,) in channels {
                let _ = sqlx::query(
                    "INSERT OR IGNORE INTO push_records (hot_event_id, channel_id, status)
                     VALUES (?, ?, 'pending')"
                )
                .bind(hot_event_id)
                .bind(channel_id)
                .execute(pool)
                .await;
            }
        }
    }

    // 6. 标记文章为已处理
    if !processed_ids.is_empty() {
        // SQLite 不支持大批量 IN 参数，分批处理
        for chunk in processed_ids.chunks(100) {
            let placeholders: Vec<String> = chunk.iter().map(|_| "?".to_string()).collect();
            let sql = format!(
                "UPDATE articles SET processed_at = datetime('now') WHERE id IN ({})",
                placeholders.join(",")
            );
            let mut query = sqlx::query(&sql);
            for id in chunk {
                query = query.bind(id);
            }
            query.execute(pool).await?;
        }
    }

    tracing::info!("Filter: processed {} articles, {} keyword hits", articles.len(), keyword_hourly_counts.len());
    Ok(())
}

/// 后台循环
pub async fn start_filter_loop(pool: SqlitePool, config: FilterConfig) {
    let mut ticker = tokio::time::interval(
        tokio::time::Duration::from_secs(config.interval_seconds)
    );

    loop {
        ticker.tick().await;
        if let Err(e) = run_filter_once(&pool, &config).await {
            tracing::error!("Filter error: {:?}", e);
        }
    }
}
```

---

## 5. Pusher 模块 `src/services/pusher.rs`

### 5.1 设计概要

- 后台异步任务，每 `config.pusher.interval_seconds`（默认 10 秒）运行一次
- 查询 `push_records` 中 `status='pending'` 或满足重试条件的记录
- 根据 `channel_type` 发送 webhook（当前仅支持 webhook）
- 重试策略：指数退避 `retry_count * retry_base_seconds`

### 5.2 完整实现

```rust
use reqwest::Client;
use serde_json::json;
use sqlx::SqlitePool;
use crate::config::PusherConfig;
use crate::models::push_record::PushRecord;
use crate::models::channel::PushChannel;
use crate::models::hot_event::HotEvent;
use crate::models::keyword::Keyword;

/// 单次推送执行（供后台循环和手动触发共用）
pub async fn run_pusher_once(pool: &SqlitePool, config: &PusherConfig) -> Result<(), sqlx::Error> {
    let client = Client::new();

    // 查询待推送记录
    let records: Vec<PushRecord> = sqlx::query_as(
        "SELECT * FROM push_records
         WHERE status = 'pending'
            OR (status = 'failed'
                AND retry_count < ?
                AND (next_retry_at IS NULL OR next_retry_at <= datetime('now')))"
    )
    .bind(config.max_retries)
    .fetch_all(pool)
    .await?;

    if records.is_empty() {
        return Ok(());
    }

    for record in records {
        // 获取渠道信息
        let channel: PushChannel = match sqlx::query_as(
            "SELECT * FROM push_channels WHERE id = ?"
        )
        .bind(record.channel_id)
        .fetch_optional(pool)
        .await? {
            Some(c) => c,
            None => continue,
        };

        // 获取热点信息（用于构造推送内容）
        let hot_event: Option<HotEvent> = sqlx::query_as(
            "SELECT * FROM hot_events WHERE id = ?"
        )
        .bind(record.hot_event_id)
        .fetch_optional(pool)
        .await?;

        let keyword: Option<Keyword> = if let Some(ref event) = hot_event {
            sqlx::query_as("SELECT * FROM keywords WHERE id = ?")
                .bind(event.keyword_id)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        // 构造推送内容
        let keyword_word = keyword.as_ref().map(|k| k.word.as_str()).unwrap_or("unknown");
        let count = hot_event.as_ref().map(|e| e.count).unwrap_or(0);
        let payload = json!({
            "msgtype": "text",
            "text": {
                "content": format!(
                    "[热点告警] 关键词「{}」在当前小时内出现 {} 次，超过历史均值！",
                    keyword_word, count
                )
            }
        });

        // 解析渠道配置获取 webhook URL
        let channel_config: serde_json::Value = serde_json::from_str(&channel.config)
            .unwrap_or_default();
        let webhook_url = channel_config["url"].as_str().unwrap_or("");

        if webhook_url.is_empty() {
            tracing::error!("Pusher: channel '{}' has no webhook URL", channel.name);
            // 标记为失败
            let _ = sqlx::query(
                "UPDATE push_records SET status = 'failed', updated_at = datetime('now') WHERE id = ?"
            )
            .bind(record.id)
            .execute(pool)
            .await;
            continue;
        }

        // 发送 webhook
        match client.post(webhook_url).json(&payload).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Pusher: successfully pushed record {}", record.id);
                let _ = sqlx::query(
                    "UPDATE push_records SET status = 'success', updated_at = datetime('now') WHERE id = ?"
                )
                .bind(record.id)
                .execute(pool)
                .await;
            }
            Ok(resp) => {
                let status = resp.status();
                tracing::warn!("Pusher: webhook returned status {} for record {}", status, record.id);
                handle_push_failure(pool, &record, config).await;
            }
            Err(e) => {
                tracing::error!("Pusher: failed to send webhook for record {}: {:?}", record.id, e);
                handle_push_failure(pool, &record, config).await;
            }
        }
    }

    Ok(())
}

async fn handle_push_failure(pool: &SqlitePool, record: &PushRecord, config: &PusherConfig) {
    let new_retry_count = record.retry_count + 1;
    let next_retry_at = if new_retry_count < config.max_retries as i32 {
        Some(format!(
            "datetime('now', '+{} seconds')",
            new_retry_count as u64 * config.retry_base_seconds
        ))
    } else {
        None
    };

    let _ = sqlx::query(
        "UPDATE push_records
         SET status = 'failed', retry_count = ?, next_retry_at = ?, updated_at = datetime('now')
         WHERE id = ?"
    )
    .bind(new_retry_count)
    .bind(
        chrono::Utc::now().naive_utc()
        + chrono::Duration::seconds((new_retry_count as i64) * (config.retry_base_seconds as i64))
    )
    .bind(record.id)
    .execute(pool)
    .await;
}

/// 后台循环
pub async fn start_pusher_loop(pool: SqlitePool, config: PusherConfig) {
    let mut ticker = tokio::time::interval(
        tokio::time::Duration::from_secs(config.interval_seconds)
    );

    loop {
        ticker.tick().await;
        if let Err(e) = run_pusher_once(&pool, &config).await {
            tracing::error!("Pusher error: {:?}", e);
        }
    }
}
```

---

## 6. 后台模块注册 `src/services/mod.rs`

```rust
pub mod parser;
pub mod filter;
pub mod pusher;
```

## 7. 在 main.rs 中启动后台任务

```rust
// 在 axum::serve 之前，使用 tokio::spawn 启动后台任务
use crate::services::{parser, filter, pusher};

// 根据启动模式决定运行哪些模块
match cli.mode.as_str() {
    "all" | "api" => {
        // API 模式下也启动后台任务
        tokio::spawn(parser::start_parser_loop(pool.clone(), config.parser.clone()));
        tokio::spawn(filter::start_filter_loop(pool.clone(), config.filter.clone()));
        tokio::spawn(pusher::start_pusher_loop(pool.clone(), config.pusher.clone()));
    }
    "parser" => {
        tokio::spawn(parser::start_parser_loop(pool.clone(), config.parser.clone()));
        // 等待永远...
        tokio::signal::ctrl_c().await?;
        return Ok(());
    }
    "filter" => {
        tokio::spawn(filter::start_filter_loop(pool.clone(), config.filter.clone()));
        tokio::signal::ctrl_c().await?;
        return Ok(());
    }
    "pusher" => {
        tokio::spawn(pusher::start_pusher_loop(pool.clone(), config.pusher.clone()));
        tokio::signal::ctrl_c().await?;
        return Ok(());
    }
    _ => {
        eprintln!("Unknown mode: {}. Use: all, api, parser, filter, pusher", cli.mode);
        return Ok(());
    }
}
```

---

## 8. 路由注册更新

在 `src/routes.rs` 的 `api_routes()` 中添加：

```rust
// 数据查询 API
.route("/articles", get(handlers::query::list_articles))
.route("/hotspots", get(handlers::query::list_hotspots))
.route("/hotspots/{id}/push-records", get(handlers::query::get_push_records))
.route("/trend/{keyword_id}", get(handlers::query::get_trend))
// 系统控制
.route("/trigger/filter", post(handlers::query::trigger_filter))
.route("/trigger/pusher", post(handlers::query::trigger_pusher))
```

更新 `src/handlers/mod.rs`：

```rust
pub mod token;
pub mod source;
pub mod keyword;
pub mod channel;
pub mod query;
```

---

## 预期文件清单

| 文件路径 | 说明 |
|---------|------|
| `src/services/mod.rs` | 后台模块声明 |
| `src/services/parser.rs` | RSS 解析器 + 调度循环 |
| `src/services/filter.rs` | 关键词匹配 + 热点检测 |
| `src/services/pusher.rs` | Webhook 推送 + 重试策略 |
| `src/handlers/query.rs` | 数据查询 + 系统控制 handler |
| `src/handlers/mod.rs` | 更新：添加 query 模块 |
| `src/routes.rs` | 更新：注册查询和控制路由 |
| `src/main.rs` | 更新：启动后台任务 |
| `Cargo.toml` | 可能需添加 async-trait 依赖 |

---

## 验证节点

```bash
# 编译检查
cargo check

# 启动服务器
cargo run -- --config config.toml all
# 观察日志：Parser/Filter/Pusher 循环启动

# 测试查询 API
curl http://localhost:8080/api/v1/articles -H "Authorization: Bearer $TOKEN"
curl http://localhost:8080/api/v1/hotspots -H "Authorization: Bearer $TOKEN"

# 手动触发过滤器
curl -X POST http://localhost:8080/api/v1/trigger/filter \
  -H "Authorization: Bearer $TOKEN"

# 手动触发推送器
curl -X POST http://localhost:8080/api/v1/trigger/pusher \
  -H "Authorization: Bearer $TOKEN"

# 测试趋势 API（假设已有关键词 ID=1）
curl "http://localhost:8080/api/v1/trend/1?hours=12" \
  -H "Authorization: Bearer $TOKEN"
```
