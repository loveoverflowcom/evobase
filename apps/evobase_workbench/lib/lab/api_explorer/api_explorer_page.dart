import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../client/api.dart' show DocsApi;
import '../../client/models.dart'
    show ColumnDocDto, RlsPolicyDocDto, TableDocDto, TableMethodsDto;
import '../../l10n.dart' show AppLocalizations;
import 'api_explorer_bloc.dart';
import 'api_explorer_event.dart';
import 'api_explorer_state.dart';

class ApiExplorerPage extends StatelessWidget {
  final DocsApi docsApi;
  final bool isAdminMode;
  final VoidCallback onEnableAdminMode;

  const ApiExplorerPage({
    super.key,
    required this.docsApi,
    required this.isAdminMode,
    required this.onEnableAdminMode,
  });

  @override
  Widget build(BuildContext context) {
    if (!isAdminMode) {
      return _AdminRequiredView(onEnableAdminMode: onEnableAdminMode);
    }

    return BlocProvider(
      create: (_) =>
          ApiExplorerBloc(docsApi: docsApi)
            ..add(const ApiExplorerLoadRequested()),
      child: const _ApiExplorerView(),
    );
  }
}

class _ApiExplorerView extends StatelessWidget {
  const _ApiExplorerView();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return BlocBuilder<ApiExplorerBloc, ApiExplorerState>(
      builder: (context, state) {
        final hasTables = state.tables.isNotEmpty;

        return Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Wrap(
                alignment: WrapAlignment.spaceBetween,
                crossAxisAlignment: WrapCrossAlignment.center,
                runSpacing: 12,
                spacing: 12,
                children: [
                  ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 760),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          l10n.apiExplorerTitle,
                          style: Theme.of(context).textTheme.headlineMedium,
                        ),
                        const SizedBox(height: 8),
                        Text(l10n.apiExplorerSubtitle),
                      ],
                    ),
                  ),
                  FilledButton.icon(
                    onPressed: () {
                      context.read<ApiExplorerBloc>().add(
                        const ApiExplorerLoadRequested(),
                      );
                    },
                    icon: const Icon(Icons.refresh),
                    label: Text(l10n.apiExplorerRefresh),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              TextField(
                onChanged: (value) {
                  context.read<ApiExplorerBloc>().add(
                    ApiExplorerSearchChanged(value),
                  );
                },
                decoration: InputDecoration(
                  prefixIcon: const Icon(Icons.search),
                  hintText: l10n.apiExplorerSearchHint,
                  border: const OutlineInputBorder(),
                ),
              ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  Chip(
                    avatar: const Icon(Icons.table_chart, size: 18),
                    label: Text(
                      '${l10n.apiExplorerTablesCount}: ${state.filteredTables.length}',
                    ),
                  ),
                  Chip(
                    avatar: const Icon(Icons.view_column, size: 18),
                    label: Text(
                      '${l10n.apiExplorerColumnsCount}: ${state.totalColumns}',
                    ),
                  ),
                ],
              ),
              if (state.errorMessage != null) ...[
                const SizedBox(height: 12),
                Material(
                  color: Theme.of(context).colorScheme.errorContainer,
                  borderRadius: BorderRadius.circular(12),
                  child: ListTile(
                    leading: Icon(
                      Icons.error_outline,
                      color: Theme.of(context).colorScheme.onErrorContainer,
                    ),
                    title: Text(
                      state.errorMessage!,
                      style: TextStyle(
                        color: Theme.of(context).colorScheme.onErrorContainer,
                      ),
                    ),
                    trailing: IconButton(
                      tooltip: l10n.apiExplorerRefresh,
                      onPressed: () {
                        context.read<ApiExplorerBloc>().add(
                          const ApiExplorerLoadRequested(),
                        );
                      },
                      icon: Icon(
                        Icons.refresh,
                        color: Theme.of(context).colorScheme.onErrorContainer,
                      ),
                    ),
                  ),
                ),
              ],
              const SizedBox(height: 16),
              Expanded(
                child: switch (state.status) {
                  ApiExplorerStatus.initial || ApiExplorerStatus.loading
                      when !hasTables =>
                    const Center(child: CircularProgressIndicator()),
                  _
                      when state.filteredTables.isEmpty &&
                          state.searchQuery.trim().isNotEmpty =>
                    _EmptyState(
                      icon: Icons.search_off,
                      title: l10n.apiExplorerNoResults,
                      subtitle: l10n.apiExplorerSelectTable,
                    ),
                  _ when state.filteredTables.isEmpty => _EmptyState(
                    icon: Icons.topic_outlined,
                    title: l10n.apiExplorerEmptyTitle,
                    subtitle: l10n.apiExplorerEmptySubtitle,
                  ),
                  _ => LayoutBuilder(
                    builder: (context, constraints) {
                      final isWide = constraints.maxWidth >= 1080;

                      if (isWide) {
                        return Row(
                          children: [
                            SizedBox(
                              width: 340,
                              child: _TableList(
                                tables: state.filteredTables,
                                selectedTableKey: state.selectedTableKey,
                              ),
                            ),
                            const SizedBox(width: 16),
                            Expanded(
                              child: _TableDetails(table: state.selectedTable),
                            ),
                          ],
                        );
                      }

                      return Column(
                        children: [
                          SizedBox(
                            height: 280,
                            child: _TableList(
                              tables: state.filteredTables,
                              selectedTableKey: state.selectedTableKey,
                            ),
                          ),
                          const SizedBox(height: 16),
                          Expanded(
                            child: _TableDetails(table: state.selectedTable),
                          ),
                        ],
                      );
                    },
                  ),
                },
              ),
            ],
          ),
        );
      },
    );
  }
}

