## MODIFIED Requirements

### Requirement: Auth page visual design

The system SHALL render the token authentication page with a full-viewport dark background (`var(--bg)`), a centered card using project CSS classes (`panel`, `p-8`, custom widths), containing the system title, instruction text, token input field using native `<input type="password">`, and submit button using project `.btn .btn-primary` classes. The page SHALL NOT use antd components (Card, Input, Button, Alert, Typography).

#### Scenario: Auth page layout

- **WHEN** user navigates to `/auth`
- **THEN** the page displays a vertically and horizontally centered div with `panel` class containing:
  - Title "AI 热点监控系统" using native `h2` element
  - Subtitle "请输入 API Token 以继续" using native `p` element
  - Native `<input type="password">` field with placeholder "粘贴你的 API Token..."
  - Native `<button className="btn btn-primary">` "验证并进入"
  - Hint text about initial Token in backend logs

### Requirement: Token validation workflow

The system SHALL validate the entered token by storing it in localStorage and calling `GET /tokens`. On success, navigate to `/dashboard`. On failure, clear the token and display an error using a custom error div (not antd Alert).

#### Scenario: Successful token validation

- **WHEN** user submits a valid token
- **THEN** the token SHALL be saved to `localStorage` as `api_token`, an API call to `GET /tokens` SHALL return successfully, and user SHALL be navigated to `/dashboard`

#### Scenario: Failed token validation

- **WHEN** user submits an invalid or expired token
- **THEN** the token SHALL NOT remain in `localStorage`, an error message "Token 无效或已过期" SHALL appear in a custom error div styled with red background and `var(--danger)` color

#### Scenario: Loading state during validation

- **WHEN** the validation API call is in progress
- **THEN** the submit button text changes to "验证中..." and the button is disabled
