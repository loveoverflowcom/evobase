import 'package:equatable/equatable.dart';

import '../../client/models.dart' show TableDocDto;

enum ApiExplorerStatus { initial, loading, success, failure }

String tableDocKey(TableDocDto table) => '${table.schema}.${table.table}';

class ApiExplorerState extends Equatable {
  final ApiExplorerStatus status;
  final List<TableDocDto> tables;
  final String searchQuery;
  final String? selectedTableKey;
  final String? errorMessage;

  const ApiExplorerState({
    this.status = ApiExplorerStatus.initial,
    this.tables = const [],
    this.searchQuery = '',
    this.selectedTableKey,
    this.errorMessage,
  });

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
    List<TableDocDto>? tables,
    String? searchQuery,
    String? selectedTableKey,
    String? errorMessage,
    bool clearSelectedTable = false,
    bool clearErrorMessage = false,
  }) {
    return ApiExplorerState(
      status: status ?? this.status,
      tables: tables ?? this.tables,
      searchQuery: searchQuery ?? this.searchQuery,
      selectedTableKey: clearSelectedTable
          ? null
          : (selectedTableKey ?? this.selectedTableKey),
      errorMessage: clearErrorMessage
          ? null
          : (errorMessage ?? this.errorMessage),
    );
  }

  @override
  List<Object?> get props => [
    status,
    tables,
    searchQuery,
    selectedTableKey,
    errorMessage,
  ];
}
