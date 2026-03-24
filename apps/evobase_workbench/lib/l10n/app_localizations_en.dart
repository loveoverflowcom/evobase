// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'EvoBase Workbench';

  @override
  String get signIn => 'Sign In';

  @override
  String get createAccount => 'Create Account';

  @override
  String get usernameLabel => 'Username';

  @override
  String get passwordLabel => 'Password';

  @override
  String get usernameRequired => 'Please enter username';

  @override
  String get passwordRequired => 'Please enter password';

  @override
  String get loginButton => 'Login';

  @override
  String get registerButton => 'Register';

  @override
  String get alreadyHaveAccountLogin => 'Already have an account? Login';

  @override
  String get needAccountRegister => 'Need an account? Register';

  @override
  String get loginFailedTitle => 'Login failed';

  @override
  String get unknownError => 'Unknown error';

  @override
  String get dashboard => 'Dashboard';

  @override
  String get apiExplorer => 'API Explorer';

  @override
  String get rlsTester => 'RLS Tester';

  @override
  String get eventMonitor => 'Event Monitor';

  @override
  String get settings => 'Settings';

  @override
  String get welcomeTitle => 'Welcome to EvoBase Workbench';

  @override
  String get welcomeSubtitle =>
      'Architecture is ready. Start building features!';

  @override
  String get logout => 'Logout';

  @override
  String get adminMode => 'Admin Mode';

  @override
  String get enableAdminMode => 'Enable Admin Mode';

  @override
  String get disableAdminMode => 'Disable Admin Mode';

  @override
  String get adminTokenTitle => 'Enter Admin Token';

  @override
  String get adminTokenLabel => 'Admin Token';

  @override
  String get cancelButton => 'Cancel';

  @override
  String get setButton => 'Set';

  @override
  String get okButton => 'OK';

  @override
  String get loadingLabel => 'Loading...';

  @override
  String get themeModeButtonTooltip => 'Theme mode';

  @override
  String get themeModeSystem => 'System';

  @override
  String get themeModeLight => 'Light';

  @override
  String get themeModeDark => 'Dark';
}
