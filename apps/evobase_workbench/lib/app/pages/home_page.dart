import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../l10n.dart' show AppLocalizations;
import '../../theme.dart' show ThemeToggleButton;
import '../blocs/auth.dart'
    show
        AuthAdminTokenCleared,
        AuthAdminTokenSet,
        AuthBloc,
        AuthLogoutRequested,
        AuthState;

class HomePage extends StatelessWidget {
  const HomePage({super.key});

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
            selectedIndex: 0,
            onDestinationSelected: (index) {
              // TODO: Navigate to different pages.
            },
          ),
          const VerticalDivider(thickness: 1, width: 1),
          Expanded(
            child: Center(
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
                          onPressed: () => _showAdminTokenDialog(context),
                        );
                      }
                      return ElevatedButton.icon(
                        icon: const Icon(Icons.cancel),
                        label: Text(l10n.disableAdminMode),
                        onPressed: () {
                          context.read<AuthBloc>().add(
                            const AuthAdminTokenCleared(),
                          );
                        },
                      );
                    },
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
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
