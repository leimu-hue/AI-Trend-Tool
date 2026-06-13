# database-schema (delta)

## MODIFIED Requirements

### Requirement: articles table

`articles` 表 SHALL 新增 `status TEXT NOT NULL DEFAULT 'pending'` 列。SHALL 创建 `idx_articles_status` 索引。原 `idx_articles_processed` 索引 SHALL 被移除。Migration SHALL 将已有 `processed_at IS NOT NULL` 的文章标记为 `matched`。

#### Scenario: 新文章使用默认 status

- **WHEN** 一篇文章被插入
- **THEN** `status` SHALL 默认为 `pending`
- **THEN** `idx_articles_status` SHALL 优化按状态查询

#### Scenario: 存量数据迁移

- **WHEN** migration `20260613000001_article_status.sql` 运行
- **THEN** `ALTER TABLE articles ADD COLUMN status TEXT NOT NULL DEFAULT 'pending'` SHALL 执行
- **THEN** `UPDATE articles SET status = 'matched' WHERE processed_at IS NOT NULL` SHALL 执行
- **THEN** `CREATE INDEX IF NOT EXISTS idx_articles_status ON articles(status)` SHALL 执行
- **THEN** `DROP INDEX IF EXISTS idx_articles_processed` SHALL 执行

### Requirement: push_records table

`push_records` 表 SHALL 新增 `last_error TEXT` 列，可为 NULL。`status` 列 SHALL 扩展支持 `dead` 值。Migration SHALL 将 `status='failed' AND next_retry_at IS NULL AND retry_count > 0` 的存量记录标记为 `dead`。

#### Scenario: push_records migration

- **WHEN** migration `20260613000002_push_record_enhancements.sql` 运行
- **THEN** `ALTER TABLE push_records ADD COLUMN last_error TEXT` SHALL 执行
- **THEN** `UPDATE push_records SET status = 'dead' WHERE status = 'failed' AND next_retry_at IS NULL AND retry_count > 0` SHALL 执行

#### Scenario: 新 push record 的 last_error 默认为 NULL

- **WHEN** 一条新的 push_record 被 Filter 创建
- **THEN** `last_error` SHALL 为 NULL
- **THEN** `status` SHALL 为 `pending`

### Requirement: Migration auto-runs on startup

`build.rs` 的 `rerun-if-changed` SHALL 包含新的 migration 文件 `docs/migrations/20260613000001_article_status.sql` 和 `docs/migrations/20260613000002_push_record_enhancements.sql`。

#### Scenario: 新增 migration 在启动时自动应用

- **WHEN** 新版本首次启动
- **THEN** `sqlx::migrate!()` SHALL 检测到两个新 migration 并应用
- **THEN** migration 执行顺序 SHALL 为 `20260613000001_article_status` → `20260613000002_push_record_enhancements`
