## Why

The project currently has token authentication (step 03) but no operational APIs to manage data sources, keywords, or push channels — the three core domain entities. Without these CRUD APIs, users cannot configure RSS feeds to monitor, define hotspot keywords, or set up alert channels. This blocks all downstream functionality (parser, filter, pusher).

## What Changes

- Add 13 new API endpoints across 3 handler modules:
  - **Data Sources** (5 endpoints): list, create, update, delete, manual fetch trigger
  - **Keywords** (4 endpoints): list, create, update, delete
  - **Push Channels** (4 endpoints): list, create, update, delete
- Create `src/handlers/mod.rs` with new module declarations
- Register all routes in `src/routes.rs` under `/api/v1/*` behind auth middleware
- All handlers delegate to existing `src/db/` module functions (following project SQL organization rules)
- Response format follows `ApiResponse` pattern from existing token handlers (`{ "data": ... }`)
- **OPEN QUESTION**: The plan doc specifies `PUT` for updates and `DELETE` for deletes, but CLAUDE.md mandates only `GET` and `POST`. Need to resolve before implementation.

## Capabilities

### New Capabilities
- `source-crud-api`: CRUD operations for RSS data sources plus manual fetch trigger
- `keyword-crud-api`: CRUD operations for hotspot keywords  
- `channel-crud-api`: CRUD operations for push notification channels

### Modified Capabilities
None. All existing capabilities remain unchanged.

## Impact

- New files: `src/handlers/source.rs`, `src/handlers/keyword.rs`, `src/handlers/channel.rs`, `src/handlers/mod.rs`
- Modified files: `src/routes.rs` (route registration)
- `src/db/source.rs`, `src/db/keyword.rs`, `src/db/channel.rs` may need new query functions
- No database schema changes required (tables already exist from step 02)
- No dependency changes required
- All endpoints protected behind existing auth middleware
