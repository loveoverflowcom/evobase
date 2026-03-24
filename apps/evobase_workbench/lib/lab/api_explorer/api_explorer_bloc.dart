import 'package:flutter_bloc/flutter_bloc.dart';

import '../../client/api.dart' show DocsApi;
import '../../client/models.dart' show DatabaseDto, TableDocDto;
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
    on<ApiExplorerTableSelectionCleared>(_onTableSelectionCleared);
    on<ApiExplorerDatabaseSelected>(_onDatabaseSelected);
    on<ApiExplorerBootstrapSubmitted>(_onBootstrapSubmitted);
  }

  Future<void> _onLoadRequested(
    ApiExplorerLoadRequested event,
    Emitter<ApiExplorerState> emit,
  ) async {
    await _loadExplorer(
      emit,
      preferredDatabaseId: event.preferredDatabaseId,
      preserveTables: state.tables.isNotEmpty,
    );
  }

  Future<void> _onDatabaseSelected(
    ApiExplorerDatabaseSelected event,
    Emitter<ApiExplorerState> emit,
  ) async {
    await _loadExplorer(
      emit,
      preferredDatabaseId: event.databaseId,
      preserveTables: false,
    );
  }

  Future<void> _onBootstrapSubmitted(
    ApiExplorerBootstrapSubmitted event,
    Emitter<ApiExplorerState> emit,
  ) async {
    emit(
      state.copyWith(
        isBootstrapping: true,
        clearErrorMessage: true,
        clearInfoMessage: true,
      ),
    );

    final result = await _docsApi.bootstrapDatabase(event.request).run();
    await result.match(
      (failure) async {
        emit(
          state.copyWith(
            status: ApiExplorerStatus.failure,
            errorMessage: failure.message,
            isBootstrapping: false,
          ),
        );
      },
      (database) async {
        emit(
          state.copyWith(
            isBootstrapping: false,
            infoMessage: database.docsAvailable
                ? 'Database `${database.databaseId}` is ready for exploration.'
                : 'Database `${database.databaseId}` was created with bootstrap status `${database.status.name}`.',
          ),
        );
        add(ApiExplorerLoadRequested(preferredDatabaseId: database.databaseId));
      },
    );
  }

  Future<void> _loadExplorer(
    Emitter<ApiExplorerState> emit, {
    String? preferredDatabaseId,
    required bool preserveTables,
  }) async {
    emit(
      state.copyWith(
        status: ApiExplorerStatus.loading,
        clearErrorMessage: true,
        tables: preserveTables ? state.tables : const [],
      ),
    );

    final catalogResult = await _docsApi.getDatabases().run();
    await catalogResult.match(
      (failure) async {
        emit(
          state.copyWith(
            status: ApiExplorerStatus.failure,
            errorMessage: failure.message,
          ),
        );
      },
      (catalog) async {
        final databases = [...catalog.databases]..sort(_compareDatabases);
        final activeDatabaseId = _resolveActiveDatabaseId(
          databases: databases,
          defaultDatabaseId: catalog.defaultDatabaseId,
          preferredDatabaseId: preferredDatabaseId,
        );
        final activeDatabase = _findDatabase(databases, activeDatabaseId);

        if (activeDatabase == null) {
          emit(
            state.copyWith(
              status: ApiExplorerStatus.success,
              databases: databases,
              defaultDatabaseId: catalog.defaultDatabaseId,
              activeDatabaseId: null,
              tables: const [],
              clearSelectedTable: true,
              errorMessage: 'No database is currently registered.',
            ),
          );
          return;
        }

        if (!activeDatabase.docsAvailable) {
          emit(
            state.copyWith(
              status: ApiExplorerStatus.success,
              databases: databases,
              defaultDatabaseId: catalog.defaultDatabaseId,
              activeDatabaseId: activeDatabase.databaseId,
              tables: const [],
              clearSelectedTable: true,
              errorMessage:
                  activeDatabase.failure?.message ??
                  'This database is not ready for docs exploration yet.',
            ),
          );
          return;
        }

        final docsResult = await _docsApi
            .getDocs(databaseId: activeDatabase.databaseId)
            .run();
        docsResult.match(
          (failure) {
            emit(
              state.copyWith(
                status: ApiExplorerStatus.failure,
                databases: databases,
                defaultDatabaseId: catalog.defaultDatabaseId,
                activeDatabaseId: activeDatabase.databaseId,
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
                databases: databases,
                defaultDatabaseId: catalog.defaultDatabaseId,
                activeDatabaseId: docs.database.databaseId,
                tables: tables,
                selectedTableKey: selectedTableKey,
                clearErrorMessage: true,
              ),
            );
          },
        );
      },
    );
  }

  String? _resolveActiveDatabaseId({
    required List<DatabaseDto> databases,
    required String defaultDatabaseId,
    String? preferredDatabaseId,
  }) {
    if (databases.isEmpty) {
      return null;
    }

    final candidates = [
      preferredDatabaseId,
      state.activeDatabaseId,
      defaultDatabaseId,
      databases.first.databaseId,
    ];

    for (final candidate in candidates) {
      if (candidate == null) {
        continue;
      }

      for (final database in databases) {
        if (database.databaseId == candidate) {
          return candidate;
        }
      }
    }

    return databases.first.databaseId;
  }

  DatabaseDto? _findDatabase(List<DatabaseDto> databases, String? databaseId) {
    if (databaseId == null) {
      return null;
    }

    for (final database in databases) {
      if (database.databaseId == databaseId) {
        return database;
      }
    }

    return null;
  }

  int _compareDatabases(DatabaseDto left, DatabaseDto right) {
    final defaultComparison = (right.isDefault ? 1 : 0).compareTo(
      left.isDefault ? 1 : 0,
    );
    if (defaultComparison != 0) {
      return defaultComparison;
    }

    return left.databaseId.compareTo(right.databaseId);
  }

  int _compareTables(TableDocDto left, TableDocDto right) {
    final schemaComparison = left.schema.compareTo(right.schema);
    if (schemaComparison != 0) {
      return schemaComparison;
    }

    return left.table.compareTo(right.table);
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

  void _onTableSelectionCleared(
    ApiExplorerTableSelectionCleared event,
    Emitter<ApiExplorerState> emit,
  ) {
    emit(state.copyWith(clearSelectedTable: true));
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
}
