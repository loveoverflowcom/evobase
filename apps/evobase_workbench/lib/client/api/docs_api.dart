import 'package:fpdart/fpdart.dart';

import '../core.dart' show ApiClient, ApiException, AuthHeaderMode;
import '../models.dart'
    show
        ApiDocsDto,
        BootstrapDatabaseRequestDto,
        DatabaseCatalogDto,
        DatabaseDto;

/// Docs API client
/// For fetching API documentation (admin endpoint)
class DocsApi {
  final ApiClient _client;

  DocsApi(this._client);

  /// Get API documentation
  /// Requires admin token
  TaskEither<ApiException, ApiDocsDto> getDocs({String? databaseId}) {
    final path = databaseId == null
        ? '/docs'
        : '/admin/databases/$databaseId/docs';

    return _client
        .get<ApiDocsDto>(
          path,
          authHeaderMode: AuthHeaderMode.adminToken,
          fromJson: (json) => ApiDocsDto.fromJson(json),
        )
        .map((response) => response.data);
  }

  TaskEither<ApiException, DatabaseCatalogDto> getDatabases() {
    return _client
        .get<DatabaseCatalogDto>(
          '/admin/databases',
          authHeaderMode: AuthHeaderMode.adminToken,
          fromJson: (json) => DatabaseCatalogDto.fromJson(json),
        )
        .map((response) => response.data);
  }

  TaskEither<ApiException, DatabaseDto> getDatabase(String databaseId) {
    return _client
        .get<DatabaseDto>(
          '/admin/databases/$databaseId',
          authHeaderMode: AuthHeaderMode.adminToken,
          fromJson: (json) => DatabaseDto.fromJson(json),
        )
        .map((response) => response.data);
  }

  TaskEither<ApiException, DatabaseDto> bootstrapDatabase(
    BootstrapDatabaseRequestDto request,
  ) {
    return _client
        .post<DatabaseDto>(
          '/admin/databases',
          data: request.toJson(),
          authHeaderMode: AuthHeaderMode.adminToken,
          fromJson: (json) => DatabaseDto.fromJson(json),
        )
        .map((response) => response.data);
  }
}
