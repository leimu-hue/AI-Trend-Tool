## Context

`src/services/filter.rs` 当前 435 行，包含 2 个 struct（`Automata`、`MatchResult`）、1 个纯函数（`compute_stats`）、5 个私有函数、2 个公开函数、1 个测试模块。所有逻辑平铺在一个文件中。项目计划引入 AI 关键字匹配（大模型语义判断），需要在现有 Aho-Corasick 匹配之外支持可替换的匹配策略。

Rust 2018+ 支持 `module_name.rs` + `module_name/` 目录共存作为子模块声明——`filter.rs` 作为模块根，`filter/` 下的 `.rs` 文件作为子模块。这是本项目采用的模式（而非过时的 `mod.rs`）。

## Goals / Non-Goals

**Goals:**
- 将 filter.rs 按功能拆分为 `filter.rs` + `filter/` 目录下的子模块
- 抽取 `KeywordMatcher` trait，定义关键字匹配的抽象接口
- 当前 Aho-Corasick 匹配封装为 `KeywordMatcher` trait 的默认实现
- 保持所有公开 API 签名不变（`run_filter_once`、`start_filter_loop`、`compute_stats`）
- 保持现有单元测试通过

**Non-Goals:**
- 不实现 AI 匹配逻辑（仅预留 trait）
- 不拆分 `parser.rs` 或 `pusher.rs`
- 不修改数据库交互层（`db/`）
- 不修改 HTTP 处理器（`handlers/`）

## Decisions

### Decision 1: 模块组织结构

选择 `filter.rs` + `filter/` 目录共存模式，而非 `filter/mod.rs`。

**理由**：用户明确要求不使用 mod.rs。Rust 2018+ 推荐的 `filter.rs` + `filter/` 模式更符合现代 Rust 风格。`filter.rs` 作为模块根声明 `mod types; mod traits; mod matching; mod detection; mod validation;`，并包含主编排逻辑。

**替代方案**：`filter/mod.rs` — 被用户否决。

### Decision 2: 子模块文件划分

```
filter.rs          — 模块根: run_filter_once, start_filter_loop, #[cfg(test)]
filter/types.rs    — Automata, MatchResult
filter/traits.rs   — KeywordMatcher trait
filter/matching.rs — build_automata, match_articles (AC 实现 KeywordMatcher)
filter/detection.rs— compute_stats, detect_and_push, upsert_hot_event_record
filter/validation.rs — claim_and_validate
```

**理由**：按功能内聚划分——匹配引擎（matching）、突发检测（detection）、前置校验（validation）。每个文件职责单一。types.rs 集中管理所有 struct。traits.rs 独立文件支持未来新增匹配策略实现者。

**替代方案**：
- 5 个文件放同一层级 → 不按目录分，功能边界模糊
- 所有 struct 各自独立文件 → 当前仅 2 个 struct，过度拆分增加文件数

### Decision 3: KeywordMatcher trait 设计

```rust
pub trait KeywordMatcher: Send + Sync {
    /// 对一批文章执行关键字匹配。
    /// 返回每个文章的匹配结果：命中的 keyword_id 列表。
    fn match_batch(&self, articles: &[Article]) -> Vec<ArticleMatches>;
}

pub struct ArticleMatches {
    pub article_id: i64,
    pub matched_keyword_ids: Vec<i64>,
}
```

**理由**：
- `Send + Sync` 允许跨线程/异步任务使用
- `match_batch` 批处理接口——一次传入多篇文章，避免逐篇调用的开销
- 返回值按文章分组，调用方（`detect_and_push`）自行聚合 hourly_counts 和 mentions

**替代方案**：
- `fn match_single(&self, text: &str) -> Vec<i64>` — 粒度太细，调用方需要显式循环
- `async fn` trait 方法 — 当前 AC 实现是同步的，用 `async_trait` 增加无关依赖。等 AI 实现需要时再改为 async。

### Decision 4: 可见性策略

- `run_filter_once`、`start_filter_loop`、`compute_stats`：保持 `pub`
- `KeywordMatcher` trait 及 `ArticleMatches`：`pub(crate)` 或 `pub`（后续 AI 实现者可能在其他 crate）
- `Automata`、`MatchResult`、所有私有函数：`pub(super)`（仅 `filter` 父模块可见）
- `build_automata`、`match_articles`、`detect_and_push`、`upsert_hot_event_record`、`claim_and_validate`：`pub(super)` 供 `filter.rs` 编排调用

**理由**：最小化公开 API 面。只有真正需要被外部调用的才 pub。子模块间的调用通过 `pub(super)` 限定了作用域。

### Decision 5: `services.rs` 声明变更

**当前**：
```rust
pub mod filter;
pub mod parser;
pub mod pusher;
```

**变更后**：
```rust
pub mod filter;
// filter 内部子模块由 filter.rs 自行声明，services.rs 无需感知
pub mod parser;
pub mod pusher;
```

**无需变更 services.rs**。filter.rs 内部 `mod types; mod traits; mod matching; mod detection; mod validation;` 即可。

## Risks / Trade-offs

- **[文件增多]** 从 1 个文件变为 6 个文件 → 接受：这是模块化的必要代价，每个文件 <100 行，职责清晰
- **[trait 对象开销]** 未来用 `Box<dyn KeywordMatcher>` 替代泛型时可能有微小动态分发开销 → `match_batch` 本身是批处理，单次调用摊销开销可忽略
- **[编译时间]** 更多模块可能略微增加编译时间 → 实际影响极小，模块都很小
- **[重构风险]** 代码移动可能引入编译错误 → 每个子文件创建后立即 `cargo check`，渐进式迁移

## Migration Plan

1. 创建 `src/services/filter/` 目录
2. 逐步迁移：types → traits → matching → detection → validation → 主文件
3. 每步 `cargo check` 验证编译
4. `cargo test` 验证测试通过
5. 无需回滚策略——纯代码重组，git revert 即可
