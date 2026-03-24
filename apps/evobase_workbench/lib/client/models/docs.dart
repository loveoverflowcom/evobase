import 'package:json_annotation/json_annotation.dart';

import 'databases.dart';

part 'docs.g.dart';

/// Mirror of evobase-protocol ApiDocsDto
@JsonSerializable()
class ApiDocsDto {
  final DatabaseDto database;
  final List<TableDocDto> tables;

  ApiDocsDto({required this.database, required this.tables});

  factory ApiDocsDto.fromJson(Map<String, dynamic> json) =>
      _$ApiDocsDtoFromJson(json);

  Map<String, dynamic> toJson() => _$ApiDocsDtoToJson(this);
}

@JsonSerializable()
class TableDocDto {
  final String name;
  final String schema;
  final String table;
  final String endpoint;
  final List<ColumnDocDto> columns;
  final TableMethodsDto methods;
  final RlsDocDto rls;
  final QueryDocDto query;

  TableDocDto({
    required this.name,
    required this.schema,
    required this.table,
    required this.endpoint,
    required this.columns,
    required this.methods,
    required this.rls,
    required this.query,
  });

  factory TableDocDto.fromJson(Map<String, dynamic> json) =>
      _$TableDocDtoFromJson(json);

  Map<String, dynamic> toJson() => _$TableDocDtoToJson(this);
}

@JsonSerializable()
class ColumnDocDto {
  final String name;

  @JsonKey(name: 'type')
  final String dataType;

  final bool nullable;

  @JsonKey(name: 'has_default')
  final bool hasDefault;

  ColumnDocDto({
    required this.name,
    required this.dataType,
    required this.nullable,
    required this.hasDefault,
  });

  factory ColumnDocDto.fromJson(Map<String, dynamic> json) =>
      _$ColumnDocDtoFromJson(json);

  Map<String, dynamic> toJson() => _$ColumnDocDtoToJson(this);
}

@JsonSerializable()
class RlsDocDto {
  final bool enabled;
  final List<RlsPolicyDocDto> policies;

  RlsDocDto({required this.enabled, required this.policies});

  factory RlsDocDto.fromJson(Map<String, dynamic> json) =>
      _$RlsDocDtoFromJson(json);

  Map<String, dynamic> toJson() => _$RlsDocDtoToJson(this);
}

@JsonSerializable()
class RlsPolicyDocDto {
  final String name;
  final String command;

  @JsonKey(name: 'using')
  final String? using;

  @JsonKey(name: 'with_check')
  final String? withCheck;

  RlsPolicyDocDto({
    required this.name,
    required this.command,
    this.using,
    this.withCheck,
  });

  factory RlsPolicyDocDto.fromJson(Map<String, dynamic> json) =>
      _$RlsPolicyDocDtoFromJson(json);

  Map<String, dynamic> toJson() => _$RlsPolicyDocDtoToJson(this);
}

@JsonSerializable()
class TableMethodsDto {
  @JsonKey(name: 'GET')
  final bool get;

  @JsonKey(name: 'POST')
  final bool post;

  @JsonKey(name: 'PATCH')
  final bool patch;

  @JsonKey(name: 'DELETE')
  final bool delete;

  TableMethodsDto({
    required this.get,
    required this.post,
    required this.patch,
    required this.delete,
  });

  factory TableMethodsDto.fromJson(Map<String, dynamic> json) =>
      _$TableMethodsDtoFromJson(json);

  Map<String, dynamic> toJson() => _$TableMethodsDtoToJson(this);
}

@JsonSerializable()
class QueryDocDto {
  @JsonKey(name: 'selectable_columns')
  final List<String> selectableColumns;

  @JsonKey(name: 'filter_operators')
  final List<String> filterOperators;

  @JsonKey(name: 'order_supported')
  final bool orderSupported;

  @JsonKey(name: 'limit_supported')
  final bool limitSupported;

  @JsonKey(name: 'offset_supported')
  final bool offsetSupported;

  QueryDocDto({
    required this.selectableColumns,
    required this.filterOperators,
    required this.orderSupported,
    required this.limitSupported,
    required this.offsetSupported,
  });

  factory QueryDocDto.fromJson(Map<String, dynamic> json) =>
      _$QueryDocDtoFromJson(json);

  Map<String, dynamic> toJson() => _$QueryDocDtoToJson(this);
}
