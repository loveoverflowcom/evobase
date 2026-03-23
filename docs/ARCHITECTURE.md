# EvoBase Architecture

## Overview

EvoBase is a Rust-based backend platform with clean separation between API layer and core business logic. The architecture follows a modular design with clear responsibilities for each crate.

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
- Application state management

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
    ├── docs.rs        # GET /docs, /docs/{table} (admin only)
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

**Dependencies**: `evobase-core`, `sqlx`

### `evobase-messaging`
In-memory messaging hub implementation:
- Server-Sent Events (SSE) connection management
- Message queuing and delivery
- Connection lifecycle tracking

**Dependencies**: `evobase-core`, `tokio`

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
```

## Authentication & Authorization

### User Authentication
- Access tokens (JWT, 15 min TTL) for API requests
- Refresh tokens (JWT, 30 day TTL) for token renewal
- Notification tokens (JWT, 30 day TTL) for SSE connections
- Middleware: `require_access_token`

### Admin Authorization
- Static admin token from `ADMIN_TOKEN` env var
- Used for sensitive endpoints like API documentation
- Middleware: `require_admin_token`

## Configuration

Environment variables (see `.env.example`):

```bash
SERVER_ADDR=0.0.0.0:3000
DATABASE_URL=postgres://user:pass@localhost:5432/db
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
- Alternative auth mechanisms (implement `AuthService`)
- API versioning (add `/v2` prefix with new handlers)
