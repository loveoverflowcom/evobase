import 'package:fpdart/fpdart.dart';

import '../api.dart' show AuthApi;
import '../core.dart'
    show
        ApiClient,
        ApiException,
        ConflictException,
        UnknownApiException,
        UnauthorizedException;
import '../models.dart' show AuthResponseDto;
import 'token_storage.dart';

/// Manages authentication flow with auto-refresh.
class AuthManager {
  final AuthApi _authApi;
  final ApiClient _apiClient;
  final TokenStorage _tokenStorage;
  bool _isRefreshing = false;

  AuthManager({
    required AuthApi authApi,
    required ApiClient apiClient,
    required TokenStorage tokenStorage,
  }) : _authApi = authApi,
       _apiClient = apiClient,
       _tokenStorage = tokenStorage {
    _initializeTokens();
  }

  void _initializeTokens() {
    final accessToken = _tokenStorage.getAccessToken();
    final adminToken = _tokenStorage.getAdminToken();

    if (accessToken != null) {
      _apiClient.setAccessToken(accessToken);
    }
    if (adminToken != null && adminToken.isNotEmpty) {
      _apiClient.setAdminToken(adminToken);
    }
  }

  /// Register new user.
  TaskEither<ApiException, AuthResponseDto> register({
    required String username,
    required String password,
  }) {
    return _authApi
        .register(username: username, password: password)
        .flatMap(_saveAuthResponse);
  }

  /// Login user.
  TaskEither<ApiException, AuthResponseDto> login({
    required String username,
    required String password,
  }) {
    return _authApi
        .login(username: username, password: password)
        .flatMap(_saveAuthResponse);
  }

  /// Refresh access token.
  TaskEither<ApiException, AuthResponseDto> refresh() {
    if (_isRefreshing) {
      return TaskEither.left(
        ConflictException(message: 'Token refresh already in progress'),
      );
    }

    final refreshToken = _tokenStorage.getRefreshToken();
    if (refreshToken == null || refreshToken.isEmpty) {
      return TaskEither.left(
        UnauthorizedException(message: 'No refresh token available'),
      );
    }

    return TaskEither.tryCatch(() async {
      _isRefreshing = true;
      try {
        final response = await _authApi.refresh(refreshToken).run();
        final authResponse = await response.match(
          (failure) async => throw failure,
          (value) async => value,
        );

        final savedResponse = await _saveAuthResponse(authResponse).run();
        return await savedResponse.match(
          (failure) async => throw failure,
          (value) async => value,
        );
      } finally {
        _isRefreshing = false;
      }
    }, _mapUnexpectedError);
  }

  /// Auto-refresh if token is expired.
  Future<void> ensureValidToken() async {
    if (_tokenStorage.isTokenExpired()) {
      final result = await refresh().run();
      result.match((failure) => throw failure, (_) {});
    }
  }

  /// Logout user.
  Future<void> logout() async {
    await _tokenStorage.clearAll();
    _apiClient.setAccessToken(null);
    _apiClient.setAdminToken(null);
  }

  /// Set admin token for admin mode.
  Future<void> setAdminToken(String token) async {
    if (token.trim().isEmpty) {
      await clearAdminToken();
      return;
    }

    await _tokenStorage.setAdminToken(token);
    _apiClient.setAdminToken(token);
  }

  /// Clear admin token.
  Future<void> clearAdminToken() async {
    await _tokenStorage.removeAdminToken();
    _apiClient.setAdminToken(null);
  }

  /// Check if user is logged in.
  bool isLoggedIn() => _tokenStorage.isLoggedIn();

  /// Check if admin mode is enabled.
  bool isAdminMode() => _tokenStorage.isAdminMode();

  /// Get current user info.
  String? getUserId() => _tokenStorage.getUserId();
  String? getUsername() => _tokenStorage.getUsername();

  TaskEither<ApiException, AuthResponseDto> _saveAuthResponse(
    AuthResponseDto response,
  ) {
    return TaskEither.tryCatch(() async {
      await Future.wait([
        _tokenStorage.setAccessToken(response.tokens.accessToken),
        _tokenStorage.setRefreshToken(response.tokens.refreshToken),
        _tokenStorage.setNotificationToken(response.tokens.notificationToken),
        _tokenStorage.setUserId(response.userId),
        _tokenStorage.setUsername(response.username),
        _tokenStorage.setExpiresAt(
          DateTime.now().add(Duration(seconds: response.tokens.expiresIn)),
        ),
      ]);

      _apiClient.setAccessToken(response.tokens.accessToken);
      return response;
    }, _mapUnexpectedError);
  }

  ApiException _mapUnexpectedError(Object error, StackTrace stackTrace) {
    if (error is ApiException) {
      return error;
    }

    return UnknownApiException(code: 'AUTH_ERROR', message: error.toString());
  }
}
