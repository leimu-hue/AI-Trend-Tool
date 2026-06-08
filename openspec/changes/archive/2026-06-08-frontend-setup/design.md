## Context

The TrendAITool backend is a Rust/Axum HTTP server on `localhost:8080`. The frontend must be a desktop application that consumes this API. The prototype in `docs/Live-Artifact/index.html` defines the visual contract: dark theme, sidebar layout, specific CSS variables.

User additional requirements beyond the original design doc:
1. **React 19** with **React Compiler** — auto-memoization eliminates manual `useMemo`/`useCallback`/`React.memo`
2. **Electron** as cross-platform desktop framework (not just a web app)

## Goals / Non-Goals

**Goals:**
- Electron desktop app with React 19 frontend, TypeScript, Vite bundler
- React Compiler enabled for automatic memoization
- Secure Electron defaults: `contextIsolation: true`, `nodeIntegration: false`, minimal preload API
- Ant Design 5 component library with dark theme ConfigProvider, Tailwind CSS for utility styling
- Design tokens from `docs/Live-Artifact/` prototype mapped to antd theme tokens + Tailwind config
- Token-based auth page, sidebar layout, routing with protected routes
- Axios API client with interceptors for all backend endpoints
- Shared components: Loading, Empty, ErrorBoundary (leveraging antd base components)

**Non-Goals:**
- Page implementations (Dashboard, Sources, Keywords, etc.) — only stubs in this change
- Electron auto-update, code signing, installer configuration
- Native OS integrations (tray, notifications, file system access)
- State management library (Zustand, Redux) — not needed for this scaffold
- End-to-end tests

## Decisions

### D1: Electron + Vite integration → `electron-vite` package

**Choice:** Use `electron-vite` (npm package) to scaffold the project.

**Rationale:**
- `electron-vite` provides a unified Vite config for main, preload, and renderer processes
- Handles HMR for renderer, auto-restart for main process changes
- Generates correct `esm` → `cjs` conversion for main/preload (Electron still requires CommonJS)
- Actively maintained, ~5k GitHub stars, used by electron-vite-react template

**Alternatives considered:**
- `electron-forge` + Vite plugin: more complex config, heavier
- Manual webpack/vite setup: error-prone, needs deep Electron bundling knowledge
- `electron-builder` alone: only packages, doesn't handle dev workflow

### D2: React Compiler → `babel-plugin-react-compiler`

**Choice:** Use `babel-plugin-react-compiler` with `@vitejs/plugin-react` (Babel mode).

**Rationale:**
- React 19 stable includes the compiler opt-in via Babel plugin
- Vite's React plugin supports Babel transforms — add the compiler plugin to the Babel config
- With compiler enabled, `useMemo`, `useCallback`, `React.memo` become unnecessary for most cases — the compiler infers memoization automatically
- Components must follow React rules (no side effects in render) — compiler enforces this

**Configuration approach:**
```ts
// vite.config.ts
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: [['babel-plugin-react-compiler', { target: '19' }]],
      },
    }),
  ],
});
```

### D3: Electron Security Architecture

**Mandatory defaults (from electron skill constraints):**
- `nodeIntegration: false` — renderer has no Node.js access
- `contextIsolation: true` — preload script runs in isolated context
- `sandbox: true` — renderer sandboxed
- `webSecurity: true` — CORS and content security enforced

**Preload pattern:** `contextBridge.exposeInMainWorld()` exposes a minimal `electronAPI` object. For now, most communication uses HTTP to the local backend. IPC is reserved for native features (future: file dialogs, notifications).

**CSP header:**
```
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self' http://localhost:8080
```

### D4: Component Library → Ant Design 5 + Tailwind CSS

**Choice:** Use Ant Design 5 (antd) with `ConfigProvider` dark theme + Tailwind CSS v4 for utility styling.

**Rationale:**
- Ant Design 5 has first-class dark theme support via `ConfigProvider` with `theme.algorithm` — `theme.darkAlgorithm` provides a complete dark token set out of the box
- Prototype CSS component classes map naturally to antd components: `.btn` → `<Button>`, `.panel` → `<Card>`, `table` → `<Table>`, `.badge` → `<Tag>`, `.modal` → `<Modal>`, `.field` → `<Form.Item>`, `.toast` → `message.success()`
- Tailwind CSS handles layout, spacing, typography, and one-off styles that don't warrant custom CSS — eliminates maintaining a `global.css` component library
- Custom design tokens from prototype override antd's default theme tokens: colors, border radius, font family — prototype's visual identity survives
- Component library handles accessibility, keyboard navigation, focus rings, and ARIA out of the box

