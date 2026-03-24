import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'theme_storage.dart';

class ThemeCubit extends Cubit<ThemeMode> {
  final ThemeStorage _themeStorage;

  ThemeCubit({required ThemeStorage themeStorage})
    : _themeStorage = themeStorage,
      super(themeStorage.getThemeMode());

  Future<void> setThemeMode(ThemeMode mode) async {
    await _themeStorage.setThemeMode(mode);
    emit(mode);
  }

  Future<void> toggle(Brightness brightness) async {
    await setThemeMode(
      brightness == Brightness.dark ? ThemeMode.light : ThemeMode.dark,
    );
  }
}
