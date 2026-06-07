# Token API

Base URL: `http://localhost:8080`

## Authentication

All `/api/v1/*` endpoints require Bearer token authentication.

| Header          | Value                  |
|-----------------|------------------------|
| `Authorization` | `Bearer <your-token>` |

Obtain a token from the initial admin token (printed on first server startup) or by calling `POST /api/v1/tokens`.

---

## Error Format

All error responses follow this structure:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable description"
  }
}
```

| Status | Code             | Description                          |
|--------|------------------|--------------------------------------|
| 400    | `BAD_REQUEST`    | Invalid request body or parameters   |
| 401    | `UNAUTHORIZED`   | Missing, invalid, expired, or revoked token |
| 404    | `NOT_FOUND`      | Resource not found                   |
| 409    | `CONFLICT`       | Unique constraint violation          |
| 500    | `DATABASE_ERROR` / `INTERNAL_ERROR` | Server-side error  |

---

## Endpoints

### GET /health

Health check. **No authentication required.**

**Response** `200 OK`

```json
{
  "status": "ok"
}
```

**Example**

```bash
curl http://localhost:8080/health
```

---

### POST /api/v1/tokens

Create a new API token. Generates a 64-character random hex token. The plaintext `token` value is **only returned in this response** — it cannot be retrieved again.

**Authentication:** Required

**Request Body** `application/json`

| Field        | Type             | Required | Description                          |
|--------------|------------------|----------|--------------------------------------|
| `name`       | `string`         | Yes      | Display name for the token           |
| `expires_at` | `string \| null` | No       | ISO 8601 datetime, e.g. `2026-12-31T23:59:59` |

**Example Request**

```json
{
  "name": "My API Token",
  "expires_at": null
}
```

**Response** `201 Created`

| Field          | Type             | Description                        |
|----------------|------------------|------------------------------------|
| `id`           | `integer`        | Token ID                           |
| `name`         | `string`         | Display name                       |
| `token`        | `string`         | 64-char hex, **only shown here**   |
| `last_used_at` | `string \| null` | ISO 8601 datetime                  |
| `created_at`   | `string`         | ISO 8601 datetime                  |
| `expires_at`   | `string \| null` | ISO 8601 datetime                  |
| `revoked`      | `boolean`        | Whether the token has been revoked |

**Example Response**

```json
{
  "data": {
    "id": 1,
    "name": "My API Token",
    "token": "a1b2c3d4e5f6...64chars",
    "last_used_at": null,
    "created_at": "2026-06-07T06:00:00",
    "expires_at": null,
    "revoked": false
  }
}
```

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "My API Token"}'
```

---

### GET /api/v1/tokens

List all tokens. The plaintext token value is **hidden** — returns `ApiTokenInfo` without the `token` field. Ordered by `created_at` descending (newest first).

**Authentication:** Required

**Response** `200 OK`

Each item in the `data` array:

| Field          | Type             | Description                        |
|----------------|------------------|------------------------------------|
| `id`           | `integer`        | Token ID                           |
| `name`         | `string`         | Display name                       |
| `last_used_at` | `string \| null` | ISO 8601 datetime                  |
| `created_at`   | `string`         | ISO 8601 datetime                  |
| `expires_at`   | `string \| null` | ISO 8601 datetime                  |
| `revoked`      | `boolean`        | Whether the token has been revoked |

**Example Response**

```json
{
  "data": [
    {
      "id": 1,
      "name": "Initial Admin Token",
      "last_used_at": "2026-06-07T06:15:30",
      "created_at": "2026-06-07T05:24:47",
      "expires_at": null,
      "revoked": false
    }
  ]
}
```

**Example**

```bash
curl http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <token>"
```

---

### POST /api/v1/tokens/revoke/{id}

Revoke a token (soft delete). Sets `revoked = 1`. The token can no longer be used for authentication.

**Authentication:** Required

**Path Parameters**

| Field | Type      | Required | Description            |
|-------|-----------|----------|------------------------|
| `id`  | `integer` | Yes      | The token ID to revoke |

**Request Body:** None

**Response** `204 No Content`

No body returned on success.

**Error Responses**

| Status | Code        | Message                             |
|--------|-------------|-------------------------------------|
| 404    | `NOT_FOUND` | `Token with id {id} not found`      |

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/tokens/revoke/1 \
  -H "Authorization: Bearer <token>"
```
