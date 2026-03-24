import 'package:fpdart/fpdart.dart';

import '../core/api_client.dart';
import '../core/api_error.dart';
import '../models/auth_models.dart';
import 'token_storage.dart';

/// Authentication manager
class AuthManager {

  AuthManager({
    required ApiClient client,
    required TokenStorage storage,
  })  : _client = client,
        _storage = storage {
    _syncTokens();
  }
  final ApiClient _client;
  final TokenStorage _storage;

  void _syncTokens() {
    final accessToken = _storage.getAccessToken();
    final adminToken = _storage.getAdminToken();
    _client.setAccessToken(accessToken);
    _client.setAdminToken(adminToken);
  }

  /// Login with username and password
  TaskEither<ApiException, AuthResponse> login({
    required String username,
    required String password,
  }) {
    return _client
        .post<Map<String, dynamic>>(
          '/auth/login',
          data: AuthRequest(username: username, password: password).toJson(),
          authHeaderMode: AuthHeaderMode.none,
          fromJson: (json) => json as Map<String, dynamic>,
        )
        .map((response) => AuthResponse.fromJson(response.data))
        .flatMap((authResponse) => TaskEither.tryCatch(
              () async {
                await _saveAuthResponse(authResponse);
                return authResponse;
              },
              (error, _) => UnknownApiException(
                code: 'STORAGE_ERROR',
                message: 'Failed to save auth tokens',
              ),
            ));
  }

  /// Register new user
  TaskEither<ApiException, AuthResponse> register({
    required String username,
    required String password,
  }) {
    return _client
        .post<Map<String, dynamic>>(
          '/auth/register',
          data: AuthRequest(username: username, password: password).toJson(),
          authHeaderMode: AuthHeaderMode.none,
          fromJson: (json) => json as Map<String, dynamic>,
        )
        .map((response) => AuthResponse.fromJson(response.data))
        .flatMap((authResponse) => TaskEither.tryCatch(
              () async {
                await _saveAuthResponse(authResponse);
                return authResponse;
              },
              (error, _) => UnknownApiException(
                code: 'STORAGE_ERROR',
                message: 'Failed to save auth tokens',
              ),
            ));
  }

  /// Refresh access token
  TaskEither<ApiException, AuthResponse> refresh() {
    final refreshToken = _storage.getRefreshToken();
    if (refreshToken == null || refreshToken.isEmpty) {
      return TaskEither.left(
        UnauthorizedException(message: 'No refresh token available'),
      );
    }

    return _client
        .post<Map<String, dynamic>>(
          '/auth/refresh',
          data: RefreshRequest(refreshToken: refreshToken).toJson(),
          authHeaderMode: AuthHeaderMode.none,
          fromJson: (json) => json as Map<String, dynamic>,
        )
        .map((response) => AuthResponse.fromJson(response.data))
        .flatMap((authResponse) => TaskEither.tryCatch(
              () async {
                await _saveAuthResponse(authResponse);
                return authResponse;
              },
              (error, _) => UnknownApiException(
                code: 'STORAGE_ERROR',
                message: 'Failed to save auth tokens',
              ),
            ));
  }

  /// Logout
  Future<void> logout() async {
    await _storage.clearAll();
    _client.setAccessToken(null);
    _client.setAdminToken(null);
  }

  /// Set admin token
  Future<void> setAdminToken(String token) async {
    await _storage.setAdminToken(token);
    _client.setAdminToken(token);
  }

  /// Clear admin token
  Future<void> clearAdminToken() async {
    await _storage.removeAdminToken();
    _client.setAdminToken(null);
  }

  /// Check if user is logged in
  bool isLoggedIn() => _storage.isLoggedIn();

  /// Check if admin mode is enabled
  bool isAdminMode() => _storage.isAdminMode();

  /// Get current user ID
  String? getUserId() => _storage.getUserId();

  /// Get current username
  String? getUsername() => _storage.getUsername();

  Future<void> _saveAuthResponse(AuthResponse response) async {
    await Future.wait([
      _storage.setAccessToken(response.accessToken),
      _storage.setRefreshToken(response.refreshToken),
      _storage.setUserId(response.userId),
      _storage.setUsername(response.username),
      _storage.setExpiresAt(response.expiresAt),
    ]);
    _syncTokens();
  }
}
