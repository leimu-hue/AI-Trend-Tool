# 步骤 02：数据库迁移 + 数据模型

## 前置依赖

- 步骤 01 已完成（项目骨架、Cargo.toml 依赖、db.rs 连接池）

## 目标

完成后拥有：
- 8 张表的 SQLite migration 文件
- 对应的 Rust 数据模型结构体（含 sqlx::FromRow）
- 数据库连接池可用，迁移自动运行

---

## 1. Migration 文件

使用 `sqlx migrate` 管理。创建以下迁移文件：

```bash
# 初始化迁移目录
sqlx migrate add init
```

在 `docs/migrations/<timestamp>_init.sql` 中写入以下全部 DDL：

```sql
-- ============================================================
-- API Token 表
-- ============================================================
CREATE TABLE IF NOT EXISTS api_tokens (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    token       TEXT    NOT NULL UNIQUE,
    last_used_at DATETIME,
    created_at  DATETIME NOT NULL DEFAULT (datetime('now')),
    expires_at  DATETIME,
    revoked     BOOLEAN NOT NULL DEFAULT 0
);

-- ============================================================
-- 数据源配置表
-- ============================================================
CREATE TABLE IF NOT EXISTS data_sources (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    type             TEXT    NOT NULL,                          -- rss, atom, json_feed...
    name             TEXT    NOT NULL,
    url              TEXT    NOT NULL,
    config           TEXT    NOT NULL DEFAULT '{}',             -- JSON 扩展配置
    enabled          BOOLEAN NOT NULL DEFAULT 1,
    interval_seconds INTEGER NOT NULL DEFAULT 300,
    last_fetched_at  DATETIME,
    created_at       DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at       DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================
-- 文章表
-- ============================================================
CREATE TABLE IF NOT EXISTS articles (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id    INTEGER NOT NULL REFERENCES data_sources(id) ON DELETE CASCADE,
    link         TEXT    NOT NULL UNIQUE,
    title        TEXT    NOT NULL DEFAULT '',
    summary      TEXT    NOT NULL DEFAULT '',
    content      TEXT    NOT NULL DEFAULT '',
    published_at DATETIME,
    fetched_at   DATETIME NOT NULL DEFAULT (datetime('now')),
    processed_at DATETIME
);

CREATE INDEX idx_articles_processed ON articles(processed_at);
CREATE INDEX idx_articles_source    ON articles(source_id);
CREATE INDEX idx_articles_fetched   ON articles(fetched_at);

-- ============================================================
-- 关键词表
-- ============================================================
CREATE TABLE IF NOT EXISTS keywords (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    word           TEXT    NOT NULL UNIQUE,
    case_sensitive BOOLEAN NOT NULL DEFAULT 0,
    enabled        BOOLEAN NOT NULL DEFAULT 1,
    std_multiplier REAL    NOT NULL DEFAULT 2.0,
    min_hot_count  INTEGER NOT NULL DEFAULT 3,
    created_at     DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================
-- 关键词命中明细表（可选，记录每次命中）
-- ============================================================
CREATE TABLE IF NOT EXISTS keyword_mentions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword_id INTEGER NOT NULL REFERENCES keywords(id) ON DELETE CASCADE,
    article_id INTEGER NOT NULL REFERENCES articles(id)  ON DELETE CASCADE,
    matched_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_mentions_keyword ON keyword_mentions(keyword_id);
CREATE INDEX idx_mentions_article ON keyword_mentions(article_id);

-- ============================================================
-- 热点事件表
-- ============================================================
CREATE TABLE IF NOT EXISTS hot_events (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword_id        INTEGER NOT NULL REFERENCES keywords(id) ON DELETE CASCADE,
    hour_bucket       TEXT    NOT NULL,                        -- 格式: YYYYMMDDHH
    count             INTEGER NOT NULL DEFAULT 0,
    mean_historical   REAL    NOT NULL DEFAULT 0.0,
    stddev_historical REAL    NOT NULL DEFAULT 0.0,
    created_at        DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_hot_events_keyword  ON hot_events(keyword_id);
CREATE INDEX idx_hot_events_bucket   ON hot_events(hour_bucket);

-- ============================================================
-- 推送渠道表
-- ============================================================
CREATE TABLE IF NOT EXISTS push_channels (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT    NOT NULL,
    channel_type TEXT    NOT NULL DEFAULT 'webhook',
    config       TEXT    NOT NULL DEFAULT '{}',               -- JSON: {"url": "..."}
    enabled      BOOLEAN NOT NULL DEFAULT 1
);

-- ============================================================
-- 推送记录表
-- ============================================================
CREATE TABLE IF NOT EXISTS push_records (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    hot_event_id INTEGER NOT NULL REFERENCES hot_events(id)    ON DELETE CASCADE,
    channel_id   INTEGER NOT NULL REFERENCES push_channels(id) ON DELETE CASCADE,
    status       TEXT    NOT NULL DEFAULT 'pending',           -- pending | success | failed
    retry_count  INTEGER NOT NULL DEFAULT 0,
    next_retry_at DATETIME,
    created_at   DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at   DATETIME NOT NULL DEFAULT (datetime('now')),
    UNIQUE(hot_event_id, channel_id)
);

CREATE INDEX idx_push_records_status ON push_records(status);
```

