## Purpose

Define the `KeywordMatcher` trait — an abstract interface for keyword matching strategies. Enables swapping the default Aho-Corasick implementation with future alternatives (e.g., AI-based semantic matching) without changing the filter orchestration logic.

## Requirements

### Requirement: KeywordMatcher trait 定义

系统 SHALL 定义 `KeywordMatcher` trait 作为关键字匹配的抽象接口，位于 `src/services/filter/traits.rs`。该 trait SHALL 声明 `match_batch` 方法，接受文章切片，返回每篇文章的关键字匹配结果。

#### Scenario: trait 声明 Send + Sync

- **WHEN** `KeywordMatcher` trait 被定义
- **THEN** 它 SHALL 约束 `Send + Sync`，允许在异步上下文中安全使用

#### Scenario: match_batch 方法签名

- **WHEN** 调用方通过 `&dyn KeywordMatcher` 调用 `match_batch`
- **THEN** 方法 SHALL 接受 `&[Article]` 参数
- **THEN** 方法 SHALL 返回 `Vec<ArticleMatches>`，包含每篇文章的匹配关键字 ID 列表

### Requirement: ArticleMatches 返回结构

系统 SHALL 定义 `ArticleMatches` 结构体，表示单篇文章的匹配结果。

#### Scenario: ArticleMatches 字段

- **WHEN** 匹配器完成批处理
- **THEN** `ArticleMatches` SHALL 包含 `article_id: i64` 字段
- **THEN** `ArticleMatches` SHALL 包含 `matched_keyword_ids: Vec<i64>` 字段

### Requirement: AhoCorasickMatcher 实现 KeywordMatcher

系统 SHALL 提供默认实现 `AhoCorasickMatcher`，使用当前 Aho-Corasick 多模式匹配算法实现 `KeywordMatcher` trait。

#### Scenario: 不区分大小写匹配

- **WHEN** 关键字 `case_sensitive = false`
- **THEN** `AhoCorasickMatcher` SHALL 使用 `ascii_case_insensitive` 模式匹配

#### Scenario: 区分大小写匹配

- **WHEN** 关键字 `case_sensitive = true`
- **THEN** `AhoCorasickMatcher` SHALL 使用精确大小写模式匹配

#### Scenario: 匹配结果按文章分组

- **WHEN** 一篇文章的 `title + summary` 命中了 3 个不同关键字
- **THEN** 该文章的 `ArticleMatches.matched_keyword_ids` SHALL 包含 3 个 keyword_id

#### Scenario: 无匹配的文章不在结果中

- **WHEN** 某篇文章未命中任何关键字
- **THEN** `match_batch` 返回的 Vec SHALL NOT 包含该文章的 `ArticleMatches` 条目

### Requirement: 匹配逻辑从编排函数解耦

`run_filter_once` SHALL 通过 `&dyn KeywordMatcher` 引用调用匹配逻辑，而非直接依赖 `Automata` 结构体或 `build_automata`/`match_articles` 函数。

#### Scenario: run_filter_once 使用 trait 对象

- **WHEN** `run_filter_once` 执行匹配阶段
- **THEN** 它 SHALL 通过 `KeywordMatcher` trait 引用调用 `match_batch`
- **THEN** 它 SHALL NOT 直接引用 `Automata` 或调用 `match_articles`
