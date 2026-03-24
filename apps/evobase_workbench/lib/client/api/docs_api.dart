import 'package:fpdart/fpdart.dart';

import '../core.dart' show ApiClient, ApiException, AuthHeaderMode;
import '../models.dart' show ApiDocsDto;

/// Docs API client
/// For fetching API documentation (admin endpoint)
class DocsApi {
  final ApiClient _client;

  DocsApi(this._client);

  /// Get API documentation
  /// Requires admin token
  TaskEither<ApiException, ApiDocsDto> getDocs() {
    return _client
        .get<ApiDocsDto>(
          '/docs',
          authHeaderMode: AuthHeaderMode.adminToken,
          fromJson: (json) => ApiDocsDto.fromJson(json),
        )
        .map((response) => response.data);
  }
}
