import 'package:fpdart/fpdart.dart';

import '../core.dart' show ApiClient, ApiException;
import '../models.dart'
    show AuthResponseDto, LoginRequest, RefreshRequest, RegisterRequest;

/// Auth API client
/// Thin wrapper around ApiClient for auth endpoints
class AuthApi {
  final ApiClient _client;

  AuthApi(this._client);

  TaskEither<ApiException, AuthResponseDto> register({
    required String username,
    required String password,
  }) {
    final request = RegisterRequest(username: username, password: password);

    return _client
        .post<AuthResponseDto>(
          '/auth/register',
          data: request.toJson(),
          fromJson: (json) => AuthResponseDto.fromJson(json),
        )
        .map((response) => response.data);
  }

  TaskEither<ApiException, AuthResponseDto> login({
    required String username,
    required String password,
  }) {
    final request = LoginRequest(username: username, password: password);

    return _client
        .post<AuthResponseDto>(
          '/auth/login',
          data: request.toJson(),
          fromJson: (json) => AuthResponseDto.fromJson(json),
        )
        .map((response) => response.data);
  }

  TaskEither<ApiException, AuthResponseDto> refresh(String refreshToken) {
    final request = RefreshRequest(refreshToken: refreshToken);

    return _client
        .post<AuthResponseDto>(
          '/auth/refresh',
          data: request.toJson(),
          fromJson: (json) => AuthResponseDto.fromJson(json),
        )
        .map((response) => response.data);
  }
}
