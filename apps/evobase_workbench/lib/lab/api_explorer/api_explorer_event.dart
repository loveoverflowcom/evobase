import 'package:equatable/equatable.dart';

sealed class ApiExplorerEvent extends Equatable {
  const ApiExplorerEvent();

  @override
  List<Object?> get props => const [];
}

final class ApiExplorerLoadRequested extends ApiExplorerEvent {
  const ApiExplorerLoadRequested();
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
