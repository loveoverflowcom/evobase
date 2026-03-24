/// Login/Register request
class AuthRequest {

  const AuthRequest({required this.username, required this.password});
  
  final String username;
  final String password;

  Map<String, dynamic> toJson() => {
        'username': username,
        'password': password,
      };
}

/// Auth response
class AuthResponse {

  AuthResponse({
    required this.accessToken,
    required this.refreshToken,
    required this.userId,
    required this.username,
    required this.expiresAt,
  });

  factory AuthResponse.fromJson(Map<String, dynamic> json) {
    return AuthResponse(
      accessToken: json['access_token'] as String,
      refreshToken: json['refresh_token'] as String,
      userId: json['user_id'] as String,
      username: json['username'] as String,
      expiresAt: DateTime.parse(json['expires_at'] as String),
    );
  }
  final String accessToken;
  final String refreshToken;
  final String userId;
  final String username;
  final DateTime expiresAt;
}

/// Refresh token request
class RefreshRequest {

  const RefreshRequest({required this.refreshToken});
  
  final String refreshToken;

  Map<String, dynamic> toJson() => {
        'refresh_token': refreshToken,
      };
}
