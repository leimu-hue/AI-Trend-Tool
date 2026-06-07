## Why

系统当前所有 `/api/v1/*` 端点无认证保护，任何请求均可访问。需要 Bearer Token 认证中间件保护所有 API 端点，并提供 Token 管理 API 供前端/管理员使用。计划文档已给出详细实现方案（docs/plans/03-auth-and-token-api.md）。

## What Changes

- 新增 Bearer Token 认证中间件，保护 `/api/v1/*` 下所有端点
- 新增 Token CRUD API（POST/GET/DELETE `/api/v1/tokens`）
- 首次启动时自动初始化 token（配置优先，无配置时自动生成随机 token 并日志输出）—— **注意：与现有 `backend-project-scaffold` spec 冲突，需确认**
- 更新路由层，对 API 路由组应用认证中间件
- 新增 `handlers` 和 `middleware` 模块

## Capabilities

### New Capabilities

- `auth-middleware`: Bearer Token 认证中间件。从 Authorization header 提取 token，验证有效性、吊销状态、过期时间，通过后将 `ApiToken` 注入 request extensions
- `token-api`: Token CRUD API。POST 创建（返回完整 token 明文）、GET 列表（隐藏 token 明文）、DELETE 吊销（软删除，设 revoked=1）
- `initial-token-bootstrap`: 首次启动引导。`api_tokens` 表为空时自动创建初始 token：优先用 `config.toml` 中 `auth.initial_token`，否则自动生成 64 字符 hex token 并通过 tracing 日志输出

### Modified Capabilities

- `backend-project-scaffold`: **待确认** — 现有 spec 声明"未配置 initial_token 时系统不创建任何 token"。计划文档要求改为自动生成随机 token。需确认最终行为。

## Impact

- **Affected code**: `src/routes.rs`（路由重构）, `src/main.rs`（初始 token 引导）, `src/middleware.rs`（模块声明）, `src/handlers.rs`（模块声明）
- **New files**: `src/middleware/auth.rs`, `src/handlers/token.rs`
- **Dependencies**: 已就绪 — `rand`、`hex`、`chrono` 已在 Cargo.toml 中
- **Models**: `ApiToken`、`ApiTokenInfo`、`CreateTokenRequest` 已存在于 `src/models/token.rs`
- **Error handling**: `AppError::Unauthorized` 已存在于 `src/error.rs`
- **⚠️ 模块约定冲突**: 计划文档建议创建 `src/middleware/mod.rs` 和 `src/handlers/mod.rs`，但现有 `module-organization` spec 要求 2018 edition 风格（`src/middleware.rs` + `src/middleware/`）。代码库已存在 `src/middleware.rs` 和 `src/handlers.rs`，将遵循现有约定。
