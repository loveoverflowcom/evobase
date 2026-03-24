import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../client/api.dart' show DocsApi;
import '../../client/models.dart'
    show
        BootstrapDatabaseRequestDto,
        BootstrapSqlScriptDto,
        ColumnDocDto,
        DatabaseBootstrapFailureDto,
        DatabaseDto,
        DatabaseStatusDto,
        ExistingDatabasePolicyDto,
        RlsPolicyDocDto,
        TableDocDto,
        TableMethodsDto;
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
    return BlocBuilder<ApiExplorerBloc, ApiExplorerState>(
      builder: (context, state) {
        final hasTables = state.tables.isNotEmpty;
        final activeDatabase = state.activeDatabase;

        return LayoutBuilder(
          builder: (context, constraints) {
            final isCompactToolbar = constraints.maxWidth < 900;
            final useStackedTableFlow = constraints.maxWidth < 1080;

            return Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _ExplorerHeader(
                    state: state,
                    isCompact: isCompactToolbar,
                  ),
                  const SizedBox(height: 16),
                  _DatabaseToolbar(
                    state: state,
                    isCompact: isCompactToolbar,
                  ),
                  const SizedBox(height: 12),
                  if (state.infoMessage != null)
                    _BannerCard(
                      icon: Icons.info_outline,
                      message: state.infoMessage!,
                      backgroundColor: Theme.of(
                        context,
                      ).colorScheme.secondaryContainer,
                      foregroundColor: Theme.of(
                        context,
                      ).colorScheme.onSecondaryContainer,
                    ),
                  if (state.infoMessage != null) const SizedBox(height: 12),
                  if (state.errorMessage != null)
                    _BannerCard(
                      icon: Icons.error_outline,
                      message: state.errorMessage!,
                      backgroundColor: Theme.of(
                        context,
                      ).colorScheme.errorContainer,
                      foregroundColor: Theme.of(
                        context,
                      ).colorScheme.onErrorContainer,
                    ),
                  if (state.errorMessage != null) const SizedBox(height: 12),
                  _MetricsStrip(state: state),
                  const SizedBox(height: 16),
                  TextField(
                    onChanged: (value) {
                      context.read<ApiExplorerBloc>().add(
                        ApiExplorerSearchChanged(value),
                      );
                    },
                    decoration: InputDecoration(
                      prefixIcon: const Icon(Icons.search),
                      hintText: AppLocalizations.of(context)!.apiExplorerSearchHint,
                      border: const OutlineInputBorder(),
                    ),
                  ),
                  const SizedBox(height: 16),
                  Expanded(
                    child: switch (state.status) {
                      ApiExplorerStatus.initial || ApiExplorerStatus.loading
                          when !hasTables &&
                              state.activeDatabase?.docsAvailable != false =>
                        const Center(child: CircularProgressIndicator()),
                      _
                          when state.activeDatabase != null &&
                              state.activeDatabase!.docsAvailable == false =>
                        _DatabaseUnavailableState(
                          database: state.activeDatabase!,
                        ),
                      _
                          when state.filteredTables.isEmpty &&
                              state.searchQuery.trim().isNotEmpty =>
                        _EmptyState(
                          icon: Icons.search_off,
                          title: AppLocalizations.of(context)!.apiExplorerNoResults,
                          subtitle:
                              AppLocalizations.of(context)!.apiExplorerSelectTable,
                        ),
                      _ when state.filteredTables.isEmpty => _EmptyState(
                        icon: Icons.topic_outlined,
                        title: AppLocalizations.of(context)!.apiExplorerEmptyTitle,
                        subtitle: activeDatabase == null
                            ? 'No database is registered yet.'
                            : 'The selected database has no explorable tables right now.',
                      ),
                      _ => _TableMasterDetailContent(
                        state: state,
                        activeDatabase: activeDatabase,
                        useStackedFlow: useStackedTableFlow,
                      ),
                    },
                  ),
                ],
              ),
            );
          },
        );
      },
    );
  }
}

class _ExplorerHeader extends StatelessWidget {
  final ApiExplorerState state;
  final bool isCompact;

