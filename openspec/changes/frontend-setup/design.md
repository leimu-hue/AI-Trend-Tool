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

**Tailwind config** maps prototype design tokens:
```js
// tailwind.config.ts — prototype tokens as Tailwind extensions
{
  theme: {
    extend: {
      colors: {
        bg: '#161412',
        surface: '#1f1d1b',
        fg: '#faf9f6',
        'fg-2': '#afaeac',
        muted: '#868584',
        meta: '#666469',
        border: 'rgba(226, 226, 226, 0.35)',
        accent: '#353534',
        'accent-on': '#afaeac',
        'accent-hover': '#454545',
        success: '#16a34a',
        warn: '#eab308',
        danger: '#dc2626',
      },
      fontFamily: {
        display: ['Inter', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        body: ['Inter', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        mono: ['ui-monospace', 'SF Mono', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
      },
      borderRadius: {
        sm: '6px',
        md: '12px',
        lg: '14px',
        pill: '9999px',
      },
    },
  },
}
```

Ant Design theme token overrides map prototype colors into antd's token system:
```ts
// theme/tokens.ts — prototype → antd token mapping
import type { ThemeConfig } from 'antd';

export const themeConfig: ThemeConfig = {
  algorithm: theme.darkAlgorithm,
  token: {
    colorBgBase: '#161412',        // --bg
    colorBgContainer: '#1f1d1b',   // --surface
    colorText: '#faf9f6',          // --fg
    colorTextSecondary: '#afaeac', // --fg-2
    colorTextTertiary: '#868584',  // --muted
    colorBorder: 'rgba(226, 226, 226, 0.35)', // --border
    colorPrimary: '#afaeac',       // --accent-on
    fontFamily: 'Inter, ui-sans-serif, system-ui, sans-serif',
    borderRadius: 12,              // --radius-md
    borderRadiusLG: 14,            // --radius-lg
    borderRadiusSM: 6,             // --radius-sm
  },
};
```

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

### D7: Toast / Notifications → antd `message` API

**Choice:** Use antd's built-in `message` API for toast notifications (via `App.useApp()` or static `message.success/error/info`).

**Rationale:**
- antd already ships with a dark-theme-aware notification system — no need for custom Toast context
- `message.success(msg)`, `message.error(msg)` match prototype toast behavior with 2-3s auto-dismiss
- `App.useApp()` pattern (antd 5 recommended) gives access to `message`, `modal`, `notification` instances that respect ConfigProvider theme
- Eliminates custom `ToastProvider` + `useToast` code entirely

**Usage pattern:**
```tsx
// In components/hooks — thin wrapper for ergonomics
import { App } from 'antd';

export function useMessage() {
  const { message } = App.useApp();
  return {
    success: (msg: string) => message.success(msg, 2),
    error: (msg: string) => message.error(msg, 3),
    info: (msg: string) => message.info(msg, 2),
  };
}
```

The API client's response interceptor also uses `App.useApp()` message — or falls back to static import for non-component usage (`message.error(msg)` outside React tree). For the interceptor (which runs outside React), static `message` import from `antd` works since antd 5 holds a singleton.

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
