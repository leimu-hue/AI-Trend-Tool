# Shared Components

## Purpose

Provide reusable UI components for loading, empty states, error boundaries, and toast notifications — wrapping antd base components with project-specific defaults.

## Requirements

### Requirement: Loading component
The system SHALL provide a `Loading` component that wraps antd `Spin` with optional full-page centering.

#### Scenario: Default loading
- **WHEN** `Loading` is rendered without props
- **THEN** a centered container (min-height 200px) displays antd `Spin` with "加载中..." text in `var(--muted)` color

#### Scenario: Full-page loading
- **WHEN** `Loading` is rendered with `fullPage` prop
- **THEN** the container has `min-height: 100vh` for full-viewport centering

### Requirement: Empty state component
The system SHALL provide an `Empty` component that wraps antd `Empty` with custom description and optional call-to-action.

#### Scenario: Empty with description only
- **WHEN** `Empty` is rendered with `description="暂无数据"`
- **THEN** antd `Empty` displays with "暂无数据" as the description

#### Scenario: Empty with action button
- **WHEN** `Empty` is rendered with `description="暂无数据源"`, `actionText="添加数据源"`, and an `onAction` handler
- **THEN** an antd `Button` with "添加数据源" is displayed below the description and calls `onAction` when clicked

### Requirement: Error boundary component
The system SHALL provide an `ErrorBoundary` class component that catches JavaScript errors in its child component tree and displays a fallback UI using antd `Result`.

#### Scenario: Error boundary catches render error
- **WHEN** a child component throws an error during rendering
- **THEN** the ErrorBoundary displays antd `Result` with:
  - `status="error"`
  - `title="页面出错了"`
  - `subTitle` set to the error message
  - `extra` containing an antd `Button` "刷新页面" that calls `window.location.reload()`

#### Scenario: Error boundary logs errors
- **WHEN** the ErrorBoundary catches an error
- **THEN** the error and component stack SHALL be logged via `console.error`

### Requirement: Notification hook
The system SHALL provide a `useMessage` hook that wraps antd's `App.useApp().notification` for type-safe notification calls with bottom-right placement.

#### Scenario: Success notification
- **WHEN** `showMessage.success("保存成功")` is called from a component within `<App>`
- **THEN** a dark-themed success notification appears at bottom-right and auto-dismisses after 2 seconds

#### Scenario: Error notification
- **WHEN** `showMessage.error("操作失败")` is called
- **THEN** a dark-themed error notification appears at bottom-right and auto-dismisses after 3 seconds

#### Scenario: Info notification
- **WHEN** `showMessage.info("提示信息")` is called
- **THEN** a dark-themed info notification appears at bottom-right and auto-dismisses after 2 seconds
