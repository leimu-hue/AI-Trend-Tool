# Query API (查询与系统控制)

Base URL: `http://localhost:8080`

## Authentication

All endpoints require Bearer token authentication.

| Header          | Value                  |
|-----------------|------------------------|
| `Authorization` | `Bearer <your-token>` |

---

## Endpoints

### GET /api/v1/articles

List articles with pagination and optional filtering.

**Authentication:** Required

**Query Parameters**

| Field        | Type      | Required | Default | Description                              |
|--------------|-----------|----------|---------|------------------------------------------|
| `page`       | `integer` | No       | `1`     | Page number (1-based)                    |
| `per_page`   | `integer` | No       | `20`    | Items per page                           |
| `source_id`  | `integer` | No       | —       | Filter by source ID                      |
| `processed`  | `boolean` | No       | —       | Filter by processed status               |

**Response** `200 OK`

```json
{
  "data": {
    "items": [
      {
        "id": 1,
        "source_id": 1,
        "link": "https://example.com/article-1",
        "title": "Article Title",
        "summary": "Article summary text",
        "content": "Full article content",
        "published_at": "2026-06-07T09:00:00",
        "fetched_at": "2026-06-07T09:10:45",
        "processed_at": null
      }
    ],
    "total": 100,
    "page": 1,
    "per_page": 20
  }
}
```

**Example**

```bash
curl "http://localhost:8080/api/v1/articles?page=1&per_page=10&source_id=1" \
  -H "Authorization: Bearer <token>"
```

---

### GET /api/v1/hotspots

List hotspots with pagination and optional keyword filter.

**Authentication:** Required

**Query Parameters**

| Field        | Type      | Required | Default | Description                              |
|--------------|-----------|----------|---------|------------------------------------------|
| `page`       | `integer` | No       | `1`     | Page number (1-based, min 1)             |
| `per_page`   | `integer` | No       | `20`    | Items per page (max 100)                 |
| `keyword_id` | `integer` | No       | —       | Filter by keyword ID                     |

**Response** `200 OK`

```json
{
  "data": {
    "items": [
      {
        "id": 1,
        "keyword_id": 1,
        "hour_bucket": "2026-06-07T09:00:00",
        "count": 15,
        "mean_historical": 5.2,
        "stddev_historical": 2.1,
        "created_at": "2026-06-07T09:30:00"
      }
    ],
    "total": 50,
    "page": 1,
    "per_page": 20
  }
}
```

**Example**

```bash
curl "http://localhost:8080/api/v1/hotspots?page=1&per_page=10&keyword_id=1" \
  -H "Authorization: Bearer <token>"
```

---

### GET /api/v1/hotspots/{id}/push-records

List push records for a specific hotspot, including channel names.

**Authentication:** Required

**Path Parameters**

| Field | Type      | Required | Description                        |
|-------|-----------|----------|------------------------------------|
| `id`  | `integer` | Yes      | The hotspot (hot event) ID         |

**Response** `200 OK`

```json
{
  "data": [
    {
      "id": 1,
      "hot_event_id": 1,
      "channel_id": 1,
      "channel_name": "DingTalk Alert",
      "status": "sent",
      "retry_count": 0,
      "next_retry_at": null,
      "created_at": "2026-06-07T09:35:00",
      "updated_at": "2026-06-07T09:35:05"
    }
  ]
}
```

**Example**

```bash
curl http://localhost:8080/api/v1/hotspots/1/push-records \
  -H "Authorization: Bearer <token>"
```

---

### GET /api/v1/trend/{keyword_id}

Get hourly mention trend data for a keyword.

**Authentication:** Required

**Path Parameters**

| Field        | Type      | Required | Description                  |
|--------------|-----------|----------|------------------------------|
| `keyword_id` | `integer` | Yes      | The keyword ID               |

**Query Parameters**

| Field   | Type      | Required | Default | Description                              |
|---------|-----------|----------|---------|------------------------------------------|
| `hours` | `integer` | No       | `24`    | Number of hours to look back             |

**Response** `200 OK`

```json
{
  "data": {
    "keyword_id": 1,
    "keyword": "GPT-5",
    "points": [
      {
        "hour_bucket": "2026-06-07T00:00:00",
        "count": 3
      },
      {
        "hour_bucket": "2026-06-07T01:00:00",
        "count": 7
      }
    ]
  }
}
```

**Error Responses**

| Status | Code        | Message                              |
|--------|-------------|--------------------------------------|
| 404    | `NOT_FOUND` | `Keyword {keyword_id} not found`     |

**Example**

```bash
curl "http://localhost:8080/api/v1/trend/1?hours=48" \
  -H "Authorization: Bearer <token>"
```

---

### GET /api/v1/settings

Return the current server configuration (sensitive fields like `database` and `auth` are excluded).

**Authentication:** Required

**Response** `200 OK`

```json
{
  "data": {
    "server": {
      "host": "0.0.0.0",
      "port": 8080
    },
    "parser": {
      "max_concurrent_fetches": 5,
      "default_user_agent": "AI-Trend-Tool/1.0",
      "default_timeout_seconds": 30,
      "interval_seconds": 30
    },
    "filter": {
      "batch_size": 100,
      "interval_seconds": 60,
      "history_hours": 168,
      "min_history_hours": 24
    },
    "pusher": {
      "interval_seconds": 30,
      "max_retries": 3,
      "retry_base_seconds": 60
    }
  }
}
```

**Example**

```bash
curl http://localhost:8080/api/v1/settings \
  -H "Authorization: Bearer <token>"
```

---

### POST /api/v1/trigger/filter

Manually run one filter iteration. Detects hotspots from unprocessed articles and triggers push if new hotspots are found.

**Authentication:** Required

**Request Body:** None

**Response** `200 OK`

```json
{
  "data": {
    "message": "Filter executed"
  }
}
```

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/trigger/filter \
  -H "Authorization: Bearer <token>"
```

---

### POST /api/v1/trigger/pusher

Manually run one pusher iteration. Sends pending push records to their configured channels.

**Authentication:** Required

**Request Body:** None

**Response** `200 OK`

```json
{
  "data": {
    "message": "Pusher executed"
  }
}
```

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/trigger/pusher \
  -H "Authorization: Bearer <token>"
```
