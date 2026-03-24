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
  String get featureComingSoon =>
      'Công cụ này sẽ được bổ sung tiếp sau khi hoàn thiện luồng API Explorer.';

  @override
  String get adminModeEnabledMessage =>
      'Admin token đang hoạt động. Các công cụ lab có thể gọi endpoint được bảo vệ.';

  @override
  String get adminModeDisabledMessage =>
      'Thêm admin token để mở khóa các công cụ lab và docs được bảo vệ.';

  @override
  String get apiExplorerTitle => 'API Docs Explorer';

  @override
  String get apiExplorerSubtitle =>
      'Duyệt schema, methods, khả năng query và chính sách RLS từ endpoint /docs đang chạy.';

  @override
  String get apiExplorerRefresh => 'Tải lại docs';

  @override
  String get apiExplorerSearchHint =>
      'Tìm theo schema, table, endpoint hoặc column';

  @override
  String get apiExplorerTablesCount => 'Bảng';

  @override
  String get apiExplorerColumnsCount => 'Cột';

  @override
  String get apiExplorerSelectableColumns => 'Cột có thể select';

  @override
  String get apiExplorerFilterOperators => 'Toán tử filter';

  @override
  String get apiExplorerQueryCapabilities => 'Khả năng query';

  @override
  String get apiExplorerMethods => 'Methods';

  @override
  String get apiExplorerColumns => 'Cột';

  @override
  String get apiExplorerRlsPolicies => 'Chính sách RLS';

  @override
  String get apiExplorerEndpoint => 'Endpoint';

  @override
  String get apiExplorerSchema => 'Schema';

  @override
  String get apiExplorerTable => 'Table';

  @override
  String get apiExplorerNoResults =>
      'Không có bảng nào khớp với từ khóa hiện tại.';

  @override
  String get apiExplorerEmptyTitle => 'Chưa có tài liệu API';

  @override
  String get apiExplorerEmptySubtitle =>
      'Endpoint /docs không trả về bảng nào.';

  @override
  String get apiExplorerSelectTable =>
      'Chọn một bảng để xem tài liệu chi tiết.';

  @override
  String get apiExplorerAdminRequiredTitle => 'Cần bật admin mode';

  @override
  String get apiExplorerAdminRequiredSubtitle =>
      'Docs explorer đọc các admin endpoint được bảo vệ. Hãy thêm admin token để tiếp tục.';

  @override
  String get apiExplorerColumnsEmpty =>
      'Không có cột nào được trả về cho bảng này.';

  @override
  String get apiExplorerPoliciesEmpty =>
      'Không có chính sách RLS nào cho bảng này.';

  @override
  String get apiExplorerRlsEnabled => 'RLS đang bật';

  @override
  String get apiExplorerRlsDisabled => 'RLS đang tắt';

  @override
  String get apiExplorerOrderSupported => 'Sắp xếp';

  @override
  String get apiExplorerLimitSupported => 'Giới hạn';

  @override
  String get apiExplorerOffsetSupported => 'Độ lệch';

  @override
  String get apiExplorerSampleRequest => 'Request mẫu';

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
