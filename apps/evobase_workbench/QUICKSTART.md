# EvoBase Workbench - Quick Start

## 🚀 Setup (5 minutes)

### 1. Install Dependencies
```bash
cd apps/evobase_workbench
flutter pub get
```

### 2. Generate Code
```bash
# Run code generation for JSON serialization
flutter pub run build_runner build --delete-conflicting-outputs

# Or use the helper script
./scripts/generate.sh
```

### 3. Run the App
```bash
# macOS
flutter run -d macos

# Windows
flutter run -d windows

# Linux
flutter run -d linux
```

## 📁 Project Structure

```
lib/
├── client/              # API Core Layer
│   ├── models/          # DTOs (mirror backend)
│   │   ├── envelope.dart      # ApiResponse<T>, ErrorEnvelope
│   │   ├── auth.dart          # Auth DTOs
│   │   ├── docs.dart          # API Docs DTOs
│   │   └── rest.dart          # REST query params
│   ├── api/             # REST clients
│   │   ├── auth_api.dart      # /auth endpoints
│   │   ├── docs_api.dart      # /docs endpoint
│   │   └── rest_api.dart      # /rest/* endpoints
│   ├── core/            # Base infrastructure
│   │   ├── api_client.dart    # HTTP client with ApiResponse parsing
│   │   └── api_error.dart     # Typed exceptions
│   └── auth/            # Token management
│       ├── token_storage.dart # Secure token storage
│       └── auth_manager.dart  # Auth flow orchestration
├── connector/           # Future extensibility
│   └── interfaces/      # Plugin & connector interfaces
├── lab/                 # Admin tools
│   └── event_monitor/   # SSE listener for /events
├── app/                 # UI Layer
│   ├── blocs/           # State management
│   │   └── auth/        # Auth Bloc
│   └── pages/           # UI screens
│       ├── login_page.dart
│       └── home_page.dart
└── config/              # Configuration
    └── app_config.dart  # Environment settings
```

## 🎯 Key Concepts

### 1. Architecture Flow
```
UI (Pages)
  ↓ dispatch events
Bloc (State Management)
  ↓ call methods
API Clients (auth_api, rest_api, docs_api)
  ↓ use
ApiClient (core HTTP client)
  ↓ parse
ApiResponse<T> or ErrorEnvelope
```

### 2. DTO Consistency
All models mirror `evobase-protocol` exactly:
- Same field names (snake_case in JSON)
- Same structure
- No business logic

### 3. Error Handling
Backend errors are mapped to typed exceptions:
```dart
try {
  await api.query('messages');
} on UnauthorizedException {
  // Handle auth error
} on ValidationException catch (e) {
  // Handle validation (e.field available)
} on NetworkException {
  // Handle network error
}
```

### 4. Token Management
- Automatic token injection
- Auto-refresh on 401
- Admin token support

## 📝 Common Tasks

### Login
```dart
context.read<AuthBloc>().add(
  AuthLoginRequested(
    username: 'user',
    password: 'pass',
  ),
);
```

### Query Data
```dart
final restApi = RestApi(apiClient);
final messages = await restApi.query(
  'messages',
  params: TableQueryParams(
    select: 'id,content,created_at',
    order: 'created_at.desc',
    limit: 10,
  ),
);
```

### Enable Admin Mode
```dart
context.read<AuthBloc>().add(
  AuthAdminTokenSet('your-admin-token'),
);
```

### Get API Docs
```dart
final docsApi = DocsApi(apiClient);
final docs = await docsApi.getDocs();
```

## 🔧 Configuration

### Change Environment
Edit `lib/config/app_config.dart`:
```dart
// Development (default)
AppConfig.setEnvironment(AppConfig.dev);

// Production
AppConfig.setEnvironment(AppConfig.prod);
```

### Backend URL
```dart
static const dev = AppConfig(
  baseUrl: 'http://localhost:3000',  // ← Change this
  environment: 'development',
);
```

## 🏗️ Next Steps

### 1. Build API Explorer (Lab)
- Browse `/docs` endpoint
- Display table schemas
- Show RLS policies

### 2. Build RLS Tester (Lab)
- Test queries with different users
- Visualize RLS policy effects

### 3. Build Event Monitor (Lab)
- Connect to `/events` SSE endpoint
- Display real-time events

### 4. Add More Features
- Table data browser
- Query builder UI
- User management
- Settings panel

## 📚 Documentation

- `README.md` - Overview & features
- `ARCHITECTURE.md` - Detailed architecture guide
- `EXAMPLES.md` - Code examples & patterns
- `QUICKSTART.md` - This file

## 🐛 Troubleshooting

### Code Generation Fails
```bash
flutter clean
flutter pub get
flutter pub run build_runner build --delete-conflicting-outputs
```

### Can't Connect to Backend
1. Check backend is running: `http://localhost:3000/health`
2. Verify `baseUrl` in `app_config.dart`
3. Check CORS settings in backend

### Token Expired
- Tokens auto-refresh on 401
- If refresh fails, user is logged out
- Check token expiry in `TokenStorage`

## 🎨 UI Customization

### Theme
Edit `main.dart`:
```dart
theme: ThemeData(
  colorScheme: ColorScheme.fromSeed(
    seedColor: Colors.blue,  // ← Change color
  ),
),
```

### Navigation
Add routes in `home_page.dart`:
```dart
NavigationRail(
  destinations: [
    // Add your destinations
  ],
)
```

## ✅ Checklist

- [ ] Dependencies installed (`flutter pub get`)
- [ ] Code generated (`build_runner`)
- [ ] Backend running (`localhost:3000`)
- [ ] App runs (`flutter run`)
- [ ] Can login
- [ ] Can query data
- [ ] Admin mode works

## 🚦 Status

✅ Core architecture complete
✅ Auth flow working
✅ API client ready
✅ State management setup
⏳ Lab features (to be implemented)
⏳ UI polish (to be implemented)

## 📞 Support

For issues or questions:
1. Check `ARCHITECTURE.md` for design details
2. Check `EXAMPLES.md` for code patterns
3. Review backend docs in `docs/`
