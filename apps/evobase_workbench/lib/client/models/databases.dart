import 'package:json_annotation/json_annotation.dart';

part 'databases.g.dart';

@JsonSerializable()
class DatabaseCatalogDto {
  @JsonKey(name: 'default_database_id')
  final String defaultDatabaseId;
  final List<DatabaseDto> databases;

  DatabaseCatalogDto({
    required this.defaultDatabaseId,
    required this.databases,
  });

  factory DatabaseCatalogDto.fromJson(Map<String, dynamic> json) =>
      _$DatabaseCatalogDtoFromJson(json);

  Map<String, dynamic> toJson() => _$DatabaseCatalogDtoToJson(this);
}

@JsonSerializable()
class DatabaseDto {
  @JsonKey(name: 'database_id')
  final String databaseId;

  @JsonKey(name: 'postgres_database')
  final String postgresDatabase;

  final String? description;

  @JsonKey(defaultValue: [])
  final List<String> tags;
  final String? owner;
  final DatabaseKindDto kind;
  final DatabaseStatusDto status;

  @JsonKey(name: 'docs_available')
  final bool docsAvailable;

  @JsonKey(name: 'is_default')
  final bool isDefault;

  @JsonKey(name: 'script_count')
  final int scriptCount;

  final DatabaseBootstrapFailureDto? failure;

  DatabaseDto({
    required this.databaseId,
    required this.postgresDatabase,
    this.description,
    required this.tags,
    this.owner,
    required this.kind,
    required this.status,
    required this.docsAvailable,
    required this.isDefault,
    required this.scriptCount,
    this.failure,
  });

  factory DatabaseDto.fromJson(Map<String, dynamic> json) =>
      _$DatabaseDtoFromJson(json);

  Map<String, dynamic> toJson() => _$DatabaseDtoToJson(this);
}

@JsonSerializable(includeIfNull: false)
class BootstrapDatabaseRequestDto {
  @JsonKey(name: 'database_id')
  final String databaseId;

  @JsonKey(name: 'postgres_database')
  final String postgresDatabase;

  final List<BootstrapSqlScriptDto> scripts;
  final String? description;
  final List<String> tags;
  final String? owner;

  @JsonKey(name: 'existing_database_policy')
  final ExistingDatabasePolicyDto existingDatabasePolicy;

  BootstrapDatabaseRequestDto({
    required this.databaseId,
    required this.postgresDatabase,
    required this.scripts,
    this.description,
    this.tags = const [],
    this.owner,
    this.existingDatabasePolicy = ExistingDatabasePolicyDto.fail,
  });

  factory BootstrapDatabaseRequestDto.fromJson(Map<String, dynamic> json) =>
      _$BootstrapDatabaseRequestDtoFromJson(json);

  Map<String, dynamic> toJson() => _$BootstrapDatabaseRequestDtoToJson(this);
}

@JsonSerializable()
class BootstrapSqlScriptDto {
  final String name;
  final String sql;

  BootstrapSqlScriptDto({required this.name, required this.sql});

  factory BootstrapSqlScriptDto.fromJson(Map<String, dynamic> json) =>
      _$BootstrapSqlScriptDtoFromJson(json);

  Map<String, dynamic> toJson() => _$BootstrapSqlScriptDtoToJson(this);
}

@JsonSerializable()
class DatabaseBootstrapFailureDto {
  final BootstrapFailureStageDto stage;
  final String message;

  @JsonKey(name: 'script_name')
  final String? scriptName;

  @JsonKey(name: 'script_index')
  final int? scriptIndex;

  DatabaseBootstrapFailureDto({
    required this.stage,
    required this.message,
    this.scriptName,
    this.scriptIndex,
  });

  factory DatabaseBootstrapFailureDto.fromJson(Map<String, dynamic> json) =>
      _$DatabaseBootstrapFailureDtoFromJson(json);

  Map<String, dynamic> toJson() => _$DatabaseBootstrapFailureDtoToJson(this);
}

@JsonEnum(fieldRename: FieldRename.snake)
enum DatabaseKindDto {
  @JsonValue('default')
  defaultDatabase,
  bootstrapped,
}

@JsonEnum(fieldRename: FieldRename.snake)
enum DatabaseStatusDto { ready, bootstrapFailed }

@JsonEnum(fieldRename: FieldRename.snake)
enum ExistingDatabasePolicyDto { fail, useExisting }

@JsonEnum(fieldRename: FieldRename.snake)
enum BootstrapFailureStageDto {
  createDatabase,
  connectDatabase,
  executeScript,
  introspectSchema,
}
