import 'package:equatable/equatable.dart';

import '../../client/models.dart' show BootstrapDatabaseRequestDto;

sealed class ApiExplorerEvent extends Equatable {
  const ApiExplorerEvent();

  @override
  List<Object?> get props => const [];
}

final class ApiExplorerLoadRequested extends ApiExplorerEvent {
  final String? preferredDatabaseId;

  const ApiExplorerLoadRequested({this.preferredDatabaseId});

  @override
  List<Object?> get props => [preferredDatabaseId];
}

final class ApiExplorerSearchChanged extends ApiExplorerEvent {
  final String query;

  const ApiExplorerSearchChanged(this.query);

  @override
  List<Object?> get props => [query];
}

final class ApiExplorerTableSelected extends ApiExplorerEvent {
  final String tableKey;

  const ApiExplorerTableSelected(this.tableKey);

  @override
  List<Object?> get props => [tableKey];
}

final class ApiExplorerTableSelectionCleared extends ApiExplorerEvent {
  const ApiExplorerTableSelectionCleared();
}

final class ApiExplorerDatabaseSelected extends ApiExplorerEvent {
  final String databaseId;

  const ApiExplorerDatabaseSelected(this.databaseId);

  @override
  List<Object?> get props => [databaseId];
}

final class ApiExplorerBootstrapSubmitted extends ApiExplorerEvent {
  final BootstrapDatabaseRequestDto request;

  const ApiExplorerBootstrapSubmitted(this.request);

  @override
  List<Object?> get props => [request];
}
