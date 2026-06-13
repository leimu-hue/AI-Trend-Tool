## Why

`src/services/filter.rs` 已膨胀至 435 行，struct、函数、测试全部混在一个文件中。后续计划引入 AI 关键字匹配（大模型语义判断替代/补充 Aho-Corasick），当前的单文件结构难以扩展——新增匹配策略时需要在一个文件中塞入更多分支逻辑，且无法通过 trait 实现策略替换。现在拆分，为后续 AI 匹配能力预留清晰的扩展点。

## What Changes

- **BREAKING** `services/filter.rs` 拆分为 `filter.rs` + `filter/` 子目录，使用 Rust 2018+ `filter.rs` + `filter/` 共存模式（非 mod.rs）
- 新增 `KeywordMatcher` trait，定义关键字匹配的抽象接口
- 当前 Aho-Corasick 匹配逻辑实现 `KeywordMatcher` trait
- 每个 struct（`Automata`、`MatchResult`）迁移到 `filter/types.rs`
- 按功能拆分子模块：`matching.rs`（匹配）、`detection.rs`（突发检测）、`validation.rs`（校验）
- `filter.rs` 仅保留主编排逻辑（`run_filter_once`、`start_filter_loop`）和单元测试
- 所有公开 API（`run_filter_once`、`start_filter_loop`、`compute_stats`）的签名和行为不变，对外调用方零改动

## Non-goals

- 不实现 AI 匹配逻辑（仅预留 trait 扩展点）
- 不修改 `parser.rs` 或 `pusher.rs` 的文件结构
- 不改变数据库 schema 或 SQL 查询
- 不改变 API 端点行为

## Capabilities

### New Capabilities

- `keyword-matcher-trait`: 定义 `KeywordMatcher` trait 抽象接口，支持替换关键字匹配策略（当前 AC 自动机，未来 AI 模型）

### Modified Capabilities

- `filter-module`: 模块内部拆分为子模块，`run_filter_once` 编排流程改为通过 `KeywordMatcher` trait 调用匹配逻辑；`Aho-Corasick keyword matching` requirement 改为 "默认 AC 实现满足 `KeywordMatcher` trait"

## Impact

- Affected code: `src/services/filter.rs` → `src/services/filter.rs` + `src/services/filter/*.rs`（新建 5 个子文件）
- `src/services.rs` 需新增子模块声明（`pub mod filter;` 后添加 filter 内部模块路径）
- `src/main.rs` 和 `src/handlers/` 无需改动——`run_filter_once` 和 `start_filter_loop` 的 pub 路径不变
- Dependencies: 无新增外部依赖
