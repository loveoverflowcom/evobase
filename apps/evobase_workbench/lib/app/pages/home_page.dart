import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../client/api.dart' show DocsApi;
import '../../l10n.dart' show AppLocalizations;
import '../../lab/api_explorer/api_explorer_page.dart';
import '../../theme.dart' show ThemeToggleButton;
import '../blocs/auth.dart'
    show
        AuthAdminTokenCleared,
        AuthAdminTokenSet,
        AuthBloc,
        AuthLogoutRequested,
        AuthState;

class HomePage extends StatefulWidget {
  final DocsApi docsApi;

  const HomePage({super.key, required this.docsApi});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.appTitle),
        actions: [
          const ThemeToggleButton(),
          BlocBuilder<AuthBloc, AuthState>(
            builder: (context, state) {
              return Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (state.isAdminMode)
                    Chip(
                      label: Text(l10n.adminMode),
                      backgroundColor: Colors.orange,
                    ),
                  const SizedBox(width: 8),
                  Text(state.username ?? ''),
                  const SizedBox(width: 8),
                  IconButton(
                    tooltip: l10n.logout,
                    icon: const Icon(Icons.logout),
                    onPressed: () {
                      context.read<AuthBloc>().add(const AuthLogoutRequested());
                    },
                  ),
                ],
              );
            },
          ),
        ],
      ),
      body: Row(
        children: [
          NavigationRail(
            extended: true,
            destinations: [
              NavigationRailDestination(
                icon: const Icon(Icons.dashboard),
                label: Text(l10n.dashboard),
              ),
              NavigationRailDestination(
                icon: const Icon(Icons.api),
                label: Text(l10n.apiExplorer),
              ),
              NavigationRailDestination(
                icon: const Icon(Icons.security),
                label: Text(l10n.rlsTester),
              ),
              NavigationRailDestination(
                icon: const Icon(Icons.monitor),
                label: Text(l10n.eventMonitor),
              ),
              NavigationRailDestination(
                icon: const Icon(Icons.settings),
                label: Text(l10n.settings),
              ),
            ],
            selectedIndex: _selectedIndex,
            onDestinationSelected: (index) {
              setState(() {
                _selectedIndex = index;
              });
            },
          ),
          const VerticalDivider(thickness: 1, width: 1),
          Expanded(child: _buildContent(context)),
        ],
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    switch (_selectedIndex) {
      case 0:
        return _DashboardView(onShowAdminTokenDialog: _showAdminTokenDialog);
      case 1:
        return BlocBuilder<AuthBloc, AuthState>(
          builder: (context, state) {
            return ApiExplorerPage(
              docsApi: widget.docsApi,
              isAdminMode: state.isAdminMode,
              onEnableAdminMode: () => _showAdminTokenDialog(context),
            );
          },
        );
      case 2:
        return FeaturePlaceholder(
          icon: Icons.security,
          title: l10n.rlsTester,
          subtitle: l10n.featureComingSoon,
        );
      case 3:
        return FeaturePlaceholder(
          icon: Icons.monitor,
          title: l10n.eventMonitor,
          subtitle: l10n.featureComingSoon,
        );
      case 4:
        return SettingsView(onShowAdminTokenDialog: _showAdminTokenDialog);
      default:
        return const SizedBox.shrink();
    }
  }

  void _showAdminTokenDialog(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final controller = TextEditingController();

    showDialog<void>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(l10n.adminTokenTitle),
        content: TextField(
          controller: controller,
          decoration: InputDecoration(
            labelText: l10n.adminTokenLabel,
            border: const OutlineInputBorder(),
          ),
          obscureText: true,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext),
            child: Text(l10n.cancelButton),
          ),
          FilledButton(
            onPressed: () {
              context.read<AuthBloc>().add(AuthAdminTokenSet(controller.text));
              Navigator.pop(dialogContext);
            },
            child: Text(l10n.setButton),
          ),
        ],
      ),
    ).whenComplete(controller.dispose);
  }
}

class _DashboardView extends StatelessWidget {
  final ValueChanged<BuildContext> onShowAdminTokenDialog;

  const _DashboardView({required this.onShowAdminTokenDialog});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.check_circle, size: 64, color: Colors.green),
            const SizedBox(height: 16),
            Text(
              l10n.welcomeTitle,
              style: Theme.of(context).textTheme.headlineMedium,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 8),
            Text(l10n.welcomeSubtitle, textAlign: TextAlign.center),
            const SizedBox(height: 32),
            BlocBuilder<AuthBloc, AuthState>(
              builder: (context, state) {
                if (!state.isAdminMode) {
                  return ElevatedButton.icon(
                    icon: const Icon(Icons.admin_panel_settings),
                    label: Text(l10n.enableAdminMode),
                    onPressed: () => onShowAdminTokenDialog(context),
                  );
                }
                return ElevatedButton.icon(
                  icon: const Icon(Icons.cancel),
                  label: Text(l10n.disableAdminMode),
                  onPressed: () {
                    context.read<AuthBloc>().add(const AuthAdminTokenCleared());
                  },
                );
              },
            ),
          ],
        ),
      ),
    );
  }
}

class FeaturePlaceholder extends StatelessWidget {
  final IconData icon;
  final String title;
  final String subtitle;

  const FeaturePlaceholder({
    super.key,
    required this.icon,
    required this.title,
    required this.subtitle,
  });

  @override
  Widget build(BuildContext context) {
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 480),
        child: Card(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
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
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class SettingsView extends StatelessWidget {
  final ValueChanged<BuildContext> onShowAdminTokenDialog;

  const SettingsView({super.key, required this.onShowAdminTokenDialog});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return BlocBuilder<AuthBloc, AuthState>(
      builder: (context, state) {
        return ListView(
          padding: const EdgeInsets.all(24),
          children: [
            Text(
              l10n.settings,
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            const SizedBox(height: 16),
            Card(
              child: ListTile(
                leading: Icon(
                  state.isAdminMode
                      ? Icons.verified_user
                      : Icons.admin_panel_settings_outlined,
                ),
                title: Text(l10n.adminMode),
                subtitle: Text(
                  state.isAdminMode
                      ? l10n.adminModeEnabledMessage
                      : l10n.adminModeDisabledMessage,
                ),
                trailing: state.isAdminMode
                    ? FilledButton.tonalIcon(
                        onPressed: () {
                          context.read<AuthBloc>().add(
                            const AuthAdminTokenCleared(),
                          );
                        },
                        icon: const Icon(Icons.cancel),
                        label: Text(l10n.disableAdminMode),
                      )
                    : FilledButton.icon(
                        onPressed: () => onShowAdminTokenDialog(context),
                        icon: const Icon(Icons.key),
                        label: Text(l10n.enableAdminMode),
                      ),
              ),
            ),
          ],
        );
      },
    );
  }
}
