import 'package:equatable/equatable.dart';

enum AuthStatus { initial, loading, authenticated, unauthenticated, error }

class AuthState extends Equatable {
  final AuthStatus status;
  final String? userId;
  final String? username;
  final bool isAdminMode;
  final String? errorMessage;

  const AuthState({
    this.status = AuthStatus.initial,
    this.userId,
    this.username,
    this.isAdminMode = false,
    this.errorMessage,
  });

  AuthState copyWith({
    AuthStatus? status,
    String? userId,
    String? username,
    bool? isAdminMode,
    String? errorMessage,
    bool clearErrorMessage = false,
  }) {
    return AuthState(
      status: status ?? this.status,
      userId: userId ?? this.userId,
      username: username ?? this.username,
      isAdminMode: isAdminMode ?? this.isAdminMode,
      errorMessage: clearErrorMessage
          ? null
          : (errorMessage ?? this.errorMessage),
    );
  }

  @override
  List<Object?> get props => [
    status,
    userId,
    username,
    isAdminMode,
    errorMessage,
  ];
}
