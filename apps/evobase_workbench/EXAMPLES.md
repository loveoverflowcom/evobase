# EvoBase Workbench - Usage Examples

## Setup & Initialization

### 1. Install Dependencies

```bash
cd apps/evobase_workbench
flutter pub get
```

### 2. Generate Code

```bash
# Run code generation
./scripts/generate.sh

# Or manually
flutter pub run build_runner build --delete-conflicting-outputs
```

### 3. Run Application

```bash
# macOS
flutter run -d macos

# Windows
flutter run -d windows

# Linux
flutter run -d linux
```

## Authentication Examples

### Login

```dart
// In UI
final authBloc = context.read<AuthBloc>();
authBloc.add(AuthLoginRequested(
  username: 'testuser',
  password: 'password123',
));

// Listen to state
BlocListener<AuthBloc, AuthState>(
  listener: (context, state) {
    if (state.status == AuthStatus.authenticated) {
      // Navigate to home
    } else if (state.status == AuthStatus.error) {
      // Show error
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(state.errorMessage ?? 'Error')),
      );
    }
  },
  child: YourWidget(),
)
```

### Register

```dart
authBloc.add(AuthRegisterRequested(
  username: 'newuser',
  password: 'securepass',
));
```

### Logout

```dart
authBloc.add(const AuthLogoutRequested());
```

### Enable Admin Mode

```dart
// Set admin token
authBloc.add(AuthAdminTokenSet('your-admin-token-here'));

// Check if admin mode is enabled
BlocBuilder<AuthBloc, AuthState>(
  builder: (context, state) {
    if (state.isAdminMode) {
      return AdminFeatures();
    }
    return RegularFeatures();
  },
)
```

## API Client Examples

### Direct API Usage

```dart
// Initialize
final apiClient = ApiClient(
  baseUrl: 'http://localhost:3000',
);

final authApi = AuthApi(apiClient);
final docsApi = DocsApi(apiClient);
final restApi = RestApi(apiClient);

// Login
try {
  final response = await authApi.login(
    username: 'user',
    password: 'pass',
  );
  print('User ID: ${response.userId}');
  print('Access Token: ${response.tokens.accessToken}');
} on UnauthorizedException catch (e) {
  print('Login failed: ${e.message}');
} on NetworkException catch (e) {
  print('Network error: ${e.message}');
}
```

### Query Table

```dart
// GET /rest/messages
final messages = await restApi.query(
  'messages',
  params: TableQueryParams(
    select: 'id,content,created_at,user_id',
    order: 'created_at.desc',
    limit: 20,
    offset: 0,
  ),
);

for (final msg in messages) {
  print('${msg['id']}: ${msg['content']}');
}
```

### Insert Data

```dart
// POST /rest/messages (single)
final inserted = await restApi.insert(
  'messages',
  InsertBody.single({
    'content': 'Hello, World!',
    'user_id': 'user-uuid-here',
  }),
);

// POST /rest/messages (multiple)
final insertedMultiple = await restApi.insert(
  'messages',
  InsertBody.multiple([
    {'content': 'Message 1', 'user_id': 'user-1'},
    {'content': 'Message 2', 'user_id': 'user-2'},
  ]),
);
```

### Update Data

```dart
// PATCH /rest/messages?id=eq.123
final updated = await restApi.update(
  'messages',
  PatchBody({'content': 'Updated content'}),
  params: TableQueryParams(
    select: 'id=eq.message-uuid-here',
  ),
);
```

### Delete Data

```dart
// DELETE /rest/messages?id=eq.123
final deleted = await restApi.delete(
  'messages',
  params: TableQueryParams(
    select: 'id=eq.message-uuid-here',
  ),
);
```

### Get API Documentation

```dart
// Requires admin token
apiClient.setAdminToken('your-admin-token');

final docs = await docsApi.getDocs();

for (final table in docs.tables) {
  print('Table: ${table.name}');
  print('Endpoint: ${table.endpoint}');
  print('Methods: GET=${table.methods.get}, POST=${table.methods.post}');
  
  for (final column in table.columns) {
    print('  - ${column.name}: ${column.dataType}');
  }
  
  if (table.rls.enabled) {
    print('RLS Policies:');
    for (final policy in table.rls.policies) {
      print('  - ${policy.name} (${policy.command})');
    }
  }
}
```

## Bloc Pattern Examples

### Create a Custom Bloc

