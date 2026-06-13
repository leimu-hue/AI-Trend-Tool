## MODIFIED Requirements

### Requirement: Electron security defaults

The system SHALL expose only the minimum necessary APIs via `contextBridge` in the preload script. Unused APIs SHALL be removed to follow the principle of least privilege.

#### Scenario: preload 不暴露未使用的 clipboard.readText

- **WHEN** preload 脚本暴露 `electronAPI` 对象
- **THEN** `clipboard.readText` SHALL NOT be exposed
- **THEN** 仅 `clipboard.writeText` SHALL 保留
- **THEN** main process 中对应的 `ipcMain.handle('clipboard:read', ...)` SHALL 被移除

#### Scenario: Electron security defaults
- **WHEN** the Electron main process creates a BrowserWindow
- **THEN** `nodeIntegration` SHALL be `false`, `contextIsolation` SHALL be `true`, and `sandbox` SHALL be `true`
