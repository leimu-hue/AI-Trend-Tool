## ADDED Requirements

### Requirement: Electron + Vite + React 19 project scaffold
The system SHALL provide a working Electron desktop application project at `web/` using Vite as the bundler, React 19 with TypeScript, and `electron-vite` for unified main/preload/renderer build configuration.

#### Scenario: Project compiles without errors
- **WHEN** user runs `cd frontend && npm install && npm run dev`
- **THEN** the Electron window opens displaying the React application without TypeScript or build errors

#### Scenario: Production build succeeds
- **WHEN** user runs `cd frontend && npm run build`
- **THEN** the project produces distributable Electron application files in `web/dist/`

#### Scenario: React Compiler auto-memoizes
- **WHEN** a component renders with derived values that would normally require `useMemo`
- **THEN** the React Compiler automatically memoizes the computation without explicit `useMemo` calls

#### Scenario: Electron security defaults
- **WHEN** the Electron main process creates a BrowserWindow
- **THEN** `nodeIntegration` SHALL be `false`, `contextIsolation` SHALL be `true`, and `sandbox` SHALL be `true`

### Requirement: Design token CSS system
The system SHALL provide CSS custom properties (design tokens) in `src/styles/tokens.css` extracted from `docs/Live-Artifact/index.html`, including colors, fonts, font sizes, border radii, shadows, motion curves, layout widths, and spacing scale.

#### Scenario: All prototype colors available as CSS variables
- **WHEN** a component uses `var(--bg)`, `var(--surface)`, `var(--fg)`, `var(--accent)`, `var(--danger)`, `var(--success)`, or `var(--warn)`
- **THEN** the correct prototype color value is applied

#### Scenario: Font scale matches prototype
- **WHEN** a component uses `var(--text-xs)` through `var(--text-2xl)`
- **THEN** the font sizes match `12px`, `14px`, `18px`, `24px`, `32px`, `48px` respectively

#### Scenario: Global CSS provides base component classes
- **WHEN** `global.css` is loaded
- **THEN** `.btn`, `.btn-primary`, `.btn-ghost`, `.btn-danger`, `.panel`, `.panel-header`, `table`, `.badge`, `.badge-success`, `.badge-warn`, `.badge-danger`, `.modal`, `.modal-overlay`, `.field`, `.toast`, and `.truncate` classes are available with prototype-matching styles

### Requirement: API client with token interceptors
The system SHALL provide an Axios instance in `src/api/client.ts` that automatically attaches the Bearer token from localStorage to all requests and handles 401 responses by clearing the token and redirecting to `/auth`.

#### Scenario: Token automatically attached to requests
- **WHEN** a valid API token exists in `localStorage` under key `api_token`
- **THEN** every outgoing request includes the header `Authorization: Bearer <token>`

#### Scenario: 401 response clears token and redirects
- **WHEN** any API request receives a 401 HTTP response
- **THEN** the token SHALL be removed from localStorage and the user SHALL be redirected to `/auth`

#### Scenario: Network error shows friendly message
- **WHEN** an API request fails due to a network error (no response received)
- **THEN** a toast notification displays "网络错误，请检查后端服务是否启动"

#### Scenario: Per-domain API modules export typed functions
- **WHEN** a page imports `tokenApi.list()` from `src/api/tokens.ts`
- **THEN** it returns a typed `Promise<TokenInfo[]>` using the shared Axios client

### Requirement: Token authentication page
The system SHALL provide a dark-themed authentication page at `/auth` where users enter an API token, validate it against the backend, and proceed to the dashboard on success.

#### Scenario: Auth page renders fullscreen dark card
- **WHEN** user navigates to `/auth` with no valid token
- **THEN** a full-page dark background (`var(--bg)`) displays a centered card (`var(--surface)`) with title "AI 热点监控系统", token input field, and "验证并进入" button

#### Scenario: Valid token navigates to dashboard
- **WHEN** user enters a valid API token and clicks "验证并进入"
- **THEN** the token is saved to `localStorage`, API call to `GET /tokens` succeeds, and user is navigated to `/dashboard`

#### Scenario: Invalid token shows error
- **WHEN** user enters an invalid or expired token
- **THEN** an error message "Token 无效或已过期" is displayed in a red alert box, and the token is NOT saved to localStorage

#### Scenario: Enter key submits form
- **WHEN** user presses Enter in the token input field
- **THEN** the form submits as if "验证并进入" was clicked

### Requirement: App layout with sidebar navigation
The system SHALL provide a layout component matching the prototype's sidebar + topbar + content area structure, with collapsible responsive behavior at 768px breakpoint.

#### Scenario: Sidebar displays navigation groups
- **WHEN** user is authenticated and on any page
- **THEN** the sidebar shows the brand logo area, navigation sections "监控" (仪表盘, 文章日志), "配置" (数据源, 关键词, 推送渠道, API 令牌), and "系统" (设置), plus a footer with LIVE status indicator

#### Scenario: Active nav item highlighted
- **WHEN** user is on `/sources`
- **THEN** the "数据源" nav button has the `active` class and distinct styling

#### Scenario: Sidebar collapses on mobile
- **WHEN** viewport width is ≤ 768px
- **THEN** the sidebar is hidden and a hamburger menu button appears in the topbar, toggling sidebar visibility

#### Scenario: Topbar shows page title and timestamp
- **WHEN** user is on any authenticated page
- **THEN** the topbar displays the current page title, a "BETA" badge, and the current UTC timestamp

### Requirement: Shared UI components
The system SHALL provide reusable components for loading, empty states, error boundaries, and toast notifications.

#### Scenario: Loading component renders spinning indicator
- **WHEN** `Loading` component is rendered
- **THEN** a `.live-dot` animated indicator and "加载中..." text are displayed, optionally full-page when `fullPage` prop is true

#### Scenario: Empty component shows message and optional action
- **WHEN** `Empty` component is rendered with `description="暂无数据源"` and `actionText="添加数据源"`
- **THEN** the description text displays with an "添加数据源" button that calls `onAction` when clicked

#### Scenario: ErrorBoundary catches render errors
- **WHEN** a child component throws an error during rendering
- **THEN** the ErrorBoundary displays "页面出错了" with the error message and a "刷新页面" button that reloads the page

#### Scenario: Toast notification appears and auto-dismisses
- **WHEN** `showToast("操作成功")` is called
- **THEN** a toast notification appears at the bottom-right corner displaying "操作成功" and automatically disappears after 2 seconds
