# EvoBase Workbench - Project Summary

## 🎯 Project Overview

**EvoBase Workbench** là ứng dụng desktop admin/management cho EvoBase, được xây dựng bằng Flutter với kiến trúc mirror backend để đảm bảo consistency và maintainability.

## 📊 Architecture Highlights

### Layered Architecture
```
┌─────────────────────────────────────┐
│    app/ (Presentation Layer)       │
│    - UI với Bloc state management  │
│    - Pages, Widgets, Routing       │
├─────────────────────────────────────┤
│    lab/ (Admin Workbench)          │
│    - API Explorer                  │
│    - RLS Tester                    │
│    - Event Monitor (SSE)           │
├─────────────────────────────────────┤
│    connector/ (Integration)        │
│    - Plugin interfaces             │
│    - Future extensibility          │
├─────────────────────────────────────┤
│    client/ (API Core Layer)        │
│    - DTOs (mirror protocol)        │
│    - REST clients                  │
│    - Auth management               │
│    - Error handling                │
└─────────────────────────────────────┘
```

### Key Design Principles

1. **DTO Consistency**: Mirror `evobase-protocol` exactly
2. **Separation of Concerns**: UI ≠ Business Logic ≠ API
3. **Type Safety**: Typed exceptions, generic ApiResponse<T>
4. **Token Management**: Auto-refresh, admin token support
5. **Extensibility**: Connector layer for future integrations

## 📁 Complete File Structure

```
apps/evobase_workbench/
├── lib/
│   ├── client/                          # API Core Layer
│   │   ├── models/                      # DTOs
│   │   │   ├── envelope.dart            # ApiResponse<T>, ErrorEnvelope
│   │   │   ├── envelope.g.dart          # Generated
│   │   │   ├── auth.dart                # Auth DTOs
│   │   │   ├── auth.g.dart              # Generated
│   │   │   ├── docs.dart                # API Docs DTOs
│   │   │   ├── docs.g.dart              # Generated
│   │   │   ├── rest.dart                # REST query params
│   │   │   ├── rest.g.dart              # Generated
│   │   │   └── models.dart              # Barrel export
│   │   ├── api/                         # REST Clients
│   │   │   ├── auth_api.dart            # /auth endpoints
│   │   │   ├── docs_api.dart            # /docs endpoint
│   │   │   └── rest_api.dart            # /rest/* endpoints
│   │   ├── core/                        # Base Infrastructure
│   │   │   ├── api_client.dart          # HTTP client + parsing
│   │   │   └── api_error.dart           # Typed exceptions
│   │   └── auth/                        # Token Management
│   │       ├── token_storage.dart       # Secure storage
│   │       └── auth_manager.dart        # Auth orchestration
│   ├── connector/                       # Integration Layer
│   │   ├── interfaces/
│   │   │   ├── data_connector.dart      # Data source interface
│   │   │   └── plugin_interface.dart    # Plugin interface
│   │   ├── drivers/                     # (Future)
│   │   └── implementations/             # (Future)
│   ├── lab/                             # Admin Workbench
│   │   ├── api_explorer/                # (To implement)
│   │   ├── rls_tester/                  # (To implement)
│   │   └── event_monitor/
│   │       └── sse_listener.dart        # SSE client for /events
│   ├── app/                             # Presentation Layer
│   │   ├── blocs/                       # State Management
│   │   │   └── auth/
│   │   │       ├── auth_bloc.dart       # Auth Bloc
│   │   │       ├── auth_event.dart      # Auth Events
│   │   │       └── auth_state.dart      # Auth States
│   │   ├── pages/                       # UI Screens
│   │   │   ├── login_page.dart          # Login/Register
│   │   │   └── home_page.dart           # Main dashboard
│   │   ├── widgets/                     # (Future)
│   │   └── routing/                     # (Future)
│   ├── config/                          # Configuration
│   │   └── app_config.dart              # Environment config
│   └── main.dart                        # App entry point
├── scripts/
│   └── generate.sh                      # Code generation helper
├── QUICKSTART.md                        # Quick start guide
├── README.md                            # Project overview
├── ARCHITECTURE.md                      # Detailed architecture
├── EXAMPLES.md                          # Code examples
├── PROJECT_SUMMARY.md                   # This file
├── pubspec.yaml                         # Dependencies
└── .gitignore                           # Git ignore rules
```

## 🔑 Core Components

### 1. ApiClient (client/core/api_client.dart)
**Purpose**: Base HTTP client cho tất cả API calls

