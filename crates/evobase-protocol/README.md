# evobase-protocol

Protocol layer for EvoBase containing DTOs (Data Transfer Objects) and API response envelopes.

## Purpose

This crate defines the wire format for HTTP requests and responses. It acts as a translation layer between HTTP JSON and internal domain types.

## Key Concepts

### DTOs vs Domain Types

- **DTOs** (this crate): What goes over the wire (JSON)
- **Domain Types** (`evobase-core`): Internal business logic types

DTOs can evolve independently from domain types. The `From`/`Into` traits provide the mapping.

### Response Envelope

All successful responses are wrapped in `ApiResponse<T>`:

```rust
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub meta: Option<ResponseMeta>,
}
```

Example:
```json
{
  "data": {
    "user_id": "...",
    "username": "john",
    "tokens": { ... }
  }
}
```

### Error Envelope

All errors use `ErrorEnvelope`:

```rust
#[derive(Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
    pub field: Option<String>,
}
```

Example:
```json
{
  "error": {
    "code": "bad_request",
    "message": "username must be between 3 and 64 characters",
    "field": "username"
  }
}
```

## Modules

### `envelope`
- `ApiResponse<T>` - Success response wrapper
- `ErrorEnvelope` - Error response wrapper
- `ResponseMeta` - Pagination metadata

### `auth`
- `RegisterRequest` - User registration
- `LoginRequest` - User login
- `RefreshRequest` - Token refresh
- `AuthResponseDto` - Authentication response
- `TokenDto` - Token bundle

### `rest`
- `TableQueryParams` - Query string parameters
- `InsertBody` - Insert request body
- `PatchBody` - Update request body

### `docs`
- `ApiDocsDto` - API documentation
- `TableDocDto` - Table documentation
- `ColumnDocDto` - Column metadata
- `RlsDocDto` - Row-level security info

## Usage Example

```rust
use evobase_protocol::{ApiResponse, LoginRequest, AuthResponseDto};

// In a handler
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,  // DTO in
) -> ApiResult<Json<ApiResponse<AuthResponseDto>>> {
    let domain_req = request.into();    // DTO → domain
    let result = state.auth_service.login(domain_req).await?;
    let dto = AuthResponseDto::from(result);  // domain → DTO
    Ok(Json(ApiResponse::new(dto)))
}
```

## Dependencies

- `evobase-core` - Domain types for conversion
- `serde` - Serialization
- `uuid` - UUID support
