## Why

The TrendAITool backend (steps 01–05) exposes a full REST API but has no graphical interface. Operators need a desktop application to manage RSS data sources, keywords, push channels, view hotspot dashboards, and authenticate via API tokens. This is the first frontend deliverable — establishing the project scaffold, visual foundation, authentication flow, and navigation structure that all subsequent feature pages will build upon.

## What Changes

- Create `web/` directory with **Electron** + **Vite** + **React 19** + **TypeScript** project scaffold
- Enable **React Compiler** (babel-plugin-react-compiler) for automatic memoization — eliminates manual `useMemo`/`useCallback`/`React.memo` in most cases
- Integrate Electron as cross-platform desktop shell (Windows, macOS, Linux) with secure defaults (`contextIsolation: true`, `nodeIntegration: false`)
- Extract design tokens from `docs/Live-Artifact/index.html` into Ant Design theme tokens (`theme.darkAlgorithm` + custom token overrides) and Tailwind CSS config
- Build UI with **Ant Design 5** components (Button, Card, Table, Tag, Modal, Form, Layout, message) styled via dark ConfigProvider, plus **Tailwind CSS** for layout/spacing/utilities
- Implement Axios API client with automatic Bearer token injection and 401 interception
- Build Token authentication page (dark-themed centered card, token validation, localStorage persistence)
- Implement app layout with collapsible sidebar navigation, topbar, and responsive breakpoints
- Create shared components: `Loading`, `Empty`, `ErrorBoundary` (leveraging antd Spin, Empty, Result)
- Wire routing with protected routes (redirect to `/auth` when no token)
- Stub all section pages (Dashboard, Sources, Keywords, Channels, Articles, Settings/Tokens)

## Capabilities

### New Capabilities

- `frontend-project-scaffold`: Electron + Vite + React 19 + TypeScript project initialization with React Compiler, dev/build scripts, and secure Electron main process
- `design-token-system`: Ant Design theme token overrides (darkAlgorithm + prototype colors/borderRadius/fontFamily) + Tailwind CSS config extending prototype design tokens — ensures all components match prototype visual identity
- `api-client-layer`: Axios instance with request interceptor (Bearer token from localStorage), response interceptor (401 redirect, error toast), and per-domain API modules (tokens, sources, keywords, channels, queries)
- `auth-page`: Token authentication page — fullscreen dark background, centered card, token input with validation, localStorage persistence, auto-redirect to dashboard on success
- `app-layout`: Sidebar + topbar + content area layout matching prototype — brand/logo area, nav sections (监控/配置/系统), LIVE status footer, responsive collapse at 768px
- `shared-components`: Loading state (antd Spin wrapper), Empty state (antd Empty wrapper with optional action), ErrorBoundary class component with reload

### Modified Capabilities

_None_ — all existing specs are backend capabilities. This change introduces only new frontend capabilities.

## Impact

- **New directory**: `web/` (Electron + Vite + React project, fully independent from Rust backend)
- **Dependencies added**: react 19, react-dom 19, react-router-dom, axios, echarts, echarts-for-react, dayjs, antd, @ant-design/icons, electron, electron-builder, vite, @vitejs/plugin-react, babel-plugin-react-compiler, tailwindcss, @tailwindcss/vite, typescript
- **Build artifacts**: Electron app packaged via electron-builder for Windows/macOS/Linux
- **Backend**: No changes — consumes existing API at `http://localhost:8080/api/v1`
- **No database changes**
