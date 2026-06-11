## MODIFIED Requirements

### Requirement: Error boundary component

The system SHALL provide an `ErrorBoundary` class component that catches JavaScript errors in its child component tree and displays a fallback UI. The UI SHALL show a generic, user-friendly message without exposing internal error details. Detailed error information SHALL only be output to `console.error`.

#### Scenario: Error boundary catches render error

- **WHEN** a child component throws an error during rendering
- **THEN** the ErrorBoundary displays a generic message "页面发生了未知错误，请刷新页面重试"
- **THEN** the detailed error SHALL be logged via `console.error`
- **THEN** the error message SHALL NOT be displayed in the UI

#### Scenario: Error boundary logs errors

- **WHEN** the ErrorBoundary catches an error
- **THEN** the error and component stack SHALL be logged via `console.error`

### Requirement: Notification hook

The system SHALL provide a `useMessage` hook that wraps antd's `App.useApp().notification` for type-safe notification calls. The `useNotificationBridge` SHALL cleanup its module-level cache (`contextApi`) on component unmount.

#### Scenario: Module cache cleaned up on unmount

- **WHEN** `App` component unmounts (e.g., HMR)
- **THEN** `contextApi` SHALL be set to `null` in the cleanup function
- **THEN** subsequent calls to notification functions SHALL not reference the stale instance

## REMOVED Requirements

### Requirement: Loading component

**Reason**: `Loading` component is dead code — not imported by any module in the project.
**Migration**: No migration needed. Use antd `Spin` directly where loading indicators are needed.
