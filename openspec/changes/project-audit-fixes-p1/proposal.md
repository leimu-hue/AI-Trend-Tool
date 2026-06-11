## Why

项目审查发现 8 个 P1 级问题：数据源列表缺少文章计数、Pusher 每次创建新 HTTP 客户端浪费连接、Pusher 竞态导致 webhook 重复发送、分页 per_page 与实际不符导致数据不可达、push_records 静默吞错、Settings 页面 fallback 默认值与后端不一致、Toast 定时器内存泄漏、Layout 页脚误导性文案。这些问题影响功能正确性、性能和用户体验。

## What Changes

- **数据源列表添加文章计数**：`list_sources` 返回 `article_count` 字段，前端显示实际计数
- **Pusher HTTP 客户端复用**：在 loop 启动时创建一次 `reqwest::Client`，传递引用给每次调用
- **Pusher 原子领取机制**：先 UPDATE pending→processing，再 SELECT processing，消除定时器与事件并发触发的竞态窗口
- **分页 per_page 响应值修正**：handler 层先 clamp per_page 再传入 DB 和响应，确保响应值与实际返回条数一致
- **push_records 插入错误日志**：区分 UNIQUE 冲突（正常跳过）和真正的数据库错误（记录日志）
- **Settings DEFAULTS 对齐**：前端 fallback 默认值与 `config.toml` 一致（max_concurrent_fetches: 10, batch_size: 1000, port: 3000）
- **Toast 定时器清理**：使用 `useRef` 收集 timer ID，组件卸载时统一 clearTimeout
- **Layout 页脚文案修正**：移除"每5分钟自动刷新"误导性描述，改为"后端监控运行中"

## Capabilities

### New Capabilities

- `source-article-count`: 数据源列表查询返回文章计数，通过 LEFT JOIN + COUNT + GROUP BY 实现
- `pusher-atomic-claim`: Pusher 在处理前先原子性地将 pending 记录标记为 processing，防止并发重复发送

### Modified Capabilities

- `source-crud-api`: `list_sources` handler 使用带计数的新查询函数
- `pusher-module`: `run_pusher_once` 接收外部传入的 `reqwest::Client` 引用；增加原子领取步骤；`insert_push_records_for_event` 区分错误类型并记录日志
- `query-apis`: 分页 handler 在构造响应前 clamp per_page，确保响应值与 DB 实际 limit 一致
- `shared-components`: Toast 组件卸载时清理所有活跃定时器
- `app-layout`: 侧边栏底部状态文案移除自动刷新暗示
- `frontend-project-scaffold`: Settings 页面 DEFAULTS 对象与 config.toml 对齐

## Non-goals

- Task 7（数据库路径迁移）：用户已选择保留现有路径
- Task 4（401 hash 路由跳转）：经分析非 Bug
- P0 安全修复（Tasks 1-3, 13-16）：已在 `project-audit-fixes-p0` 中处理
- P2 改进（Tasks 8-12, 23-31）：将在 `project-audit-fixes-p2` 中处理

## Impact

- **后端**：`src/db/source.rs`（新查询）、`src/handlers/source.rs`（使用新查询）、`src/services/pusher.rs`（client 复用 + 原子领取）、`src/db/push_record.rs`（错误日志）、`src/handlers/query.rs`（per_page clamp + trigger_pusher 传 client）
- **前端**：`web/src/renderer/src/pages/Settings.tsx`（DEFAULTS）、`web/src/renderer/src/components/Toast.tsx`（定时器清理）、`web/src/renderer/src/components/Layout.tsx`（页脚文案）
- **无破坏性变更**：所有修改向后兼容
