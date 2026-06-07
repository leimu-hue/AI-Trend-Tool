# Source API (数据源)

Base URL: `http://localhost:8080`

## Authentication

All endpoints require Bearer token authentication.

| Header          | Value                  |
|-----------------|------------------------|
| `Authorization` | `Bearer <your-token>` |

---

## Endpoints

### GET /api/v1/sources

List all data sources. Ordered by `created_at` descending (newest first).

**Authentication:** Required

**Response** `200 OK`

Each item in the `data` array:

| Field              | Type             | Description                          |
|--------------------|------------------|--------------------------------------|
| `id`               | `integer`        | Source ID                            |
| `type`             | `string`         | Feed type (e.g. `rss`, `atom`)       |
| `name`             | `string`         | Display name                         |
| `url`              | `string`         | Feed URL                             |
| `config`           | `string`         | JSON extension config                |
| `enabled`          | `boolean`        | Whether this source is active        |
| `interval_seconds` | `integer`        | Fetch interval in seconds            |
| `last_fetched_at`  | `string \| null` | ISO 8601 datetime                    |
| `created_at`       | `string`         | ISO 8601 datetime                    |
| `updated_at`       | `string`         | ISO 8601 datetime                    |

**Example Response**

```json
{
  "data": [
    {
      "id": 1,
      "type": "rss",
      "name": "Hacker News",
      "url": "https://hnrss.org/frontpage",
      "config": "{}",
      "enabled": true,
      "interval_seconds": 300,
      "last_fetched_at": null,
      "created_at": "2026-06-07T09:10:45",
      "updated_at": "2026-06-07T09:10:45"
    }
  ]
}
```

**Example**

```bash
curl http://localhost:8080/api/v1/sources \
  -H "Authorization: Bearer <token>"
```

---

### POST /api/v1/sources

Create a new data source.

**Authentication:** Required

**Request Body** `application/json`

| Field              | Type     | Required | Default | Description                       |
|--------------------|----------|----------|---------|-----------------------------------|
| `type`             | `string` | Yes      | —       | Feed type, e.g. `rss`             |
| `name`             | `string` | Yes      | —       | Display name for this source      |
| `url`              | `string` | Yes      | —       | Feed URL                          |
| `interval_seconds` | `integer`| No       | `300`   | Polling interval in seconds       |
| `config`           | `string` | No       | `"{}"`  | JSON extension config             |

**Example Request**

```json
{
  "type": "rss",
  "name": "Hacker News",
  "url": "https://hnrss.org/frontpage",
  "interval_seconds": 600
}
```

**Response** `201 Created`

Returns the full `DataSource` object (same fields as GET response above).

**Example Response**

```json
{
  "data": {
    "id": 1,
    "type": "rss",
    "name": "Hacker News",
    "url": "https://hnrss.org/frontpage",
    "config": "{}",
    "enabled": true,
    "interval_seconds": 600,
    "last_fetched_at": null,
    "created_at": "2026-06-07T09:10:45",
    "updated_at": "2026-06-07T09:10:45"
  }
}
```

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/sources \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"type":"rss","name":"Hacker News","url":"https://hnrss.org/frontpage"}'
```

---

### POST /api/v1/sources/{id}/update

Update a data source. All fields are optional — only the fields provided are updated.

**Authentication:** Required

**Path Parameters**

| Field | Type      | Required | Description              |
|-------|-----------|----------|--------------------------|
| `id`  | `integer` | Yes      | The source ID to update  |

**Request Body** `application/json`

| Field              | Type      | Required | Description                       |
|--------------------|-----------|----------|-----------------------------------|
| `name`             | `string`  | No       | New display name                  |
| `url`              | `string`  | No       | New feed URL                      |
| `enabled`          | `boolean` | No       | Enable or disable this source     |
| `interval_seconds` | `integer` | No       | New polling interval in seconds   |
| `config`           | `string`  | No       | New JSON extension config         |

**Example Request**

```json
{
  "name": "Hacker News Updated",
  "interval_seconds": 600
}
```

**Response** `200 OK`

Returns the full updated `DataSource` object.

**Error Responses**

| Status | Code        | Message                       |
|--------|-------------|-------------------------------|
| 404    | `NOT_FOUND` | `Source {id} not found`       |

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/sources/1/update \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"interval_seconds":600}'
```

---

### POST /api/v1/sources/{id}/delete

Delete a data source.

**Authentication:** Required

**Path Parameters**

| Field | Type      | Required | Description              |
|-------|-----------|----------|--------------------------|
| `id`  | `integer` | Yes      | The source ID to delete  |

**Request Body:** None

**Response** `204 No Content`

No body returned on success.

**Error Responses**

| Status | Code        | Message                       |
|--------|-------------|-------------------------------|
| 404    | `NOT_FOUND` | `Source {id} not found`       |

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/sources/1/delete \
  -H "Authorization: Bearer <token>"
```

---

### POST /api/v1/sources/{id}/fetch

Manually trigger a fetch for the given source. Resets `last_fetched_at` to `NULL` so the Parser picks it up on its next cycle.

**Authentication:** Required

**Path Parameters**

| Field | Type      | Required | Description               |
|-------|-----------|----------|---------------------------|
| `id`  | `integer` | Yes      | The source ID to trigger  |

**Request Body:** None

**Response** `200 OK`

```json
{
  "data": {
    "message": "Fetch triggered for source 1"
  }
}
```

**Error Responses**

| Status | Code        | Message                       |
|--------|-------------|-------------------------------|
| 404    | `NOT_FOUND` | `Source {id} not found`       |

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/sources/1/fetch \
  -H "Authorization: Bearer <token>"
```