class _AdminRequiredView extends StatelessWidget {
  final VoidCallback onEnableAdminMode;

  const _AdminRequiredView({required this.onEnableAdminMode});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return _EmptyState(
      icon: Icons.lock_outline,
      title: l10n.apiExplorerAdminRequiredTitle,
      subtitle: l10n.apiExplorerAdminRequiredSubtitle,
      action: FilledButton.icon(
        onPressed: onEnableAdminMode,
        icon: const Icon(Icons.admin_panel_settings),
        label: Text(l10n.enableAdminMode),
      ),
    );
  }
}

class _TableList extends StatelessWidget {
  final List<TableDocDto> tables;
  final String? selectedTableKey;

  const _TableList({required this.tables, required this.selectedTableKey});

  @override
  Widget build(BuildContext context) {
    return Card(
      clipBehavior: Clip.antiAlias,
      child: ListView.separated(
        itemCount: tables.length,
        separatorBuilder: (_, _) => const Divider(height: 1),
        itemBuilder: (context, index) {
          final table = tables[index];
          final tableKey = tableDocKey(table);

          return ListTile(
            selected: tableKey == selectedTableKey,
            selectedTileColor: Theme.of(
              context,
            ).colorScheme.secondaryContainer.withValues(alpha: 0.45),
            leading: CircleAvatar(
              child: Text(table.table.characters.first.toUpperCase()),
            ),
            title: Text(tableKey),
            subtitle: Text(
              '${table.endpoint} • ${table.columns.length} columns',
            ),
            trailing: Text(_enabledMethodCountLabel(table.methods)),
            onTap: () {
              context.read<ApiExplorerBloc>().add(
                ApiExplorerTableSelected(tableKey),
              );
            },
          );
        },
      ),
    );
  }
}

class _TableDetails extends StatelessWidget {
  final TableDocDto? table;

  const _TableDetails({required this.table});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final table = this.table;

    if (table == null) {
      return _EmptyState(
        icon: Icons.description_outlined,
        title: l10n.apiExplorerSelectTable,
        subtitle: l10n.apiExplorerSubtitle,
      );
    }

    return Card(
      clipBehavior: Clip.antiAlias,
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              tableDocKey(table),
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 12),
            _DetailsGrid(table: table),
            const SizedBox(height: 20),
            Text(
              l10n.apiExplorerMethods,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: _methodChips(table.methods),
            ),
            const SizedBox(height: 20),
            Text(
              l10n.apiExplorerSampleRequest,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            _CodeBlock(text: _sampleRequest(table)),
            const SizedBox(height: 20),
            Text(
              l10n.apiExplorerQueryCapabilities,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                _CapabilityChip(
                  label: l10n.apiExplorerOrderSupported,
                  enabled: table.query.orderSupported,
                ),
                _CapabilityChip(
                  label: l10n.apiExplorerLimitSupported,
                  enabled: table.query.limitSupported,
                ),
                _CapabilityChip(
                  label: l10n.apiExplorerOffsetSupported,
                  enabled: table.query.offsetSupported,
                ),
              ],
            ),
            const SizedBox(height: 12),
            _NamedValuesWrap(
              title: l10n.apiExplorerSelectableColumns,
              values: table.query.selectableColumns,
            ),
            const SizedBox(height: 12),
            _NamedValuesWrap(
              title: l10n.apiExplorerFilterOperators,
              values: table.query.filterOperators,
            ),
            const SizedBox(height: 20),
            Text(
              l10n.apiExplorerColumns,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            if (table.columns.isEmpty)
              Text(l10n.apiExplorerColumnsEmpty)
            else
              ...table.columns.map((column) => _ColumnCard(column: column)),
            const SizedBox(height: 20),
            Text(
              l10n.apiExplorerRlsPolicies,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Chip(
              avatar: Icon(
                table.rls.enabled ? Icons.verified : Icons.gpp_bad_outlined,
                size: 18,
              ),
              label: Text(
                table.rls.enabled
                    ? l10n.apiExplorerRlsEnabled
                    : l10n.apiExplorerRlsDisabled,
              ),
            ),
            const SizedBox(height: 12),
            if (table.rls.policies.isEmpty)
              Text(l10n.apiExplorerPoliciesEmpty)
            else
              ...table.rls.policies.map(
                (policy) => _PolicyCard(policy: policy),
              ),
          ],
        ),
      ),
    );
  }
}

