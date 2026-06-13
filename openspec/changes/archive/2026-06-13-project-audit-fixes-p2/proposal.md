## Why

项目审查发现 14 个 P2 级改进项，涵盖代码卫生、可靠性、用户体验和安全性。这些问题不影响核心功能，但累积会导致技术债务加重、调试困难、用户体验下降。

## What Changes

- **输入验证统一**：引入 `validator` crate，替换各 handler 中重复的手动验证代码
- **Auth 页面组件统一**：将 antd 组件替换为项目自定义组件 + Tailwind，与其他管理页面风格一致
- **日志级别动态配置**：优先读取 `RUST_LOG` 环境变量，默认回退 `info`
- **核心逻辑单元测试**：为 config 验证、error 转换、filter 统计计算添加测试
- **前端分页控件**：Articles 页面添加分页 UI，利用后端已有的分页参数
- **Parser 子任务跟踪**：使用 `JoinSet` 跟踪 spawn 的子任务，关闭时等待完成
- **DB 连接池扩容**：`max_connections` 设为 `>= max_concurrent_fetches + 5`
- **hours 参数安全 clamp**：防止 i64→i32 截断为负数
- **DB 路径 unwrap 安全化**：根路径边界情况处理
- **ErrorBoundary 消息脱敏**：UI 显示通用提示，详细错误仅输出到 console
- **Tokens 页面 clipboard API 现代化**：使用 preload 暴露的 IPC 桥接替代 `document.execCommand('copy')`
- **死代码清理**：删除 `useApi.ts`、`Loading.tsx`、移除 `clipboard.readText` 的 preload 暴露
- **notification 模块缓存清理**：`useNotificationBridge` 添加 cleanup 逻辑
- **Auth 中间件日志增强**：auth 失败时记录原因，方便排查（内嵌于其他修改中）

## Capabilities

### New Capabilities

- `validator-integration`: 引入 validator crate，为 Create/Update 请求结构体添加 derive 验证
- `unit-test-coverage`: config、error、filter 模块的单元测试
- `frontend-pagination`: Articles 列表的分页 UI 控件
- `parser-joinset-tracking`: Parser 使用 JoinSet 管理子任务生命周期

### Modified Capabilities

- `input-validation`: 手动验证代码替换为 validator derive + `req.validate()?`
- `auth-page`: antd 组件替换为自定义组件 + Tailwind
- `backend-project-scaffold`: 日志级别支持 RUST_LOG 环境变量；DB 路径 unwrap 安全化；DB 连接池大小与并发任务数匹配
- `query-apis`: hours 参数 clamp 到 1..8760；添加分页查询支持
- `shared-components`: ErrorBoundary 错误消息脱敏；删除 Loading.tsx 死代码；notification 模块 cleanup
- `token-api`: Tokens 页面使用 IPC clipboard 替代 execCommand
- `frontend-project-scaffold`: 移除 preload 中未使用的 clipboard.readText
- `parser-module`: 子任务 JoinHandle 收集和取消等待
- `config-validation`: 添加配置验证的单元测试
- `filter-module`: 提取 compute_stats 纯函数并添加测试

## Non-goals

- P0 安全修复（Tasks 1-3, 13-16）：已在 `project-audit-fixes-p0` 中处理
- P1 Bug 修复（Tasks 5-6, 17-22）：已在 `project-audit-fixes-p1` 中处理
- Task 7（数据库路径迁移）：用户已选择保留现有路径
- 非 RESTful 路由更改：按项目规范不做更改

## Impact

- **后端**：`Cargo.toml`（新增 validator 依赖）、`src/models/*.rs`（derive Validate）、`src/handlers/*.rs`（调用 validate）、`src/error.rs`（From impl + 测试）、`src/main.rs`（EnvFilter + unwrap 修复）、`src/db.rs`（连接池大小）、`src/services/parser.rs`（JoinSet）、`src/services/filter.rs`（compute_stats + 测试）、`src/config.rs`（测试）
- **前端**：`web/src/renderer/src/pages/Auth.tsx`、`web/src/renderer/src/pages/Articles.tsx`、`web/src/renderer/src/pages/Tokens.tsx`、`web/src/renderer/src/components/ErrorBoundary.tsx`、`web/src/preload/index.ts`、`web/src/main/index.ts`、删除 `useApi.ts` / `Loading.tsx`、`lib/notification.ts`
- **破坏性变更**：无。validator 验证规则与现有手动验证等价
