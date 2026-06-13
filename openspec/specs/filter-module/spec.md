# filter-module

## Purpose

Background service that matches unprocessed articles against enabled keywords using Aho-Corasick multi-pattern matching, accumulates hourly bucket counts, and detects trending hotspots via statistical burst detection (moving average + standard deviation).

## Requirements

### Requirement: Filter background scheduling

The system SHALL run the Filter module as an asynchronous background task that uses `tokio::select!` to listen for three signals: cancellation token (graceful shutdown), configurable interval tick (fallback polling), and `articles_ready_rx` channel (event-driven trigger from Parser). The system SHALL expose a shared `run_filter_once` function that returns `bool` indicating whether new push records were created.

#### Scenario: Filter runs on interval

- **WHEN** the Filter loop is running and the interval ticks
- **THEN** it SHALL call `run_filter_once` and if it returns `true`, send `PipelineEvent::NewData` via `push_ready_tx.try_send()`

#### Scenario: Filter runs on Parser notification

- **WHEN** the Filter loop receives `PipelineEvent::NewData` via `articles_rx.recv()`
- **THEN** it SHALL immediately call `run_filter_once` and if it returns `true`, send `PipelineEvent::NewData` via `push_ready_tx.try_send()`

#### Scenario: run_filter_once returns true when push records created

- **WHEN** `run_filter_once` executes and burst detection identifies a hotspot, creating one or more push records
- **THEN** the function SHALL return `true`

#### Scenario: run_filter_once returns false when no new push records

- **WHEN** `run_filter_once` executes and no hotspots are detected (or no unprocessed articles exist)
- **THEN** the function SHALL return `false`

#### Scenario: Filter shuts down gracefully

- **WHEN** the global `CancellationToken` is cancelled
- **THEN** the Filter SHALL log a shutdown message and break out of its loop

#### Scenario: Manual filter trigger calls same function

- **WHEN** the `POST /api/v1/trigger/filter` endpoint is called
- **THEN** it SHALL execute the same `run_filter_once` function that the background loop uses

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

### Requirement: Aho-Corasick keyword matching

系统 SHALL 通过 `AhoCorasickMatcher`（实现 `KeywordMatcher` trait）构建 Aho-Corasick 自动机并对每篇文章的 `title + summary` 文本执行关键字匹配。`AhoCorasickMatcher`、`Automata` 结构体及相关构建逻辑 SHALL 位于 `src/services/filter/matching.rs`；`Automata` 结构体定义 SHALL 位于 `src/services/filter/types.rs`。

#### Scenario: Build automaton from enabled keywords

- **WHEN** `AhoCorasickMatcher` 被构造
- **THEN** 它 SHALL 接收所有启用关键字的切片
- **AND** SHALL 构建一个区分大小写自动机和一个不区分大小写自动机

#### Scenario: Case-insensitive matching

- **WHEN** 关键字 `case_sensitive = false`
- **THEN** `AhoCorasickMatcher` SHALL 使用不区分大小写自动机匹配（`ascii_case_insensitive` 模式）

#### Scenario: Record keyword mentions in batch

- **WHEN** 关键字在文章中匹配
- **THEN** 系统 SHALL 收集 (keyword_id, article_id) 对用于批量插入

#### Scenario: No enabled keywords — mark all skipped

- **WHEN** 无启用关键字
- **THEN** 系统 SHALL 将所有 processing 文章标记为 skipped 并返回

### Requirement: Hourly bucket counting

The system SHALL accumulate matched keyword counts per current UTC hour bucket (format `YYYYMMDDHH`).

#### Scenario: Accumulate counts per keyword per hour

- **WHEN** multiple articles match the same keyword in the same filter run
- **THEN** the system SHALL sum the match counts per keyword for the current hour bucket

### Requirement: Statistical burst detection

The system SHALL detect hotspots using a moving average + standard deviation model over historical hourly counts.

#### Scenario: Calculate statistics from historical counts

- **WHEN** a keyword has at least `config.filter.min_history_hours` of historical data in `hot_events`
- **THEN** the system SHALL calculate mean and standard deviation from the past `config.filter.history_hours` hourly counts

#### Scenario: Hotspot detected — count exceeds threshold

- **WHEN** `current_hour_count > mean + (keyword.std_multiplier * stddev)` AND `current_hour_count >= keyword.min_hot_count`
- **THEN** the system SHALL create a `hot_events` record and SHALL insert `push_records` (status='pending') for every enabled `push_channel`

#### Scenario: No hotspot — count within normal range

- **WHEN** `current_hour_count <= threshold` OR `current_hour_count < min_hot_count`
- **THEN** the system SHALL still record the hourly count in `hot_events` but SHALL NOT create push records

#### Scenario: Insufficient historical data

- **WHEN** a keyword has fewer than `min_history_hours` of historical hot_events
- **THEN** the system SHALL record the current count but SHALL NOT perform burst detection

### Requirement: Keyword mention recording uses batch insert

