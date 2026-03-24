import 'package:http/http.dart' as http;

import 'auth/auth_manager.dart';
import 'auth/token_storage.dart';
import 'core/api_client.dart';
import 'realtime/realtime_client.dart';
import 'rest/rest_client.dart';

/// Main Evobase client
/// 
/// Example:
/// ```dart
/// // Initialize client
/// final client = await EvobaseClient.create(
///   baseUrl: 'http://localhost:3000',
/// );
/// 
/// // Login
/// final loginResult = await client.auth.login(
///   username: 'john',
///   password: 'password123',
/// ).run();
/// 
/// loginResult.match(
///   (error) => print('Login failed: ${error.message}'),
///   (response) => print('Logged in as ${response.username}'),
/// );
/// 
/// // Query data
/// final queryResult = await client.rest.query(
///   'users',
///   params: QueryParams(
///     select: ['id', 'username'],
///     limit: 10,
///   ),
/// ).run();
/// 
/// // Listen to realtime events
/// client.realtime.listen().listen((event) {
///   print('Event: ${event.type}');
///   print('Data: ${event.jsonData}');
/// });
/// ```
class EvobaseClient {

  EvobaseClient._({
    required ApiClient apiClient,
    required TokenStorage tokenStorage,
    required AuthManager authManager,
    required RestClient restClient,
    required RealtimeClient realtimeClient,
  })  : _apiClient = apiClient,
        _authManager = authManager,
        _restClient = restClient,
        _realtimeClient = realtimeClient;
  final ApiClient _apiClient;
  final AuthManager _authManager;
  final RestClient _restClient;
  final RealtimeClient _realtimeClient;

  /// Create a new Evobase client
  static Future<EvobaseClient> create({
    required String baseUrl,
    http.Client? httpClient,
    Duration connectTimeout = const Duration(seconds: 30),
    Duration receiveTimeout = const Duration(seconds: 30),
    bool enableLogging = true,
  }) async {
    final tokenStorage = await TokenStorage.create();
    final apiClient = ApiClient(
      baseUrl: baseUrl,
      client: httpClient,
      connectTimeout: connectTimeout,
      receiveTimeout: receiveTimeout,
      enableLogging: enableLogging,
    );

    final authManager = AuthManager(
      client: apiClient,
      storage: tokenStorage,
    );

    final restClient = RestClient(apiClient);
    final realtimeClient = RealtimeClient(
      baseUrl: baseUrl,
      enableLogging: enableLogging,
    );

    // Sync realtime token
    final accessToken = tokenStorage.getAccessToken();
    if (accessToken != null) {
      realtimeClient.setAccessToken(accessToken);
    }

    return EvobaseClient._(
      apiClient: apiClient,
      tokenStorage: tokenStorage,
      authManager: authManager,
      restClient: restClient,
      realtimeClient: realtimeClient,
    );
  }

  /// Authentication manager
  AuthManager get auth => _authManager;

  /// REST API client
  RestClient get rest => _restClient;

  /// Realtime client
  RealtimeClient get realtime => _realtimeClient;

  /// Close all connections
  void close() {
    _apiClient.close();
    _realtimeClient.close();
  }
}
