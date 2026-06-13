## MODIFIED Requirements

### Requirement: Typed API modules per domain

The system SHALL provide per-domain API modules (`tokens.ts`, `sources.ts`, `keywords.ts`, `channels.ts`, `queries.ts`) that export typed functions wrapping the shared Axios client.

#### Scenario: Token API module

- **WHEN** `tokenApi.list()` is called
- **THEN** it sends `GET /tokens` and returns `Promise<TokenInfo[]>`
- **WHEN** `tokenApi.create({ name: "my-token" })` is called
- **THEN** it sends `POST /tokens` with the request body and returns `Promise<CreateTokenResponse>`
- **WHEN** `tokenApi.revoke(1)` is called
- **THEN** it sends `POST /tokens/revoke/1`

#### Scenario: Each API module follows same pattern

- **WHEN** any API module function is called
- **THEN** it uses the shared Axios client, returns a typed Promise, and uses GET/POST methods only (matching backend API conventions)

## ADDED Requirements

### Requirement: Article 接口包含 status 字段

`queries.ts` 中的 `Article` 接口 SHALL 包含 `status: string` 字段，值为 `'pending' | 'processing' | 'matched' | 'skipped'` 之一。`processed_at` 字段 SHALL 保留以维持向后兼容。

#### Scenario: 新 Article 数据包含 status

- **WHEN** 后端返回 Article JSON 包含 `"status": "matched"` 和 `"processed_at": "2026-01-01T00:00:00Z"`
- **THEN** 前端解析后的 `Article` 对象 `status` 为 `"matched"`，`processed_at` 为对应时间戳

#### Scenario: 旧 Article 数据不含 status（向后兼容）

- **WHEN** 后端返回 Article JSON 不包含 `status` 字段但包含 `"processed_at": null`
- **THEN** 前端解析后的 `Article` 对象 `status` 为 `undefined`，`processed_at` 为 `null`

### Requirement: PushRecord 接口包含 last_error 字段并支持 dead 状态

`queries.ts` 中的 `PushRecord` 接口 SHALL 包含 `last_error: string | null` 字段。`status` 字段 SHALL 支持 `'dead'` 值（重试耗尽终态）。

#### Scenario: PushRecord 包含 last_error

- **WHEN** 后端返回 PushRecord JSON 包含 `"last_error": "connection refused"`
- **THEN** 前端解析后的 `PushRecord` 对象 `last_error` 为 `"connection refused"`

#### Scenario: PushRecord 状态为 dead

- **WHEN** 后端返回 PushRecord JSON 包含 `"status": "dead"`
- **THEN** 前端解析后的 `PushRecord` 对象 `status` 为 `"dead"`

### Requirement: getArticles 支持 status 参数

`queryApi.getArticles()` 方法的 `params` 参数 SHALL 支持 `status?: string` 字段，可选值为 `'pending'`、`'processing'`、`'matched'`、`'skipped'`。`processed?: boolean` 参数 SHALL 保留以向后兼容。

#### Scenario: 用 status 参数查询

- **WHEN** 调用 `queryApi.getArticles({ status: 'matched' })`
- **THEN** 发送 `GET /api/v1/articles?status=matched`，返回 `Promise<PaginatedResponse<Article>>`

#### Scenario: status 和 processed 同时存在时优先 status

- **WHEN** 调用 `queryApi.getArticles({ status: 'matched', processed: true })`
- **THEN** 请求同时携带两个参数，后端自行处理优先级（后端优先使用 `status`）