  const _ExplorerHeader({required this.state, required this.isCompact});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final title = Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          l10n.apiExplorerTitle,
          style: Theme.of(context).textTheme.headlineMedium,
        ),
        const SizedBox(height: 8),
        Text(
          'Browse docs from the default database or any admin-bootstrapped database.',
        ),
      ],
    );

    final actions = isCompact
        ? Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _CreateDatabaseButton(isBootstrapping: state.isBootstrapping),
              const SizedBox(height: 12),
              _RefreshExplorerButton(
                preferredDatabaseId: state.activeDatabaseId,
              ),
            ],
          )
        : Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              _CreateDatabaseButton(isBootstrapping: state.isBootstrapping),
              const SizedBox(width: 12),
              _RefreshExplorerButton(
                preferredDatabaseId: state.activeDatabaseId,
              ),
            ],
          );

    if (isCompact) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          title,
          const SizedBox(height: 16),
          SizedBox(width: double.infinity, child: actions),
        ],
      );
    }

    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(child: title),
        const SizedBox(width: 16),
        actions,
      ],
    );
  }
}

class _DatabaseToolbar extends StatelessWidget {
  final ApiExplorerState state;
  final bool isCompact;

  const _DatabaseToolbar({required this.state, required this.isCompact});

  @override
  Widget build(BuildContext context) {
    final selector = DropdownButtonFormField<String>(
      initialValue: state.activeDatabaseId,
      decoration: const InputDecoration(
        labelText: 'Active database',
        border: OutlineInputBorder(),
      ),
      items: state.databases
          .map(
            (database) => DropdownMenuItem(
              value: database.databaseId,
              child: Text(_databaseDropdownLabel(database)),
            ),
          )
          .toList(growable: false),
      onChanged: (value) {
        if (value == null) {
          return;
        }

        context.read<ApiExplorerBloc>().add(ApiExplorerDatabaseSelected(value));
      },
    );

    if (isCompact) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          selector,
          if (state.activeDatabase != null) ...[
            const SizedBox(height: 12),
            _DatabaseSummaryCard(database: state.activeDatabase!),
          ],
        ],
      );
    }

    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(width: 320, child: selector),
        if (state.activeDatabase != null) ...[
          const SizedBox(width: 12),
          Expanded(child: _DatabaseSummaryCard(database: state.activeDatabase!)),
        ],
      ],
    );
  }
}

class _TableMasterDetailContent extends StatelessWidget {
  final ApiExplorerState state;
  final DatabaseDto? activeDatabase;
  final bool useStackedFlow;

  const _TableMasterDetailContent({
    required this.state,
    required this.activeDatabase,
    required this.useStackedFlow,
  });

  @override
  Widget build(BuildContext context) {
    if (!useStackedFlow) {
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
            child: _TableDetails(
              table: state.selectedTable,
              activeDatabase: activeDatabase,
            ),
          ),
        ],
      );
    }

    if (state.selectedTable != null) {
      return _TableDetails(
        table: state.selectedTable,
        activeDatabase: activeDatabase,
        onBack: () {
          context.read<ApiExplorerBloc>().add(
            const ApiExplorerTableSelectionCleared(),
          );
        },
      );
    }

    return _TableList(
      tables: state.filteredTables,
      selectedTableKey: state.selectedTableKey,
    );
  }
}

class _CreateDatabaseButton extends StatelessWidget {
  final bool isBootstrapping;

  const _CreateDatabaseButton({required this.isBootstrapping});

  @override
  Widget build(BuildContext context) {
    return OutlinedButton.icon(
      onPressed: isBootstrapping
          ? null
          : () async {
              final request = await showModalBottomSheet<BootstrapDatabaseRequestDto>(
                context: context,
                isScrollControlled: true,
                builder: (_) => const _CreateDatabaseSheet(),
              );

              if (request != null && context.mounted) {
                context.read<ApiExplorerBloc>().add(
                  ApiExplorerBootstrapSubmitted(request),
                );
              }
            },
      icon: isBootstrapping
          ? const SizedBox.square(
              dimension: 16,
              child: CircularProgressIndicator(strokeWidth: 2),
            )
          : const Icon(Icons.add_circle_outline),
      label: Text(isBootstrapping ? 'Bootstrapping...' : 'Create database'),
    );
  }
}

class _RefreshExplorerButton extends StatelessWidget {
  final String? preferredDatabaseId;

  const _RefreshExplorerButton({required this.preferredDatabaseId});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return FilledButton.icon(
      onPressed: () {
        context.read<ApiExplorerBloc>().add(
          ApiExplorerLoadRequested(preferredDatabaseId: preferredDatabaseId),
        );
      },
      icon: const Icon(Icons.refresh),
      label: Text(l10n.apiExplorerRefresh),
    );
  }
}