**Features**:
- Generic methods: `get<T>()`, `post<T>()`, `patch<T>()`, `delete<T>()`
- Auto token injection (access + admin)
- ApiResponse parsing
- Error mapping to typed exceptions
- Request/response logging

**Usage**:
```dart
final client = ApiClient(baseUrl: 'http://localhost:3000');
final response = await client.get<AuthResponseDto>(
  '/auth/login',
  fromJson: (json) => AuthResponseDto.fromJson(json),
);
```

### 2. DTOs (client/models/)
**Purpose**: Mirror backend `evobase-protocol` DTOs

**Features**:
- `json_serializable` for code generation
- Exact field name matching (snake_case ↔ camelCase)
- No business logic
- Type-safe serialization

**Example**:
```dart
@JsonSerializable()
class AuthResponseDto {
  @JsonKey(name: 'user_id')
  final String userId;
  final String username;
  final TokenDto tokens;
}
```

### 3. AuthManager (client/auth/auth_manager.dart)
**Purpose**: Orchestrate authentication flow

**Features**:
- Login/Register/Refresh
- Auto token refresh on 401
- Token storage management
- Admin token support

**Usage**:
```dart
final authManager = AuthManager(
  apiClient: apiClient,
  tokenStorage: tokenStorage,
);
await authManager.login(username: 'user', password: 'pass');
```

### 4. AuthBloc (app/blocs/auth/)
**Purpose**: State management cho authentication

**Features**:
- Event-driven architecture
- Typed events & states
- UI orchestration
- Error handling

**Usage**:
```dart
authBloc.add(AuthLoginRequested(
  username: 'user',
  password: 'pass',
));
```

### 5. REST API Clients (client/api/)
**Purpose**: Thin wrappers cho specific endpoints

