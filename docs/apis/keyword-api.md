# Keyword API (关键词)

Base URL: `http://localhost:8080`

## Authentication

All endpoints require Bearer token authentication.

| Header          | Value                  |
|-----------------|------------------------|
| `Authorization` | `Bearer <your-token>` |

---

## Endpoints

### GET /api/v1/keywords

List all keywords. Ordered by `created_at` descending (newest first).

**Authentication:** Required

**Response** `200 OK`

Each item in the `data` array:

| Field            | Type      | Description                                   |
|------------------|-----------|-----------------------------------------------|
| `id`             | `integer` | Keyword ID                                    |
| `word`           | `string`  | Keyword text                                  |
| `case_sensitive` | `boolean` | Whether matching is case-sensitive            |
| `enabled`        | `boolean` | Whether this keyword is active                |
| `std_multiplier` | `number`  | Hotspot detection sensitivity multiplier      |
| `min_hot_count`  | `integer` | Minimum mentions per hour to trigger hotspot  |
| `created_at`     | `string`  | ISO 8601 datetime                             |

**Example Response**

```json
{
  "data": [
    {
      "id": 1,
      "word": "GPT-5",
      "case_sensitive": false,
      "enabled": true,
      "std_multiplier": 2.0,
      "min_hot_count": 3,
      "created_at": "2026-06-07T09:10:57"
    }
  ]
}
```

**Example**

```bash
curl http://localhost:8080/api/v1/keywords \
  -H "Authorization: Bearer <token>"
```

---

### POST /api/v1/keywords

Create a new keyword.

**Authentication:** Required

**Request Body** `application/json`

| Field            | Type      | Required | Default | Description                                   |
|------------------|-----------|----------|---------|-----------------------------------------------|
| `word`           | `string`  | Yes      | —       | Keyword text (unique)                         |
| `case_sensitive` | `boolean` | No       | `false` | Whether matching is case-sensitive            |
| `std_multiplier` | `number`  | No       | `2.0`   | Hotspot sensitivity (higher = less sensitive) |
| `min_hot_count`  | `integer` | No       | `3`     | Minimum hourly mentions to trigger            |

**Example Request**

```json
{
  "word": "GPT-5",
  "std_multiplier": 2.0,
  "min_hot_count": 3
}
```

**Response** `201 Created`

Returns the full `Keyword` object.

**Example Response**

```json
{
  "data": {
    "id": 1,
    "word": "GPT-5",
    "case_sensitive": false,
    "enabled": true,
    "std_multiplier": 2.0,
    "min_hot_count": 3,
    "created_at": "2026-06-07T09:10:57"
  }
}
```

**Error Responses**

| Status | Code      | Message                              |
|--------|-----------|--------------------------------------|
| 409    | `CONFLICT`| `Keyword '<word>' already exists`    |

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/keywords \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"word":"GPT-5","std_multiplier":2.0,"min_hot_count":3}'
```

---

### POST /api/v1/keywords/{id}/update

Update a keyword. All fields are optional — only the fields provided are updated.

**Authentication:** Required

**Path Parameters**

| Field | Type      | Required | Description                |
|-------|-----------|----------|----------------------------|
| `id`  | `integer` | Yes      | The keyword ID to update   |

**Request Body** `application/json`

| Field            | Type      | Required | Description                                   |
|------------------|-----------|----------|-----------------------------------------------|
| `word`           | `string`  | No       | New keyword text                              |
| `case_sensitive` | `boolean` | No       | Whether matching is case-sensitive            |
| `enabled`        | `boolean` | No       | Enable or disable this keyword                |
| `std_multiplier` | `number`  | No       | Hotspot detection sensitivity multiplier      |
| `min_hot_count`  | `integer` | No       | Minimum hourly mentions to trigger            |

**Example Request**

```json
{
  "std_multiplier": 3.0,
  "enabled": false
}
```

**Response** `200 OK`

Returns the full updated `Keyword` object.

**Error Responses**

| Status | Code        | Message                         |
|--------|-------------|---------------------------------|
| 404    | `NOT_FOUND` | `Keyword {id} not found`        |

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/keywords/1/update \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"std_multiplier":3.0}'
```

---

### POST /api/v1/keywords/{id}/delete

Delete a keyword.

**Authentication:** Required

**Path Parameters**

| Field | Type      | Required | Description                |
|-------|-----------|----------|----------------------------|
| `id`  | `integer` | Yes      | The keyword ID to delete   |

**Request Body:** None

**Response** `204 No Content`

No body returned on success.

**Error Responses**

| Status | Code        | Message                         |
|--------|-------------|---------------------------------|
| 404    | `NOT_FOUND` | `Keyword {id} not found`        |

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/keywords/1/delete \
  -H "Authorization: Bearer <token>"
```