The system SHALL collect all keyword-article mention pairs during the matching phase and insert them in a single batch operation using `INSERT OR IGNORE INTO keyword_mentions (keyword_id, article_id) VALUES (?, ?), (?, ?), ...` in chunks of 100.

#### Scenario: Batch insert keyword mentions

- **WHEN** multiple keywords match in multiple articles during one filter run
- **THEN** the system SHALL collect all (keyword_id, article_id) pairs into a Vec
- **THEN** SHALL insert them in chunks of 100 using a single INSERT with multiple VALUES
- **THEN** SHALL use `INSERT OR IGNORE` to preserve deduplication semantics

#### Scenario: No matches — skip batch insert

- **WHEN** no keywords match any article in the batch
- **THEN** the system SHALL NOT execute any INSERT for keyword mentions

### Requirement: Hot event upsert uses ON CONFLICT

The system SHALL upsert hot_event records using SQLite's `ON CONFLICT(keyword_id, hour_bucket) DO UPDATE` syntax to preserve the row's `id` and maintain foreign key integrity with `push_records`. The upsert and push_records insert SHALL be wrapped in a single database transaction.

#### Scenario: First hotspot for keyword in hour — insert

- **WHEN** a hotspot is detected and no existing row matches the (keyword_id, hour_bucket) pair
- **THEN** the system SHALL INSERT a new `hot_events` row within a transaction
- **THEN** push_records SHALL be created referencing the new row's id within the same transaction
- **THEN** the transaction SHALL COMMIT before marking articles as processed

#### Scenario: Repeat detection in same hour — update

- **WHEN** a hotspot is re-detected for the same (keyword_id, hour_bucket) pair
- **THEN** the system SHALL UPDATE the existing row's `count`, `mean_historical`, and `stddev_historical`
- **THEN** the row's `id` SHALL NOT change
- **THEN** existing push_records referencing this hot_event SHALL remain valid

#### Scenario: Transaction failure rolls back

- **WHEN** push_records insert fails within the transaction (e.g., FK violation)
- **THEN** the transaction SHALL ROLLBACK
- **THEN** hot_event changes SHALL be reverted
- **THEN** articles SHALL NOT be marked as processed

### Requirement: Historical statistics use batch query

The system SHALL load all keywords' hourly counts in a single query rather than per-keyword queries. Statistics (mean, stddev) SHALL be calculated in memory from the batched result.

#### Scenario: Batch load all hourly counts

- **WHEN** `run_filter_once` executes hotspot detection
- **THEN** the system SHALL query all (keyword_id, hour_bucket, total_count) in one SQL call
- **THEN** SHALL group results by keyword_id in memory
- **THEN** SHALL compute mean and stddev per keyword from the grouped data

#### Scenario: Keyword with no history skipped

- **WHEN** a keyword has zero rows in the batched result
- **THEN** the system SHALL skip burst detection for that keyword (insufficient data)

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

### Requirement: compute_stats 纯函数

系统 SHALL 提取 `compute_stats(counts: &[i32]) -> (f64, f64)` 纯函数，计算数组的均值和标准差。该函数 SHALL 有单元测试覆盖。

#### Scenario: 空数组
- **WHEN** `compute_stats(&[])` 被调用
- **THEN** 返回值 SHALL 为 `(0.0, 0.0)`

#### Scenario: 单元素
- **WHEN** `compute_stats(&[5])` 被调用
- **THEN** 均值 SHALL 为 5.0
- **THEN** 标准差 SHALL 为 0.0

#### Scenario: 多元素正常分布
- **WHEN** `compute_stats(&[2, 4, 4, 4, 5, 5, 7, 9])` 被调用
- **THEN** 均值 SHALL 约等于 5.0
- **THEN** 标准差 SHALL 约等于 2.0

### Requirement: Filter 子模块组织

系统 SHALL 将 filter 模块拆分为 `src/services/filter.rs`（模块根）+ `src/services/filter/` 子目录，包含以下子模块：
- `types.rs` — `Automata` 结构体定义
- `traits.rs` — `KeywordMatcher` trait 和 `ArticleMatches` 结构体定义
- `matching.rs` — `AhoCorasickMatcher` 实现
- `detection.rs` — `compute_stats`、`detect_and_push`、`upsert_hot_event_record`
- `validation.rs` — `claim_and_validate`

#### Scenario: filter.rs 仅含编排逻辑

- **WHEN** 查看 `filter.rs` 文件内容
- **THEN** 它 SHALL 只包含 `run_filter_once`、`start_filter_loop`、`#[cfg(test)]` 模块
- **THEN** 它 SHALL NOT 包含 struct 定义、trait 定义或子功能函数实现

#### Scenario: 公开 API 路径不变

- **WHEN** 外部代码通过 `crate::services::filter::run_filter_once` 调用
- **THEN** 该路径 SHALL 仍然有效
- **THEN** 函数签名 SHALL 保持不变
