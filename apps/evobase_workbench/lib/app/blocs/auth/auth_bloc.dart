import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../client/auth.dart' show AuthManager;
import '../../../client/core.dart' show ApiException;
import 'auth_event.dart';
import 'auth_state.dart';

/// Auth Bloc - orchestrates authentication flow.
/// UI → Bloc → AuthManager → ApiClient
class AuthBloc extends Bloc<AuthEvent, AuthState> {
  final AuthManager _authManager;

  AuthBloc({required AuthManager authManager})
    : _authManager = authManager,
      super(const AuthState()) {
    on<AuthLoginRequested>(_onLoginRequested);
    on<AuthRegisterRequested>(_onRegisterRequested);
    on<AuthLogoutRequested>(_onLogoutRequested);
    on<AuthRefreshRequested>(_onRefreshRequested);
    on<AuthCheckRequested>(_onCheckRequested);
    on<AuthAdminTokenSet>(_onAdminTokenSet);
    on<AuthAdminTokenCleared>(_onAdminTokenCleared);
  }

  Future<void> _onLoginRequested(
    AuthLoginRequested event,
    Emitter<AuthState> emit,
  ) async {
    emit(state.copyWith(status: AuthStatus.loading, clearErrorMessage: true));

    final result = await _authManager
        .login(username: event.username, password: event.password)
        .run();

    result.match(
      (failure) => _emitFailure(emit, failure),
      (response) =>
          _emitAuthenticated(emit, response.userId, response.username),
    );
  }

  Future<void> _onRegisterRequested(
    AuthRegisterRequested event,
    Emitter<AuthState> emit,
  ) async {
    emit(state.copyWith(status: AuthStatus.loading, clearErrorMessage: true));

    final result = await _authManager
        .register(username: event.username, password: event.password)
        .run();

    result.match(
      (failure) => _emitFailure(emit, failure),
      (response) =>
          _emitAuthenticated(emit, response.userId, response.username),
    );
  }

  Future<void> _onLogoutRequested(
    AuthLogoutRequested event,
    Emitter<AuthState> emit,
  ) async {
    await _authManager.logout();
    emit(const AuthState(status: AuthStatus.unauthenticated));
  }

  Future<void> _onRefreshRequested(
    AuthRefreshRequested event,
    Emitter<AuthState> emit,
  ) async {
    final result = await _authManager.refresh().run();
    result.match(
      (failure) => _emitFailure(emit, failure),
      (response) => emit(
        state.copyWith(
          status: AuthStatus.authenticated,
          userId: response.userId,
          username: response.username,
          isAdminMode: _authManager.isAdminMode(),
          clearErrorMessage: true,
        ),
      ),
    );
  }

  Future<void> _onCheckRequested(
    AuthCheckRequested event,
    Emitter<AuthState> emit,
  ) async {
    if (_authManager.isLoggedIn()) {
      emit(
        state.copyWith(
          status: AuthStatus.authenticated,
          userId: _authManager.getUserId(),
          username: _authManager.getUsername(),
          isAdminMode: _authManager.isAdminMode(),
          clearErrorMessage: true,
        ),
      );
    } else {
      emit(const AuthState(status: AuthStatus.unauthenticated));
    }
  }

  Future<void> _onAdminTokenSet(
    AuthAdminTokenSet event,
    Emitter<AuthState> emit,
  ) async {
    await _authManager.setAdminToken(event.token);
    emit(state.copyWith(isAdminMode: _authManager.isAdminMode()));
  }

  Future<void> _onAdminTokenCleared(
    AuthAdminTokenCleared event,
    Emitter<AuthState> emit,
  ) async {
    await _authManager.clearAdminToken();
    emit(state.copyWith(isAdminMode: false));
  }

  void _emitAuthenticated(
    Emitter<AuthState> emit,
    String userId,
    String username,
  ) {
    emit(
      state.copyWith(
        status: AuthStatus.authenticated,
        userId: userId,
        username: username,
        isAdminMode: _authManager.isAdminMode(),
        clearErrorMessage: true,
      ),
    );
  }

  void _emitFailure(Emitter<AuthState> emit, ApiException failure) {
    emit(
      state.copyWith(
        status: AuthStatus.error,
        errorMessage: failure.message,
        isAdminMode: _authManager.isAdminMode(),
      ),
    );
  }
}
