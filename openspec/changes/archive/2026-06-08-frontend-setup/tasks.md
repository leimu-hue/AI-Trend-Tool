## 1. Project Scaffold

- [x] 1.1 Scaffold Electron + Vite + React 19 + TypeScript project using `electron-vite` (or `npm create @quick-start/electron`) in `web/`
- [x] 1.2 Install dependencies: `react@19`, `react-dom@19`, `react-router-dom`, `axios`, `echarts`, `echarts-for-react`, `dayjs`, `antd`, `@ant-design/icons`, `electron`, `electron-builder`, `electron-vite`
- [x] 1.3 Install dev dependencies: `@vitejs/plugin-react`, `babel-plugin-react-compiler`, `tailwindcss`, `@tailwindcss/vite`, `typescript`, `@types/react`, `@types/react-dom`
- [x] 1.4 Configure `vite.config.ts` with React Compiler Babel plugin (`babel-plugin-react-compiler` with `target: '19'`) and Tailwind CSS Vite plugin (`@tailwindcss/vite`)
- [x] 1.5 Configure Electron main process (`src/main/index.ts`) with security defaults: `nodeIntegration: false`, `contextIsolation: true`, `sandbox: true`
- [x] 1.6 Configure Electron preload script (`src/preload/index.ts`) with `contextBridge.exposeInMainWorld()` for minimal native API
- [x] 1.7 Create `web/.env.development` with `VITE_API_BASE_URL=http://localhost:8080/api/v1`
- [x] 1.8 Verify `npm run dev` launches Electron window without errors
- [x] 1.9 Verify `npm run build` produces production output without TypeScript errors

## 2. Design Tokens & Theme System

- [x] 2.1 Extract design tokens from `docs/Live-Artifact/index.html` `:root` block into `src/theme/tokens.ts` as Ant Design `ThemeConfig` overrides (colors, borderRadius, fontFamily)
- [x] 2.2 Create `src/theme/config.tsx` — `ConfigProvider` wrapper with `theme.darkAlgorithm` + custom token overrides from tokens.ts
- [x] 2.3 Configure Tailwind CSS v4: define `@theme` block in `src/styles/index.css` with `--color-*`, `--font-family-*`, `--radius-*` CSS custom properties mapped from prototype tokens; register `@tailwindcss/vite` plugin in `electron.vite.config.ts`
- [x] 2.4 Create `src/styles/index.css` — Tailwind directives (`@tailwind base/components/utilities`) + minimal reset override for Electron quirks
- [x] 2.5 Verify dark theme: render a test page with antd Button, Card, Table, Tag, Modal — colors match prototype `var(--bg)`, `var(--surface)`, `var(--fg)`, `var(--border)`
- [x] 2.6 Verify Tailwind utilities: `bg-bg`, `text-fg`, `text-muted`, `border-border`, `font-display`, `font-mono`, `rounded-md`, `rounded-lg` classes produce correct prototype values
- [x] 2.7 Ensure antd ConfigProvider wraps the entire app in `main.tsx` (above Router)

## 3. API Client Layer

- [x] 3.1 Create `src/api/client.ts` with Axios instance (baseURL from env, 30s timeout, JSON headers)
- [x] 3.2 Add request interceptor: read token from `localStorage`, set `Authorization: Bearer` header
- [x] 3.3 Add response interceptor: 401 clears token + redirects to `/auth`, server errors show toast, network errors show connection message
- [x] 3.4 Create `src/api/tokens.ts` — `tokenApi.list()`, `tokenApi.create()`, `tokenApi.revoke()` with TypeScript interfaces
- [x] 3.5 Create `src/api/sources.ts` — `sourceApi` with typed CRUD functions (list, create, update, delete)
- [x] 3.6 Create `src/api/keywords.ts` — `keywordApi` with typed CRUD functions
- [x] 3.7 Create `src/api/channels.ts` — `channelApi` with typed CRUD functions
- [x] 3.8 Create `src/api/queries.ts` — `queryApi` with typed query functions (hotspots, articles, stats)

## 4. Shared Components

