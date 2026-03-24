import 'package:json_annotation/json_annotation.dart';

part 'envelope.g.dart';

/// Mirror of evobase-protocol ApiResponse
@JsonSerializable(genericArgumentFactories: true)
class ApiResponse<T> {
  final T data;
  final ResponseMeta? meta;

  ApiResponse({required this.data, this.meta});

  factory ApiResponse.fromJson(
    Map<String, dynamic> json,
    T Function(Object? json) fromJsonT,
  ) => _$ApiResponseFromJson(json, fromJsonT);

  Map<String, dynamic> toJson(Object Function(T value) toJsonT) =>
      _$ApiResponseToJson(this, toJsonT);
}

@JsonSerializable()
class ResponseMeta {
  final int? total;
  final int? limit;
  final int? offset;

  ResponseMeta({this.total, this.limit, this.offset});

  factory ResponseMeta.fromJson(Map<String, dynamic> json) =>
      _$ResponseMetaFromJson(json);

  Map<String, dynamic> toJson() => _$ResponseMetaToJson(this);
}

/// Mirror of evobase-protocol ErrorEnvelope
@JsonSerializable()
class ErrorEnvelope {
  final ErrorDetail error;

  ErrorEnvelope({required this.error});

  factory ErrorEnvelope.fromJson(Map<String, dynamic> json) =>
      _$ErrorEnvelopeFromJson(json);

  Map<String, dynamic> toJson() => _$ErrorEnvelopeToJson(this);
}

@JsonSerializable()
class ErrorDetail {
  final String code;
  final String message;
  final String? field;

  ErrorDetail({required this.code, required this.message, this.field});

  factory ErrorDetail.fromJson(Map<String, dynamic> json) =>
      _$ErrorDetailFromJson(json);

  Map<String, dynamic> toJson() => _$ErrorDetailToJson(this);
}
