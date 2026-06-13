## MODIFIED Requirements

### Requirement: Notification hook

The system SHALL provide a `useMessage` hook that wraps antd's `App.useApp().notification` for type-safe notification calls with bottom-right placement. The `ToastProvider` component SHALL clean up all active timers on unmount to prevent memory leaks and React warnings.

#### Scenario: Success notification

- **WHEN** `showMessage.success("保存成功")` is called from a component within `<App>`
- **THEN** a dark-themed success notification appears at bottom-right and auto-dismisses after 2 seconds

#### Scenario: Toast timers cleaned up on unmount

- **WHEN** `ToastProvider` component unmounts (e.g., HMR hot reload)
- **THEN** all active `setTimeout` timers SHALL be cleared via `clearTimeout`
- **THEN** no React state update warnings SHALL appear in the console
