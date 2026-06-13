# filter-module (delta)

## MODIFIED Requirements

### Requirement: Unprocessed article batching

系统 SHALL 通过原子 claim 获取待处理文章：调用 `db::article::claim_pending_articles(pool, batch_size)` 将 `pending` 文章原子更新为 `processing` 并返回。完成匹配后，系统 SHALL 分别调用 `mark_articles_matched` 和 `mark_articles_skipped` 分类标记文章状态。原 `SELECT WHERE processed_at IS NULL` 方式 SHALL 被移除。

#### Scenario: 原子领取待处理文章

- **WHEN** `run_filter_once` 执行
- **THEN** 系统 SHALL 调用 `claim_pending_articles(pool, config.batch_size as i64)` 原子领取最多 `batch_size` 篇 pending 文章
- **THEN** 领取的文章状态 SHALL 被原子更新为 `processing`

#### Scenario: 无 pending 文章 — 提前返回

- **WHEN** `claim_pending_articles` 返回空列表
- **THEN** filter SHALL 返回 `false` 且不执行任何后续操作

#### Scenario: 无启用关键字 — 标记所有为 skipped

- **WHEN** 有 processing 文章但无启用关键字
- **THEN** filter SHALL 将所有 processing 文章标记为 `skipped`
- **THEN** filter SHALL 返回 `false`

### Requirement: Mark articles processed

系统 SHALL 在匹配完成后分类标记文章：命中关键字的文章标记为 `matched`，未命中的标记为 `skipped`。两种标记 SHALL 同步更新 `processed_at = datetime('now')`。原"所有文章统一标记 processed_at"的方式 SHALL 被移除。

#### Scenario: 命中文章标记为 matched

- **WHEN** 某篇文章至少命中一个关键字
- **THEN** 该文章的 `status` SHALL 更新为 `matched`
- **THEN** `processed_at` SHALL 设置为当前时间

#### Scenario: 未命中文章标记为 skipped

- **WHEN** 某篇文章未命中任何关键字
- **THEN** 该文章的 `status` SHALL 更新为 `skipped`
- **THEN** `processed_at` SHALL 设置为当前时间

#### Scenario: 批量标记分块执行

- **WHEN** matched 或 skipped 文章数量超过 100
- **THEN** 系统 SHALL 每 100 个 ID 一批执行 UPDATE
