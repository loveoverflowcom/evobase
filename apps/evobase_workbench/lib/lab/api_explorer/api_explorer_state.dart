import 'package:equatable/equatable.dart';

import '../../client/models.dart' show DatabaseDto, TableDocDto;

enum ApiExplorerStatus { initial, loading, success, failure }

String tableDocKey(TableDocDto table) => '${table.schema}.${table.table}';

class ApiExplorerState extends Equatable {
  final ApiExplorerStatus status;
  final List<DatabaseDto> databases;
  final String? defaultDatabaseId;
  final String? activeDatabaseId;
  final List<TableDocDto> tables;
  final String searchQuery;
  final String? selectedTableKey;
  final String? errorMessage;
  final String? infoMessage;
  final bool isBootstrapping;

  const ApiExplorerState({
    this.status = ApiExplorerStatus.initial,
    this.databases = const [],
    this.defaultDatabaseId,
    this.activeDatabaseId,
    this.tables = const [],
    this.searchQuery = '',
    this.selectedTableKey,
    this.errorMessage,
    this.infoMessage,
    this.isBootstrapping = false,
  });

  DatabaseDto? get activeDatabase {
    final activeDatabaseId = this.activeDatabaseId;
    if (activeDatabaseId == null) {
      return null;
    }

    for (final database in databases) {
      if (database.databaseId == activeDatabaseId) {
        return database;
      }
    }

    return null;
  }

  List<TableDocDto> get filteredTables {
    final query = searchQuery.trim().toLowerCase();
    if (query.isEmpty) {
      return tables;
    }

    return tables
        .where((table) {
          final haystacks = [
            table.name,
            table.schema,
            table.table,
            table.endpoint,
            ...table.columns.map((column) => column.name),
          ];

          return haystacks.any((value) => value.toLowerCase().contains(query));
        })
        .toList(growable: false);
  }

  TableDocDto? get selectedTable {
    final key = selectedTableKey;
    if (key == null) {
      return null;
    }

    for (final table in filteredTables) {
      if (tableDocKey(table) == key) {
        return table;
      }
    }

    return null;
  }

  int get totalColumns =>
      tables.fold(0, (total, table) => total + table.columns.length);

  ApiExplorerState copyWith({
    ApiExplorerStatus? status,
    List<DatabaseDto>? databases,
    String? defaultDatabaseId,
    String? activeDatabaseId,
    List<TableDocDto>? tables,
    String? searchQuery,
    String? selectedTableKey,
    String? errorMessage,
    String? infoMessage,
    bool? isBootstrapping,
    bool clearInfoMessage = false,
    bool clearSelectedTable = false,
    bool clearErrorMessage = false,
  }) {
    return ApiExplorerState(
      status: status ?? this.status,
      databases: databases ?? this.databases,
      defaultDatabaseId: defaultDatabaseId ?? this.defaultDatabaseId,
      activeDatabaseId: activeDatabaseId ?? this.activeDatabaseId,
      tables: tables ?? this.tables,
      searchQuery: searchQuery ?? this.searchQuery,
      selectedTableKey: clearSelectedTable
          ? null
          : (selectedTableKey ?? this.selectedTableKey),
      errorMessage: clearErrorMessage
          ? null
          : (errorMessage ?? this.errorMessage),
      infoMessage: clearInfoMessage ? null : (infoMessage ?? this.infoMessage),
      isBootstrapping: isBootstrapping ?? this.isBootstrapping,
    );
  }

  @override
  List<Object?> get props => [
    status,
    databases,
    defaultDatabaseId,
    activeDatabaseId,
    tables,
    searchQuery,
    selectedTableKey,
    errorMessage,
    infoMessage,
    isBootstrapping,
  ];
}
