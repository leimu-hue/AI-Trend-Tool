## MODIFIED Requirements

### Requirement: Sidebar navigation structure

The system SHALL render a fixed-width sidebar (`--sidebar-w: 220px`) containing the brand area, grouped navigation items, and a status footer, exactly matching the prototype structure.

#### Scenario: Brand area

- **WHEN** the layout renders
- **THEN** the sidebar top area displays the logo icon "◈", brand name "AI 热点监控", and subtitle "多源实时监控 · 关键词告警 · 趋势分析"

#### Scenario: Navigation groups

- **WHEN** the layout renders
- **THEN** navigation items are grouped under section headers:
  - "监控": 仪表盘, 文章日志
  - "配置": 数据源, 关键词, 推送渠道, API 令牌
  - "系统": 设置

#### Scenario: Active navigation item

- **WHEN** user is viewing `/sources`
- **THEN** the "数据源" navigation item has the `active` CSS class, visually distinguishing it from inactive items

#### Scenario: Navigation click navigates and closes mobile sidebar

- **WHEN** user clicks a navigation item
- **THEN** the router navigates to the corresponding route AND the sidebar closes if in mobile (open) state

#### Scenario: Sidebar footer

- **WHEN** the layout renders
- **THEN** the sidebar footer displays a `.live-dot` indicator with "LIVE" text and "监控中" status text
- **THEN** the footer SHALL NOT claim any specific auto-refresh interval
