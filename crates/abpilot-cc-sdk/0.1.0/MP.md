# cc_abpilot_mp API Documentation

Lambda Function URL: `https://wpyi6ctkdvfcxbqtmy6d6tkesi0yzzid.lambda-url.us-east-1.on.aws`

## Authentication

Most endpoints require authentication. Three authentication methods are supported:

### 1. JWT Token (Bearer Token)
```
Authorization: Bearer <jwt_token>
```

### 2. API Key
```
X-Api-Key: sk_<uuid>
```

### 3. App/World Signature (for file operations)
```
Signature = HMAC-SHA256(app_id/world_id + timestamp, secret)
```

Headers:
- `X-App-Id` or `X-World-Id`
- `X-Signature`: HMAC-SHA256 signature
- `X-Timestamp`: Unix timestamp (valid within 5 minutes)

---

## User Authentication

### POST /auth/send-code
Send a 6-digit verification code to email.

**Authentication**: None

**Request**:
```json
{
  "email": "user@example.com"
}
```

**Response**:
```json
{
  "message": "Code sent"
}
```

**Notes**:
- Code is valid for 5 minutes
- Code is sent via SMTP

---

### POST /auth/verify-code
Verify the code and get JWT token.

**Authentication**: None

**Request**:
```json
{
  "email": "user@example.com",
  "code": "123456"
}
```

**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_id": "a1b2c3d4"
}
```

**Error Response** (401):
```json
{
  "error": "Invalid or expired code"
}
```

**Notes**:
- User is auto-created on first login
- User ID is 8-character alphanumeric string

---

## API Key Management

### POST /apikey
Create a new API key.

**Authentication**: Token or API Key

**Request**:
```json
{
  "name": "My API Key"
}
```

**Response**:
```json
{
  "apikey": "sk_a1b2c3d4e5f6...",
  "name": "My API Key"
}
```

---

### DELETE /apikey
Delete an API key.

**Authentication**: Token or API Key

**Request**:
```json
{
  "apikey": "sk_a1b2c3d4e5f6..."
}
```

**Response**:
```json
{
  "message": "API key deleted"
}
```

---

### GET /apikey
List all API keys for the authenticated user.

**Authentication**: Token or API Key

**Response**:
```json
{
  "apikeys": [
    {
      "apikey": "sk_a1b2c3d4e5f6...",
      "name": "My API Key",
      "created_at": 1234567890
    }
  ]
}
```

---

## App Management

### POST /app
Create a new app.

**Authentication**: Token or API Key

**Request**:
```json
{
  "name": "My App"
}
```

**Response**:
```json
{
  "app_id": "rImLACph7Ayr8tu1",
  "name": "My App",
  "secret": "5d75a1ee1cd34b2e9122b44d36ddf798"
}
```

**Notes**:
- App ID is 16-character alphanumeric string
- Secret is 32-character hex string

---

### DELETE /app
Delete an app.

**Authentication**: Token or API Key

**Request**:
```json
{
  "app_id": "rImLACph7Ayr8tu1"
}
```

**Response**:
```json
{
  "message": "App deleted"
}
```

---

### GET /app
List all apps for the authenticated user.

**Authentication**: Token or API Key

**Response**:
```json
{
  "apps": [
    {
      "app_id": "rImLACph7Ayr8tu1",
      "name": "My App",
      "created_at": 1234567890
    }
  ]
}
```

---

### POST /app/reset-secret
Reset app secret.

**Authentication**: Token or API Key

**Request**:
```json
{
  "app_id": "rImLACph7Ayr8tu1"
}
```

**Response**:
```json
{
  "app_id": "rImLACph7Ayr8tu1",
  "secret": "new_secret_here"
}
```

---

### POST /app/upload
Get presigned URLs for uploading files to S3.

**Authentication**: Token or API Key

**Request**:
```json
{
  "app_id": "rImLACph7Ayr8tu1",
  "files": ["file1.png", "file2.json"]
}
```

**Response**:
```json
{
  "urls": [
    "https://app.abpilot.cc.s3.amazonaws.com/...",
    "https://app.abpilot.cc.s3.amazonaws.com/..."
  ]
}
```

**Notes**:
- URLs are valid for 1 hour
- Files are stored at `s3://app.abpilot.cc/{app_id}/{filename}`

---

### POST /app/files
Get presigned URLs for downloading files from S3.

**Authentication**: Token, API Key, or App Signature

**Request**:
```json
{
  "app_id": "rImLACph7Ayr8tu1",
  "files": ["file1.png", "file2.json"]
}
```

**Response**:
```json
{
  "urls": [
    "https://app.abpilot.cc.s3.amazonaws.com/...",
    "https://app.abpilot.cc.s3.amazonaws.com/..."
  ]
}
```

**Notes**:
- URLs are valid for 1 hour
- Can use App Signature for authentication (no user login required)

---

## World Management

### POST /world
Create a new world.

**Authentication**: Token or API Key

**Request**:
```json
{
  "name": "My World"
}
```

**Response**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "name": "My World",
  "secret": "c26087c463474bf0b2b1f5018ae07f05"
}
```

**Notes**:
- World ID is 16-character alphanumeric string
- Secret is 32-character hex string

---

### DELETE /world
Delete a world.

**Authentication**: Token or API Key

**Request**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx"
}
```

