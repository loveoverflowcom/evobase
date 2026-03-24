# Evobase Flutter Client

Official Flutter client for Evobase - a modern backend-as-a-service platform.

## Features

- 🔐 **Authentication** - Login, register, token refresh
- 📊 **REST API** - Full CRUD operations on any table
- ⚡ **Realtime** - Server-Sent Events (SSE) for live updates
- 🎯 **Type-safe** - Functional programming with fpdart
- 🔧 **Easy to use** - Simple, intuitive API

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  evobase_flutter:
    path: ../path/to/evobase_flutter
```

Or from pub.dev (when published):

```yaml
dependencies:
  evobase_flutter: ^0.1.0
```

## Quick Start

### 1. Initialize the client

```dart
import 'package:evobase_flutter/evobase.dart';

final client = await EvobaseClient.create(
  baseUrl: 'http://localhost:3000',
);
```

### 2. Authentication

```dart
// Register
final registerResult = await client.auth.register(
  username: 'john',
  password: 'password123',
).run();

registerResult.match(
  (error) => print('Error: ${error.message}'),
  (response) => print('Welcome ${response.username}!'),
);

// Login
final loginResult = await client.auth.login(
  username: 'john',
  password: 'password123',
).run();

// Check login status
if (client.auth.isLoggedIn()) {
  print('User is logged in');
}

// Logout
await client.auth.logout();
```

### 3. REST API Operations

#### Query (SELECT)

```dart
final result = await client.rest.query(
  'users',
  params: QueryParams(
    select: ['id', 'username', 'email'],
    filter: {'status': 'active'},
    order: ['created_at.desc'],
    limit: 10,
    offset: 0,
  ),
).run();

result.match(
  (error) => print('Error: ${error.message}'),
  (users) => print('Found ${users.length} users'),
);
```

#### Insert

```dart
final result = await client.rest.insert(
  'users',
  InsertBody(
    records: [
      {'username': 'alice', 'email': 'alice@example.com'},
      {'username': 'bob', 'email': 'bob@example.com'},
    ],
    returning: ['id', 'username'],
  ),
).run();
```

#### Update

```dart
final result = await client.rest.update(
  'users',
  UpdateBody(
    set: {'status': 'inactive'},
    returning: ['id', 'status'],
  ),
  params: QueryParams(
    filter: {'id.eq': '123'},
  ),
).run();
```

#### Delete

```dart
final result = await client.rest.delete(
  'users',
  params: QueryParams(
    filter: {'id.eq': '123'},
  ),
).run();
```

### 4. Realtime Events

```dart
// Listen to all events
client.realtime.listen().listen((event) {
  print('Event type: ${event.type}');
  print('Event data: ${event.jsonData}');
  
  // Handle different event types
  switch (event.type) {
    case 'insert':
      print('New record inserted');
      break;
    case 'update':
      print('Record updated');
      break;
    case 'delete':
      print('Record deleted');
      break;
  }
});

// Close connection when done
await client.realtime.close();
```

## Advanced Usage

### Error Handling

The client uses functional programming with `fpdart` for type-safe error handling:

```dart
final result = await client.rest.query('users').run();

result.match(
  (error) {
    // Handle specific error types
    if (error is UnauthorizedException) {
      print('Please login first');
    } else if (error is NetworkException) {
      print('Network error: ${error.message}');
    } else if (error is ValidationException) {
      print('Validation error: ${error.message}');
    } else {
      print('Unknown error: ${error.message}');
    }
  },
  (users) {
    // Success
    print('Got ${users.length} users');
  },
);
```

### Admin Mode

```dart
// Set admin token
await client.auth.setAdminToken('your-admin-token');

// Check admin mode
if (client.auth.isAdminMode()) {
  print('Admin mode enabled');
}

// Clear admin token
await client.auth.clearAdminToken();
```

### Custom HTTP Client

```dart
import 'package:http/http.dart' as http;

final customClient = http.Client();

final client = await EvobaseClient.create(
  baseUrl: 'http://localhost:3000',
  httpClient: customClient,
  connectTimeout: Duration(seconds: 60),
  receiveTimeout: Duration(seconds: 60),
  enableLogging: false, // Disable logging
);
```

### Token Refresh

```dart
final refreshResult = await client.auth.refresh().run();

refreshResult.match(
  (error) => print('Refresh failed: ${error.message}'),
  (response) => print('Token refreshed'),
);
```

## Complete Example

```dart
import 'package:flutter/material.dart';
import 'package:evobase_flutter/evobase.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  final client = await EvobaseClient.create(
    baseUrl: 'http://localhost:3000',
  );
  
  runApp(MyApp(client: client));
}

class MyApp extends StatelessWidget {
  final EvobaseClient client;
  
  const MyApp({required this.client, super.key});
  
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: HomePage(client: client),
    );
  }
}

class HomePage extends StatefulWidget {
  final EvobaseClient client;
  
  const HomePage({required this.client, super.key});
  
  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  List<Map<String, dynamic>> users = [];
  
  @override
  void initState() {
    super.initState();
    _loadUsers();
    _listenToEvents();
  }
  
  Future<void> _loadUsers() async {
    final result = await widget.client.rest.query(
      'users',
      params: QueryParams(limit: 10),
    ).run();
    
    result.match(
      (error) => print('Error: ${error.message}'),
      (data) => setState(() => users = data),
    );
  }
  
  void _listenToEvents() {
    widget.client.realtime.listen().listen((event) {
      print('Realtime event: ${event.type}');
      _loadUsers(); // Reload on changes
    });
  }
  
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Evobase Demo')),
      body: ListView.builder(
        itemCount: users.length,
        itemBuilder: (context, index) {
          final user = users[index];
          return ListTile(
            title: Text(user['username'] ?? ''),
            subtitle: Text(user['email'] ?? ''),
          );
        },
      ),
    );
  }
}
```

## API Reference

### EvobaseClient

Main client class that provides access to all features.

**Methods:**
- `create()` - Create a new client instance
- `auth` - Access authentication manager
- `rest` - Access REST API client
- `realtime` - Access realtime client
- `close()` - Close all connections

### AuthManager

**Methods:**
- `login()` - Login with username/password
- `register()` - Register new user
- `refresh()` - Refresh access token
- `logout()` - Logout and clear tokens
- `setAdminToken()` - Set admin token
- `clearAdminToken()` - Clear admin token
- `isLoggedIn()` - Check if logged in
- `isAdminMode()` - Check if admin mode
- `getUserId()` - Get current user ID
- `getUsername()` - Get current username

### RestClient

**Methods:**
- `query()` - Query records (SELECT)
- `insert()` - Insert records (INSERT)
- `update()` - Update records (UPDATE)
- `delete()` - Delete records (DELETE)

### RealtimeClient

**Methods:**
- `listen()` - Listen to realtime events
- `close()` - Close connection
- `setAccessToken()` - Set access token

## Error Types

- `UnauthorizedException` - Authentication required
- `ForbiddenException` - Permission denied
- `NotFoundException` - Resource not found
- `ValidationException` - Validation error
- `ConflictException` - Conflict error
- `InternalServerException` - Server error
- `NetworkException` - Network error
- `TimeoutException` - Request timeout
- `UnknownApiException` - Unknown error

## License

MIT
