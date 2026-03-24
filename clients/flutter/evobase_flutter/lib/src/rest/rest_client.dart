import 'package:fpdart/fpdart.dart';

import '../core/api_client.dart';
import '../core/api_error.dart';
import '../models/query_params.dart';

/// REST API client for table operations
class RestClient {
  const RestClient(this._client);
  final ApiClient _client;

  /// Query records from a table
  /// 
  /// Example:
  /// ```dart
  /// final result = await client.rest.query(
  ///   'users',
  ///   params: QueryParams(
  ///     select: ['id', 'username', 'email'],
  ///     filter: {'status': 'active'},
  ///     order: ['created_at.desc'],
  ///     limit: 10,
  ///   ),
  /// ).run();
  /// ```
  TaskEither<ApiException, List<Map<String, dynamic>>> query(
    String table, {
    QueryParams? params,
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

  /// Insert records into a table
  /// 
  /// Example:
  /// ```dart
  /// final result = await client.rest.insert(
  ///   'users',
  ///   InsertBody(
  ///     records: [
  ///       {'username': 'john', 'email': 'john@example.com'},
  ///       {'username': 'jane', 'email': 'jane@example.com'},
  ///     ],
  ///     returning: ['id', 'username'],
  ///   ),
  /// ).run();
  /// ```
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

  /// Update records in a table
  /// 
  /// Example:
  /// ```dart
  /// final result = await client.rest.update(
  ///   'users',
  ///   UpdateBody(
  ///     set: {'status': 'inactive'},
  ///     returning: ['id', 'status'],
  ///   ),
  ///   params: QueryParams(
  ///     filter: {'id.eq': '123'},
  ///   ),
  /// ).run();
  /// ```
  TaskEither<ApiException, List<Map<String, dynamic>>> update(
    String table,
    UpdateBody body, {
    QueryParams? params,
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

  /// Delete records from a table
  /// 
  /// Example:
  /// ```dart
  /// final result = await client.rest.delete(
  ///   'users',
  ///   params: QueryParams(
  ///     filter: {'id.eq': '123'},
  ///   ),
  /// ).run();
  /// ```
  TaskEither<ApiException, List<Map<String, dynamic>>> delete(
    String table, {
    QueryParams? params,
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
