// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Vietnamese (`vi`).
class AppLocalizationsVi extends AppLocalizations {
  AppLocalizationsVi([String locale = 'vi']) : super(locale);

  @override
  String get appTitle => 'EvoBase Workbench';

  @override
  String get signIn => 'Đăng nhập';

  @override
  String get createAccount => 'Tạo tài khoản';

  @override
  String get usernameLabel => 'Tên đăng nhập';

  @override
  String get passwordLabel => 'Mật khẩu';

  @override
  String get usernameRequired => 'Vui lòng nhập tên đăng nhập';

  @override
  String get passwordRequired => 'Vui lòng nhập mật khẩu';

  @override
  String get loginButton => 'Đăng nhập';

  @override
  String get registerButton => 'Đăng ký';

  @override
  String get alreadyHaveAccountLogin => 'Đã có tài khoản? Đăng nhập';

  @override
  String get needAccountRegister => 'Chưa có tài khoản? Đăng ký';

  @override
  String get loginFailedTitle => 'Đăng nhập thất bại';

  @override
  String get unknownError => 'Lỗi không xác định';

  @override
  String get dashboard => 'Bảng điều khiển';

  @override
  String get apiExplorer => 'Khám phá API';

  @override
  String get rlsTester => 'RLS Tester';

  @override
  String get eventMonitor => 'Event Monitor';

  @override
  String get settings => 'Cài đặt';

  @override
  String get welcomeTitle => 'Chào mừng đến với EvoBase Workbench';

  @override
  String get welcomeSubtitle =>
      'Kiến trúc đã sẵn sàng. Bắt đầu xây dựng tính năng!';

  @override
  String get logout => 'Đăng xuất';

  @override
  String get adminMode => 'Chế độ Admin';

  @override
  String get enableAdminMode => 'Bật chế độ Admin';

  @override
  String get disableAdminMode => 'Tắt chế độ Admin';

  @override
  String get adminTokenTitle => 'Nhập Admin Token';

  @override
  String get adminTokenLabel => 'Admin Token';

  @override
  String get cancelButton => 'Hủy';

  @override
  String get setButton => 'Thiết lập';

  @override
  String get okButton => 'OK';

  @override
  String get loadingLabel => 'Đang tải...';

  @override
  String get themeModeButtonTooltip => 'Chế độ giao diện';

  @override
  String get themeModeSystem => 'Giống hệ thống';

  @override
  String get themeModeLight => 'Sáng';

  @override
  String get themeModeDark => 'Tối';
}
