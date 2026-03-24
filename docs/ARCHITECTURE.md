# EvoBase Architecture

## Overview

EvoBase is a Rust-based backend platform with clean separation between API layer and core business logic. The architecture now supports a default PostgreSQL database from `DATABASE_URL` plus additional runtime databases that admins can bootstrap from ordered SQL scripts.

## Module Structure

```
evobase-server
└── evobase-gateway
    ├── evobase-protocol    → DTOs, request/response models
    ├── evobase-auth        → Authentication service implementation
    ├── evobase-db          → Database storage adapter
    ├── evobase-messaging   → Messaging service implementation
    └── evobase-core        → Domain traits, types, and contracts
```

## Module Responsibilities

### `evobase-core`
Domain layer containing:
- Trait definitions (`AuthService`, `StorageAdapter`, `MessagingService`)
- Domain types (`TableSelect`, `TableInsert`, `QualifiedTable`)
- Runtime database management (`DatabaseRegistry`, `DatabaseManager`, bootstrap request/result types)
- Core error types (`AppError`, `AppResult`)
- Configuration structs

**Dependencies**: None (only workspace deps like `serde`, `uuid`)

### `evobase-protocol`
Protocol/DTO layer containing:
- Request DTOs (`RegisterRequest`, `LoginRequest`)
- Response DTOs (`AuthResponseDto`, `TokenDto`, `ApiDocsDto`)
- API envelope types (`ApiResponse<T>`, `ErrorEnvelope`)
- Conversion traits (`From<CoreType> for DtoType`)

**Dependencies**: `evobase-core`, `serde`, `uuid`

### `evobase-gateway`
HTTP gateway layer containing:
- Route definitions and router assembly
- HTTP handlers (thin, no business logic)
- Middleware (authentication, admin authorization)
- Request parsing and response mapping
- Application state management, including default storage and runtime database manager

**Structure**:
```
src/
├── lib.rs
├── error.rs           # ApiError wrapper for HTTP responses
├── state.rs           # AppState with services and config
├── middleware.rs      # require_access_token, require_admin_token
├── routes.rs          # Router assembly
├── rest.rs            # Query string parsing utilities
└── handlers/
    ├── mod.rs
    ├── auth.rs        # POST /auth/register, /login, /refresh
    ├── rest.rs        # CRUD operations on /rest/{table}
    ├── databases.rs   # /admin/databases bootstrap + catalog endpoints
    ├── docs.rs        # Legacy default-database docs endpoints
    ├── health.rs      # GET /healthz
    └── messaging.rs   # GET /events, POST /messages/send
```

**Dependencies**: `evobase-core`, `evobase-protocol`, `axum`, `tower-http`

### `evobase-auth`
Authentication service implementation:
- JWT token generation and verification
- Password hashing with Argon2
- Token lifecycle management (access, refresh, notification tokens)

**Dependencies**: `evobase-core`, `jsonwebtoken`, `argon2`

### `evobase-db`
Database storage adapter implementation:
- PostgreSQL connection management
- Row-level security (RLS) enforcement
- Dynamic SQL query building
- Schema introspection for API docs
- PostgreSQL database bootstrap orchestration from admin SQL scripts

**Dependencies**: `evobase-core`, `sqlx`

### `evobase-messaging`
In-memory messaging hub implementation:
- Server-Sent Events (SSE) connection management
- Message queuing and delivery
- Connection lifecycle tracking

**Dependencies**: `evobase-core`, `tokio`

## Runtime Database Model

EvoBase now keeps two layers of database wiring at runtime:

- `storage: Arc<dyn StorageAdapter>` in `AppState`
  Used by the existing auth and public REST flow. This remains bound to the default database from `DATABASE_URL` for backward compatibility.
- `database_manager: Arc<DatabaseManager>` in `AppState`
  Owns a `DatabaseRegistry` and resolves which database an admin/docs request should target.

The registry distinguishes:

- Default database
  Booted during startup from `DATABASE_URL`, identified by `DATABASE_DEFAULT_ID` or `default`.
- Bootstrapped runtime databases
  Created through admin API requests, registered under a stable `database_id`, and stored in-memory for the lifetime of the server process.

### Request Target Resolution

- `GET /docs` and `GET /docs/{table}`
  Always target the default database for backward compatibility.
- `GET /admin/databases/{database_id}/docs...`
  Target the database resolved by `database_id`.
- Public `/rest/{table}` and auth endpoints
  Continue to use only the default database.

## Request Flow

```
HTTP Request
    │
    ▼
[Gateway Handler]
    │  deserialize via DTO
    ▼
Request DTO (protocol)
    │  convert to domain type
    ▼
[Service Method] (core trait)
    │  business logic execution
    ▼
Domain Result (core type)
    │  convert to response DTO
    ▼
Response DTO (protocol)
    │  wrap in ApiResponse<T>
    ▼
JSON Response

On Error:
AppError → ApiError → ErrorEnvelope → JSON (4xx/5xx)
```

