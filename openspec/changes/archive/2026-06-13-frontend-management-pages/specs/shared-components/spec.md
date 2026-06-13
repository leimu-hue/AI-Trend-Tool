# Shared Components — Delta

## ADDED Requirements

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
