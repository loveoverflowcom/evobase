# EvoBase Workbench - Implementation Checklist

## ✅ Core Architecture (COMPLETED)

### Project Setup
- [x] Flutter project created with desktop support (macOS, Windows, Linux)
- [x] Dependencies configured in `pubspec.yaml`
- [x] Code generation setup with `build_runner`
- [x] `.gitignore` configured
- [x] Helper scripts created

### Folder Structure
- [x] `lib/client/` - API Core Layer
- [x] `lib/connector/` - Integration Layer
- [x] `lib/lab/` - Admin Workbench
- [x] `lib/app/` - Presentation Layer
- [x] `lib/config/` - Configuration

### client/ - API Core Layer
- [x] `models/envelope.dart` - ApiResponse<T>, ErrorEnvelope
- [x] `models/auth.dart` - Auth DTOs (RegisterRequest, LoginRequest, AuthResponseDto, TokenDto)
- [x] `models/docs.dart` - API Docs DTOs (ApiDocsDto, TableDocDto, etc.)
- [x] `models/rest.dart` - REST DTOs (TableQueryParams, InsertBody, PatchBody)
- [x] `models/models.dart` - Barrel export
- [x] `core/api_client.dart` - Base HTTP client with generic parsing
- [x] `core/api_error.dart` - Typed exceptions hierarchy
- [x] `auth/token_storage.dart` - Secure token storage
- [x] `auth/auth_manager.dart` - Auth flow orchestration
- [x] `api/auth_api.dart` - Auth endpoints client
- [x] `api/docs_api.dart` - Docs endpoint client
- [x] `api/rest_api.dart` - REST endpoints client

### connector/ - Integration Layer
- [x] `interfaces/data_connector.dart` - Data source interface
- [x] `interfaces/plugin_interface.dart` - Plugin interface
- [x] `drivers/` - Folder created (future implementations)
- [x] `implementations/` - Folder created (future implementations)

### lab/ - Admin Workbench
- [x] `event_monitor/sse_listener.dart` - SSE client for /events
- [x] `api_explorer/` - Folder created (to implement)
- [x] `rls_tester/` - Folder created (to implement)

### app/ - Presentation Layer
- [x] `blocs/auth/auth_bloc.dart` - Auth Bloc
- [x] `blocs/auth/auth_event.dart` - Auth Events
- [x] `blocs/auth/auth_state.dart` - Auth States
- [x] `pages/login_page.dart` - Login/Register UI
- [x] `pages/home_page.dart` - Main dashboard UI
- [x] `widgets/` - Folder created (future components)
- [x] `routing/` - Folder created (future routing)

### config/
- [x] `app_config.dart` - Environment configuration

### Main Entry
- [x] `main.dart` - App initialization & dependency injection

### Documentation
- [x] `README.md` - Project overview
- [x] `ARCHITECTURE.md` - Detailed architecture guide
- [x] `EXAMPLES.md` - Code examples & patterns
- [x] `QUICKSTART.md` - Quick start guide
- [x] `PROJECT_SUMMARY.md` - Complete project summary
- [x] `CHECKLIST.md` - This file

### Scripts
- [x] `scripts/generate.sh` - Code generation helper

## ⏳ Lab Features (TO IMPLEMENT)

### API Explorer
- [ ] Create `lab/api_explorer/api_explorer_page.dart`
- [ ] Create `lab/api_explorer/api_explorer_bloc.dart`
- [ ] Fetch and display API docs from `/docs`
- [ ] Show table schemas with columns
- [ ] Display RLS policies
- [ ] Show available methods per table
- [ ] Interactive query builder UI
- [ ] Test query execution
- [ ] Response viewer

### RLS Tester
- [ ] Create `lab/rls_tester/rls_tester_page.dart`
- [ ] Create `lab/rls_tester/rls_tester_bloc.dart`
- [ ] User selector (test as different users)
- [ ] Query input with syntax highlighting
- [ ] Execute query with selected user context
- [ ] Display results
- [ ] Compare results between users
- [ ] Visualize RLS policy effects
- [ ] Policy explanation viewer

### Event Monitor
- [ ] Create `lab/event_monitor/event_monitor_page.dart`
- [ ] Create `lab/event_monitor/event_monitor_bloc.dart`
- [ ] Connect to `/events` SSE endpoint
- [ ] Display events in real-time
- [ ] Event type filtering
- [ ] Event history with timestamps
- [ ] JSON viewer for event data
- [ ] Export events to file
- [ ] Clear history button

## 🔮 Future Enhancements (PLANNED)

### Connector Implementations
- [ ] PostgreSQL direct connector
- [ ] SQLite local cache connector
- [ ] REST API connector (generic)
- [ ] GraphQL connector
- [ ] Custom connector template

### Plugin System
- [ ] Plugin loader
- [ ] Plugin registry
- [ ] Data transformer plugins
- [ ] Export/import plugins
- [ ] Custom visualization plugins

### Advanced Features
- [ ] Query history with search
- [ ] Saved queries library
- [ ] Query templates
- [ ] Data visualization (charts, graphs)
- [ ] Batch operations UI
- [ ] Schema migration tools
- [ ] Backup/restore functionality
- [ ] Multi-environment management

