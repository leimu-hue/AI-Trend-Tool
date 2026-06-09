# Design Token System — Delta

## MODIFIED Requirements

### Requirement: Global CSS entry point
The system SHALL provide `src/styles/index.css` that contains the Tailwind v4 `@theme` block with prototype design tokens, imports Tailwind layers, includes minimal overrides for Electron quirks, and includes custom CSS component classes for management pages.

#### Scenario: Tailwind layers are available
- **WHEN** `index.css` is imported in `main.tsx`
- **THEN** all Tailwind v4 utility classes and `@theme` custom properties are available in components

#### Scenario: Component base classes defined
- **WHEN** the CSS is inspected
- **THEN** the following custom component CSS classes SHALL be defined: `.panel` / `.panel-header` / `.panel-title` (panel layout), `.btn` / `.btn-primary` / `.btn-ghost` / `.btn-sm` / `.btn-danger` (button variants), `.badge` / `.badge-success` / `.badge-danger` / `.badge-neutral` (status badges), `.modal-overlay` / `.modal` / `.modal-actions` (modal dialog), `.table-wrap` / `table` (table styles), `.field` / `.field-help` (form fields), `.settings-grid` / `.settings-group` / `.setting-row` / `.setting-label` / `.setting-value` (settings layout), `.toast-container` / `.toast` / `.toast-success` / `.toast-error` / `.toast-info` (toast notifications). All classes SHALL reference design token CSS custom properties (`var(--fg)`, `var(--surface)`, `var(--border)`, etc.) for color values.