**Components**:
- `AuthApi`: /auth endpoints
- `DocsApi`: /docs endpoint (admin)
- `RestApi`: /rest/* dynamic table operations

**Usage**:
```dart
final restApi = RestApi(apiClient);
final messages = await restApi.query('messages', params: ...);
```

## 🔄 Data Flow Examples

### Login Flow
```
LoginPage
  ↓ User submits form
  ↓ dispatch AuthLoginRequested
AuthBloc
  ↓ call authManager.login()
AuthManager
  ↓ call authApi.login()
AuthApi
  ↓ call apiClient.post()
ApiClient
  ↓ HTTP POST /auth/login
  ↓ parse ApiResponse<AuthResponseDto>
  ↓ return response
AuthManager
  ↓ save tokens to TokenStorage
  ↓ inject token to ApiClient
  ↓ return AuthResponseDto
AuthBloc
  ↓ emit AuthState.authenticated
LoginPage
  ↓ BlocListener detects state change
  ↓ Navigate to HomePage
```

### Query Flow with Auto-Refresh
```
UI
  ↓ dispatch LoadMessagesRequested
MessagesBloc
  ↓ call restApi.query('messages')
RestApi
  ↓ call apiClient.get()
ApiClient
  ↓ inject access token
  ↓ HTTP GET /rest/messages
Backend
  ↓ 401 Unauthorized (token expired)
ApiClient
  ↓ catch 401
  ↓ call authManager.refresh()
AuthManager
  ↓ POST /auth/refresh
  ↓ save new tokens
  ↓ return success
ApiClient
  ↓ retry original request with new token
  ↓ HTTP GET /rest/messages
Backend
  ↓ 200 OK with data
ApiClient
  ↓ parse ApiResponse<List<dynamic>>
  ↓ return data
MessagesBloc
  ↓ emit MessagesState.loaded
UI
  ↓ display messages
```

## 🛠️ Technology Stack

### Core
- **Flutter**: 3.41.5
- **Dart**: 3.11.3

### Dependencies
- `http`: HTTP client
- `flutter_bloc`: State management
- `equatable`: Value equality
- `json_annotation`: JSON serialization
- `shared_preferences`: Local storage
- `hive`: NoSQL database (future)
- `logger`: Logging
- `uuid`: UUID generation

### Dev Dependencies
- `build_runner`: Code generation
- `json_serializable`: JSON codegen
- `hive_generator`: Hive codegen
- `flutter_lints`: Linting

## 📋 Implementation Status

### ✅ Completed
- [x] Project structure
- [x] Core ApiClient with generic parsing
- [x] All DTOs (envelope, auth, docs, rest)
- [x] Error handling with typed exceptions
- [x] Token management (storage + auto-refresh)
- [x] Auth flow (login, register, refresh)
- [x] AuthBloc with full state management
- [x] Login/Home UI pages
- [x] Admin mode support
- [x] SSE listener for event monitoring
- [x] Configuration system
- [x] Code generation setup
- [x] Documentation (README, ARCHITECTURE, EXAMPLES, QUICKSTART)

### ⏳ To Implement (Lab Features)
- [ ] API Explorer UI
  - [ ] Browse /docs endpoint
  - [ ] Display table schemas
  - [ ] Show RLS policies
  - [ ] Interactive query builder
- [ ] RLS Tester UI
  - [ ] Test queries with different users
  - [ ] Visualize policy effects
  - [ ] Compare results
- [ ] Event Monitor UI
  - [ ] Display SSE events in real-time
  - [ ] Filter by event type
  - [ ] Event history
- [ ] Settings Panel
  - [ ] Environment switcher
  - [ ] Admin token management
  - [ ] Theme customization

### 🔮 Future Enhancements
- [ ] Connector implementations
  - [ ] PostgreSQL direct connection
  - [ ] SQLite local cache
  - [ ] External API integrations
- [ ] Plugin system
  - [ ] Custom data transformers
  - [ ] Export/import plugins
- [ ] Advanced features
  - [ ] Query history
  - [ ] Saved queries
  - [ ] Data visualization
  - [ ] Batch operations

## 🎓 Learning Resources

### For New Developers

1. **Start Here**:
   - Read `QUICKSTART.md` for setup
   - Run the app and explore UI
   - Read `ARCHITECTURE.md` for design

2. **Understand Flow**:
   - Trace login flow in code
   - See how Bloc orchestrates
   - Understand ApiClient parsing

3. **Add Features**:
   - Start with simple Bloc (e.g., MessagesBloc)
   - Add corresponding UI page
   - Follow existing patterns

4. **Code Examples**:
   - Check `EXAMPLES.md` for patterns
   - Copy-paste and adapt
   - Follow best practices

### Key Patterns to Learn

1. **Bloc Pattern**:
   - Events → Bloc → States → UI
   - No business logic in UI
   - Pure functions in Bloc

2. **API Client Pattern**:
   - Generic `ApiResponse<T>` parsing
   - Typed exceptions
   - Automatic error mapping

3. **DTO Pattern**:
   - Mirror backend exactly
   - Use `json_serializable`
   - No logic in DTOs

4. **Token Management**:
   - Automatic injection
   - Auto-refresh on 401
   - Secure storage

## 🚀 Getting Started

### Quick Setup (5 minutes)
```bash
cd apps/evobase_workbench
flutter pub get
flutter pub run build_runner build --delete-conflicting-outputs
flutter run -d macos
```

### First Task: Add a Feature
1. Create a new Bloc (copy `auth_bloc` pattern)
2. Add corresponding UI page
3. Wire up in `main.dart`
4. Test the flow

### Example: Messages Feature
```dart
// 1. Create MessagesBloc
class MessagesBloc extends Bloc<MessagesEvent, MessagesState> {
  final RestApi restApi;
  // ... implement
}

// 2. Create MessagesPage
class MessagesPage extends StatelessWidget {
  // ... implement with BlocBuilder
}

// 3. Add to navigation
NavigationRail(
  destinations: [
    NavigationRailDestination(
      icon: Icon(Icons.message),
      label: Text('Messages'),
    ),
  ],
)
```

## 📞 Support & Contribution

### Questions?
1. Check documentation files
2. Review code examples
3. Trace existing implementations

### Contributing
1. Follow existing patterns
2. Mirror backend DTOs exactly
3. Keep UI logic-free
4. Use Bloc for orchestration
5. Handle all error cases

## 🎯 Project Goals

### Primary Goals
✅ Mirror backend architecture
✅ Type-safe API communication
✅ Clean separation of concerns
✅ Extensible design
✅ Admin tools for debugging

### Success Criteria
✅ Can login/register users
✅ Can query/insert/update/delete data
✅ Auto token refresh works
✅ Admin mode functional
⏳ Lab tools implemented
⏳ Production-ready UI

## 📈 Next Steps

### Immediate (Week 1)
1. Implement API Explorer UI
2. Add table data browser
3. Build query builder

### Short-term (Month 1)
1. Complete RLS Tester
2. Finish Event Monitor UI
3. Add settings panel
4. Polish UI/UX

### Long-term (Quarter 1)
1. Implement connector drivers
2. Add plugin system
3. Advanced features
4. Performance optimization

---

**Status**: Core architecture complete, ready for feature development
**Last Updated**: 2026-03-24
**Version**: 1.0.0
