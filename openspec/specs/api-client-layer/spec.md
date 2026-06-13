# API Client Layer

## Purpose

Provide a configured Axios HTTP client with automatic Bearer token injection, error handling, and per-domain typed API modules for all backend endpoints.

## Requirements

### Requirement: Axios instance with base configuration
The system SHALL create and export a configured Axios instance at `src/api/client.ts` with the base URL from environment variable `VITE_API_BASE_URL`, 30-second timeout, and JSON content type header.

#### Scenario: Axios instance created with defaults
- **WHEN** the client module is imported
- **THEN** an Axios instance exists with `baseURL` matching `VITE_API_BASE_URL` (default `http://localhost:3000/api/v1`), `timeout: 30000`, and `Content-Type: application/json` header

### Requirement: Request interceptor adds Bearer token
The system SHALL add a request interceptor that reads the token from `localStorage.getItem('api_token')` and sets the `Authorization: Bearer <token>` header on every outgoing request when a token exists.

#### Scenario: Token added when present
- **WHEN** `localStorage` contains `api_token: "test-token-123"`
- **THEN** outgoing requests include header `Authorization: Bearer test-token-123`

#### Scenario: No header when token missing
- **WHEN** `localStorage` has no `api_token` key
- **THEN** outgoing requests do NOT include an `Authorization` header

### Requirement: Response interceptor handles errors
The system SHALL add a response interceptor that handles HTTP errors: 401 clears the token and redirects to `/auth`, other server errors display the backend error message, and network errors display a connection failure message.

#### Scenario: 401 clears token and redirects
- **WHEN** server responds with HTTP 401
- **THEN** `api_token` SHALL be removed from `localStorage` and `window.location.hash` SHALL be set to `#/auth`

#### Scenario: Server error displays backend message
- **WHEN** server responds with HTTP 400/404/500 and body contains `{ "error": { "message": "关键词不存在" } }`
- **THEN** a toast notification displays "关键词不存在"

#### Scenario: Network error displays connection message
- **WHEN** request fails without receiving any response (network down, server stopped)
- **THEN** a toast notification displays "网络错误，请检查后端服务是否启动"

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
