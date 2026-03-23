# Refactoring Notes

## What Changed

### New Module: `evobase-protocol`

Created a new crate to hold all DTOs and API response formats. This separates the wire format from internal domain types.

**Benefits**:
- DTOs can evolve independently from domain logic
- Clear boundary between HTTP layer and business logic
- Easier to version APIs in the future
- Can be shared with client SDKs

### Gateway Restructuring

Reorganized `evobase-gateway` into a cleaner structure:

**Before**:
```
src/
├── lib.rs
├── error.rs
├── middleware.rs
├── rest.rs
├── routes.rs (contained all handlers)
└── state.rs
```

**After**:
```
src/
├── lib.rs
├── error.rs
├── middleware.rs
├── rest.rs
├── routes.rs (only router assembly)
├── state.rs
└── handlers/
    ├── mod.rs
    ├── auth.rs
    ├── rest.rs
    ├── docs.rs
    ├── health.rs
    └── messaging.rs
```

**Benefits**:
- Each handler module is focused on one domain
- Easier to find and modify specific endpoints
- Better code organization for future growth

### Admin Authorization

Added admin token authentication for sensitive endpoints.

**Changes**:
1. Added `ADMIN_TOKEN` to configuration
2. Created `require_admin_token` middleware
3. Protected `/docs` endpoints with admin auth

**Usage**:
```bash
# Set in .env
ADMIN_TOKEN=your-secret-admin-token

# Call admin endpoints
curl -H "Authorization: Bearer your-secret-admin-token" \
  http://localhost:3000/docs
```

### Response Format Standardization

All responses now use consistent envelopes:

**Success**:
```json
{
  "data": { ... },
  "meta": { ... }  // optional
}
```

**Error**:
```json
{
  "error": {
    "code": "error_code",
    "message": "Human readable message",
    "field": "field_name"  // optional
  }
}
```

## Migration Guide

### For Existing Code

If you have code calling the API, update to handle the new response format:

**Before**:
```javascript
const response = await fetch('/auth/login', { ... });
const data = await response.json();
console.log(data.user_id);  // Direct access
```

**After**:
```javascript
const response = await fetch('/auth/login', { ... });
const result = await response.json();
console.log(result.data.user_id);  // Access via .data
```

**Error handling**:
```javascript
if (!response.ok) {
  const error = await response.json();
  console.log(error.error.code);     // "unauthorized"
  console.log(error.error.message);  // "unauthorized"
}
```

### For New Endpoints

When adding new endpoints, follow this pattern:

1. **Define DTOs in `evobase-protocol`**:
```rust
// crates/evobase-protocol/src/my_feature.rs
#[derive(Deserialize)]
pub struct MyRequest {
    pub field: String,
}

#[derive(Serialize)]
pub struct MyResponseDto {
    pub result: String,
}

impl From<core::MyResponse> for MyResponseDto {
    fn from(r: core::MyResponse) -> Self {
        Self { result: r.result }
    }
}
```

2. **Create handler in `evobase-gateway/src/handlers/`**:
```rust
// crates/evobase-gateway/src/handlers/my_feature.rs
use evobase_protocol::{ApiResponse, MyRequest, MyResponseDto};

pub async fn my_handler(
    State(state): State<AppState>,
    Json(request): Json<MyRequest>,
) -> ApiResult<Json<ApiResponse<MyResponseDto>>> {
    let domain_req = request.into();
    let result = state.my_service.do_something(domain_req).await?;
    let dto = MyResponseDto::from(result);
    Ok(Json(ApiResponse::new(dto)))
}
```

3. **Register route in `routes.rs`**:
```rust
Router::new()
    .route("/my-endpoint", post(handlers::my_feature::my_handler))
```

## Breaking Changes

### Response Format
All endpoints now return data wrapped in `{ "data": ... }`. Update client code accordingly.

### Admin Endpoints
`/docs` endpoints now require admin token. Set `ADMIN_TOKEN` in environment and include in Authorization header.

### Error Format
Error responses now use structured format with `code`, `message`, and optional `field`.

## Non-Breaking Changes

- Internal code organization (handlers split into modules)
- DTO layer (internal only, wire format unchanged except envelope)
- Middleware structure (behavior unchanged)

## Testing

After refactoring, test these scenarios:

1. **User registration and login** - Verify token generation
2. **Protected endpoints** - Verify access token validation
3. **Admin endpoints** - Verify admin token validation
4. **Error responses** - Verify error format
5. **SSE connections** - Verify messaging still works

## Future Work

This refactoring sets the foundation for:

1. **API Versioning**: Add `/v2` prefix with new handlers
2. **Admin API Expansion**: Add more admin endpoints (user management, system stats)
3. **Plugin System**: Services can be swapped via trait implementations
4. **Client SDK**: Share `evobase-protocol` with client libraries
5. **GraphQL Layer**: Add alongside REST using same services
