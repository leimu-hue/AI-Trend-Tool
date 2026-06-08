# API Client Layer

## Purpose

Provide a configured Axios HTTP client with automatic Bearer token injection, error handling, and per-domain typed API modules for all backend endpoints.

## Requirements

### Requirement: Axios instance with base configuration
The system SHALL create and export a configured Axios instance at `src/api/client.ts` with the base URL from environment variable `VITE_API_BASE_URL`, 30-second timeout, and JSON content type header.

#### Scenario: Axios instance created with defaults
- **WHEN** the client module is imported
- **THEN** an Axios instance exists with `baseURL` matching `VITE_API_BASE_URL` (default `http://localhost:8080/api/v1`), `timeout: 30000`, and `Content-Type: application/json` header

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
