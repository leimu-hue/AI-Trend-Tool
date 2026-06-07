# channel-crud-api

## Purpose

API endpoints for managing push notification channels (DingTalk/Feishu webhooks): list, create, update, and delete. All endpoints require Bearer token authentication.

## ADDED Requirements

### Requirement: List channels endpoint

The system SHALL expose `GET /api/v1/channels` to list all push channels. Results SHALL be ordered by `id ASC`.

#### Scenario: List all channels

- **WHEN** `GET /api/v1/channels` is called with a valid Bearer token
- **THEN** the response SHALL be HTTP 200 with body `{"data": [<PushChannel>, ...]}`
- **THEN** each item SHALL contain all fields: `id`, `name`, `channel_type`, `config`, `enabled`

#### Scenario: List channels without authentication

- **WHEN** `GET /api/v1/channels` is called without a valid Authorization header
- **THEN** the request SHALL be rejected with HTTP 401

### Requirement: Create channel endpoint

The system SHALL expose `POST /api/v1/channels` to add a new push channel. The request body SHALL contain `name` (required), `config` (required, JSON string), and `channel_type` (optional, default "webhook"). The response SHALL be HTTP 201 with the created `PushChannel` object.

#### Scenario: Create channel with required fields

- **WHEN** `POST /api/v1/channels` is called with body `{"name": "DingTalk Alert", "config": "{\"url\": \"https://oapi.dingtalk.com/robot/send?access_token=xxx\"}"}`
- **THEN** the system SHALL insert a row into `push_channels` with `channel_type = "webhook"`
- **THEN** the response SHALL be HTTP 201 with body `{"data": {"id": <id>, "name": "DingTalk Alert", ...}}`

#### Scenario: Create channel with explicit type

- **WHEN** `POST /api/v1/channels` is called with `channel_type` specified as "feishu"
- **THEN** the inserted row SHALL use `channel_type = "feishu"` instead of the default "webhook"

#### Scenario: Create channel without authentication

- **WHEN** `POST /api/v1/channels` is called without a valid Bearer token
- **THEN** the request SHALL be rejected with HTTP 401

### Requirement: Update channel endpoint

The system SHALL expose `POST /api/v1/channels/{id}/update` to update an existing push channel. All fields in the request body SHALL be optional — only provided fields are updated. The response SHALL be HTTP 200 with the updated `PushChannel` object.

#### Scenario: Partial update

- **WHEN** `POST /api/v1/channels/1/update` is called with body `{"name": "Updated Channel", "enabled": false}`
- **THEN** only `name` and `enabled` SHALL be updated in the database
- **THEN** the response SHALL be HTTP 200 with the full updated `PushChannel`

#### Scenario: Update non-existent channel

- **WHEN** `POST /api/v1/channels/999/update` is called and no channel with id=999 exists
- **THEN** the response SHALL be HTTP 404 with body `{"error": {"code": "NOT_FOUND", "message": "Channel 999 not found"}}`

#### Scenario: Update with empty body

- **WHEN** `POST /api/v1/channels/1/update` is called with body `{}`
- **THEN** no fields SHALL be modified
- **THEN** the response SHALL be HTTP 200 with the existing `PushChannel`

### Requirement: Delete channel endpoint

The system SHALL expose `POST /api/v1/channels/{id}/delete` to delete a push channel. The response SHALL be HTTP 204 No Content on success.

#### Scenario: Delete existing channel

- **WHEN** `POST /api/v1/channels/1/delete` is called with a valid Bearer token and channel id=1 exists
- **THEN** the row SHALL be deleted from `push_channels`
- **THEN** the response SHALL be HTTP 204 with no body

#### Scenario: Delete non-existent channel

- **WHEN** `POST /api/v1/channels/999/delete` is called and no channel with id=999 exists
- **THEN** the response SHALL be HTTP 404 with body `{"error": {"code": "NOT_FOUND", "message": "Channel 999 not found"}}`

### Requirement: Channel handlers follow project conventions

Channel handlers SHALL reside at `src/handlers/channel.rs` and be declared via `pub mod channel;` in `src/handlers.rs`. All SQL operations SHALL delegate to `src/db/channel.rs` functions. Responses SHALL use `ApiResponse` for consistent wrapping.

#### Scenario: Handler module compiles

- **WHEN** `cargo check` is run
- **THEN** `src/handlers/channel.rs` SHALL compile as a submodule of `handlers`
- **THEN** no raw SQL strings SHALL appear in the handler file
