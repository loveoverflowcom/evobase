import 'package:shared_preferences/shared_preferences.dart';

/// Secure token storage using SharedPreferences
/// In production, consider using flutter_secure_storage
class TokenStorage {
  static const _keyAccessToken = 'access_token';
  static const _keyRefreshToken = 'refresh_token';
  static const _keyNotificationToken = 'notification_token';
  static const _keyAdminToken = 'admin_token';
  static const _keyUserId = 'user_id';
  static const _keyUsername = 'username';
  static const _keyExpiresAt = 'expires_at';

  final SharedPreferences _prefs;

  TokenStorage(this._prefs);

  static Future<TokenStorage> create() async {
    final prefs = await SharedPreferences.getInstance();
    return TokenStorage(prefs);
  }

  // Access Token
  Future<void> setAccessToken(String token) async {
    await _prefs.setString(_keyAccessToken, token);
  }

  String? getAccessToken() {
    return _prefs.getString(_keyAccessToken);
  }

  // Refresh Token
  Future<void> setRefreshToken(String token) async {
    await _prefs.setString(_keyRefreshToken, token);
  }

  String? getRefreshToken() {
    return _prefs.getString(_keyRefreshToken);
  }

  // Notification Token
  Future<void> setNotificationToken(String token) async {
    await _prefs.setString(_keyNotificationToken, token);
  }

  String? getNotificationToken() {
    return _prefs.getString(_keyNotificationToken);
  }

  // Admin Token (static)
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

  // User Info
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

  // Token Expiry
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

  // Clear all tokens
  Future<void> clearAll() async {
    await Future.wait([
      _prefs.remove(_keyAccessToken),
      _prefs.remove(_keyRefreshToken),
      _prefs.remove(_keyNotificationToken),
      _prefs.remove(_keyAdminToken),
      _prefs.remove(_keyUserId),
      _prefs.remove(_keyUsername),
      _prefs.remove(_keyExpiresAt),
    ]);
  }

  // Check if user is logged in
  bool isLoggedIn() {
    final accessToken = getAccessToken();
    return accessToken != null && accessToken.isNotEmpty && !isTokenExpired();
  }

  // Check if admin mode is enabled
  bool isAdminMode() {
    final token = getAdminToken();
    return token != null && token.isNotEmpty;
  }
}
