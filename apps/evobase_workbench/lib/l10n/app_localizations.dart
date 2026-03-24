import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import 'app_localizations_en.dart';
import 'app_localizations_vi.dart';

abstract class AppLocalizations {
  AppLocalizations(this.locale);

  final Locale locale;

  static const supportedLocales = <Locale>[Locale('en'), Locale('vi')];

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  static AppLocalizations of(BuildContext context) {
    final localizations = Localizations.of<AppLocalizations>(
      context,
      AppLocalizations,
    );
    assert(
      localizations != null,
      'AppLocalizations are not available in this context.',
    );
    return localizations!;
  }

  String get appTitle;
  String get signIn;
  String get createAccount;
  String get usernameLabel;
  String get passwordLabel;
  String get usernameRequired;
  String get passwordRequired;
  String get loginButton;
  String get registerButton;
  String get alreadyHaveAccountLogin;
  String get needAccountRegister;
  String get loginFailedTitle;
  String get unknownError;
  String get dashboard;
  String get apiExplorer;
  String get rlsTester;
  String get eventMonitor;
  String get settings;
  String get welcomeTitle;
  String get welcomeSubtitle;
  String get logout;
  String get adminMode;
  String get enableAdminMode;
  String get disableAdminMode;
  String get adminTokenTitle;
  String get adminTokenLabel;
  String get cancelButton;
  String get setButton;
  String get okButton;
  String get loadingLabel;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) => AppLocalizations.supportedLocales.any(
    (supportedLocale) => supportedLocale.languageCode == locale.languageCode,
  );

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(_lookupAppLocalizations(locale));
  }

  @override
  bool shouldReload(covariant LocalizationsDelegate<AppLocalizations> old) =>
      false;
}

AppLocalizations _lookupAppLocalizations(Locale locale) {
  switch (locale.languageCode) {
    case 'vi':
      return AppLocalizationsVi();
    case 'en':
    default:
      return AppLocalizationsEn();
  }
}