> 说明：SQLite 的 `datetime('now')` 用于设置默认时间。生产环境可考虑使用应用层注入时间。

---

## 2. Rust 数据模型 `src/models/`

### 2.1 `src/models.rs`

```rust
pub mod token;
pub mod source;
pub mod article;
pub mod keyword;
pub mod hot_event;
pub mod channel;
pub mod push_record;
```

### 2.2 `src/models/token.rs`

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct ApiToken {
    pub id: i64,
    pub name: String,
    pub token: String,
    pub last_used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub revoked: bool,
}

/// 列表响应中隐藏 token 明文
#[derive(Debug, Serialize)]
pub struct ApiTokenInfo {
    pub id: i64,
    pub name: String,
    pub last_used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub revoked: bool,
}

impl From<ApiToken> for ApiTokenInfo {
    fn from(t: ApiToken) -> Self {
        ApiTokenInfo {
            id: t.id,
            name: t.name,
            last_used_at: t.last_used_at,
            created_at: t.created_at,
            expires_at: t.expires_at,
            revoked: t.revoked,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub expires_at: Option<NaiveDateTime>,
}
```

### 2.3 `src/models/source.rs`

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DataSource {
    pub id: i64,
    #[serde(rename = "type")]
    pub source_type: String,
    pub name: String,
    pub url: String,
    pub config: String,             // JSON string
    pub enabled: bool,
    pub interval_seconds: i64,
    pub last_fetched_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateSourceRequest {
    #[serde(rename = "type")]
    pub source_type: String,
    pub name: String,
    pub url: String,
    pub interval_seconds: Option<i64>,
    pub config: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSourceRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub enabled: Option<bool>,
    pub interval_seconds: Option<i64>,
    pub config: Option<String>,
}
```

### 2.4 `src/models/article.rs`

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
    pub processed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub source_id: Option<i64>,
    pub processed: Option<bool>,
}
```

### 2.5 `src/models/keyword.rs`

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct Keyword {
    pub id: i64,
    pub word: String,
    pub case_sensitive: bool,
    pub enabled: bool,
    pub std_multiplier: f64,
    pub min_hot_count: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeywordRequest {
    pub word: String,
    pub case_sensitive: Option<bool>,
    pub std_multiplier: Option<f64>,
    pub min_hot_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeywordRequest {
    pub word: Option<String>,
    pub case_sensitive: Option<bool>,
    pub enabled: Option<bool>,
    pub std_multiplier: Option<f64>,
    pub min_hot_count: Option<i32>,
}
```

### 2.6 `src/models/hot_event.rs`

```rust
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct HotEvent {
    pub id: i64,
    pub keyword_id: i64,
    pub hour_bucket: String,
    pub count: i32,
    pub mean_historical: f64,
    pub stddev_historical: f64,
    pub created_at: NaiveDateTime,
}
```

### 2.7 `src/models/channel.rs`

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct PushChannel {
    pub id: i64,
    pub name: String,
    pub channel_type: String,
    pub config: String,             // JSON string
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub channel_type: Option<String>,
    pub config: String,             // JSON string: {"url": "https://..."}
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub config: Option<String>,
    pub enabled: Option<bool>,
}
```

### 2.8 `src/models/push_record.rs`

```rust
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct PushRecord {
    pub id: i64,
    pub hot_event_id: i64,
    pub channel_id: i64,
    pub status: String,
    pub retry_count: i32,
    pub next_retry_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

---

## 预期文件清单

| 文件路径 | 说明 |
|---------|------|
| `docs/migrations/<timestamp>_init.sql` | 全部建表 DDL |
| `src/models.rs` | 模块声明 |
| `src/models/token.rs` | ApiToken 模型 |
| `src/models/source.rs` | DataSource 模型 |
| `src/models/article.rs` | Article 模型 |
| `src/models/keyword.rs` | Keyword 模型 |
| `src/models/hot_event.rs` | HotEvent 模型 |
| `src/models/channel.rs` | PushChannel 模型 |
| `src/models/push_record.rs` | PushRecord 模型 |

---

## 验证节点

```bash
# 编译检查（模型结构体应与 migration SQL 匹配）
cargo check

# 启动服务器，确认迁移成功运行（日志中无错误）
cargo run -- --config config.toml all

# 用 sqlite3 检查表是否创建成功
sqlite3 ./docs/data/hotspot.db ".tables"
# 预期输出: api_tokens  articles  data_sources  hot_events  keyword_mentions  keywords  push_channels  push_records
```
