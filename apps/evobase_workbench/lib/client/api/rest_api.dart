import 'package:fpdart/fpdart.dart';

import '../core.dart' show ApiClient, ApiException;
import '../models.dart' show InsertBody, PatchBody, TableQueryParams;

/// REST API client for dynamic table operations
/// Supports GET, POST, PATCH, DELETE on any table
class RestApi {
  final ApiClient _client;

  RestApi(this._client);

  /// GET /rest/{table}
  TaskEither<ApiException, List<Map<String, dynamic>>> query(
    String table, {
    TableQueryParams? params,
  }) {
    return _client
        .get<List<dynamic>>(
          '/rest/$table',
          queryParameters: params?.toQueryMap(),
          fromJson: (json) => json as List<dynamic>,
        )
        .map(
          (response) =>
              response.data.map((e) => e as Map<String, dynamic>).toList(),
        );
  }

  /// POST /rest/{table}
  TaskEither<ApiException, List<Map<String, dynamic>>> insert(
    String table,
    InsertBody body,
  ) {
    return _client
        .post<List<dynamic>>(
          '/rest/$table',
          data: body.toJson(),
          fromJson: (json) => json as List<dynamic>,
        )
        .map(
          (response) =>
              response.data.map((e) => e as Map<String, dynamic>).toList(),
        );
  }

  /// PATCH /rest/{table}
  TaskEither<ApiException, List<Map<String, dynamic>>> update(
    String table,
    PatchBody body, {
    TableQueryParams? params,
  }) {
    return _client
        .patch<List<dynamic>>(
          '/rest/$table',
          data: body.toJson(),
          queryParameters: params?.toQueryMap(),
          fromJson: (json) => json as List<dynamic>,
        )
        .map(
          (response) =>
              response.data.map((e) => e as Map<String, dynamic>).toList(),
        );
  }

  /// DELETE /rest/{table}
  TaskEither<ApiException, List<Map<String, dynamic>>> delete(
    String table, {
    TableQueryParams? params,
  }) {
    return _client
        .delete<List<dynamic>>(
          '/rest/$table',
          queryParameters: params?.toQueryMap(),
          fromJson: (json) => json as List<dynamic>,
        )
        .map(
          (response) =>
              response.data.map((e) => e as Map<String, dynamic>).toList(),
        );
  }
}
