-- ============================================================
-- API Token 表
-- ============================================================
CREATE TABLE IF NOT EXISTS api_tokens (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    token       TEXT    NOT NULL,
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

CREATE INDEX IF NOT EXISTS idx_articles_processed ON articles(processed_at);
CREATE INDEX IF NOT EXISTS idx_articles_source    ON articles(source_id);
CREATE INDEX IF NOT EXISTS idx_articles_fetched   ON articles(fetched_at);

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
-- 关键词命中明细表
-- ============================================================
CREATE TABLE IF NOT EXISTS keyword_mentions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword_id INTEGER NOT NULL REFERENCES keywords(id) ON DELETE CASCADE,
    article_id INTEGER NOT NULL REFERENCES articles(id)  ON DELETE CASCADE,
    matched_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_mentions_keyword ON keyword_mentions(keyword_id);
CREATE INDEX IF NOT EXISTS idx_mentions_article ON keyword_mentions(article_id);

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

CREATE INDEX IF NOT EXISTS idx_hot_events_keyword  ON hot_events(keyword_id);
CREATE INDEX IF NOT EXISTS idx_hot_events_bucket   ON hot_events(hour_bucket);

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

CREATE INDEX IF NOT EXISTS idx_push_records_status ON push_records(status);
