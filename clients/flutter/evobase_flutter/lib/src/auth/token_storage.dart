import 'package:shared_preferences/shared_preferences.dart';

/// Token storage using SharedPreferences
class TokenStorage {

  const TokenStorage(this._prefs);
  static const _keyAccessToken = 'evobase_access_token';
  static const _keyRefreshToken = 'evobase_refresh_token';
  static const _keyAdminToken = 'evobase_admin_token';
  static const _keyUserId = 'evobase_user_id';
  static const _keyUsername = 'evobase_username';
  static const _keyExpiresAt = 'evobase_expires_at';

  final SharedPreferences _prefs;

  static Future<TokenStorage> create() async {
    final prefs = await SharedPreferences.getInstance();
    return TokenStorage(prefs);
  }

  Future<void> setAccessToken(String token) async {
    await _prefs.setString(_keyAccessToken, token);
  }

  String? getAccessToken() {
    return _prefs.getString(_keyAccessToken);
  }

  Future<void> setRefreshToken(String token) async {
    await _prefs.setString(_keyRefreshToken, token);
  }

  String? getRefreshToken() {
    return _prefs.getString(_keyRefreshToken);
  }

  Future<void> setAdminToken(String token) async {
    if (token.trim().isEmpty) {
      await _prefs.remove(_keyAdminToken);
      return;
    }
    await _prefs.setString(_keyAdminToken, token);
  }

  String? getAdminToken() {
    return _prefs.getString(_keyAdminToken);
  }

  Future<void> removeAdminToken() async {
    await _prefs.remove(_keyAdminToken);
  }

  Future<void> setUserId(String userId) async {
    await _prefs.setString(_keyUserId, userId);
  }

  String? getUserId() {
    return _prefs.getString(_keyUserId);
  }

  Future<void> setUsername(String username) async {
    await _prefs.setString(_keyUsername, username);
  }

  String? getUsername() {
    return _prefs.getString(_keyUsername);
  }

  Future<void> setExpiresAt(DateTime expiresAt) async {
    await _prefs.setInt(_keyExpiresAt, expiresAt.millisecondsSinceEpoch);
  }

  DateTime? getExpiresAt() {
    final timestamp = _prefs.getInt(_keyExpiresAt);
    return timestamp != null
        ? DateTime.fromMillisecondsSinceEpoch(timestamp)
        : null;
  }

  bool isTokenExpired() {
    final expiresAt = getExpiresAt();
    if (expiresAt == null) return true;
    return DateTime.now().isAfter(expiresAt);
  }

  Future<void> clearAll() async {
    await Future.wait([
      _prefs.remove(_keyAccessToken),
      _prefs.remove(_keyRefreshToken),
      _prefs.remove(_keyAdminToken),
      _prefs.remove(_keyUserId),
      _prefs.remove(_keyUsername),
      _prefs.remove(_keyExpiresAt),
    ]);
  }

  bool isLoggedIn() {
    final accessToken = getAccessToken();
    return accessToken != null && accessToken.isNotEmpty && !isTokenExpired();
  }

  bool isAdminMode() {
    final token = getAdminToken();
    return token != null && token.isNotEmpty;
  }
}
