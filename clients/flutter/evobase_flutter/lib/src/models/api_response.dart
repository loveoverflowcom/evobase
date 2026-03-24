/// API response wrapper
class ApiResponse<T> {

  ApiResponse({
    required this.data,
    this.count,
    this.meta,
  });

  factory ApiResponse.fromJson(
    Map<String, dynamic> json,
    T Function(dynamic) fromJson,
  ) {
    return ApiResponse<T>(
      data: fromJson(json['data']),
      count: json['count'] as int?,
      meta: json['meta'] as Map<String, dynamic>?,
    );
  }
  final T data;
  final int? count;
  final Map<String, dynamic>? meta;
}

/// Error envelope from backend
class ErrorEnvelope {

  const ErrorEnvelope({required this.error});

  factory ErrorEnvelope.fromJson(Map<String, dynamic> json) {
    return ErrorEnvelope(
      error: ErrorDetail.fromJson(json['error'] as Map<String, dynamic>),
    );
  }
  final ErrorDetail error;
}

/// Error detail
class ErrorDetail {

  const ErrorDetail({
    required this.code,
    required this.message,
    this.field,
  });

  factory ErrorDetail.fromJson(Map<String, dynamic> json) {
    return ErrorDetail(
      code: json['code'] as String,
      message: json['message'] as String,
      field: json['field'] as String?,
    );
  }
  final String code;
  final String message;
  final String? field;
}
