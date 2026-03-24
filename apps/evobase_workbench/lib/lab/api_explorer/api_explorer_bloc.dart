import 'package:flutter_bloc/flutter_bloc.dart';

import '../../client/api.dart' show DocsApi;
import '../../client/models.dart' show TableDocDto;
import 'api_explorer_event.dart';
import 'api_explorer_state.dart';

class ApiExplorerBloc extends Bloc<ApiExplorerEvent, ApiExplorerState> {
  final DocsApi _docsApi;

  ApiExplorerBloc({required DocsApi docsApi})
    : _docsApi = docsApi,
      super(const ApiExplorerState()) {
    on<ApiExplorerLoadRequested>(_onLoadRequested);
    on<ApiExplorerSearchChanged>(_onSearchChanged);
    on<ApiExplorerTableSelected>(_onTableSelected);
  }

  Future<void> _onLoadRequested(
    ApiExplorerLoadRequested event,
    Emitter<ApiExplorerState> emit,
  ) async {
    emit(
      state.copyWith(
        status: ApiExplorerStatus.loading,
        clearErrorMessage: true,
      ),
    );

    final result = await _docsApi.getDocs().run();
    result.match(
      (failure) {
        emit(
          state.copyWith(
            status: ApiExplorerStatus.failure,
            errorMessage: failure.message,
          ),
        );
      },
      (docs) {
        final tables = [...docs.tables]..sort(_compareTables);
        final selectedTableKey = _resolveSelectedTableKey(
          tables: tables,
          preferredKey: state.selectedTableKey,
          query: state.searchQuery,
        );

        emit(
          state.copyWith(
            status: ApiExplorerStatus.success,
            tables: tables,
            selectedTableKey: selectedTableKey,
            clearErrorMessage: true,
          ),
        );
      },
    );
  }

  void _onSearchChanged(
    ApiExplorerSearchChanged event,
    Emitter<ApiExplorerState> emit,
  ) {
    emit(
      state.copyWith(
        searchQuery: event.query,
        selectedTableKey: _resolveSelectedTableKey(
          tables: state.tables,
          preferredKey: state.selectedTableKey,
          query: event.query,
        ),
      ),
    );
  }

  void _onTableSelected(
    ApiExplorerTableSelected event,
    Emitter<ApiExplorerState> emit,
  ) {
    emit(state.copyWith(selectedTableKey: event.tableKey));
  }

  String? _resolveSelectedTableKey({
    required List<TableDocDto> tables,
    required String? preferredKey,
    required String query,
  }) {
    final filteredTables = _filterTables(tables, query);
    if (filteredTables.isEmpty) {
      return null;
    }

    if (preferredKey != null) {
      for (final table in filteredTables) {
        if (tableDocKey(table) == preferredKey) {
          return preferredKey;
        }
      }
    }

    return tableDocKey(filteredTables.first);
  }

  List<TableDocDto> _filterTables(List<TableDocDto> tables, String query) {
    final normalizedQuery = query.trim().toLowerCase();
    if (normalizedQuery.isEmpty) {
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

          return haystacks.any(
            (value) => value.toLowerCase().contains(normalizedQuery),
          );
        })
        .toList(growable: false);
  }

  int _compareTables(TableDocDto left, TableDocDto right) {
    final schemaComparison = left.schema.compareTo(right.schema);
    if (schemaComparison != 0) {
      return schemaComparison;
    }

    return left.table.compareTo(right.table);
  }
}
