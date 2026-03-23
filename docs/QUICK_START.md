# Quick Start Guide

## Setup

1. **Copy environment file**:
```bash
cp .env.example .env
```

2. **Configure environment variables**:
```bash
# Edit .env and set:
DATABASE_URL=postgres://user:pass@localhost:5432/evobase
ACCESS_TOKEN_SECRET=your-secret-key-here
REFRESH_TOKEN_SECRET=another-secret-key-here
NOTIFICATION_TOKEN_SECRET=yet-another-secret-key-here
ADMIN_TOKEN=your-admin-token-here
```

3. **Run database migrations**:
```bash
# Apply migrations from db/migrations/
psql $DATABASE_URL < db/migrations/0001_init.sql
```

4. **Build and run**:
```bash
cargo run
```

Server will start on `http://localhost:3000`

## API Examples

### Register a User

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john",
    "password": "password123"
  }'
```

Response:
```json
{
  "data": {
    "user_id": "...",
    "username": "john",
    "tokens": {
      "access_token": "...",
      "refresh_token": "...",
      "notification_token": "...",
      "token_type": "Bearer",
      "expires_in": 900
    }
  }
}
```

### Login

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john",
    "password": "password123"
  }'
```

### Access Protected Endpoint

```bash
# Save access token from login response
ACCESS_TOKEN="your-access-token"

# Query a table
curl http://localhost:3000/rest/messages?limit=10 \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### Admin Endpoints

```bash
# List API documentation (requires admin token)
curl http://localhost:3000/docs \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# Get specific table docs
curl http://localhost:3000/docs/public.messages \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

## REST API Query Syntax

### Select with Filters

```bash
# Get messages where user_id equals a value
curl "http://localhost:3000/rest/messages?user_id=eq.123" \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# Multiple filters
curl "http://localhost:3000/rest/messages?user_id=eq.123&created_at=gt.2024-01-01" \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### Select Specific Columns

```bash
curl "http://localhost:3000/rest/messages?select=id,content,created_at" \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### Ordering

```bash
# Order by created_at descending
curl "http://localhost:3000/rest/messages?order=created_at.desc" \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### Pagination

```bash
curl "http://localhost:3000/rest/messages?limit=20&offset=40" \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### Insert Rows

```bash
# Insert single row
curl -X POST http://localhost:3000/rest/messages \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello world",
    "user_id": "123"
  }'

# Insert multiple rows
curl -X POST http://localhost:3000/rest/messages \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '[
    {"content": "Message 1", "user_id": "123"},
    {"content": "Message 2", "user_id": "123"}
  ]'
```

### Update Rows

```bash
curl -X PATCH "http://localhost:3000/rest/messages?id=eq.1" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Updated content"
  }'
```

### Delete Rows

```bash
curl -X DELETE "http://localhost:3000/rest/messages?id=eq.1" \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

## Filter Operators

- `eq` - Equals
- `neq` - Not equals
- `gt` - Greater than
- `gte` - Greater than or equal
- `lt` - Less than
- `lte` - Less than or equal
- `like` - SQL LIKE (case-sensitive)
- `ilike` - SQL ILIKE (case-insensitive)

## Server-Sent Events (SSE)

Connect to receive real-time messages:

```bash
# Using notification token
curl -N http://localhost:3000/events?token=$NOTIFICATION_TOKEN

# Or using Authorization header
curl -N http://localhost:3000/events \
  -H "Authorization: Bearer $NOTIFICATION_TOKEN"
```

Send a message to another user:

```bash
curl -X POST http://localhost:3000/messages/send \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "to_user_id": "recipient-user-id",
    "event": "chat.message",
    "payload": {
      "text": "Hello!"
    }
  }'
```

## Health Check

```bash
curl http://localhost:3000/healthz
```

Response:
```json
{
  "data": {
    "status": "ok"
  }
}
```

## Error Handling

All errors follow this format:

```json
{
  "error": {
    "code": "bad_request",
    "message": "username must be between 3 and 64 characters",
    "field": "username"
  }
}
```

Common error codes:
- `bad_request` (400)
- `unauthorized` (401)
- `not_found` (404)
- `conflict` (409)
- `database_error` (502)
- `internal_error` (500)

## Development

### Run with logs

```bash
RUST_LOG=debug cargo run
```

### Check code

```bash
cargo check --workspace
```

### Format code

```bash
cargo fmt --all
```

### Run linter

```bash
cargo clippy --workspace
```

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed architecture documentation.
