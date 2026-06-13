## MODIFIED Requirements

### Requirement: Bearer token extraction

The auth middleware SHALL extract a Bearer token from the `Authorization` header of incoming requests. If the header is missing or does not use the `Bearer` scheme, the middleware SHALL return HTTP 401 with error code `UNAUTHORIZED`.

#### Scenario: Missing Authorization header

- **WHEN** a request arrives at `/api/v1/*` without an `Authorization` header
- **THEN** the middleware SHALL return HTTP 401 with body `{"error": {"code": "UNAUTHORIZED", "message": "Missing Authorization header"}}`

#### Scenario: Non-Bearer Authorization header

- **WHEN** a request arrives with `Authorization: Basic dXNlcjpwYXNz`
- **THEN** the middleware SHALL return HTTP 401 with body `{"error": {"code": "UNAUTHORIZED", "message": "Invalid Authorization format, expected Bearer"}}`

## ADDED Requirements

### Requirement: CORS 显式方法白名单

CORS 中间件 SHALL 使用显式 HTTP 方法白名单替代 `CorsLayer::permissive()`。允许的方法包括 GET、POST、PUT、DELETE、OPTIONS。Origin 和 Headers 保持 `Any`（兼容 Electron file:// 协议）。

#### Scenario: 允许的 HTTP 方法通过 CORS

- **WHEN** 跨域请求使用 GET、POST、PUT、DELETE 或 OPTIONS 方法
- **THEN** CORS 中间件 SHALL 返回对应的 `Access-Control-Allow-Methods` 头
- **THEN** 请求 SHALL 正常通过

#### Scenario: 不允许的 HTTP 方法被拒绝

- **WHEN** 跨域请求使用 PATCH、HEAD 或其他未列出的方法
- **THEN** CORS 中间件 SHALL 拒绝该预检请求
