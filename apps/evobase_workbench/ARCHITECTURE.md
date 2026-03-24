# EvoBase Workbench - Architecture Guide

## Overview

EvoBase Workbench is a Flutter desktop application that mirrors the backend architecture for consistency, type-safety, and maintainability.

## Design Principles

### 1. Mirror Backend Architecture
- DTOs match `evobase-protocol` exactly
- Same naming conventions (snake_case in JSON, camelCase in Dart)
- Strict separation: Domain (backend) ↔ Protocol (DTOs) ↔ Client (Flutter)

### 2. Layered Architecture

```
┌─────────────────────────────────────┐
│         Presentation (app/)         │  ← UI, Blocs, Routing
├─────────────────────────────────────┤
│      Integration (connector/)       │  ← Future extensibility
├─────────────────────────────────────┤
│        API Client (client/)         │  ← HTTP, DTOs, Auth
├─────────────────────────────────────┤
│         Backend (EvoBase)           │  ← REST API
└─────────────────────────────────────┘
```

### 3. Strict Boundaries
- **UI**: Render only, no business logic
- **Bloc**: Orchestrate state, call APIs
- **Client**: Pure API communication
- **Models**: DTOs only, no logic

## Module Details

### client/ - API Core Layer

#### models/
DTOs that mirror `evobase-protocol`:

```dart
// envelope.dart - ApiResponse<T> & ErrorEnvelope
ApiResponse<T> {
  T data;
  ResponseMeta? meta;
}

ErrorEnvelope {
  ErrorDetail error;
}
```

**Rules**:
- Use `json_serializable` for code generation
- Match backend field names exactly
- Use `@JsonKey(name: 'snake_case')` for Rust compatibility
- No business logic

#### core/
Base infrastructure:

```dart
// api_client.dart
class ApiClient {
  Future<ApiResponse<T>> get<T>(...);
  Future<ApiResponse<T>> post<T>(...);
  Future<ApiResponse<T>> patch<T>(...);
  Future<ApiResponse<T>> delete<T>(...);
}
```

**Features**:
- Automatic token injection
- ApiResponse parsing
- Error mapping to typed exceptions
- Request/response logging

```dart
// api_error.dart
abstract class ApiException {
  String code;
  String message;
  String? field;
}

// Typed exceptions
UnauthorizedException
ForbiddenException
NotFoundException
ValidationException
ConflictException
InternalServerException
```

#### auth/
Token management:

```dart
// token_storage.dart
class TokenStorage {
  String? getAccessToken();
  String? getRefreshToken();
  String? getAdminToken();
  bool isTokenExpired();
}

// auth_manager.dart
class AuthManager {
  Future<AuthResponseDto> login(...);
  Future<AuthResponseDto> register(...);
  Future<AuthResponseDto> refresh();
  Future<void> ensureValidToken();
}
```

**Flow**:
1. Login → Save tokens
2. API call → Inject access token
3. 401 response → Auto refresh
4. Refresh success → Retry original request

#### api/
REST API clients:

```dart
// auth_api.dart
class AuthApi {
  Future<AuthResponseDto> login(...);
  Future<AuthResponseDto> register(...);
  Future<AuthResponseDto> refresh(...);
}

// docs_api.dart
class DocsApi {
  Future<ApiDocsDto> getDocs();  // Requires admin token
}

// rest_api.dart
class RestApi {
  Future<List<Map>> query(String table, ...);
  Future<List<Map>> insert(String table, ...);
  Future<List<Map>> update(String table, ...);
  Future<List<Map>> delete(String table, ...);
}
```

### connector/ - Integration Layer

Prepare for future extensibility:

```dart
// interfaces/data_connector.dart
abstract class DataConnector {
  Future<void> connect();
  Future<Map<String, dynamic>> query(String query);
}

// interfaces/plugin_interface.dart
abstract class PluginInterface {
  String get name;
  Future<void> initialize();
  Future<dynamic> execute(String action, Map params);
}
```

**Use cases**:
- External system integration
- Local database (SQLite)
- Custom plugins
- Data import/export

### lab/ - Admin Workbench

Admin-only debugging tools:

```dart
// event_monitor/sse_listener.dart
class SseListener {
  Stream<SseEvent> listen();  // /events endpoint
}
```

**Features**:
- API Docs explorer (`/docs`)
- RLS policy testing
- SSE event monitoring (`/events`)
- Admin-only access (requires ADMIN_TOKEN)

### app/ - Presentation Layer

#### blocs/
State management with Bloc pattern:

```dart
// auth/auth_bloc.dart
class AuthBloc extends Bloc<AuthEvent, AuthState> {
  // Events
  AuthLoginRequested
  AuthRegisterRequested
  AuthLogoutRequested
  AuthRefreshRequested
  AuthAdminTokenSet
  
  // States
  AuthStatus: initial, loading, authenticated, unauthenticated, error
}
```

**Rules**:
- UI dispatches events
- Bloc handles business logic
- Bloc calls API clients
- UI reacts to state changes

#### pages/
Screen implementations:

```dart
// login_page.dart
class LoginPage extends StatefulWidget {
  // Form validation
  // Dispatch AuthLoginRequested
  // Listen to AuthState
}

// home_page.dart
class HomePage extends StatelessWidget {
  // Navigation rail
  // Admin mode toggle
  // Feature routing
}
```

