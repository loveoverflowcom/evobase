# evobase

`evobase` is a minimal, modular Rust backend platform inspired by Supabase/PostgREST.
It keeps application logic thin by pushing authorization and row-level access into PostgreSQL.

## Architecture

- `evobase-core`: shared types, traits, config, and errors
- `evobase-db`: PostgreSQL adapter and PostgREST-like table access
- `evobase-auth`: username/password auth with JWT issuance
- `evobase-messaging`: in-memory SSE notification hub
- `evobase-gateway`: HTTP routing, middleware, and request parsing
- `evobase-server`: bootstrap binary

## Features

- Username/password register, login, and refresh
- Access, refresh, and notification JWTs
- Authenticated SSE connections at `/events`
- In-memory fan-out messaging at `/messages/send`
- PostgREST-like REST table gateway at `/rest/:table`
- Request-scoped PostgreSQL claim forwarding via `set_config`

## Quick Start

1. Copy `.env.example` to `.env` and update the secrets.
2. Run the migration in [`db/migrations/0001_init.sql`](/Users/manhblue/Documents/personal/open_source/evobase/db/migrations/0001_init.sql).
3. Start the server:

```bash
cargo run -p evobase-server
```

## Example cURL

Register:

```bash
curl -X POST http://127.0.0.1:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"super-secret-password"}'
```

Login:

```bash
curl -X POST http://127.0.0.1:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"super-secret-password"}'
```

Refresh:

```bash
curl -X POST http://127.0.0.1:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token":"<refresh-token>"}'
```

Open an SSE stream:

```bash
curl -N "http://127.0.0.1:3000/events?token=<notification-token>"
```

Send a message:

```bash
curl -X POST http://127.0.0.1:3000/messages/send \
  -H "Authorization: Bearer <access-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "to_user_id":"<target-user-id>",
    "event":"chat.message",
    "payload":{"body":"hello from evobase"}
  }'
```

Read from a table with RLS applied:

```bash
curl "http://127.0.0.1:3000/rest/public.notes?select=id,body,created_at&order=created_at.desc&limit=10" \
  -H "Authorization: Bearer <access-token>"
```

Insert through the REST gateway:

```bash
curl -X POST http://127.0.0.1:3000/rest/public.notes \
  -H "Authorization: Bearer <access-token>" \
  -H "Content-Type: application/json" \
  -d '{"owner_id":"<user-id>","body":"created via REST gateway"}'
```

## Supported REST Query Syntax

- `select=id,body,created_at`
- `limit=20`
- `offset=0`
- `order=created_at.desc,id.asc`
- Filters: `column=eq.value`, `column=neq.value`, `column=gt.value`, `column=gte.value`, `column=lt.value`, `column=lte.value`, `column=like.%foo%`, `column=ilike.%foo%`

## Project Layout

```text
.
|-- Cargo.toml
|-- .env.example
|-- docs/
|-- README.md
|-- db/
|   `-- migrations/
|       `-- 0001_init.sql
|-- crates/
|   |-- evobase-auth/
|   |-- evobase-core/
|   |-- evobase-db/
|   |-- evobase-gateway/
|   `-- evobase-messaging/
`-- apps/
    `-- evobase-server/
```

## Architecture Diagrams

PlantUML diagrams are available in `docs/diagrams/`.
Start with:

- `docs/diagrams/01_component_overview.puml`
- `docs/diagrams/02_auth_rls_sequence.puml`
- `docs/diagrams/03_messaging_sse_flow.puml`
- `docs/diagrams/09_rest_gateway_flow.puml`
