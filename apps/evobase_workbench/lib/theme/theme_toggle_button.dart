import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../l10n.dart' show AppLocalizations;
import 'theme_cubit.dart';

class ThemeToggleButton extends StatelessWidget {
  const ThemeToggleButton({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return BlocBuilder<ThemeCubit, ThemeMode>(
      builder: (context, themeMode) {
        final icon = switch (themeMode) {
          ThemeMode.dark => Icons.dark_mode,
          ThemeMode.light => Icons.light_mode,
          ThemeMode.system => Icons.brightness_auto,
        };

        return PopupMenuButton<ThemeMode>(
          tooltip: l10n.themeModeButtonTooltip,
          icon: Icon(icon),
          initialValue: themeMode,
          onSelected: (mode) => context.read<ThemeCubit>().setThemeMode(mode),
          itemBuilder: (context) => [
            CheckedPopupMenuItem<ThemeMode>(
              value: ThemeMode.system,
              checked: themeMode == ThemeMode.system,
              child: Text(l10n.themeModeSystem),
            ),
            CheckedPopupMenuItem<ThemeMode>(
              value: ThemeMode.light,
              checked: themeMode == ThemeMode.light,
              child: Text(l10n.themeModeLight),
            ),
            CheckedPopupMenuItem<ThemeMode>(
              value: ThemeMode.dark,
              checked: themeMode == ThemeMode.dark,
              child: Text(l10n.themeModeDark),
            ),
          ],
        );
      },
    );
  }
}