- [x] 4.1 Create `src/components/Loading.tsx` — wraps antd `Spin` with optional `fullPage` prop, centered layout with "加载中..." text in `var(--muted)`
- [x] 4.2 Create `src/components/Empty.tsx` — wraps antd `Empty` with custom description + optional action button (`actionText` + `onAction` props), matching prototype empty state style
- [x] 4.3 Create `src/components/ErrorBoundary.tsx` — class component with `getDerivedStateFromError`, fallback UI using antd `Result` with error status, error message, and reload button
- [x] 4.4 Create `src/hooks/useMessage.ts` — thin wrapper around `App.useApp().message` for ergonomic `success(msg)`, `error(msg)`, `info(msg)` calls within components
- [x] 4.5 Create `src/hooks/useApi.ts` — generic `useApi<T>(apiFn)` hook returning `{ data, loading, error, execute }`, uses antd `message` for error display via `useMessage`

## 5. Auth Page

- [x] 5.1 Create `src/pages/Auth.tsx` — fullscreen dark background (`bg-bg`), centered antd `Card` with prototype border/radius styling
- [x] 5.2 Implement token input using antd `Input.Password` with Enter key support (`onPressEnter`)
- [x] 5.3 Implement token validation: save to localStorage, call `GET /tokens` via client, navigate to `/dashboard` on success
- [x] 5.4 Implement error display: antd `Alert` type="error" with "Token 无效或已过期" message on validation failure
- [x] 5.5 Implement loading state: antd `Button` with `loading` prop, text "验证中..."
- [x] 5.6 Implement auto-redirect: check localStorage token on mount, navigate to `/dashboard` if present

## 6. App Layout & Routing

- [x] 6.1 Create `src/components/Layout.tsx` — sidebar + topbar + `<Outlet />` structure
- [x] 6.2 Implement sidebar: brand area (logo "◈", name, subtitle), navigation groups (监控/配置/系统), LIVE status footer
- [x] 6.3 Implement navigation items with route mapping, active state highlighting, and click-to-navigate
- [x] 6.4 Implement topbar: hamburger menu button (mobile), page title, BETA badge, UTC timestamp
- [x] 6.5 Implement responsive sidebar: fixed positioned off-screen at ≤768px, slide-in toggle, auto-reset on resize >768px
- [x] 6.6 Create `src/App.tsx` — BrowserRouter + Routes with `ProtectedRoute` wrapper, route-to-title mapping
- [x] 6.7 Create `src/main.tsx` — ReactDOM.createRoot, wrap with antd `App` > ConfigProvider (theme) > ErrorBoundary > BrowserRouter, import `src/styles/index.css`

## 7. Page Stubs

- [x] 7.1 Create `src/pages/Dashboard.tsx` — placeholder page with panel and page title
- [x] 7.2 Create `src/pages/Sources.tsx` — placeholder page with panel and page title
- [x] 7.3 Create `src/pages/Keywords.tsx` — placeholder page with panel and page title
- [x] 7.4 Create `src/pages/Channels.tsx` — placeholder page with panel and page title
- [x] 7.5 Create `src/pages/Tokens.tsx` — placeholder page with panel and page title (导航组：配置)
- [x] 7.6 Create `src/pages/Articles.tsx` — placeholder page with panel and page title
- [x] 7.7 Create `src/pages/Settings.tsx` — placeholder page with panel and page title

## 8. Verification

- [x] 8.1 Run `npm run dev` — Electron window opens, redirects to `/auth`
- [x] 8.2 Enter valid API token — navigates to `/dashboard`, sidebar + topbar render correctly
- [x] 8.3 Navigate all sidebar links — each stub page renders with correct page title in topbar
- [x] 8.4 Test responsive: resize to ≤768px — sidebar collapses, hamburger menu works
- [x] 8.5 Test 401: use invalid/expired token — redirected to `/auth` with error clearing
- [x] 8.6 Run `npm run build` — production build succeeds without errors
- [x] 8.7 Verify React Compiler: `npm run build` includes React Compiler transform (no compiler errors)
