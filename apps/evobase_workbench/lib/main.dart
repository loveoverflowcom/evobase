import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'app/blocs/auth.dart'
    show AuthBloc, AuthCheckRequested, AuthState, AuthStatus;
import 'app/pages.dart' show HomePage, LoginPage;
import 'client/api.dart' show AuthApi, DocsApi, RestApi;
import 'client/auth.dart' show AuthManager, TokenStorage;
import 'client/core.dart' show ApiClient;
import 'config.dart' show AppConfig;
import 'l10n.dart' show AppLocalizations;
import 'theme.dart' show ThemeCubit, ThemeStorage;

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final tokenStorage = await TokenStorage.create();
  final themeStorage = await ThemeStorage.create();
  final apiClient = ApiClient(
    baseUrl: AppConfig.current.baseUrl,
    connectTimeout: AppConfig.current.connectTimeout,
    receiveTimeout: AppConfig.current.receiveTimeout,
  );

  final authApi = AuthApi(apiClient);
  final docsApi = DocsApi(apiClient);
  final restApi = RestApi(apiClient);

  final authManager = AuthManager(
    authApi: authApi,
    apiClient: apiClient,
    tokenStorage: tokenStorage,
  );

  runApp(
    EvobaseWorkbench(
      authManager: authManager,
      authApi: authApi,
      docsApi: docsApi,
      restApi: restApi,
      themeStorage: themeStorage,
    ),
  );
}

class EvobaseWorkbench extends StatelessWidget {
  final AuthManager authManager;
  final AuthApi authApi;
  final DocsApi docsApi;
  final RestApi restApi;
  final ThemeStorage themeStorage;

  const EvobaseWorkbench({
    super.key,
    required this.authManager,
    required this.authApi,
    required this.docsApi,
    required this.restApi,
    required this.themeStorage,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => ThemeCubit(themeStorage: themeStorage),
      child: BlocProvider(
        create: (context) =>
            AuthBloc(authManager: authManager)..add(const AuthCheckRequested()),
        child: BlocBuilder<ThemeCubit, ThemeMode>(
          builder: (context, themeMode) {
            return MaterialApp(
              onGenerateTitle: (context) =>
                  AppLocalizations.of(context)!.appTitle,
              debugShowCheckedModeBanner: false,
              localizationsDelegates: const [
                AppLocalizations.delegate,
                GlobalMaterialLocalizations.delegate,
                GlobalWidgetsLocalizations.delegate,
                GlobalCupertinoLocalizations.delegate,
              ],
              supportedLocales: AppLocalizations.supportedLocales,
              theme: ThemeData(
                colorScheme: ColorScheme.fromSeed(
                  seedColor: Colors.blue,
                  brightness: Brightness.light,
                ),
                useMaterial3: true,
              ),
              darkTheme: ThemeData(
                colorScheme: ColorScheme.fromSeed(
                  seedColor: Colors.blue,
                  brightness: Brightness.dark,
                ),
                useMaterial3: true,
              ),
              themeMode: themeMode,
              home: AuthGate(docsApi: docsApi),
            );
          },
        ),
      ),
    );
  }
}

class AuthGate extends StatelessWidget {
  final DocsApi docsApi;

  const AuthGate({super.key, required this.docsApi});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    const authenticated = AuthStatus.authenticated;
    const unauthenticated = AuthStatus.unauthenticated;
    const error = AuthStatus.error;
    const initial = AuthStatus.initial;
    const loading = AuthStatus.loading;

    return BlocBuilder<AuthBloc, AuthState>(
      builder: (context, state) {
        switch (state.status) {
          case authenticated:
            return HomePage(docsApi: docsApi);
          case unauthenticated:
          case error:
            return const LoginPage();
          case initial:
          case loading:
            return Scaffold(body: Center(child: Text(l10n.loadingLabel)));
        }
      },
    );
  }
}
