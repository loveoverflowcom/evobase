import 'dart:async' as async;
import 'dart:convert';
import 'dart:io';

import 'package:fpdart/fpdart.dart';
import 'package:http/http.dart' as http;
import 'package:logger/logger.dart';

import '../models.dart' show ApiResponse, ErrorDetail, ErrorEnvelope;
import 'api_error.dart';

enum AuthHeaderMode { accessToken, adminToken, none }

/// Base API client for all HTTP communication.
///
/// Handles:
/// - ApiResponse parsing
/// - ErrorEnvelope parsing
/// - Token injection
/// - Error mapping to typed exceptions
class ApiClient {
  final http.Client _client;
  final Logger _logger;
  final Uri _baseUri;
  final Duration connectTimeout;
  final Duration receiveTimeout;
  final bool _ownsClient;
  String? _accessToken;
  String? _adminToken;

  ApiClient({
    required String baseUrl,
    http.Client? client,
    this.connectTimeout = const Duration(seconds: 30),
    this.receiveTimeout = const Duration(seconds: 30),
  }) : _client = client ?? http.Client(),
       _ownsClient = client == null,
       _baseUri = Uri.parse(baseUrl),
       _logger = Logger(
         printer: PrettyPrinter(
           methodCount: 0,
           errorMethodCount: 5,
           lineLength: 80,
           colors: true,
           printEmojis: true,
         ),
       );

  /// Set access token for authenticated requests.
  void setAccessToken(String? token) {
    _accessToken = token;
  }

  /// Set admin token for admin endpoints.
  void setAdminToken(String? token) {
    _adminToken = token;
  }

  /// GET request with ApiResponse parsing.
  TaskEither<ApiException, ApiResponse<T>> get<T>(
    String path, {
    Map<String, dynamic>? queryParameters,
    AuthHeaderMode authHeaderMode = AuthHeaderMode.accessToken,
    required T Function(dynamic json) fromJson,
  }) {
    return _send<T>(
      'GET',
      path,
      queryParameters: queryParameters,
      authHeaderMode: authHeaderMode,
      fromJson: fromJson,
    );
  }

  /// POST request with ApiResponse parsing.
  TaskEither<ApiException, ApiResponse<T>> post<T>(
    String path, {
    dynamic data,
    Map<String, dynamic>? queryParameters,
    AuthHeaderMode authHeaderMode = AuthHeaderMode.accessToken,
    required T Function(dynamic json) fromJson,
  }) {
    return _send<T>(
      'POST',
      path,
      data: data,
      queryParameters: queryParameters,
      authHeaderMode: authHeaderMode,
      fromJson: fromJson,
    );
  }

  /// PATCH request with ApiResponse parsing.
  TaskEither<ApiException, ApiResponse<T>> patch<T>(
    String path, {
    dynamic data,
    Map<String, dynamic>? queryParameters,
    AuthHeaderMode authHeaderMode = AuthHeaderMode.accessToken,
    required T Function(dynamic json) fromJson,
  }) {
    return _send<T>(
      'PATCH',
      path,
      data: data,
      queryParameters: queryParameters,
      authHeaderMode: authHeaderMode,
      fromJson: fromJson,
    );
  }

  /// DELETE request with ApiResponse parsing.
  TaskEither<ApiException, ApiResponse<T>> delete<T>(
    String path, {
    Map<String, dynamic>? queryParameters,
    AuthHeaderMode authHeaderMode = AuthHeaderMode.accessToken,
    required T Function(dynamic json) fromJson,
  }) {
    return _send<T>(
      'DELETE',
      path,
      queryParameters: queryParameters,
      authHeaderMode: authHeaderMode,
      fromJson: fromJson,
    );
  }

  TaskEither<ApiException, ApiResponse<T>> _send<T>(
    String method,
    String path, {
    dynamic data,
    Map<String, dynamic>? queryParameters,
    AuthHeaderMode authHeaderMode = AuthHeaderMode.accessToken,
    required T Function(dynamic json) fromJson,
  }) {
    return TaskEither.tryCatch(() async {
      final request = http.Request(method, _buildUri(path, queryParameters));
      _applyHeaders(
        request.headers,
        hasBody: data != null,
        authHeaderMode: authHeaderMode,
      );

      if (data != null) {
        request.body = jsonEncode(data);
      }

      _logRequest(request, data: data, queryParameters: queryParameters);

      final streamedResponse = await _client
          .send(request)
          .timeout(connectTimeout);
      final body = await streamedResponse.stream.bytesToString().timeout(
        receiveTimeout,
      );

      _logger.i('← ${streamedResponse.statusCode} $method ${request.url.path}');

      return _parseResponse<T>(
        statusCode: streamedResponse.statusCode,
        body: body,
        fromJson: fromJson,
      );
    }, (error, stackTrace) => _mapError(error, stackTrace));
  }

