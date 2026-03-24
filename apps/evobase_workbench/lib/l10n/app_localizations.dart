import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_vi.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('vi'),
  ];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'EvoBase Workbench'**
  String get appTitle;

  /// No description provided for @signIn.
  ///
  /// In en, this message translates to:
  /// **'Sign In'**
  String get signIn;

  /// No description provided for @createAccount.
  ///
  /// In en, this message translates to:
  /// **'Create Account'**
  String get createAccount;

  /// No description provided for @usernameLabel.
  ///
  /// In en, this message translates to:
  /// **'Username'**
  String get usernameLabel;

  /// No description provided for @passwordLabel.
  ///
  /// In en, this message translates to:
  /// **'Password'**
  String get passwordLabel;

  /// No description provided for @usernameRequired.
  ///
  /// In en, this message translates to:
  /// **'Please enter username'**
  String get usernameRequired;

  /// No description provided for @passwordRequired.
  ///
  /// In en, this message translates to:
  /// **'Please enter password'**
  String get passwordRequired;

  /// No description provided for @loginButton.
  ///
  /// In en, this message translates to:
  /// **'Login'**
  String get loginButton;

  /// No description provided for @registerButton.
  ///
  /// In en, this message translates to:
  /// **'Register'**
  String get registerButton;

  /// No description provided for @alreadyHaveAccountLogin.
  ///
  /// In en, this message translates to:
  /// **'Already have an account? Login'**
  String get alreadyHaveAccountLogin;

  /// No description provided for @needAccountRegister.
  ///
  /// In en, this message translates to:
  /// **'Need an account? Register'**
  String get needAccountRegister;

  /// No description provided for @loginFailedTitle.
  ///
  /// In en, this message translates to:
  /// **'Login failed'**
  String get loginFailedTitle;

  /// No description provided for @unknownError.
  ///
  /// In en, this message translates to:
  /// **'Unknown error'**
  String get unknownError;

  /// No description provided for @dashboard.
  ///
  /// In en, this message translates to:
  /// **'Dashboard'**
  String get dashboard;

  /// No description provided for @apiExplorer.
  ///
  /// In en, this message translates to:
  /// **'API Explorer'**
  String get apiExplorer;

  /// No description provided for @rlsTester.
  ///
  /// In en, this message translates to:
  /// **'RLS Tester'**
  String get rlsTester;

  /// No description provided for @eventMonitor.
  ///
  /// In en, this message translates to:
  /// **'Event Monitor'**
  String get eventMonitor;

  /// No description provided for @settings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settings;

  /// No description provided for @welcomeTitle.
  ///
  /// In en, this message translates to:
  /// **'Welcome to EvoBase Workbench'**
  String get welcomeTitle;

  /// No description provided for @welcomeSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Architecture is ready. Start building features!'**
  String get welcomeSubtitle;

  /// No description provided for @logout.
  ///
  /// In en, this message translates to:
  /// **'Logout'**
  String get logout;

  /// No description provided for @adminMode.
  ///
  /// In en, this message translates to:
  /// **'Admin Mode'**
  String get adminMode;

  /// No description provided for @enableAdminMode.
  ///
  /// In en, this message translates to:
  /// **'Enable Admin Mode'**
  String get enableAdminMode;

  /// No description provided for @disableAdminMode.
  ///
  /// In en, this message translates to:
  /// **'Disable Admin Mode'**
  String get disableAdminMode;

  /// No description provided for @adminTokenTitle.
  ///
  /// In en, this message translates to:
  /// **'Enter Admin Token'**
  String get adminTokenTitle;

  /// No description provided for @adminTokenLabel.
  ///
  /// In en, this message translates to:
  /// **'Admin Token'**
  String get adminTokenLabel;

  /// No description provided for @cancelButton.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancelButton;

  /// No description provided for @setButton.
  ///
  /// In en, this message translates to:
  /// **'Set'**
  String get setButton;

  /// No description provided for @okButton.
  ///
  /// In en, this message translates to:
  /// **'OK'**
  String get okButton;

  /// No description provided for @featureComingSoon.
  ///
  /// In en, this message translates to:
  /// **'This tool is next in line and will be added after the API Explorer flow.'**
  String get featureComingSoon;

  /// No description provided for @adminModeEnabledMessage.
  ///
  /// In en, this message translates to:
  /// **'Admin token is active. Lab-only tools can call protected endpoints.'**
  String get adminModeEnabledMessage;

  /// No description provided for @adminModeDisabledMessage.
  ///
  /// In en, this message translates to:
  /// **'Add an admin token to unlock lab-only tools and protected docs.'**
  String get adminModeDisabledMessage;

  /// No description provided for @apiExplorerTitle.
  ///
  /// In en, this message translates to:
  /// **'API Docs Explorer'**
  String get apiExplorerTitle;

  /// No description provided for @apiExplorerSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Browse schema, methods, query capabilities, and RLS policies from the live /docs endpoint.'**
  String get apiExplorerSubtitle;

  /// No description provided for @apiExplorerRefresh.
  ///
  /// In en, this message translates to:
  /// **'Refresh docs'**
  String get apiExplorerRefresh;

  /// No description provided for @apiExplorerSearchHint.
  ///
  /// In en, this message translates to:
  /// **'Search by schema, table, endpoint, or column'**
  String get apiExplorerSearchHint;

  /// No description provided for @apiExplorerTablesCount.
  ///
  /// In en, this message translates to:
  /// **'Tables'**
  String get apiExplorerTablesCount;

  /// No description provided for @apiExplorerColumnsCount.
  ///
  /// In en, this message translates to:
  /// **'Columns'**
  String get apiExplorerColumnsCount;

  /// No description provided for @apiExplorerSelectableColumns.
  ///
  /// In en, this message translates to:
  /// **'Selectable columns'**
  String get apiExplorerSelectableColumns;

  /// No description provided for @apiExplorerFilterOperators.
  ///
  /// In en, this message translates to:
  /// **'Filter operators'**
  String get apiExplorerFilterOperators;

  /// No description provided for @apiExplorerQueryCapabilities.
  ///
  /// In en, this message translates to:
  /// **'Query capabilities'**
  String get apiExplorerQueryCapabilities;

  /// No description provided for @apiExplorerMethods.
  ///
  /// In en, this message translates to:
  /// **'Methods'**
  String get apiExplorerMethods;

  /// No description provided for @apiExplorerColumns.
  ///
  /// In en, this message translates to:
  /// **'Columns'**
  String get apiExplorerColumns;

  /// No description provided for @apiExplorerRlsPolicies.
  ///
  /// In en, this message translates to:
  /// **'RLS policies'**
  String get apiExplorerRlsPolicies;

  /// No description provided for @apiExplorerEndpoint.
  ///
  /// In en, this message translates to:
  /// **'Endpoint'**
  String get apiExplorerEndpoint;

  /// No description provided for @apiExplorerSchema.
  ///
  /// In en, this message translates to:
  /// **'Schema'**
  String get apiExplorerSchema;

  /// No description provided for @apiExplorerTable.
  ///
  /// In en, this message translates to:
  /// **'Table'**
  String get apiExplorerTable;

  /// No description provided for @apiExplorerNoResults.
  ///
  /// In en, this message translates to:
  /// **'No tables match the current search.'**
  String get apiExplorerNoResults;

  /// No description provided for @apiExplorerEmptyTitle.
  ///
  /// In en, this message translates to:
  /// **'No documentation available'**
  String get apiExplorerEmptyTitle;

  /// No description provided for @apiExplorerEmptySubtitle.
  ///
  /// In en, this message translates to:
  /// **'The /docs endpoint returned no tables.'**
  String get apiExplorerEmptySubtitle;

  /// No description provided for @apiExplorerSelectTable.
  ///
  /// In en, this message translates to:
  /// **'Select a table to inspect its documentation.'**
  String get apiExplorerSelectTable;

  /// No description provided for @apiExplorerAdminRequiredTitle.
  ///
  /// In en, this message translates to:
  /// **'Admin mode is required'**
  String get apiExplorerAdminRequiredTitle;

  /// No description provided for @apiExplorerAdminRequiredSubtitle.
  ///
  /// In en, this message translates to:
  /// **'The docs explorer reads protected admin endpoints. Add the admin token to continue.'**
  String get apiExplorerAdminRequiredSubtitle;

  /// No description provided for @apiExplorerColumnsEmpty.
  ///
  /// In en, this message translates to:
  /// **'No columns were returned for this table.'**
  String get apiExplorerColumnsEmpty;

  /// No description provided for @apiExplorerPoliciesEmpty.
  ///
  /// In en, this message translates to:
  /// **'No RLS policies are defined for this table.'**
  String get apiExplorerPoliciesEmpty;

  /// No description provided for @apiExplorerRlsEnabled.
  ///
  /// In en, this message translates to:
  /// **'RLS enabled'**
  String get apiExplorerRlsEnabled;

  /// No description provided for @apiExplorerRlsDisabled.
  ///
  /// In en, this message translates to:
  /// **'RLS disabled'**
  String get apiExplorerRlsDisabled;

  /// No description provided for @apiExplorerOrderSupported.
  ///
  /// In en, this message translates to:
  /// **'Ordering'**
  String get apiExplorerOrderSupported;

  /// No description provided for @apiExplorerLimitSupported.
  ///
  /// In en, this message translates to:
  /// **'Limit'**
  String get apiExplorerLimitSupported;

  /// No description provided for @apiExplorerOffsetSupported.
  ///
  /// In en, this message translates to:
  /// **'Offset'**
  String get apiExplorerOffsetSupported;

  /// No description provided for @apiExplorerSampleRequest.
  ///
  /// In en, this message translates to:
  /// **'Sample request'**
  String get apiExplorerSampleRequest;

  /// No description provided for @loadingLabel.
  ///
  /// In en, this message translates to:
  /// **'Loading...'**
  String get loadingLabel;

  /// No description provided for @themeModeButtonTooltip.
  ///
  /// In en, this message translates to:
  /// **'Theme mode'**
  String get themeModeButtonTooltip;

  /// No description provided for @themeModeSystem.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get themeModeSystem;

  /// No description provided for @themeModeLight.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get themeModeLight;

  /// No description provided for @themeModeDark.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get themeModeDark;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'vi'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'vi':
      return AppLocalizationsVi();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
