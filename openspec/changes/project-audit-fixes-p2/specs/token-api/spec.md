## ADDED Requirements

### Requirement: Tokens 页面使用 IPC clipboard API

前端 Tokens 管理页面的复制功能 SHALL 使用 preload 脚本暴露的 `window.electronAPI.clipboard.writeText()` IPC 桥接，而不是已废弃的 `document.execCommand('copy')`。

#### Scenario: 复制 token 到剪贴板

- **WHEN** 用户在 Tokens 页面点击复制按钮
- **THEN** 系统 SHALL 调用 `window.electronAPI.clipboard.writeText(tokenString)`
- **THEN** 系统 SHALL NOT 使用 `document.execCommand('copy')`
- **THEN** 复制成功 SHALL 显示提示信息