class _MetricsStrip extends StatelessWidget {
  final ApiExplorerState state;

  const _MetricsStrip({required this.state});

  @override
  Widget build(BuildContext context) {
    final activeDatabase = state.activeDatabase;
    final chips = <Widget>[
      Chip(
        avatar: const Icon(Icons.storage, size: 18),
        label: Text('Databases: ${state.databases.length}'),
      ),
      if (activeDatabase != null)
        Chip(
          avatar: const Icon(Icons.storage_outlined, size: 18),
          label: Text('Active: ${activeDatabase.databaseId}'),
        ),
      if (activeDatabase != null) _DatabaseStatusChip(database: activeDatabase),
      Chip(
        avatar: const Icon(Icons.table_chart, size: 18),
        label: Text(
          '${AppLocalizations.of(context)!.apiExplorerTablesCount}: ${state.filteredTables.length}',
        ),
      ),
      Chip(
        avatar: const Icon(Icons.view_column, size: 18),
        label: Text(
          '${AppLocalizations.of(context)!.apiExplorerColumnsCount}: ${state.totalColumns}',
        ),
      ),
    ];

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        children: [
          for (var index = 0; index < chips.length; index++) ...[
            if (index > 0) const SizedBox(width: 8),
            chips[index],
          ],
        ],
      ),
    );
  }
}

class _DatabaseSummaryCard extends StatelessWidget {
  final DatabaseDto database;

  const _DatabaseSummaryCard({required this.database});

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Postgres DB: ${database.postgresDatabase}'),
            if (database.owner != null) ...[
              const SizedBox(height: 6),
              Text('Owner: ${database.owner}'),
            ],
            const SizedBox(height: 6),
            Text('Scripts: ${database.scriptCount}'),
            if (database.tags.isNotEmpty) ...[
              const SizedBox(height: 6),
              Text('Tags: ${database.tags.join(', ')}'),
            ],
          ],
        ),
      ),
    );
  }
}

class _DatabaseStatusChip extends StatelessWidget {
  final DatabaseDto database;

  const _DatabaseStatusChip({required this.database});

  @override
  Widget build(BuildContext context) {
    final ready = database.status == DatabaseStatusDto.ready;

    return Chip(
      avatar: Icon(
        ready ? Icons.verified_outlined : Icons.warning_amber_outlined,
        size: 18,
      ),
      label: Text(ready ? 'Ready' : 'Bootstrap failed'),
    );
  }
}

class _DatabaseUnavailableState extends StatelessWidget {
  final DatabaseDto database;

  const _DatabaseUnavailableState({required this.database});

