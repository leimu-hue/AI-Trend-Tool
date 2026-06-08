# Design Token System

## Purpose

Map prototype design tokens from `docs/Live-Artifact/` to Ant Design theme tokens and Tailwind CSS v4 configuration, ensuring all UI components match the prototype visual identity.

## Requirements

### Requirement: Ant Design theme token overrides from prototype
The system SHALL provide `src/theme/tokens.ts` exporting an Ant Design `ThemeConfig` object that maps prototype design tokens to antd theme tokens using `theme.darkAlgorithm` as the base algorithm.

#### Scenario: Color tokens override antd defaults
- **WHEN** `ConfigProvider` wraps the app with the custom theme config
- **THEN** antd components render with `colorBgBase: 'var(--color-bg)'`, `colorBgContainer: 'var(--color-surface)'`, `colorText: 'var(--color-fg)'`, `colorTextSecondary: 'var(--color-fg-2)'`, `colorTextTertiary: 'var(--color-muted)'`, and `colorBorder: 'var(--color-border)'`, resolving to prototype values via CSS custom properties from the Tailwind `@theme` block

#### Scenario: Border radius and font family overrides
- **WHEN** `ConfigProvider` wraps the app with the custom theme config
- **THEN** antd components use `borderRadius: 12` (--radius-md), `borderRadiusLG: 14` (--radius-lg), `borderRadiusSM: 6` (--radius-sm), and `fontFamily: 'Inter, ui-sans-serif, system-ui, sans-serif'`

#### Scenario: Semantic color tokens
- **WHEN** antd components render with success/warning/error states
- **THEN** `colorSuccess: '#16a34a'`, `colorWarning: '#eab308'`, `colorError: '#dc2626'` SHALL be applied

### Requirement: Tailwind CSS v4 configuration with prototype tokens
The system SHALL configure Tailwind CSS v4 via `@theme` block in `src/styles/index.css`, mapping prototype design tokens as CSS custom properties for use as Tailwind utilities.

#### Scenario: Tailwind v4 CSS-based config
- **WHEN** the project is built
- **THEN** `src/styles/index.css` SHALL contain an `@theme` block defining `--color-*`, `--font-family-*`, and `--radius-*` custom properties matching prototype values, and `@tailwindcss/vite` plugin SHALL be registered in `electron.vite.config.ts`

#### Scenario: Color utilities from prototype
- **WHEN** a component uses Tailwind classes `bg-bg`, `bg-surface`, `text-fg`, `text-fg-2`, `text-muted`, `text-meta`, `border-border`, `bg-accent`, `text-accent-on`, `text-success`, `text-warn`, `text-danger`
- **THEN** the correct prototype color values SHALL be applied

#### Scenario: Font family utilities
- **WHEN** a component uses `font-display`, `font-body`, or `font-mono` classes
- **THEN** the correct font stack from the prototype SHALL be applied

#### Scenario: Border radius utilities
- **WHEN** a component uses `rounded-sm`, `rounded-md`, `rounded-lg`, or `rounded-pill` classes
- **THEN** the correct border radius values (6px, 12px, 14px, 9999px) SHALL be applied

#### Scenario: No Tailwind preflight conflict
- **WHEN** Tailwind CSS v4 processes styles
- **THEN** no CSS reset conflicts SHALL occur with antd's base styles (Tailwind v4 removes the aggressive preflight by default)

### Requirement: Global CSS entry point
The system SHALL provide `src/styles/index.css` that contains the Tailwind v4 `@theme` block with prototype design tokens, imports Tailwind layers, and includes only minimal overrides necessary for Electron quirks.

#### Scenario: Tailwind layers are available
- **WHEN** `index.css` is imported in `main.tsx`
- **THEN** all Tailwind v4 utility classes and `@theme` custom properties are available in components

#### Scenario: No component base classes defined
- **WHEN** the CSS is inspected
- **THEN** there SHALL be NO manually defined `.btn`, `.panel`, `.table`, `.badge`, `.modal`, `.field`, or `.toast` classes — component styling SHALL use antd components or Tailwind utilities

### Requirement: ConfigProvider wraps application root
The system SHALL create `src/theme/config.tsx` exporting a `<ThemeProvider>` wrapper that combines ConfigProvider with `theme.darkAlgorithm`, custom token overrides, and the antd `App` component for message/modal/notification context.

#### Scenario: App wrapped with theme
- **WHEN** the application renders
- **THEN** `<ThemeProvider>` SHALL be the outermost wrapper in `main.tsx`, containing `ConfigProvider` with dark algorithm + custom tokens, wrapping antd `<App>` which wraps `<BrowserRouter>`