  Uri _buildUri(String path, Map<String, dynamic>? queryParameters) {
    final resolvedPath = path.startsWith('/') ? path.substring(1) : path;
    final uri = _baseUri.resolve(resolvedPath);

    if (queryParameters == null || queryParameters.isEmpty) {
      return uri;
    }

    return uri.replace(
      queryParameters: queryParameters.map(
        (key, value) => MapEntry(key, value.toString()),
      ),
    );
  }

  void _applyHeaders(
    Map<String, String> headers, {
    required bool hasBody,
    required AuthHeaderMode authHeaderMode,
  }) {
    headers['Accept'] = 'application/json';

    if (hasBody) {
      headers['Content-Type'] = 'application/json';
    }

    switch (authHeaderMode) {
      case AuthHeaderMode.accessToken:
        if (_accessToken != null && _accessToken!.isNotEmpty) {
          headers['Authorization'] = 'Bearer $_accessToken';
        }
      case AuthHeaderMode.adminToken:
        if (_adminToken != null && _adminToken!.isNotEmpty) {
          headers['Authorization'] = 'Bearer $_adminToken';
          headers['X-Admin-Token'] = _adminToken!;
        }
      case AuthHeaderMode.none:
        break;
    }
  }

  void _logRequest(
    http.Request request, {
    dynamic data,
    Map<String, dynamic>? queryParameters,
  }) {
    _logger.d('→ ${request.method} ${request.url}');
    if (data != null) {
      _logger.d('  Body: $data');
    }
    if (queryParameters != null && queryParameters.isNotEmpty) {
      _logger.d('  Query: $queryParameters');
    }
  }

  ApiResponse<T> _parseResponse<T>({
    required int statusCode,
    required String body,
    required T Function(dynamic json) fromJson,
  }) {
    final decoded = body.isEmpty ? null : jsonDecode(body);

    if (statusCode < 200 || statusCode >= 300) {
      throw _toApiException(
        statusCode: statusCode,
        decodedBody: decoded,
        rawBody: body,
      );
    }

    if (decoded is! Map<String, dynamic>) {
      throw ApiException.fromErrorDetail(
        ErrorDetail(code: 'PARSE_ERROR', message: 'Invalid response format'),
      );
    }

    return ApiResponse<T>.fromJson(decoded, fromJson);
  }

  ApiException _toApiException({
    required int statusCode,
    required dynamic decodedBody,
    required String rawBody,
  }) {
    if (decodedBody is Map<String, dynamic>) {
      try {
        final errorEnvelope = ErrorEnvelope.fromJson(decodedBody);
        return ApiException.fromErrorDetail(errorEnvelope.error);
      } catch (_) {
        // Fall through to the generic mapping below.
      }
    }

    final message =
        _extractErrorMessage(decodedBody) ??
        (rawBody.isNotEmpty
            ? rawBody
            : 'Request failed with status code $statusCode');

    return UnknownApiException(code: 'HTTP_$statusCode', message: message);
  }

  String? _extractErrorMessage(dynamic decodedBody) {
    if (decodedBody is Map<String, dynamic>) {
      final errorValue = decodedBody['error'];
      if (errorValue is Map<String, dynamic>) {
        final message = errorValue['message'];
        if (message is String && message.isNotEmpty) {
          return message;
        }
      }

      final message = decodedBody['message'];
      if (message is String && message.isNotEmpty) {
        return message;
      }
    }

    return null;
  }

  ApiException _mapError(Object error, StackTrace stackTrace) {
    if (error is ApiException) {
      return error;
    }

    if (error is async.TimeoutException) {
      return TimeoutException(message: 'Request timeout');
    }

    if (error is SocketException ||
        error is HandshakeException ||
        error is http.ClientException) {
      return NetworkException(
        message:
            'Network connection failed. Check localhost permissions and the backend URL.',
      );
    }

    if (error is FormatException) {
      return UnknownApiException(code: 'PARSE_ERROR', message: error.message);
    }

    return UnknownApiException(
      code: 'UNKNOWN_ERROR',
      message: error.toString(),
    );
  }

  /// Dispose the underlying HTTP client when this ApiClient owns it.
  void close() {
    if (_ownsClient) {
      _client.close();
    }
  }
}