```dart
// 1. Define Events
abstract class MessagesEvent extends Equatable {
  const MessagesEvent();
}

class MessagesLoadRequested extends MessagesEvent {
  @override
  List<Object?> get props => [];
}

class MessageSendRequested extends MessagesEvent {
  final String content;
  
  const MessageSendRequested(this.content);
  
  @override
  List<Object?> get props => [content];
}

// 2. Define State
enum MessagesStatus { initial, loading, loaded, error }

class MessagesState extends Equatable {
  final MessagesStatus status;
  final List<Map<String, dynamic>> messages;
  final String? errorMessage;
  
  const MessagesState({
    this.status = MessagesStatus.initial,
    this.messages = const [],
    this.errorMessage,
  });
  
  MessagesState copyWith({
    MessagesStatus? status,
    List<Map<String, dynamic>>? messages,
    String? errorMessage,
  }) {
    return MessagesState(
      status: status ?? this.status,
      messages: messages ?? this.messages,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
  
  @override
  List<Object?> get props => [status, messages, errorMessage];
}

// 3. Implement Bloc
class MessagesBloc extends Bloc<MessagesEvent, MessagesState> {
  final RestApi _restApi;
  
  MessagesBloc({required RestApi restApi})
      : _restApi = restApi,
        super(const MessagesState()) {
    on<MessagesLoadRequested>(_onLoadRequested);
    on<MessageSendRequested>(_onSendRequested);
  }
  
  Future<void> _onLoadRequested(
    MessagesLoadRequested event,
    Emitter<MessagesState> emit,
  ) async {
    emit(state.copyWith(status: MessagesStatus.loading));
    
    try {
      final messages = await _restApi.query(
        'messages',
        params: TableQueryParams(
          select: 'id,content,created_at,user_id',
          order: 'created_at.desc',
          limit: 50,
        ),
      );
      
      emit(state.copyWith(
        status: MessagesStatus.loaded,
        messages: messages,
      ));
    } on ApiException catch (e) {
      emit(state.copyWith(
        status: MessagesStatus.error,
        errorMessage: e.message,
      ));
    }
  }
  
  Future<void> _onSendRequested(
    MessageSendRequested event,
    Emitter<MessagesState> emit,
  ) async {
    try {
      await _restApi.insert(
        'messages',
        InsertBody.single({
          'content': event.content,
        }),
      );
      
      // Reload messages
      add(MessagesLoadRequested());
    } on ApiException catch (e) {
      emit(state.copyWith(
        status: MessagesStatus.error,
        errorMessage: e.message,
      ));
    }
  }
}

// 4. Use in UI
class MessagesPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => MessagesBloc(
        restApi: context.read<RestApi>(),
      )..add(MessagesLoadRequested()),
      child: BlocBuilder<MessagesBloc, MessagesState>(
        builder: (context, state) {
          if (state.status == MessagesStatus.loading) {
            return CircularProgressIndicator();
          }
          
          if (state.status == MessagesStatus.error) {
            return Text('Error: ${state.errorMessage}');
          }
          
          return ListView.builder(
            itemCount: state.messages.length,
            itemBuilder: (context, index) {
              final msg = state.messages[index];
              return ListTile(
                title: Text(msg['content']),
                subtitle: Text(msg['created_at']),
              );
            },
          );
        },
      ),
    );
  }
}
```

## Error Handling Examples

### Typed Exception Handling

```dart
try {
  final result = await restApi.query('messages');
} on UnauthorizedException catch (e) {
  // Token expired or invalid
  print('Auth error: ${e.message}');
  // Trigger re-login
  authBloc.add(const AuthLogoutRequested());
} on ForbiddenException catch (e) {
  // No permission
  print('Access denied: ${e.message}');
} on NotFoundException catch (e) {
  // Resource not found
  print('Not found: ${e.message}');
} on ValidationException catch (e) {
  // Validation failed
  print('Validation error on ${e.field}: ${e.message}');
} on NetworkException catch (e) {
  // Network issue
  print('Network error: ${e.message}');
  // Show retry dialog
} on ApiException catch (e) {
  // Generic API error
  print('API error [${e.code}]: ${e.message}');
}
```

## SSE Event Monitor Example

