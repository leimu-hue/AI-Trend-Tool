# Channel API (推送渠道)

Base URL: `http://localhost:8080`

## Authentication

All endpoints require Bearer token authentication.

| Header          | Value                  |
|-----------------|------------------------|
| `Authorization` | `Bearer <your-token>` |

---

## Endpoints

### GET /api/v1/channels

List all push channels. Ordered by `id` ascending.

**Authentication:** Required

**Response** `200 OK`

Each item in the `data` array:

| Field          | Type      | Description                                    |
|----------------|-----------|------------------------------------------------|
| `id`           | `integer` | Channel ID                                     |
| `name`         | `string`  | Display name                                   |
| `channel_type` | `string`  | Channel type, e.g. `webhook`, `feishu`         |
| `config`       | `string`  | JSON config, typically `{"url": "..."}`         |
| `enabled`      | `boolean` | Whether this channel is active                 |

**Example Response**

```json
{
  "data": [
    {
      "id": 1,
      "name": "DingTalk Alert",
      "channel_type": "webhook",
      "config": "{\"url\": \"https://oapi.dingtalk.com/robot/send?access_token=xxx\"}",
      "enabled": true
    }
  ]
}
```

**Example**

```bash
curl http://localhost:8080/api/v1/channels \
  -H "Authorization: Bearer <token>"
```

---

### POST /api/v1/channels

Create a new push channel.

**Authentication:** Required

**Request Body** `application/json`

| Field          | Type     | Required | Default     | Description                                    |
|----------------|----------|----------|-------------|------------------------------------------------|
| `name`         | `string` | Yes      | —           | Display name for this channel                  |
| `config`       | `string` | Yes      | —           | JSON config, typically `{"url": "..."}`         |
| `channel_type` | `string` | No       | `"webhook"` | Channel type, e.g. `webhook`, `feishu`         |

**Example Request**

```json
{
  "name": "DingTalk Alert",
  "config": "{\"url\": \"https://oapi.dingtalk.com/robot/send?access_token=xxx\"}"
}
```

**Response** `201 Created`

Returns the full `PushChannel` object.

**Example Response**

```json
{
  "data": {
    "id": 1,
    "name": "DingTalk Alert",
    "channel_type": "webhook",
    "config": "{\"url\": \"https://oapi.dingtalk.com/robot/send?access_token=xxx\"}",
    "enabled": true
  }
}
```

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/channels \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"DingTalk Alert","config":"{\"url\": \"https://oapi.dingtalk.com/robot/send?access_token=xxx\"}"}'
```

---

### POST /api/v1/channels/{id}/update

Update a push channel. All fields are optional — only the fields provided are updated.

**Authentication:** Required

**Path Parameters**

| Field | Type      | Required | Description                |
|-------|-----------|----------|----------------------------|
| `id`  | `integer` | Yes      | The channel ID to update   |

**Request Body** `application/json`

| Field     | Type      | Required | Description                            |
|-----------|-----------|----------|----------------------------------------|
| `name`    | `string`  | No       | New display name                       |
| `config`  | `string`  | No       | New JSON config                        |
| `enabled` | `boolean` | No       | Enable or disable this channel         |

**Example Request**

```json
{
  "name": "Updated Channel",
  "enabled": false
}
```

**Response** `200 OK`

Returns the full updated `PushChannel` object.

**Error Responses**

| Status | Code        | Message                         |
|--------|-------------|---------------------------------|
| 404    | `NOT_FOUND` | `Channel {id} not found`        |

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/channels/1/update \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"Updated Channel"}'
```

---

### POST /api/v1/channels/{id}/delete

Delete a push channel.

**Authentication:** Required

**Path Parameters**

| Field | Type      | Required | Description                |
|-------|-----------|----------|----------------------------|
| `id`  | `integer` | Yes      | The channel ID to delete   |

**Request Body:** None

**Response** `204 No Content`

No body returned on success.

**Error Responses**

| Status | Code        | Message                         |
|--------|-------------|---------------------------------|
| 404    | `NOT_FOUND` | `Channel {id} not found`        |

**Example**

```bash
curl -X POST http://localhost:8080/api/v1/channels/1/delete \
  -H "Authorization: Bearer <token>"
```
