# EvoBase Workbench

Desktop admin/management application for EvoBase built with Flutter.

## Architecture

This application mirrors the backend architecture for consistency and maintainability:

```
lib/
├── client/              # API Core Layer
│   ├── models/          # DTOs (mirror evobase-protocol)
│   │   ├── envelope.dart
│   │   ├── auth.dart
│   │   ├── docs.dart
│   │   └── rest.dart
│   ├── api/             # REST clients
│   │   ├── auth_api.dart
│   │   ├── docs_api.dart
│   │   └── rest_api.dart
│   ├── core/            # Base ApiClient & error handling
│   │   ├── api_client.dart
│   │   └── api_error.dart
│   └── auth/            # Token management
│       ├── token_storage.dart
│       └── auth_manager.dart
├── connector/           # Integration Layer (extensibility)
│   ├── interfaces/
│   ├── drivers/
│   └── implementations/
├── lab/                 # Admin Workbench
│   ├── api_explorer/
│   ├── rls_tester/
│   └── event_monitor/
├── app/                 # Presentation Layer
│   ├── pages/
│   ├── widgets/
│   ├── blocs/
│   └── routing/
└── config/              # Configuration
    └── app_config.dart
```

## Key Features

### 1. DTO Consistency
All models mirror `evobase-protocol` DTOs:
- Same naming conventions as Rust
- `json_serializable` for type-safe serialization
- No business logic in DTOs

### 2. API Response Handling
Unified response format:
```dart
ApiResponse<T> {
  data: T,
  meta: ResponseMeta?
}

ErrorEnvelope {
  error: ErrorDetail
}
```

### 3. Base ApiClient
Reusable HTTP client with:
- Automatic token injection
- ApiResponse parsing
- Error mapping to typed exceptions
- Logging

### 4. Auth Strategy
- Auto-attach AccessToken
- Auto-refresh on 401
- Support ADMIN_TOKEN for admin APIs
- Admin Guard in UI

### 5. State Management
- **Bloc** for state orchestration
- UI = render only
- Bloc = orchestrates
- Client = pure API layer

## Getting Started

### Prerequisites
- Flutter 3.0+
- Dart 3.0+

### Installation

1. Install dependencies:
```bash
flutter pub get
```

2. Generate code:
```bash
flutter pub run build_runner build --delete-conflicting-outputs
```

3. Run the app:
```bash
flutter run -d macos  # or windows, linux
```

## Configuration

Edit `lib/config/app_config.dart` to set environment:

```dart
// Development
AppConfig.setEnvironment(AppConfig.dev);

// Production
AppConfig.setEnvironment(AppConfig.prod);
```

## Usage Examples

### Login
```dart
final authBloc = context.read<AuthBloc>();
authBloc.add(AuthLoginRequested(
  username: 'user',
  password: 'pass',
));
```

### Enable Admin Mode
```dart
authBloc.add(AuthAdminTokenSet('your-admin-token'));
```

### Fetch API Docs
```dart
final docsApi = DocsApi(apiClient);
final docs = await docsApi.getDocs();
```

### Query Table
```dart
final restApi = RestApi(apiClient);
final results = await restApi.query(
  'messages',
  params: TableQueryParams(
    select: 'id,content,created_at',
    order: 'created_at.desc',
    limit: 10,
  ),
);
```

## Development

### Code Generation
When adding/modifying models:
```bash
flutter pub run build_runner watch
```

### Architecture Rules
1. **DTO Consistency**: Mirror backend exactly
2. **No Business Logic in UI**: Use Blocs
3. **Typed Exceptions**: Use ApiException hierarchy
4. **Token Management**: Use AuthManager
5. **Admin Guard**: Check `isAdminMode` before accessing lab features

## Project Structure

- `client/` - Pure API layer, no UI dependencies
- `connector/` - Future extensibility (plugins, external systems)
- `lab/` - Admin-only debugging tools
- `app/` - Flutter UI with Bloc state management
- `config/` - Environment configuration

## Backend Integration

This app integrates with EvoBase backend:
- `evobase-gateway` - REST API endpoints
- `evobase-protocol` - DTO definitions
- `evobase-auth` - Authentication service

All DTOs are kept in sync with backend protocol layer.

## License

Private - EvoBase Project
