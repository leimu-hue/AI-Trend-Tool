## MODIFIED Requirements

### Requirement: Electron + Vite + React 19 project scaffold
The system SHALL provide a working Electron desktop application project at `web/` using Vite as the bundler, React 19 with TypeScript, and `electron-vite` for unified main/preload/renderer build configuration.

#### Scenario: Project compiles without errors
- **WHEN** user runs `cd web && npm install && npm run dev`
- **THEN** the Electron window opens displaying the React application without TypeScript or build errors

#### Scenario: Production build succeeds
- **WHEN** user runs `cd web && npm run build`
- **THEN** the project produces distributable Electron application files in `web/dist/`

#### Scenario: React Compiler auto-memoizes
- **WHEN** a component renders with derived values that would normally require `useMemo`
- **THEN** the React Compiler automatically memoizes the computation without explicit `useMemo` calls

#### Scenario: Electron security defaults
- **WHEN** the Electron main process creates a BrowserWindow
- **THEN** `nodeIntegration` SHALL be `false`, `contextIsolation` SHALL be `true`, and `sandbox` SHALL be `true`

#### Scenario: 生产环境 CSP 使用端口通配符
- **WHEN** 应用以生产模式构建
- **THEN** CSP `connect-src` SHALL 为 `'self' http://localhost:*`
- **THEN** 允许连接到 localhost 的任意端口，匹配用户配置的后端端口
