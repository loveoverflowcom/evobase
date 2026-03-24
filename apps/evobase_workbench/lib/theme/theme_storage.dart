import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

class ThemeStorage {
  static const _keyThemeMode = 'theme_mode';

  final SharedPreferences _prefs;

  ThemeStorage(this._prefs);

  static Future<ThemeStorage> create() async {
    final prefs = await SharedPreferences.getInstance();
    return ThemeStorage(prefs);
  }

  ThemeMode getThemeMode() {
    final rawValue = _prefs.getString(_keyThemeMode);
    return switch (rawValue) {
      'light' => ThemeMode.light,
      'dark' => ThemeMode.dark,
      'system' => ThemeMode.system,
      _ => ThemeMode.system,
    };
  }

  Future<void> setThemeMode(ThemeMode mode) async {
    await _prefs.setString(_keyThemeMode, mode.name);
  }
}