  @override
  Widget build(BuildContext context) {
    final failure = database.failure;

    return _EmptyState(
      icon: Icons.report_problem_outlined,
      title: 'Database is not ready',
      subtitle: failure == null
          ? 'Bootstrap has not completed successfully for `${database.databaseId}`.'
          : _failureSummary(failure),
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
  final DatabaseDto? activeDatabase;
  final VoidCallback? onBack;

  const _TableDetails({
    required this.table,
    required this.activeDatabase,
    this.onBack,
  });

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final table = this.table;

    if (table == null) {
      return _EmptyState(
        icon: Icons.description_outlined,
        title: l10n.apiExplorerSelectTable,
        subtitle: activeDatabase == null
            ? l10n.apiExplorerSubtitle
            : 'Select a table from `${activeDatabase!.databaseId}` to inspect its docs.',
      );
    }

    return Card(
      clipBehavior: Clip.antiAlias,
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                if (onBack != null) ...[
                  IconButton(
                    onPressed: onBack,
                    icon: const Icon(Icons.arrow_back),
                    tooltip: 'Back to tables',
                  ),
                  const SizedBox(width: 8),
                ],
                Expanded(
                  child: Text(
                    tableDocKey(table),
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            _DetailsGrid(table: table, activeDatabase: activeDatabase),
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
            _CodeBlock(text: _sampleRequest(table, activeDatabase)),
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
  final DatabaseDto? activeDatabase;

  const _DetailsGrid({required this.table, required this.activeDatabase});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Wrap(
      spacing: 12,
      runSpacing: 12,
      children: [
        if (activeDatabase != null)
          _InfoCard(label: 'Database', value: activeDatabase!.databaseId),
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

class _BannerCard extends StatelessWidget {
  final IconData icon;
  final String message;
  final Color backgroundColor;
  final Color foregroundColor;

  const _BannerCard({
    required this.icon,
    required this.message,
    required this.backgroundColor,
    required this.foregroundColor,
  });

  @override
  Widget build(BuildContext context) {
    return Material(
      color: backgroundColor,
      borderRadius: BorderRadius.circular(12),
      child: ListTile(
        leading: Icon(icon, color: foregroundColor),
        title: Text(message, style: TextStyle(color: foregroundColor)),
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

class _CreateDatabaseSheet extends StatefulWidget {
  const _CreateDatabaseSheet();

  @override
  State<_CreateDatabaseSheet> createState() => _CreateDatabaseSheetState();
}

class _CreateDatabaseSheetState extends State<_CreateDatabaseSheet> {
  final _databaseIdController = TextEditingController();
  final _postgresDatabaseController = TextEditingController();
  final _descriptionController = TextEditingController();
  final _tagsController = TextEditingController();
  final _ownerController = TextEditingController();
  ExistingDatabasePolicyDto _policy = ExistingDatabasePolicyDto.fail;
  final List<_ScriptDraft> _scripts = [_ScriptDraft()];

  @override
  void dispose() {
    _databaseIdController.dispose();
    _postgresDatabaseController.dispose();
    _descriptionController.dispose();
    _tagsController.dispose();
    _ownerController.dispose();
    for (final script in _scripts) {
      script.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final bottomInset = MediaQuery.of(context).viewInsets.bottom;

    return SafeArea(
      child: Padding(
        padding: EdgeInsets.fromLTRB(24, 24, 24, 24 + bottomInset),
        child: SizedBox(
          height: 720,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Bootstrap database from SQL scripts',
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              const SizedBox(height: 8),
              Text(
                'The server will create a PostgreSQL database, execute the scripts in order, and then refresh the API explorer catalog.',
              ),
              const SizedBox(height: 16),
              Expanded(
                child: ListView(
                  children: [
                    TextField(
                      controller: _databaseIdController,
                      decoration: const InputDecoration(
                        labelText: 'Database ID',
                        hintText: 'tenant_alpha',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      controller: _postgresDatabaseController,
                      decoration: const InputDecoration(
                        labelText: 'PostgreSQL database name',
                        hintText: 'tenant_alpha',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      controller: _descriptionController,
                      maxLines: 2,
                      decoration: const InputDecoration(
                        labelText: 'Description',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      controller: _ownerController,
                      decoration: const InputDecoration(
                        labelText: 'Owner metadata',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      controller: _tagsController,
                      decoration: const InputDecoration(
                        labelText: 'Tags',
                        hintText: 'demo, reporting',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 12),
                    DropdownButtonFormField<ExistingDatabasePolicyDto>(
                      initialValue: _policy,
                      decoration: const InputDecoration(
                        labelText: 'If PostgreSQL database already exists',
                        border: OutlineInputBorder(),
                      ),
                      items: const [
                        DropdownMenuItem(
                          value: ExistingDatabasePolicyDto.fail,
                          child: Text('Fail the request'),
                        ),
                        DropdownMenuItem(
                          value: ExistingDatabasePolicyDto.useExisting,
                          child: Text('Use the existing database'),
                        ),
                      ],
                      onChanged: (value) {
                        if (value == null) {
                          return;
                        }
                        setState(() {
                          _policy = value;
                        });
                      },
                    ),
                    const SizedBox(height: 20),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          'SQL scripts',
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                        TextButton.icon(
                          onPressed: () {
                            setState(() {
                              _scripts.add(_ScriptDraft());
                            });
                          },
                          icon: const Icon(Icons.add),
                          label: const Text('Add script'),
                        ),
                      ],
                    ),
                    const SizedBox(height: 8),
                    ..._buildScriptEditors(),
                  ],
                ),
              ),
              const SizedBox(height: 16),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  TextButton(
                    onPressed: () => Navigator.of(context).pop(),
                    child: const Text('Cancel'),
                  ),
                  const SizedBox(width: 12),
                  FilledButton.icon(
                    onPressed: _submit,
                    icon: const Icon(Icons.play_circle_outline),
                    label: const Text('Bootstrap'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  List<Widget> _buildScriptEditors() {
    final widgets = <Widget>[];

    for (var index = 0; index < _scripts.length; index++) {
      final script = _scripts[index];
      widgets.add(
        Card(
          margin: const EdgeInsets.only(bottom: 12),
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Text('Script ${index + 1}'),
                    const Spacer(),
                    IconButton(
                      tooltip: 'Move up',
                      onPressed: index == 0
                          ? null
                          : () {
                              setState(() {
                                final previous = _scripts[index - 1];
                                _scripts[index - 1] = _scripts[index];
                                _scripts[index] = previous;
                              });
                            },
                      icon: const Icon(Icons.arrow_upward),
                    ),
                    IconButton(
                      tooltip: 'Move down',
                      onPressed: index == _scripts.length - 1
                          ? null
                          : () {
                              setState(() {
                                final next = _scripts[index + 1];
                                _scripts[index + 1] = _scripts[index];
                                _scripts[index] = next;
                              });
                            },
                      icon: const Icon(Icons.arrow_downward),
                    ),
                    IconButton(
                      tooltip: 'Remove script',
                      onPressed: _scripts.length == 1
                          ? null
                          : () {
                              setState(() {
                                final removed = _scripts.removeAt(index);
                                removed.dispose();
                              });
                            },
                      icon: const Icon(Icons.delete_outline),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                TextField(
                  controller: script.nameController,
                  decoration: const InputDecoration(
                    labelText: 'Script name',
                    hintText: '001_init_schema.sql',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: script.sqlController,
                  minLines: 6,
                  maxLines: 12,
                  decoration: const InputDecoration(
                    labelText: 'SQL',
                    alignLabelWithHint: true,
                    border: OutlineInputBorder(),
                  ),
                  style: const TextStyle(fontFamily: 'monospace'),
                ),
              ],
            ),
          ),
        ),
      );
    }

    return widgets;
  }

  void _submit() {
    final databaseId = _databaseIdController.text.trim();
    final postgresDatabase = _postgresDatabaseController.text.trim();
    final scripts = _scripts
        .map(
          (script) => BootstrapSqlScriptDto(
            name: script.nameController.text.trim(),
            sql: script.sqlController.text.trim(),
          ),
        )
        .toList(growable: false);

    if (databaseId.isEmpty || postgresDatabase.isEmpty) {
      _showMessage('Database ID and PostgreSQL database name are required.');
      return;
    }

    if (scripts.any((script) => script.name.isEmpty || script.sql.isEmpty)) {
      _showMessage('Every script needs both a name and SQL content.');
      return;
    }

    final tags = _tagsController.text
        .split(',')
        .map((value) => value.trim())
        .where((value) => value.isNotEmpty)
        .toList(growable: false);

    Navigator.of(context).pop(
      BootstrapDatabaseRequestDto(
        databaseId: databaseId,
        postgresDatabase: postgresDatabase,
        scripts: scripts,
        description: _optionalText(_descriptionController.text),
        tags: tags,
        owner: _optionalText(_ownerController.text),
        existingDatabasePolicy: _policy,
      ),
    );
  }

  String? _optionalText(String value) {
    final trimmed = value.trim();
    return trimmed.isEmpty ? null : trimmed;
  }

  void _showMessage(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }
}

class _ScriptDraft {
  final nameController = TextEditingController();
  final sqlController = TextEditingController();

  void dispose() {
    nameController.dispose();
    sqlController.dispose();
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

String _databaseDropdownLabel(DatabaseDto database) {
  final suffix = database.isDefault ? ' (default)' : '';
  return '${database.databaseId}$suffix';
}

String _failureSummary(DatabaseBootstrapFailureDto failure) {
  final scriptLabel = failure.scriptName == null
      ? ''
      : ' Script `${failure.scriptName}`';
  return '${failure.stage.name}$scriptLabel: ${failure.message}';
}

String _sampleRequest(TableDocDto table, DatabaseDto? database) {
  final columns = table.query.selectableColumns.take(3).join(',');
  final querySegments = <String>[
    if (columns.isNotEmpty) 'select=$columns',
    if (table.query.limitSupported) 'limit=20',
    if (table.query.orderSupported && table.query.selectableColumns.isNotEmpty)
      'order=${table.query.selectableColumns.first}.desc',
  ];

  final prefix = database == null ? '' : '# Database: ${database.databaseId}\n';

  if (querySegments.isEmpty) {
    return '${prefix}GET ${table.endpoint}';
  }

  return '${prefix}GET ${table.endpoint}?${querySegments.join('&')}';
}
