## MODIFIED Requirements

### Requirement: Shared UI components
The system SHALL provide reusable components for loading, empty states, error boundaries, and toast notifications.

#### Scenario: Loading component renders spinning indicator
- **WHEN** `Loading` component is rendered
- **THEN** an antd `Spin` component with "加载中..." text is displayed, optionally full-page when `fullPage` prop is true

#### Scenario: Settings page DEFAULTS matches config.toml
- **WHEN** Settings page loads and API request fails (using DEFAULTS fallback)
- **THEN** `DEFAULTS.parser.max_concurrent_fetches` SHALL be 10
- **THEN** `DEFAULTS.filter.batch_size` SHALL be 1000
- **THEN** `DEFAULTS.server.port` SHALL be 3000
