# article-status-machine

## Purpose

为 Article 引入 `pending → processing → matched/skipped` 四态状态机，替代原有的 `processed_at IS NULL/NOT NULL` 二元判断。提供原子 claim 机制防止 Filter 并发重复处理同一批文章。

## Requirements

### Requirement: Article 状态字段

系统 SHALL 在 `articles` 表中维护 `status TEXT NOT NULL DEFAULT 'pending'` 列，取值限制为 `pending`、`processing`、`matched`、`skipped`。`processed_at` 字段 SHALL 保留，在状态变为 `matched` 或 `skipped` 时同步设置为当前时间。

#### Scenario: 新入库文章状态为 pending

- **WHEN** Parser 插入一篇新文章
- **THEN** `status` SHALL 为 `pending`
- **THEN** `processed_at` SHALL 为 NULL

#### Scenario: 存量数据迁移

- **WHEN** migration `20260613000001_article_status.sql` 应用于已有数据库
- **THEN** 所有 `processed_at IS NOT NULL` 的文章的 `status` SHALL 被更新为 `matched`
- **THEN** 所有 `processed_at IS NULL` 的文章的 `status` SHALL 保持默认值 `pending`

### Requirement: Filter 原子领取待处理文章

Filter 服务 SHALL 通过 `db::article::claim_pending_articles` 原子领取待处理文章：先用 `UPDATE articles SET status='processing' WHERE id IN (SELECT id FROM articles WHERE status='pending' ORDER BY fetched_at ASC LIMIT ?)` 标记，再 SELECT 已标记的 processing 文章返回。两次 `run_filter_once` 并发调用时，后执行的 UPDATE 影响 0 行从而不会重复处理。

#### Scenario: 正常领取 pending 文章

- **WHEN** `claim_pending_articles(pool, limit)` 被调用且存在 status='pending' 的文章
- **THEN** 函数 SHALL 将最多 `limit` 篇文章的状态原子更新为 `processing`
- **THEN** 函数 SHALL 返回这些 processing 文章（按 fetched_at 升序）

#### Scenario: 无 pending 文章时返回空

- **WHEN** `claim_pending_articles(pool, limit)` 被调用且不存在 status='pending' 的文章
- **THEN** UPDATE 影响 0 行
- **THEN** SELECT 返回空 Vec

#### Scenario: 并发调用时第二批无文章可领

- **WHEN** `claim_pending_articles` (A) 已原子领取所有 pending 文章
- **AND** `claim_pending_articles` (B) 在 (A) 提交后执行
- **THEN** (B) 的 UPDATE 影响 0 行
- **THEN** (B) 返回空 Vec

### Requirement: 批量标记 matched

系统 SHALL 提供 `db::article::mark_articles_matched(pool, ids: &[i64])` 函数，批量将指定文章的状态从 `processing` 更新为 `matched`，同时设置 `processed_at = datetime('now')`。更新 SHALL 以每 100 个 ID 一批分块执行。

#### Scenario: 批量标记 matched

- **WHEN** `mark_articles_matched(pool, &[1, 2, 3])` 被调用且这些文章当前状态为 `processing`
- **THEN** 三篇文章的 `status` SHALL 更新为 `matched`
- **THEN** `processed_at` SHALL 设置为当前时间
- **THEN** 只有 status='processing' 的文章被更新（WHERE 条件）

#### Scenario: 空 ID 列表无操作

- **WHEN** `mark_articles_matched(pool, &[])` 被调用
- **THEN** 无任何数据库操作执行

### Requirement: 批量标记 skipped

系统 SHALL 提供 `db::article::mark_articles_skipped(pool, ids: &[i64])` 函数，批量将指定文章的状态从 `processing` 更新为 `skipped`，同时设置 `processed_at = datetime('now')`。更新 SHALL 以每 100 个 ID 一批分块执行。

#### Scenario: 批量标记 skipped

- **WHEN** `mark_articles_skipped(pool, &[4, 5])` 被调用且这些文章当前状态为 `processing`
- **THEN** 两篇文章的 `status` SHALL 更新为 `skipped`
- **THEN** `processed_at` SHALL 设置为当前时间

#### Scenario: skipped 文章可被定期清理识别

- **WHEN** 一篇文章的 `status` 为 `skipped`
- **THEN** 该文章 SHALL 可被 `WHERE status='skipped'` 查询定位
- **THEN** 后续可据此实现存储清理策略