## Bootstrap Flow

Admin database bootstrap is handled by the database manager and PostgreSQL provisioner:

1. Admin sends `POST /admin/databases` with `database_id`, PostgreSQL database name, metadata, and ordered SQL scripts.
2. `DatabaseManager` validates the request and checks for registry conflicts.
3. `PostgresDatabaseProvisioner` connects to `DATABASE_ADMIN_URL`.
4. The provisioner creates the PostgreSQL database, or reuses it when the request explicitly sets `existing_database_policy=use_existing`.
5. A new `PostgresStorage` is opened against the target database.
6. SQL scripts are executed sequentially in request order.
7. The manager runs schema introspection to confirm docs can be served.
8. The registry stores the database as either:
   - `ready`
   - `bootstrap_failed`, including stage/message/script metadata

### Failure Policy

- Registry insertion is non-destructive: EvoBase does not auto-drop partially created databases on bootstrap failure.
- A failed bootstrap is still registered with status metadata so the admin UI and API can inspect what happened.
- Docs endpoints only resolve databases whose status is `ready`.

## API Endpoints

### Public Endpoints

```
GET    /healthz                    # Health check
POST   /auth/register              # User registration
POST   /auth/login                 # User login
POST   /auth/refresh               # Token refresh
GET    /events                     # SSE connection (notification token)
GET    /rest/{table}               # Select rows (requires auth)
POST   /rest/{table}               # Insert rows (requires auth)
PATCH  /rest/{table}               # Update rows (requires auth)
DELETE /rest/{table}               # Delete rows (requires auth)
POST   /messages/send              # Send message (requires auth)
```

### Admin Endpoints (require admin token)

```
GET    /docs                       # List all table documentation
GET    /docs/{table}               # Get specific table documentation
GET    /admin/databases            # List runtime database catalog
POST   /admin/databases            # Bootstrap/register a runtime database
GET    /admin/databases/{id}       # Get database metadata + bootstrap status
GET    /admin/databases/{id}/docs  # List table documentation for the target database
GET    /admin/databases/{id}/docs/{table}
                                 # Get table documentation for the target database
```

## Authentication & Authorization

### User Authentication
- Access tokens (JWT, 15 min TTL) for API requests
- Refresh tokens (JWT, 30 day TTL) for token renewal
- Notification tokens (JWT, 30 day TTL) for SSE connections
- Middleware: `require_access_token`

### Admin Authorization
- Static admin token from `ADMIN_TOKEN` env var
- Used for sensitive endpoints like API documentation and runtime database bootstrap
- Middleware: `require_admin_token`

## Configuration

Environment variables (see `.env.example`):

```bash
SERVER_ADDR=0.0.0.0:3000
DATABASE_URL=postgres://user:pass@localhost:5432/db
DATABASE_ADMIN_URL=postgres://user:pass@localhost:5432/postgres
DATABASE_DEFAULT_ID=default
ACCESS_TOKEN_SECRET=...
REFRESH_TOKEN_SECRET=...
NOTIFICATION_TOKEN_SECRET=...
ADMIN_TOKEN=...                    # Static token for admin endpoints
ACCESS_TOKEN_TTL_SECS=900
REFRESH_TOKEN_TTL_SECS=2592000
NOTIFICATION_TOKEN_TTL_SECS=2592000
```

## Response Format

### Success Response

```json
{
  "data": { ... },
  "meta": {
    "total": 100,
    "limit": 20,
    "offset": 0
  }
}
```

### Error Response

```json
{
  "error": {
    "code": "unauthorized",
    "message": "unauthorized",
    "field": null
  }
}
```

Error codes:
- `config_error` - Configuration error
- `bad_request` - Invalid request
- `unauthorized` - Authentication failed
- `not_found` - Resource not found
- `conflict` - Resource conflict
- `database_error` - Database operation failed
- `internal_error` - Internal server error

## Design Principles

1. **Separation of Concerns**: Handlers only map HTTP ↔ DTOs, services contain business logic
2. **Type Safety**: Strong typing with domain types separate from DTOs
3. **Explicit Conversions**: `From`/`Into` traits for domain ↔ DTO mapping
4. **Trait-Based Services**: Core defines traits, implementations are pluggable
5. **Zero Business Logic in Handlers**: All logic lives in service implementations
6. **Admin Separation**: Admin endpoints use separate authentication mechanism

## Future Extensibility

The architecture is designed to support:
- Additional API modules (e.g., `evobase-admin-api` as separate crate)
- Plugin system via trait implementations
- Multiple storage backends (implement `StorageAdapter`)
- Persistent database catalog storage beyond the current in-memory runtime registry
- Alternative auth mechanisms (implement `AuthService`)
- API versioning (add `/v2` prefix with new handlers)
