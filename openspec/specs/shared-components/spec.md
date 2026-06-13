# Shared Components

## Purpose

Provide reusable UI components for loading, empty states, error boundaries, and toast notifications — wrapping antd base components with project-specific defaults. Includes a custom CSS Toast notification system (useToast hook) for management pages.

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

### Requirement: Toast notification hook (custom CSS)
系统 SHALL 提供基于自定义 CSS 的 Toast 通知系统（`useToast` hook + `ToastProvider`），支持 success / error / info 三种类型，不依赖 antd。

#### Scenario: ToastProvider wraps app root
- **WHEN** `<ToastProvider>` 包裹应用根组件
- **THEN** 所有子组件可通过 `useToast()` hook 触发通知

#### Scenario: Success toast
- **WHEN** 组件调用 `toast.success("操作成功")`
- **THEN** 底部居中显示绿色 `.toast-success` 气泡，3 秒后自动消失

#### Scenario: Error toast
- **WHEN** 组件调用 `toast.error("操作失败")`
- **THEN** 底部居中显示红色 `.toast-error` 气泡，3 秒后自动消失

#### Scenario: Info toast
- **WHEN** 组件调用 `toast.info("已复制")`
- **THEN** 底部居中显示中性 `.toast-info` 气泡，2 秒后自动消失

#### Scenario: Multiple toasts stack
- **WHEN** 连续触发多个 toast
- **THEN** 它们垂直堆叠显示，每个独立计时消失

#### Scenario: Toast renders outside React tree context
- **WHEN** `useToast` 在 `ToastProvider` 外调用
- **THEN** 静默失败，不抛出异常，不显示 toast
