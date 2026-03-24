import 'package:equatable/equatable.dart';

abstract class AuthEvent extends Equatable {
  const AuthEvent();

  @override
  List<Object?> get props => [];
}

class AuthLoginRequested extends AuthEvent {
  final String username;
  final String password;

  const AuthLoginRequested({required this.username, required this.password});

  @override
  List<Object?> get props => [username, password];
}

class AuthRegisterRequested extends AuthEvent {
  final String username;
  final String password;

  const AuthRegisterRequested({required this.username, required this.password});

  @override
  List<Object?> get props => [username, password];
}

class AuthLogoutRequested extends AuthEvent {
  const AuthLogoutRequested();
}

class AuthRefreshRequested extends AuthEvent {
  const AuthRefreshRequested();
}

class AuthCheckRequested extends AuthEvent {
  const AuthCheckRequested();
}

class AuthAdminTokenSet extends AuthEvent {
  final String token;

  const AuthAdminTokenSet(this.token);

  @override
  List<Object?> get props => [token];
}

class AuthAdminTokenCleared extends AuthEvent {
  const AuthAdminTokenCleared();
}
