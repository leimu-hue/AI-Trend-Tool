## 1. 创建子目录和 types.rs

- [x] 1.1 创建 `src/services/filter/` 目录
- [x] 1.2 创建 `src/services/filter/types.rs`，迁移 `Automata` 和 `MatchResult` 结构体定义，添加需要的 `use` 声明
- [x] 1.3 `cargo check` 验证 types.rs 编译通过



## 2. 创建 traits.rs（KeywordMatcher trait）

- [x] 2.1 创建 `src/services/filter/traits.rs`，定义 `KeywordMatcher` trait 和 `ArticleMatches` 结构体
- [x] 2.2 `cargo check` 验证 traits.rs 编译通过



## 3. 创建 matching.rs（AhoCorasickMatcher 实现）

- [x] 3.1 创建 `src/services/filter/matching.rs`：迁移 `build_automata` 函数，实现 `AhoCorasickMatcher` struct（封装 `Automata`），为 `AhoCorasickMatcher` 实现 `KeywordMatcher` trait（`match_batch` 方法）
- [x] 3.2 `cargo check` 验证 matching.rs 编译通过



## 4. 创建 detection.rs（突发检测）

- [x] 4.1 创建 `src/services/filter/detection.rs`：迁移 `compute_stats`、`detect_and_push`、`upsert_hot_event_record` 函数
- [x] 4.2 `cargo check` 验证 detection.rs 编译通过



## 5. 创建 validation.rs（前置校验）

- [x] 5.1 创建 `src/services/filter/validation.rs`：迁移 `claim_and_validate` 函数
- [x] 5.2 `cargo check` 验证 validation.rs 编译通过



## 6. 重写 filter.rs（主编排逻辑）

- [x] 6.1 重写 `src/services/filter.rs`：保留 `run_filter_once` 和 `start_filter_loop`，声明子模块 `mod types; mod traits; mod matching; mod detection; mod validation;`，`run_filter_once` 内部改为通过 `AhoCorasickMatcher` + `KeywordMatcher` trait 调用匹配逻辑
- [x] 6.2 `cargo check` 验证完整编译通过

## 7. 移动测试并验证

- [x] 7.1 将 `#[cfg(test)]` 模块及 `compute_stats` 测试保留在 `filter.rs`，`cargo test` 验证全部测试通过
