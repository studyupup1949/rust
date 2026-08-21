# cc_abpilot_app API Documentation

Lambda Function URL: `https://opnqqwytt7sgobosrlk6kxp5de0rolbu.lambda-url.us-east-1.on.aws`

## Authentication

All endpoints require signature-based authentication using HMAC-SHA256.

### App Signature (for `/assets/list`, `/assets/get`, `/world/device/create`)
```
Signature = HMAC-SHA256(app_id + timestamp, app_secret)
```

Headers:
- `X-App-Id`: App ID
- `X-Signature`: HMAC-SHA256 signature
- `X-Timestamp`: Unix timestamp (valid within 5 minutes)

### World Signature (for `/assets/add`, `/world/node/*`, `/world/device/get`)
```
Signature = HMAC-SHA256(world_id + timestamp, world_secret)
```

Headers:
- `X-World-Id`: World ID
- `X-Signature`: HMAC-SHA256 signature
- `X-Timestamp`: Unix timestamp (valid within 5 minutes)

---

## Asset Management

### POST /assets/list
List all assets for a device in a world.

**Authentication**: App Signature

**Request**:
```json
{
  "device_id": "device_001",
  "world_id": "KrUc1wbBULtQ53Jx"
}
```

**Response**:
```json
{
  "assets": [
    {
      "type": "gold",
      "id": "001",
      "value": 150
    }
  ]
}
```

---

### POST /assets/get
Get a specific asset for a device.

**Authentication**: App Signature

**Request**:
```json
{
  "device_id": "device_001",
  "world_id": "KrUc1wbBULtQ53Jx",
  "type": "gold",
  "id": "001"
}
```

**Response**:
```json
{
  "type": "gold",
  "id": "001",
  "value": 150
}
```

**Error Response** (404):
```json
{
  "error": "Asset not found"
}
```

---

### POST /assets/add
Add or deduct asset value (delta can be positive or negative).

**Authentication**: World Signature

**Request**:
```json
{
  "device_id": "device_001",
  "world_id": "KrUc1wbBULtQ53Jx",
  "type": "gold",
  "id": "001",
  "delta": 100
}
```

**Response**:
```json
{
  "type": "gold",
  "id": "001",
  "value": 250
}
```

**Error Response** (500):
```json
{
  "error": "Insufficient balance"
}
```

**Notes**:
- Negative delta is allowed but will fail if resulting value < 0
- The `world_id` in request body must match the `X-World-Id` header

---

## World Node Management

### POST /world/node/update
Add or update a world node (base_url + tags).

**Authentication**: World Signature

**Request**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "base_url": "https://node1.example.com",
  "tags": "cn|us"
}
```

**Response**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "base_url": "https://node1.example.com",
  "tags": "cn|us"
}
```

**Notes**:
- One world can have multiple nodes (different base_url)
- Each node can have different tags

---

### POST /world/node/delete
Delete a specific world node.

**Authentication**: World Signature

**Request**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "base_url": "https://node1.example.com"
}
```

**Response**:
```json
{
  "message": "World node deleted"
}
```

---

## World Device Management

### POST /world/device/create
Create a device login token with TTL.

**Authentication**: App Signature

**Request**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "device_id": "device_001",
  "info": {
    "platform": "ios",
    "version": "1.0"
  },
  "ttl": 3600
}
```

**Response**:
```json
{
  "token": "3b22e41c622c4527be879625caf5beef",
  "items": [
    {
      "base_url": "https://node1.example.com",
      "tags": "cn|us"
    },
    {
      "base_url": "https://node2.example.com",
      "tags": "eu"
    }
  ]
}
```

**Notes**:
- `info` can be any JSON object
- `ttl` is in seconds
- Returns all world nodes with their base_url and tags

---

### POST /world/device/get
Get device information by token.

**Authentication**: World Signature

**Request**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "token": "3b22e41c622c4527be879625caf5beef"
}
```

**Response**:
```json
{
  "device_id": "device_001",
  "info": {
    "platform": "ios",
    "version": "1.0"
  },
  "world_id": "KrUc1wbBULtQ53Jx",
  "base_url": [
    "https://node1.example.com",
    "https://node2.example.com"
  ]
}
```

**Error Response** (500):
```json
{
  "error": "Token not found"
}
```

```json
{
  "error": "Token expired"
}
```

---

## DynamoDB Tables

### cc_abpilot_mp_asset
- **Primary Key**: `pk` (device_id#world_id)
- **Sort Key**: `index` (type#id)
- **Attributes**: `world_id`, `value`

### cc_abpilot_mp_world_node
- **Primary Key**: `pk` (world_id)
- **Sort Key**: `base_url`
- **Attributes**: `tags`

### cc_abpilot_mp_world_device
- **Primary Key**: `pk` (world_id)
- **Sort Key**: `token`
- **GSI**: `token-index` (token as partition key)
- **Attributes**: `device_id`, `info`, `expire_at`
- **TTL**: `expire_at`

---

## Example: Python Client

```python
import hmac
import hashlib
import time
import requests

def generate_app_signature(app_id, secret):
    timestamp = str(int(time.time()))
    signature = hmac.new(
        secret.encode(),
        f"{app_id}{timestamp}".encode(),
        hashlib.sha256
    ).hexdigest()
    return signature, timestamp

def generate_world_signature(world_id, secret):
    timestamp = str(int(time.time()))
    signature = hmac.new(
        secret.encode(),
        f"{world_id}{timestamp}".encode(),
        hashlib.sha256
    ).hexdigest()
    return signature, timestamp

# List assets (App Signature)
app_id = "rImLACph7Ayr8tu1"
app_secret = "5d75a1ee1cd34b2e9122b44d36ddf798"
signature, timestamp = generate_app_signature(app_id, app_secret)

response = requests.post(
    "https://opnqqwytt7sgobosrlk6kxp5de0rolbu.lambda-url.us-east-1.on.aws/assets/list",
    headers={
        "X-App-Id": app_id,
        "X-Signature": signature,
        "X-Timestamp": timestamp
    },
    json={
        "device_id": "device_001",
        "world_id": "KrUc1wbBULtQ53Jx"
    }
)

# Add asset (World Signature)
world_id = "KrUc1wbBULtQ53Jx"
world_secret = "c26087c463474bf0b2b1f5018ae07f05"
signature, timestamp = generate_world_signature(world_id, world_secret)

response = requests.post(
    "https://opnqqwytt7sgobosrlk6kxp5de0rolbu.lambda-url.us-east-1.on.aws/assets/add",
    headers={
        "X-World-Id": world_id,
        "X-Signature": signature,
        "X-Timestamp": timestamp
    },
    json={
        "device_id": "device_001",
        "world_id": "KrUc1wbBULtQ53Jx",
        "type": "gold",
        "id": "001",
        "delta": 100
    }
)
```

---

## Environment Variables

- `APP_TABLE_NAME`: cc_abpilot_mp_app
- `WORLD_TABLE_NAME`: cc_abpilot_mp_world
- `WORLD_NODE_TABLE_NAME`: cc_abpilot_mp_world_node
- `ASSET_TABLE_NAME`: cc_abpilot_mp_asset
- `WORLD_DEVICE_TABLE_NAME`: cc_abpilot_mp_world_device
