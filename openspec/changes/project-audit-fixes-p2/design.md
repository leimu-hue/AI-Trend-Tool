## Context

P2 级改进涉及 14 个跨领域优化项：代码统一性（validator crate、Auth 页面组件风格）、可观测性（日志级别）、可测试性（单元测试）、用户体验（分页、错误提示）、可靠性（Parser 子任务跟踪、连接池大小、类型截断、unwrap panic）、安全性（错误消息脱敏、preload 最小权限）、代码卫生（死代码清理、废弃 API 替换、缓存清理）。

所有改进为渐进式优化，不改变核心业务逻辑，不引入破坏性变更。

## Goals / Non-Goals

**Goals:**
- 用 `validator` crate 统一输入验证，消除重复手动验证代码
- Auth 页面使用项目自定义组件，与管理页面风格统一
- 日志级别支持 `RUST_LOG` 环境变量
- 核心逻辑（config 验证、filter 统计、error 转换）有单元测试覆盖
- 前端 Articles 页面有分页控件
- Parser 关闭时等待子任务完成，避免数据丢失
- DB 连接池大小匹配并发需求
- hours 参数安全 clamp、DB 路径边界处理
- ErrorBoundary 错误消息脱敏
- 死代码清理、废弃 API 替换、缓存清理

**Non-Goals:**
- 添加集成测试或 e2e 测试
- 完整的前端响应式设计改造
- 新的业务功能

## Decisions

### Decision 1: validator 0.19 而非手动提取验证函数

**选择**：引入 `validator = { version = "0.19", features = ["derive"] }`，为各 Request 结构体添加 `#[derive(Validate)]`

**备选**：提取手动验证到独立函数。减少依赖但代码量不降。
**理由**：validator 是 Rust 生态成熟方案，derive 宏消除样板代码，错误消息自动生成。

### Decision 2: JoinSet 而非 Vec<JoinHandle>

**选择**：`tokio::task::JoinSet` 管理 Parser 子任务。

**备选**：`Vec<JoinHandle>` 手动管理。
**理由**：JoinSet 提供 `join_next()` 方法，自然处理任务完成顺序，无需手动追踪。

### Decision 3: 分页控件使用项目统一按钮样式

**选择**：使用 `btn btn-ghost btn-sm` 类，与现有管理页面操作按钮风格一致。

**备选**：使用 antd Pagination 组件。引入新的组件依赖，风格不统一。
**理由**：保持前端组件风格一致性，项目已明确使用自定义内联样式组件模式。

## Risks / Trade-offs

- **validator 引入额外依赖**：增加编译时间 ~5-10s，但消除 ~100 行重复验证代码。
- **DB 连接池增大**：从 5 增加到 15+，SQLite 单写者模式不受影响（WAL 模式支持并发读）。
- **JoinSet 等待**：关闭时等待所有子任务完成，如果某个 fetch 卡住会延长关闭时间。→ reqwest 已有 30s 超时，最坏情况 30s 内关闭。
