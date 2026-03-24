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
  String get featureComingSoon =>
      'This tool is next in line and will be added after the API Explorer flow.';

  @override
  String get adminModeEnabledMessage =>
      'Admin token is active. Lab-only tools can call protected endpoints.';

  @override
  String get adminModeDisabledMessage =>
      'Add an admin token to unlock lab-only tools and protected docs.';

  @override
  String get apiExplorerTitle => 'API Docs Explorer';

  @override
  String get apiExplorerSubtitle =>
      'Browse schema, methods, query capabilities, and RLS policies from the live /docs endpoint.';

  @override
  String get apiExplorerRefresh => 'Refresh docs';

  @override
  String get apiExplorerSearchHint =>
      'Search by schema, table, endpoint, or column';

  @override
  String get apiExplorerTablesCount => 'Tables';

  @override
  String get apiExplorerColumnsCount => 'Columns';

  @override
  String get apiExplorerSelectableColumns => 'Selectable columns';

  @override
  String get apiExplorerFilterOperators => 'Filter operators';

  @override
  String get apiExplorerQueryCapabilities => 'Query capabilities';

  @override
  String get apiExplorerMethods => 'Methods';

  @override
  String get apiExplorerColumns => 'Columns';

  @override
  String get apiExplorerRlsPolicies => 'RLS policies';

  @override
  String get apiExplorerEndpoint => 'Endpoint';

  @override
  String get apiExplorerSchema => 'Schema';

  @override
  String get apiExplorerTable => 'Table';

  @override
  String get apiExplorerNoResults => 'No tables match the current search.';

  @override
  String get apiExplorerEmptyTitle => 'No documentation available';

  @override
  String get apiExplorerEmptySubtitle =>
      'The /docs endpoint returned no tables.';

  @override
  String get apiExplorerSelectTable =>
      'Select a table to inspect its documentation.';

  @override
  String get apiExplorerAdminRequiredTitle => 'Admin mode is required';

  @override
  String get apiExplorerAdminRequiredSubtitle =>
      'The docs explorer reads protected admin endpoints. Add the admin token to continue.';

  @override
  String get apiExplorerColumnsEmpty =>
      'No columns were returned for this table.';

  @override
  String get apiExplorerPoliciesEmpty =>
      'No RLS policies are defined for this table.';

  @override
  String get apiExplorerRlsEnabled => 'RLS enabled';

  @override
  String get apiExplorerRlsDisabled => 'RLS disabled';

  @override
  String get apiExplorerOrderSupported => 'Ordering';

  @override
  String get apiExplorerLimitSupported => 'Limit';

  @override
  String get apiExplorerOffsetSupported => 'Offset';

  @override
  String get apiExplorerSampleRequest => 'Sample request';

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
