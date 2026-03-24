import 'package:json_annotation/json_annotation.dart';

part 'auth.g.dart';

/// Mirror of evobase-protocol RegisterRequest
@JsonSerializable()
class RegisterRequest {
  final String username;
  final String password;

  RegisterRequest({required this.username, required this.password});

  factory RegisterRequest.fromJson(Map<String, dynamic> json) =>
      _$RegisterRequestFromJson(json);

  Map<String, dynamic> toJson() => _$RegisterRequestToJson(this);
}

/// Mirror of evobase-protocol LoginRequest
@JsonSerializable()
class LoginRequest {
  final String username;
  final String password;

  LoginRequest({required this.username, required this.password});

  factory LoginRequest.fromJson(Map<String, dynamic> json) =>
      _$LoginRequestFromJson(json);

  Map<String, dynamic> toJson() => _$LoginRequestToJson(this);
}

/// Mirror of evobase-protocol RefreshRequest
@JsonSerializable()
class RefreshRequest {
  @JsonKey(name: 'refresh_token')
  final String refreshToken;

  RefreshRequest({required this.refreshToken});

  factory RefreshRequest.fromJson(Map<String, dynamic> json) =>
      _$RefreshRequestFromJson(json);

  Map<String, dynamic> toJson() => _$RefreshRequestToJson(this);
}

/// Mirror of evobase-protocol AuthResponseDto
@JsonSerializable()
class AuthResponseDto {
  @JsonKey(name: 'user_id')
  final String userId;
  final String username;
  final TokenDto tokens;

  AuthResponseDto({
    required this.userId,
    required this.username,
    required this.tokens,
  });

  factory AuthResponseDto.fromJson(Map<String, dynamic> json) =>
      _$AuthResponseDtoFromJson(json);

  Map<String, dynamic> toJson() => _$AuthResponseDtoToJson(this);
}

/// Mirror of evobase-protocol TokenDto
@JsonSerializable()
class TokenDto {
  @JsonKey(name: 'access_token')
  final String accessToken;

  @JsonKey(name: 'refresh_token')
  final String refreshToken;

  @JsonKey(name: 'notification_token')
  final String notificationToken;

  @JsonKey(name: 'token_type')
  final String tokenType;

  @JsonKey(name: 'expires_in')
  final int expiresIn;

  TokenDto({
    required this.accessToken,
    required this.refreshToken,
    required this.notificationToken,
    required this.tokenType,
    required this.expiresIn,
  });

  factory TokenDto.fromJson(Map<String, dynamic> json) =>
      _$TokenDtoFromJson(json);

  Map<String, dynamic> toJson() => _$TokenDtoToJson(this);
}