class _DetailsGrid extends StatelessWidget {
  final TableDocDto table;

  const _DetailsGrid({required this.table});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Wrap(
      spacing: 12,
      runSpacing: 12,
      children: [
        _InfoCard(label: l10n.apiExplorerEndpoint, value: table.endpoint),
        _InfoCard(label: l10n.apiExplorerSchema, value: table.schema),
        _InfoCard(label: l10n.apiExplorerTable, value: table.table),
      ],
    );
  }
}

class _InfoCard extends StatelessWidget {
  final String label;
  final String value;

  const _InfoCard({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 220,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.surfaceContainerHighest,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(label, style: Theme.of(context).textTheme.labelLarge),
              const SizedBox(height: 8),
              SelectableText(value),
            ],
          ),
        ),
      ),
    );
  }
}

class _NamedValuesWrap extends StatelessWidget {
  final String title;
  final List<String> values;

  const _NamedValuesWrap({required this.title, required this.values});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: values.map((value) => Chip(label: Text(value))).toList(),
        ),
      ],
    );
  }
}

class _ColumnCard extends StatelessWidget {
  final ColumnDocDto column;

  const _ColumnCard({required this.column});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 10),
      child: ListTile(
        leading: const Icon(Icons.view_column),
        title: Text(column.name),
        subtitle: Text(column.dataType),
        trailing: Wrap(
          spacing: 8,
          children: [
            Chip(label: Text(column.nullable ? 'nullable' : 'required')),
            if (column.hasDefault) const Chip(label: Text('default')),
          ],
        ),
      ),
    );
  }
}

class _PolicyCard extends StatelessWidget {
  final RlsPolicyDocDto policy;

  const _PolicyCard({required this.policy});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 10),
      child: ExpansionTile(
        leading: const Icon(Icons.policy_outlined),
        title: Text(policy.name),
        subtitle: Text(policy.command),
        childrenPadding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
        expandedCrossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (policy.using != null) ...[
            Text('USING', style: Theme.of(context).textTheme.labelLarge),
            const SizedBox(height: 6),
            _CodeBlock(text: policy.using!),
            const SizedBox(height: 12),
          ],
          if (policy.withCheck != null) ...[
            Text('WITH CHECK', style: Theme.of(context).textTheme.labelLarge),
            const SizedBox(height: 6),
            _CodeBlock(text: policy.withCheck!),
          ],
        ],
      ),
    );
  }
}

class _CapabilityChip extends StatelessWidget {
  final String label;
  final bool enabled;

  const _CapabilityChip({required this.label, required this.enabled});

  @override
  Widget build(BuildContext context) {
    return Chip(
      avatar: Icon(
        enabled ? Icons.check_circle_outline : Icons.remove_circle_outline,
        size: 18,
      ),
      label: Text(label),
    );
  }
}

class _CodeBlock extends StatelessWidget {
  final String text;

  const _CodeBlock({required this.text});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(12),
      ),
      child: SelectableText(
        text,
        style: Theme.of(
          context,
        ).textTheme.bodyMedium?.copyWith(fontFamily: 'monospace'),
      ),
    );
  }
}

class _EmptyState extends StatelessWidget {
  final IconData icon;
  final String title;
  final String subtitle;
  final Widget? action;

  const _EmptyState({
    required this.icon,
    required this.title,
    required this.subtitle,
    this.action,
  });

  @override
  Widget build(BuildContext context) {
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 560),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(icon, size: 56),
              const SizedBox(height: 16),
              Text(
                title,
                style: Theme.of(context).textTheme.headlineSmall,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 8),
              Text(subtitle, textAlign: TextAlign.center),
              if (action != null) ...[const SizedBox(height: 20), action!],
            ],
          ),
        ),
      ),
    );
  }
}

List<Widget> _methodChips(TableMethodsDto methods) {
  final items = <String>[
    if (methods.get) 'GET',
    if (methods.post) 'POST',
    if (methods.patch) 'PATCH',
    if (methods.delete) 'DELETE',
  ];

  return items
      .map(
        (method) => Chip(avatar: const Icon(Icons.http), label: Text(method)),
      )
      .toList(growable: false);
}

String _enabledMethodCountLabel(TableMethodsDto methods) {
  final count = [
    methods.get,
    methods.post,
    methods.patch,
    methods.delete,
  ].where((value) => value).length;

  return '$count methods';
}

String _sampleRequest(TableDocDto table) {
  final columns = table.query.selectableColumns.take(3).join(',');
  final querySegments = <String>[
    if (columns.isNotEmpty) 'select=$columns',
    if (table.query.limitSupported) 'limit=20',
    if (table.query.orderSupported && table.query.selectableColumns.isNotEmpty)
      'order=${table.query.selectableColumns.first}.desc',
  ];

  if (querySegments.isEmpty) {
    return 'GET ${table.endpoint}';
  }

  return 'GET ${table.endpoint}?${querySegments.join('&')}';
}
