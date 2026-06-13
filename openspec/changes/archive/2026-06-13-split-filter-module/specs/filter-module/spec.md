## MODIFIED Requirements

### Requirement: Aho-Corasick keyword matching

系统 SHALL 通过 `AhoCorasickMatcher`（实现 `KeywordMatcher` trait）构建 Aho-Corasick 自动机并对每篇文章的 `title + summary` 文本执行关键字匹配。`AhoCorasickMatcher`、`Automata` 结构体及相关构建逻辑 SHALL 位于 `src/services/filter/matching.rs`；`Automata` 和 `MatchResult` 结构体定义 SHALL 位于 `src/services/filter/types.rs`。

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

## ADDED Requirements

### Requirement: Filter 子模块组织

系统 SHALL 将 filter 模块拆分为 `src/services/filter.rs`（模块根）+ `src/services/filter/` 子目录，包含以下子模块：
- `types.rs` — `Automata`、`MatchResult` 结构体定义
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
