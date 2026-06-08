## ADDED Requirements

### Requirement: Auth page visual design
The system SHALL render the token authentication page with a full-viewport dark background (`var(--bg)`), a centered modal-style card (`var(--surface)`, `var(--elev-raised)` shadow, `var(--radius-lg)` radius, `var(--border)` border), containing the system title, instruction text, token input field, and submit button.

#### Scenario: Auth page layout
- **WHEN** user navigates to `/auth`
- **THEN** the page displays a vertically and horizontally centered card with:
  - Title "AI 热点监控系统" in `var(--fg)` using `var(--font-display)` at `var(--text-lg)`
  - Subtitle "请输入 API Token 以继续" in `var(--muted)` using `var(--font-mono)` at `var(--text-xs)` with uppercase and letter-spacing
  - Password input field with placeholder "粘贴你的 API Token..."
  - Submit button "验证并进入" using `.btn-primary` class
  - Hint text about initial Token in backend logs (small, `var(--meta)` color)

### Requirement: Token validation workflow
The system SHALL validate the entered token by storing it in localStorage and calling `GET /tokens`. On success, navigate to `/dashboard`. On failure, clear the token and display an error.

#### Scenario: Successful token validation
- **WHEN** user submits a valid token
- **THEN** the token SHALL be saved to `localStorage` as `api_token`, an API call to `GET /tokens` SHALL return successfully, and user SHALL be navigated to `/dashboard`

#### Scenario: Failed token validation
- **WHEN** user submits an invalid or expired token
- **THEN** the token SHALL NOT remain in `localStorage`, an error message "Token 无效或已过期" SHALL appear in a red alert box styled with `var(--danger)` color and semi-transparent red background

#### Scenario: Loading state during validation
- **WHEN** the validation API call is in progress
- **THEN** the submit button text changes to "验证中..." and the button is disabled

### Requirement: Redirect when token exists
The system SHALL redirect authenticated users away from the auth page when a valid token is already present.

#### Scenario: Already authenticated redirect
- **WHEN** user navigates to `/auth` while `localStorage` already contains a valid `api_token`
- **THEN** user SHALL be redirected to `/dashboard` immediately