**Alternatives considered:**
- Custom CSS only: high maintenance burden, no a11y guarantees, reinventing wheels
- MUI: heavier, Material Design aesthetic conflicts with prototype
- Radix UI + Tailwind: headless components need more wiring, no unified theme system
- shadcn/ui: requires manual component copying, no dark theme engine

**Theme architecture:**
```
src/
├── theme/
│   ├── tokens.ts       # Ant Design theme token overrides (colors, radius, font) from prototype
│   └── config.ts       # ConfigProvider wrapper with darkAlgorithm + custom tokens
├── styles/
│   └── index.css       # Tailwind directives (@tailwind base/components/utilities) + minimal reset overrides
```

**Tailwind v4 CSS-based config** maps prototype design tokens via `@theme` in `src/styles/index.css`:
```css
/* src/styles/index.css — prototype tokens as Tailwind v4 @theme */
@import 'tailwindcss/utilities';
@import 'tailwindcss/theme' layer(theme);

@theme {
  --color-bg: #161412;
  --color-surface: #1f1d1b;
  --color-fg: #faf9f6;
  --color-fg-2: #afaeac;
  --color-muted: #868584;
  --color-meta: #666469;
  --color-border: rgba(226, 226, 226, 0.35);
  --color-accent: #353534;
  --color-accent-on: #afaeac;
  --color-accent-hover: #454545;
  --color-success: #16a34a;
  --color-warn: #eab308;
  --color-danger: #dc2626;
  --font-family-display: 'Inter', ui-sans-serif, system-ui, sans-serif;
  --font-family-body: 'Inter', ui-sans-serif, system-ui, sans-serif;
  --font-family-mono: ui-monospace, 'SF Mono', Menlo, Monaco, Consolas, monospace;
  --radius-sm: 6px;
  --radius-md: 12px;
  --radius-lg: 14px;
  --radius-pill: 9999px;
}
```

Tailwind v4 removes the need for a JS config file — `@theme` defines design tokens directly in CSS. No `preflight: false` needed (v4 dropped the aggressive reset). The `@tailwindcss/vite` plugin handles processing.

Ant Design theme token overrides map prototype colors into antd's token system. Tokens reference Tailwind v4 `@theme` CSS custom properties via `var()` instead of hardcoded hex values. This allows antd's cssinjs output to resolve against the same `@theme` block defined in `index.css`, keeping antd components and Tailwind utilities in sync from a single source of truth:

```ts
// theme/tokens.ts — prototype → antd token mapping (CSS variable references)
import type { ThemeConfig } from 'antd';

export const themeTokens: ThemeConfig = {
  token: {
    colorBgBase: 'var(--color-bg)',
    colorBgContainer: 'var(--color-surface)',
    colorBgElevated: 'var(--color-surface)',
    colorText: 'var(--color-fg)',
    colorTextSecondary: 'var(--color-fg-2)',
    colorTextTertiary: 'var(--color-muted)',
    colorTextQuaternary: 'var(--color-meta)',
    colorBorder: 'var(--color-border)',
    colorBorderSecondary: 'var(--color-border)',
    colorPrimary: 'var(--color-accent-on)',
    colorPrimaryBg: 'var(--color-accent)',
    colorPrimaryBgHover: 'var(--color-accent-hover)',
    colorSuccess: 'var(--color-success)',
    colorWarning: 'var(--color-warn)',
    colorError: 'var(--color-danger)',
    fontFamily: 'var(--font-family-body)',
    fontSize: 14,
    borderRadius: 12,
    borderRadiusLG: 14,
    borderRadiusSM: 6,
    borderRadiusXS: 4,
    controlHeight: 36,
    lineHeight: 1.5,
    colorLink: 'var(--color-accent-on)',
    colorLinkHover: 'var(--color-fg)',
  },
};
```

The `@theme` block in `src/styles/index.css` remains the single source of truth for all design token values. antd's cssinjs emits these `var()` references as-is into generated CSS, where the browser resolves them from `:root`-scoped custom properties defined by Tailwind.

### D5: Token Storage → localStorage

**Choice:** Store API token in `localStorage` under key `api_token`.

**Rationale:**
- Simple, works across browser and Electron without IPC
- Token is a Bearer token (opaque string), not a JWT with expiry claims client can read
- Revocation is server-side only — client removes on 401

**Security note:** In Electron, `localStorage` is stored in the user's app data directory. This is acceptable for a local monitoring tool. Not suitable for shared/public kiosks (non-goal).

### D6: Routing → React Router v7 (latest)

