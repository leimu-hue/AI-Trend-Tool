# App Layout

## Purpose

Provide a sidebar + topbar + content area layout matching the prototype, with collapsible responsive behavior at 768px breakpoint and route-based page titles.

## Requirements

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

### Requirement: Topbar header
The system SHALL render a topbar above the main content area displaying the current page title, a BETA badge, a hamburger menu button (mobile), and the current UTC timestamp.

#### Scenario: Page title from route
- **WHEN** user navigates to `/sources`
- **THEN** the topbar displays "数据源管理" as the page title

#### Scenario: Hamburger menu on mobile
- **WHEN** viewport is ≤ 768px
- **THEN** a "☰" hamburger button appears in the topbar and toggles sidebar visibility when clicked

#### Scenario: UTC timestamp updates
- **WHEN** the layout renders
- **THEN** the topbar right section displays the current UTC timestamp in `YYYY-MM-DD HH:MM:SS` format with `var(--font-mono)` font at 11px

### Requirement: Main content area
The system SHALL render the routed page content inside a scrollable main area via React Router's `<Outlet />`.

#### Scenario: Outlet renders child routes
- **WHEN** navigating between `/dashboard`, `/sources`, `/keywords`, etc.
- **THEN** the corresponding page component renders inside the `<main className="main-content">` container

### Requirement: Responsive behavior
The system SHALL adapt the sidebar behavior at the 768px breakpoint for mobile devices and close the sidebar automatically when resizing above 768px.

#### Scenario: Sidebar hidden on mobile by default
- **WHEN** viewport is ≤ 768px
- **THEN** the sidebar SHALL be positioned fixed and translated off-screen (`transform: translateX(-100%)`)

#### Scenario: Sidebar visible when toggled on mobile
- **WHEN** user taps the hamburger menu on mobile
- **THEN** the sidebar SHALL slide in (`transform: translateX(0)`) with the `.open` class

#### Scenario: Sidebar resets on resize above mobile
- **WHEN** viewport resizes from ≤ 768px to > 768px
- **THEN** the sidebar SHALL return to its default visible position and the mobile menu state SHALL reset
