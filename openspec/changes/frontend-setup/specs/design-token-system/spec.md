## ADDED Requirements

### Requirement: Ant Design theme token overrides from prototype
The system SHALL provide `src/theme/tokens.ts` exporting an Ant Design `ThemeConfig` object that maps prototype design tokens to antd theme tokens using `theme.darkAlgorithm` as the base algorithm.

#### Scenario: Color tokens override antd defaults
- **WHEN** `ConfigProvider` wraps the app with the custom theme config
- **THEN** antd components render with `colorBgBase: '#161412'` (--bg), `colorBgContainer: '#1f1d1b'` (--surface), `colorText: '#faf9f6'` (--fg), `colorTextSecondary: '#afaeac'` (--fg-2), `colorTextTertiary: '#868584'` (--muted), and `colorBorder: 'rgba(226, 226, 226, 0.35)'` (--border)

#### Scenario: Border radius and font family overrides
- **WHEN** `ConfigProvider` wraps the app with the custom theme config
- **THEN** antd components use `borderRadius: 12` (--radius-md), `borderRadiusLG: 14` (--radius-lg), `borderRadiusSM: 6` (--radius-sm), and `fontFamily: 'Inter, ui-sans-serif, system-ui, sans-serif'`

#### Scenario: Semantic color tokens
- **WHEN** antd components render with success/warning/error states
- **THEN** `colorSuccess: '#16a34a'`, `colorWarning: '#eab308'`, `colorError: '#dc2626'` SHALL be applied

### Requirement: Tailwind CSS configuration with prototype tokens
The system SHALL provide `tailwind.config.ts` that extends the default Tailwind theme with prototype design tokens as named utilities.

#### Scenario: Color utilities from prototype
- **WHEN** a component uses Tailwind classes `bg-bg`, `bg-surface`, `text-fg`, `text-fg-2`, `text-muted`, `text-meta`, `border-border`, `bg-accent`, `text-accent-on`, `text-success`, `text-warn`, `text-danger`
- **THEN** the correct prototype color values SHALL be applied

#### Scenario: Font family utilities
- **WHEN** a component uses `font-display`, `font-body`, or `font-mono` classes
- **THEN** the correct font stack from the prototype SHALL be applied

#### Scenario: Border radius utilities
- **WHEN** a component uses `rounded-sm`, `rounded-md`, `rounded-lg`, or `rounded-pill` classes
- **THEN** the correct border radius values (6px, 12px, 14px, 9999px) SHALL be applied

#### Scenario: Tailwind preflight disabled
- **WHEN** Tailwind CSS processes styles
- **THEN** `corePlugins.preflight` SHALL be `false` to prevent CSS reset conflicts with antd's base styles

### Requirement: Global CSS entry point
The system SHALL provide `src/styles/index.css` that imports Tailwind directives (`@tailwind base/components/utilities`) and includes only minimal overrides necessary for Electron quirks.

#### Scenario: Tailwind layers are available
- **WHEN** `index.css` is imported in `main.tsx`
- **THEN** all Tailwind utility classes are available in components

#### Scenario: No component base classes defined
- **WHEN** the CSS is inspected
- **THEN** there SHALL be NO manually defined `.btn`, `.panel`, `.table`, `.badge`, `.modal`, `.field`, or `.toast` classes — component styling SHALL use antd components or Tailwind utilities

### Requirement: ConfigProvider wraps application root
The system SHALL create `src/theme/config.tsx` exporting a `<ThemeProvider>` wrapper that combines ConfigProvider with `theme.darkAlgorithm`, custom token overrides, and the antd `App` component for message/modal/notification context.

#### Scenario: App wrapped with theme
- **WHEN** the application renders
- **THEN** `<ThemeProvider>` SHALL be the outermost wrapper in `main.tsx`, containing `ConfigProvider` with dark algorithm + custom tokens, wrapping antd `<App>` which wraps `<BrowserRouter>`