### UI/UX Improvements
- [ ] Dark/light theme toggle
- [ ] Custom theme builder
- [ ] Keyboard shortcuts
- [ ] Multi-window support
- [ ] Drag-and-drop interface
- [ ] Responsive layout
- [ ] Accessibility improvements
- [ ] Internationalization (i18n)

### Performance
- [ ] Query result caching
- [ ] Lazy loading for large datasets
- [ ] Virtual scrolling
- [ ] Background sync
- [ ] Offline mode support

### Testing
- [ ] Unit tests for Blocs
- [ ] Unit tests for API clients
- [ ] Widget tests for UI
- [ ] Integration tests
- [ ] E2E tests
- [ ] Performance tests

### DevOps
- [ ] CI/CD pipeline
- [ ] Automated builds
- [ ] Release automation
- [ ] Crash reporting
- [ ] Analytics integration
- [ ] Update mechanism

## 📊 Progress Summary

### Overall Progress: ~60%

#### Completed (60%)
- ✅ Core architecture (100%)
- ✅ API client layer (100%)
- ✅ Auth flow (100%)
- ✅ State management (100%)
- ✅ Basic UI (100%)
- ✅ Documentation (100%)

#### In Progress (0%)
- ⏳ Lab features (0%)

#### Planned (0%)
- 🔮 Advanced features (0%)
- 🔮 UI polish (0%)
- 🔮 Testing (0%)

## 🎯 Next Immediate Tasks

### Priority 1: API Explorer (Week 1)
1. Create API Explorer page layout
2. Implement DocsBloc for state management
3. Fetch and parse `/docs` endpoint
4. Display table list with schemas
5. Show RLS policies per table
6. Add query builder UI
7. Test query execution

### Priority 2: RLS Tester (Week 2)
1. Create RLS Tester page layout
2. Implement RlsBloc for state management
3. Add user selector dropdown
4. Create query input with syntax highlighting
5. Execute queries with user context
6. Display and compare results
7. Add policy explanation viewer

### Priority 3: Event Monitor (Week 3)
1. Create Event Monitor page layout
2. Implement EventMonitorBloc
3. Connect SSE listener to UI
4. Display events in real-time list
5. Add event type filters
6. Implement event history
7. Add JSON viewer for event data

### Priority 4: Polish & Testing (Week 4)
1. Add error boundaries
2. Improve loading states
3. Add empty states
4. Write unit tests for Blocs
5. Write widget tests
6. Fix any bugs found
7. Performance optimization

## 🚀 How to Continue Development

### For API Explorer
```bash
# 1. Create files
touch lib/lab/api_explorer/api_explorer_page.dart
touch lib/lab/api_explorer/api_explorer_bloc.dart
touch lib/lab/api_explorer/api_explorer_event.dart
touch lib/lab/api_explorer/api_explorer_state.dart

# 2. Follow pattern from auth_bloc
# 3. Use DocsApi to fetch data
# 4. Build UI with BlocBuilder
```

### For RLS Tester
```bash
# 1. Create files
touch lib/lab/rls_tester/rls_tester_page.dart
touch lib/lab/rls_tester/rls_tester_bloc.dart
touch lib/lab/rls_tester/rls_tester_event.dart
touch lib/lab/rls_tester/rls_tester_state.dart

# 2. Use RestApi for query execution
# 3. Add user context switching
# 4. Build comparison UI
```

### For Event Monitor
```bash
# 1. Create files
touch lib/lab/event_monitor/event_monitor_page.dart
touch lib/lab/event_monitor/event_monitor_bloc.dart
touch lib/lab/event_monitor/event_monitor_event.dart
touch lib/lab/event_monitor/event_monitor_state.dart

# 2. Use SseListener (already created)
# 3. Stream events to Bloc
# 4. Build real-time UI
```

## 📝 Notes

### Code Generation
Remember to run after adding/modifying models:
```bash
flutter pub run build_runner build --delete-conflicting-outputs
```

### Testing Backend Connection
Before implementing features, ensure backend is running:
```bash
curl http://localhost:3000/health
curl http://localhost:3000/docs -H "X-Admin-Token: your-token"
```

### Admin Token
Set admin token in UI to access lab features:
```dart
authBloc.add(AuthAdminTokenSet('your-admin-token'));
```

### Debugging
Enable verbose logging in ApiClient for debugging:
```dart
// Already configured in api_client.dart
// Check console for request/response logs
```

## ✅ Verification

### Quick Verification Checklist
- [x] Project compiles without errors
- [x] Code generation successful
- [x] All DTOs have `.g.dart` files
- [x] Can run on desktop (macOS/Windows/Linux)
- [x] Login page displays
- [x] Can navigate to home page
- [x] Admin mode toggle works
- [ ] Can connect to backend (requires backend running)
- [ ] Can login successfully (requires backend + DB)
- [ ] Can fetch API docs (requires admin token)

### File Count Verification
- Total Dart files: 26 (19 source + 7 generated)
- Documentation files: 6
- Configuration files: 3
- Scripts: 1

---

**Last Updated**: 2026-03-24
**Status**: Core architecture complete, ready for lab feature implementation
**Next Milestone**: API Explorer implementation