**Response**:
```json
{
  "message": "World deleted"
}
```

---

### GET /world
List all worlds for the authenticated user.

**Authentication**: Token or API Key

**Response**:
```json
{
  "worlds": [
    {
      "world_id": "KrUc1wbBULtQ53Jx",
      "name": "My World",
      "created_at": 1234567890
    }
  ]
}
```

---

### POST /world/get
Get world details.

**Authentication**: Token or API Key

**Request**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx"
}
```

**Response**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "name": "My World",
  "created_at": 1234567890
}
```

---

### POST /world/reset-secret
Reset world secret.

**Authentication**: Token or API Key

**Request**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx"
}
```

**Response**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "secret": "new_secret_here"
}
```

---

### POST /world/upload
Get presigned URLs for uploading files to S3.

**Authentication**: Token or API Key

**Request**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "files": ["world1.dat", "world2.json"]
}
```

**Response**:
```json
{
  "urls": [
    "https://world.abpilot.cc.s3.amazonaws.com/...",
    "https://world.abpilot.cc.s3.amazonaws.com/..."
  ]
}
```

**Notes**:
- URLs are valid for 1 hour
- Files are stored at `s3://world.abpilot.cc/{world_id}/{filename}`

---

### POST /world/files
Get presigned URLs for downloading files from S3.

**Authentication**: Token, API Key, or World Signature

**Request**:
```json
{
  "world_id": "KrUc1wbBULtQ53Jx",
  "files": ["world1.dat", "world2.json"]
}
```

**Response**:
```json
{
  "urls": [
    "https://world.abpilot.cc.s3.amazonaws.com/...",
    "https://world.abpilot.cc.s3.amazonaws.com/..."
  ]
}
```

**Notes**:
- URLs are valid for 1 hour
- Can use World Signature for authentication (no user login required)

---

## DynamoDB Tables

### cc_abpilot_mp_auth
- **Primary Key**: `pk` (code#email)
- **Attributes**: `code`, `expire_at`
- **TTL**: `expire_at`

### cc_abpilot_mp_user
- **Primary Key**: `user_id` (8-char string)
- **GSI**: `email-index` (email as partition key)
- **Attributes**: `email`, `created_at`

### cc_abpilot_mp_apikey
- **Primary Key**: `apikey` (sk_uuid format)
- **GSI**: `user_id-index` (user_id as partition key)
- **Attributes**: `user_id`, `name`, `created_at`

### cc_abpilot_mp_app
- **Primary Key**: `app_id` (16-char string)
- **GSI**: `user_id-index` (user_id as partition key)
- **Attributes**: `user_id`, `name`, `secret`, `created_at`

### cc_abpilot_mp_world
- **Primary Key**: `world_id` (16-char string)
- **GSI**: `user_id-index` (user_id as partition key)
- **Attributes**: `user_id`, `name`, `secret`, `created_at`

---

## S3 Buckets

- **app.abpilot.cc**: App files storage
- **world.abpilot.cc**: World files storage

---

## Example: Python Client

```python
import hmac
import hashlib
import time
import requests

# 1. Send verification code
response = requests.post(
    "https://wpyi6ctkdvfcxbqtmy6d6tkesi0yzzid.lambda-url.us-east-1.on.aws/auth/send-code",
    json={"email": "user@example.com"}
)

# 2. Verify code and get token
response = requests.post(
    "https://wpyi6ctkdvfcxbqtmy6d6tkesi0yzzid.lambda-url.us-east-1.on.aws/auth/verify-code",
    json={"email": "user@example.com", "code": "123456"}
)
token = response.json()["token"]

# 3. Create app with token
response = requests.post(
    "https://wpyi6ctkdvfcxbqtmy6d6tkesi0yzzid.lambda-url.us-east-1.on.aws/app",
    headers={"Authorization": f"Bearer {token}"},
    json={"name": "My App"}
)
app_id = response.json()["app_id"]
app_secret = response.json()["secret"]

# 4. Get app files with signature (no login required)
timestamp = str(int(time.time()))
signature = hmac.new(
    app_secret.encode(),
    f"{app_id}{timestamp}".encode(),
    hashlib.sha256
).hexdigest()

response = requests.post(
    "https://wpyi6ctkdvfcxbqtmy6d6tkesi0yzzid.lambda-url.us-east-1.on.aws/app/files",
    headers={
        "X-App-Id": app_id,
        "X-Signature": signature,
        "X-Timestamp": timestamp
    },
    json={
        "app_id": app_id,
        "files": ["file1.png"]
    }
)
```

---

## Environment Variables

- `AUTH_TABLE_NAME`: cc_abpilot_mp_auth
- `USER_TABLE_NAME`: cc_abpilot_mp_user
- `APIKEY_TABLE_NAME`: cc_abpilot_mp_apikey
- `APP_TABLE_NAME`: cc_abpilot_mp_app
- `WORLD_TABLE_NAME`: cc_abpilot_mp_world
- `S3_BUCKET`: app.abpilot.cc
- `WORLD_S3_BUCKET`: world.abpilot.cc
- `JWT_SECRET`: Secret key for JWT signing
- `SMTP_HOST`: SMTP server host
- `SMTP_PORT`: SMTP server port (default: 465)
- `SMTP_USER`: SMTP username
- `SMTP_PASSWORD`: SMTP password