**Choice:** Use `react-router-dom` v7 (latest compatible with React 19).

**Route structure:**
```
/auth          → AuthPage (public)
/              → ProtectedRoute → AppLayout
  /dashboard   → Dashboard (stub)
  /sources     → Sources (stub)
  /keywords    → Keywords (stub)
  /channels    → Channels (stub)
  /articles    → Articles (stub)
  /tokens      → Settings/Tokens (stub)
  /settings    → Settings (stub)
```

`ProtectedRoute` checks `localStorage.getItem('api_token')` — redirects to `/auth` if missing. No token validation on mount (the API interceptor handles 401).

**Note:** Electron uses `HashRouter` instead of `BrowserRouter` because `file://` protocol does not support HTML5 history API (`pushState`/`replaceState`). Routes become `#/auth`, `#/dashboard`, etc. — functionally identical, only the URL format differs.

### D7: Toast / Notifications → antd `notification` API

**Choice:** Use antd's built-in `notification` API for toast notifications (via `App.useApp()` or static `notification.success/error/info`).

**Rationale:**
- `notification` supports `placement` control (`bottomRight`) — matches prototype toast position better than `message` (which only does top)
- antd already ships with a dark-theme-aware notification system — no need for custom Toast context
- `App.useApp()` pattern (antd 5 recommended) gives access to `message`, `modal`, `notification` instances that respect ConfigProvider theme
- Eliminates custom `ToastProvider` + `useToast` code entirely

**Usage pattern:**
```tsx
// In components/hooks — thin wrapper for ergonomics
import { App } from 'antd';

export function useMessage() {
  const { notification } = App.useApp();
  const notify = (type: 'success' | 'error' | 'info', msg: string, duration: number) => {
    notification[type]({ message: msg, placement: 'bottomRight', duration, closeIcon: false });
  };
  return {
    success: (msg: string) => notify('success', msg, 2),
    error: (msg: string) => notify('error', msg, 3),
    info: (msg: string) => notify('info', msg, 2),
  };
}
```

The API client's response interceptor runs outside the React tree — it uses static `notification` import from `antd` directly, which works since antd 5 holds a singleton.

### D8: UI Design Taste Guardrails

**Choice:** When a UI component or page layout is not fully specified by the prototype (`docs/Live-Artifact/`), consult UI design skills (`design-taste-frontend`, `minimalist-ui`) before writing code. Ant Design provides the component toolbox — design skills guide the composition.

**Rationale:**
- Prototype defines the core visual contract but doesn't cover every state (empty, error, edge cases) or every future page
- Ant Design components are powerful but unopinionated on composition — it's easy to assemble them into generic "enterprise dashboard" layouts that all look identical
- `design-taste-frontend` provides anti-slop guardrails: typography-first, intentional whitespace, honest materials (no gratuitous shadows/gradients), distinctive without being loud
- `minimalist-ui` aligns naturally with the existing dark theme: warm monochrome, typographic contrast, flat bento grids, no heavy gradients

**When to apply:**
- Designing a page or component state the prototype doesn't cover
- Choosing between multiple layout options for a data display (antd Table vs Card grid vs custom list)
- Deciding spacing, typographic hierarchy, or when to break from antd defaults
- Reviewing a finished component — if it looks "like every other SaaS dashboard," iterate with design skill guidance
- Styling with Tailwind: use design skills to decide values, not just pick from config

## Risks / Trade-offs

- **[Risk] Electron + Vite build complexity** → Mitigation: Use `electron-vite` which abstracts main/preload/renderer builds. Test `npm run build` early.
- **[Risk] React Compiler might reject non-idiomatic code** → Mitigation: Compiler runs as a linter pass; errors surface at build time, not runtime. Fix violations as they appear.
- **[Risk] Ant Design dark theme mismatch with prototype** → Mitigation: `themeConfig.token` overrides map prototype colors into antd's token system. Validate auth page and layout against prototype screenshots early. Tailwind `extend.colors` provides escape hatch for values antd doesn't cover.
- **[Risk] Tailwind + antd style conflicts** → Mitigation: Tailwind `preflight: false` (or `corePlugins.preflight: false`) to avoid CSS reset conflicts with antd's base styles. Use Tailwind `important: '#root'` or prefix to control specificity.
- **[Trade-off] antd bundle size (~400KB gzipped)** → Acceptable: desktop Electron app, not a mobile PWA. Component library eliminates months of custom widget maintenance.
- **[Trade-off] localStorage vs cookie/session** → Token persists until explicitly removed or 401. Simpler than cookie management in Electron.
