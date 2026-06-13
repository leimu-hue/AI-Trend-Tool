## MODIFIED Requirements

### Requirement: Source handler input validation

The system SHALL validate input for `POST /api/v1/sources` and `POST /api/v1/sources/{id}/update` handlers using `validator` crate derive macros on request structs, returning 400 Bad Request for invalid input.

#### Scenario: Empty name rejected for create source

- **WHEN** `POST /api/v1/sources` body contains `name` as empty string or whitespace-only
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "<validation error>"}}`

#### Scenario: Invalid URL rejected for create source

- **WHEN** `POST /api/v1/sources` body contains `url` not parseable as a valid URL
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "<validation error>"}}`

### Requirement: Keyword handler input validation

The system SHALL validate input for `POST /api/v1/keywords` and `POST /api/v1/keywords/{id}/update` handlers using `validator` crate derive macros.

#### Scenario: Empty word rejected for create keyword

- **WHEN** `POST /api/v1/keywords` body contains `word` as empty string
- **THEN** the response SHALL be HTTP 400 with `{"error": {"code": "BAD_REQUEST", "message": "<validation error>"}}`

#### Scenario: Non-positive std_multiplier rejected for create keyword

- **WHEN** `POST /api/v1/keywords` body contains `std_multiplier` <= 0
- **THEN** the response SHALL be HTTP 400

#### Scenario: min_hot_count below 1 rejected for create keyword

- **WHEN** `POST /api/v1/keywords` body contains `min_hot_count` < 1
- **THEN** the response SHALL be HTTP 400

### Requirement: Token handler input validation

The system SHALL validate input for `POST /api/v1/tokens` handler using `validator` crate derive macros.

#### Scenario: Empty name rejected for create token

- **WHEN** `POST /api/v1/tokens` body contains `name` as empty string
- **THEN** the response SHALL be HTTP 400

## REMOVED Requirements

### Requirement: Channel handler input validation

**Reason**: Channel 模块的 JSON 格式验证不适用 `validator` 的 URL/长度注解，保留当前手动验证方式。其他模块替换为 validator。
**Migration**: Channel handler 保持现有手动验证代码不变。