## Data Flow

### Authentication Flow

```
UI (LoginPage)
  ↓ dispatch AuthLoginRequested
AuthBloc
  ↓ call login()
AuthManager
  ↓ call login()
AuthApi
  ↓ POST /auth/login
ApiClient
  ↓ parse ApiResponse<AuthResponseDto>
  ↓ save tokens
TokenStorage
  ↓ emit AuthState.authenticated
AuthBloc
  ↓ rebuild
UI (HomePage)
```

### API Call Flow

```
UI
  ↓ dispatch event
Bloc
  ↓ call API
ApiClient
  ↓ inject token
  ↓ HTTP request
Backend
  ↓ ApiResponse<T> or ErrorEnvelope
ApiClient
  ↓ parse response
  ↓ map errors to exceptions
Bloc
  ↓ emit new state
UI
```

### Auto-Refresh Flow

```
ApiClient
  ↓ 401 Unauthorized
  ↓ check if refresh available
AuthManager
  ↓ call refresh()
  ↓ POST /auth/refresh
  ↓ save new tokens
  ↓ retry original request
ApiClient
```

## Error Handling

### Backend Error Format

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid credentials",
    "field": null
  }
}
```

### Client Mapping

```dart
try {
  final result = await api.query('messages');
} on UnauthorizedException catch (e) {
  // Handle auth error
} on ValidationException catch (e) {
  // Handle validation error (e.field available)
} on NetworkException catch (e) {
  // Handle network error
} on ApiException catch (e) {
  // Handle any API error
}
```

## Admin Mode

### Enabling Admin Mode

```dart
// Set admin token
authBloc.add(AuthAdminTokenSet('your-admin-token'));

// Check admin mode
if (authState.isAdminMode) {
  // Show lab features
}
```

### Admin-Protected Features

- API Docs explorer (`/docs`)
- RLS policy viewer
- SSE event monitor (`/events`)
- System configuration

## Configuration

### Environment Switching

```dart
// lib/config/app_config.dart
class AppConfig {
  static const dev = AppConfig(
    baseUrl: 'http://localhost:3000',
    environment: 'development',
  );
  
  static const prod = AppConfig(
    baseUrl: 'https://api.evobase.com',
    environment: 'production',
  );
}

// In main.dart
void main() {
  AppConfig.setEnvironment(AppConfig.dev);
  // ...
}
```

## Code Generation

### Models with json_serializable

```dart
@JsonSerializable()
class AuthResponseDto {
  @JsonKey(name: 'user_id')
  final String userId;
  final String username;
  final TokenDto tokens;
  
  // Constructor, fromJson, toJson
}
```

### Generate Code

```bash
# One-time generation
flutter pub run build_runner build --delete-conflicting-outputs

# Watch mode (auto-regenerate on changes)
flutter pub run build_runner watch
```

## Testing Strategy

### Unit Tests
- Test Blocs in isolation
- Mock API clients
- Test error handling

### Integration Tests
- Test API client with mock server
- Test auth flow end-to-end
- Test token refresh

### Widget Tests
- Test UI components
- Test form validation
- Test navigation

## Best Practices

### 1. DTO Consistency
✅ DO: Mirror backend exactly
```dart
@JsonKey(name: 'user_id')  // Match Rust snake_case
final String userId;
```

❌ DON'T: Rename or restructure
```dart
final String id;  // Wrong! Backend uses user_id
```

### 2. Business Logic Location
✅ DO: Put logic in Bloc
```dart
class AuthBloc {
  Future<void> _onLoginRequested(...) async {
    // Validation, API calls, state updates
  }
}
```

❌ DON'T: Put logic in UI
```dart
class LoginPage {
  void _submit() {
    // ❌ Don't validate or call API directly
  }
}
```

### 3. Error Handling
✅ DO: Use typed exceptions
```dart
try {
  await api.login(...);
} on UnauthorizedException {
  // Specific handling
}
```

❌ DON'T: Catch generic exceptions
```dart
try {
  await api.login(...);
} catch (e) {
  // ❌ Too generic
}
```

### 4. Token Management
✅ DO: Use AuthManager
```dart
await authManager.ensureValidToken();
final result = await api.query(...);
```

❌ DON'T: Manage tokens manually
```dart
// ❌ Don't inject tokens yourself
```

## Future Enhancements

### Planned Features
1. **API Explorer**: Interactive docs browser
2. **RLS Tester**: Test RLS policies with different users
3. **Event Monitor**: Real-time SSE event viewer
4. **Query Builder**: Visual query constructor
5. **Schema Inspector**: Database schema viewer

### Extensibility Points
- `connector/` for external integrations
- `PluginInterface` for custom plugins
- `DataConnector` for alternative data sources

## Troubleshooting

### Code Generation Issues
```bash
# Clean and regenerate
flutter clean
flutter pub get
flutter pub run build_runner build --delete-conflicting-outputs
```

### Token Refresh Loop
- Check token expiry calculation
- Verify refresh endpoint response
- Check TokenStorage implementation

### Admin Mode Not Working
- Verify ADMIN_TOKEN is set correctly
- Check backend middleware configuration
- Ensure admin endpoints require token

## References

- Backend: `crates/evobase-protocol/`
- DTOs: `crates/evobase-protocol/src/*.rs`
- API Docs: `docs/ARCHITECTURE.md`