```dart
// For lab/event_monitor
final sseListener = SseListener(
  baseUrl: 'http://localhost:3000',
  accessToken: tokenStorage.getAccessToken(),
);

final stream = sseListener.listen();

stream.listen(
  (event) {
    print('Event: ${event.type}');
    print('Data: ${event.data}');
    
    // Parse JSON data
    final json = event.jsonData;
    if (json != null) {
      print('Parsed: $json');
    }
  },
  onError: (error) {
    print('SSE error: $error');
  },
  onDone: () {
    print('SSE connection closed');
  },
);

// Close when done
sseListener.close();
```

## Token Management Examples

### Manual Token Refresh

```dart
final authManager = AuthManager(
  apiClient: apiClient,
  tokenStorage: tokenStorage,
);

// Check if token is expired
if (tokenStorage.isTokenExpired()) {
  try {
    await authManager.refresh();
    print('Token refreshed successfully');
  } on UnauthorizedException {
    print('Refresh token expired, please login again');
    // Redirect to login
  }
}
```

### Auto-Refresh Before API Call

```dart
// Ensure valid token before making request
await authManager.ensureValidToken();

// Now make API call
final result = await restApi.query('messages');
```

## Configuration Examples

### Switch Environment

```dart
// In main.dart
void main() {
  // Development
  AppConfig.setEnvironment(AppConfig.dev);
  
  // Production
  // AppConfig.setEnvironment(AppConfig.prod);
  
  // Custom
  // AppConfig.setEnvironment(AppConfig(
  //   baseUrl: 'https://staging.evobase.com',
  //   environment: 'staging',
  // ));
  
  runApp(MyApp());
}
```

### Access Current Config

```dart
final config = AppConfig.current;
print('Environment: ${config.environment}');
print('Base URL: ${config.baseUrl}');
```

## Testing Examples

### Mock API Client

```dart
class MockApiClient extends Mock implements ApiClient {}

void main() {
  group('AuthBloc', () {
    late MockApiClient mockApiClient;
    late AuthManager authManager;
    late AuthBloc authBloc;
    
    setUp(() {
      mockApiClient = MockApiClient();
      authManager = AuthManager(
        apiClient: mockApiClient,
        tokenStorage: MockTokenStorage(),
      );
      authBloc = AuthBloc(authManager: authManager);
    });
    
    test('login success emits authenticated state', () async {
      // Arrange
      when(() => mockApiClient.post<AuthResponseDto>(
        any(),
        data: any(named: 'data'),
        fromJson: any(named: 'fromJson'),
      )).thenAnswer((_) async => ApiResponse(
        data: AuthResponseDto(
          userId: 'user-123',
          username: 'testuser',
          tokens: TokenDto(...),
        ),
      ));
      
      // Act
      authBloc.add(AuthLoginRequested(
        username: 'testuser',
        password: 'password',
      ));
      
      // Assert
      await expectLater(
        authBloc.stream,
        emitsInOrder([
          isA<AuthState>().having(
            (s) => s.status,
            'status',
            AuthStatus.loading,
          ),
          isA<AuthState>().having(
            (s) => s.status,
            'status',
            AuthStatus.authenticated,
          ),
        ]),
      );
    });
  });
}
```

## Tips & Best Practices

### 1. Always Use Blocs for State
❌ Don't:
```dart
class MyPage extends StatefulWidget {
  void _loadData() async {
    final data = await restApi.query('messages');  // ❌ Direct API call
    setState(() => _messages = data);
  }
}
```

✅ Do:
```dart
class MyPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => MessagesBloc(restApi: restApi)
        ..add(MessagesLoadRequested()),  // ✅ Use Bloc
      child: BlocBuilder<MessagesBloc, MessagesState>(...),
    );
  }
}
```

### 2. Handle All Error Cases
```dart
try {
  await api.query('messages');
} on UnauthorizedException {
  // Re-login
} on NetworkException {
  // Show retry
} on ApiException catch (e) {
  // Generic error handling
}
```

### 3. Use Typed Query Parameters
```dart
// ✅ Good
final params = TableQueryParams(
  select: 'id,content',
  order: 'created_at.desc',
  limit: 10,
);

// ❌ Avoid raw strings
final url = '/rest/messages?select=id,content&order=created_at.desc';
```

### 4. Keep DTOs Pure
```dart
// ✅ Good - Pure DTO
@JsonSerializable()
class MessageDto {
  final String id;
  final String content;
  // No methods, no logic
}

// ❌ Bad - Logic in DTO
class MessageDto {
  String get formattedContent => content.toUpperCase();  // ❌
}
```
