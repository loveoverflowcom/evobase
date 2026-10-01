export const screens = [
  {
    "id": "workspace",
    "pack": "01-foundation",
    "surface": "Workspace",
    "section": "Ứng dụng",
    "kind": "hub",
    "title": "Một nơi cho công việc",
    "kicker": "KHÔNG GIAN / KHÔNG GIAN BÁN HÀNG",
    "description": "Bắt đầu từ bảng. Tạo công cụ vừa với đội của bạn.",
    "action": "Tạo ứng dụng",
    "status": "Không gian làm việc",
    "sections": [
      [
        "Bán hàng",
        "4 bảng · 3 chế độ xem · Draft 12",
        "Tiếp tục xây dựng"
      ],
      [
        "Vận hành",
        "Phiên bản 1.2 · Hosted",
        "Mở Runtime"
      ],
      [
        "Thư viện mẫu",
        "Định nghĩa không chứa dữ liệu khách hàng",
        "Khám phá mẫu"
      ]
    ],
    "note": "Định nghĩa ứng dụng, dữ liệu vận hành và kết nối tài khoản có vòng đời riêng."
  },
  {
    "id": "new-app",
    "pack": "01-foundation",
    "surface": "Workspace",
    "section": "Ứng dụng",
    "kind": "form",
    "title": "Bắt đầu với một bảng",
    "kicker": "ỨNG DỤNG MỚI",
    "description": "Chọn một nền tảng nhỏ, mở rộng khi công việc cần.",
    "action": "Tạo bản nháp",
    "status": "Chưa được xuất bản",
    "fields": [
      [
        "Tên ứng dụng",
        "Bán hàng"
      ],
      [
        "Bắt đầu từ",
        "Mẫu Khách hàng / Đơn hàng"
      ],
      [
        "Ngôn ngữ nhãn",
        "Tiếng Việt"
      ],
      [
        "Không gian sở hữu",
        "Bán hàng"
      ]
    ],
    "sections": [
      [
        "Mẫu có kiểu",
        "Customers, Products, Orders, OrderLines",
        "Stable IDs và Ref; dữ liệu ví dụ tùy chọn"
      ],
      [
        "Không gian trống",
        "Tự định nghĩa bảng đầu tiên",
        "Không cần SQL hoặc Rust"
      ]
    ],
    "note": "Tạo draft không tạo tenant database và không gửi thông báo."
  },
  {
    "id": "builder-data",
    "pack": "02-data-grid",
    "surface": "Builder",
    "section": "Bảng dữ liệu",
    "kind": "grid",
    "title": "Đơn hàng",
    "kicker": "BÁN HÀNG / BẢN NHÁP 12",
    "description": "Liên kết khách hàng và theo dõi đơn trong bản nháp của bạn.",
    "action": "Thêm bản ghi mẫu",
    "status": "Đã lưu bản nháp",
    "columns": [
      "Mã đơn",
      "Khách hàng ↗",
      "Trạng thái",
      "Tổng (VND) ƒ",
      "Phụ trách"
    ],
    "rows": [
      [
        "ORD-1048",
        "Minh An",
        "Đã gửi",
        "12.480.000 ₫",
        "Linh"
      ],
      [
        "ORD-1047",
        "An Phú",
        "Nháp",
        "4.620.000 ₫",
        "Bạn"
      ],
      [
        "ORD-1046",
        "Hải Yến",
        "Đã duyệt",
        "8.950.000 ₫",
        "Linh"
      ],
      [
        "ORD-1045",
        "Đông Nam",
        "Nháp",
        "2.180.000 ₫",
        "Bạn"
      ],
      [
        "ORD-1044",
        "Bình Minh",
        "Đã gửi",
        "6.740.000 ₫",
        "An"
      ]
    ],
    "inspector": [
      [
        "KIỂU CỘT",
        "Liên kết → Khách hàng"
      ],
      [
        "Hiển thị",
        "Tên khách hàng"
      ],
      [
        "Bắt buộc",
        "Có"
      ],
      [
        "Khi xóa",
        "Chặn khi còn đơn"
      ],
      [
        "Chiều ngược",
        "Các đơn hàng (tự tạo)"
      ],
      [
        "Nâng cao",
        "field_customer · Ref<Customers>"
      ]
    ],
    "note": "Ô hiển thị nhãn, lưu record identity. Không mở quyền đọc chỉ vì có quan hệ."
  },
  {
    "id": "field-inspector",
    "pack": "02-data-grid",
    "surface": "Builder",
    "section": "Bảng dữ liệu",
    "kind": "form",
    "title": "Một cột, một ý nghĩa",
    "kicker": "ORDERS / KHÁCH HÀNG",
    "description": "Chọn kiểu và ràng buộc trước khi nhập dữ liệu.",
    "action": "Lưu cấu hình cột",
    "status": "Thay đổi trong draft",
    "fields": [
      [
        "Tên cột",
        "Khách hàng"
      ],
      [
        "Kiểu dữ liệu",
        "Liên kết đến bảng Khách hàng"
      ],
      [
        "Hiển thị bằng",
        "Tên khách hàng"
      ],
      [
        "Số bản ghi được chọn",
        "Một khách hàng cho mỗi đơn"
      ],
      [
        "Bắt buộc chọn",
        "Có"
      ],
      [
        "Khi xóa khách hàng",
        "Chặn nếu vẫn còn đơn hàng"
      ]
    ],
    "sections": [
      [
        "Giữ nguyên liên kết",
        "Đổi tên hoặc kéo cột không làm mất liên kết",
        "Mã cố định được giữ phía sau"
      ],
      [
        "Danh sách chọn",
        "Chỉ những khách hàng được phép xem",
        "Quyền của ứng dụng được kiểm tra khi chọn"
      ]
    ],
    "note": "Chuyển kiểu có dữ liệu cần impact preview; không đổi thầm sau một lần click."
  },
  {
    "id": "import-resolve",
    "pack": "02-data-grid",
    "surface": "Builder",
    "section": "Bảng dữ liệu",
    "kind": "review",
    "title": "Khớp đúng trước khi nhập",
    "kicker": "IMPORT / KIỂM TRA THAM CHIẾU",
    "description": "Hai tên bị trùng. Chọn record đúng hoặc bỏ dòng đó.",
    "action": "Tiếp tục 28 dòng hợp lệ",
    "status": "2 dòng cần giải quyết",
    "sections": [
      [
        "Dòng 12 · Minh An",
        "2 mục tiêu cùng tên",
        "CUS-008 · Minh An / Hà Nội"
      ],
      [
        "Dòng 19 · Đại Lộc",
        "Không tìm thấy record hợp lệ",
        "Tạo riêng hoặc bỏ dòng; không đoán Ref"
      ],
      [
        "28 dòng đã kiểm",
        "Kiểu và ràng buộc hợp lệ",
        "Chưa ghi vào Runtime"
      ]
    ],
    "inspector": [
      [
        "Nguồn",
        "customers-demo.csv (fixture)"
      ],
      [
        "Mapping",
        "Khách hàng → field_customer"
      ],
      [
        "Cách ghi",
        "Bản nháp / dữ liệu mẫu"
      ],
      [
        "Lỗi",
        "Không có phép tự chọn"
      ]
    ],
    "note": "Paste và IME phải giữ text đến khi commit; lỗi theo ô/dòng, undo có ranh giới rõ."
  },
  {
    "id": "relations",
    "pack": "03-structure-formulas",
    "surface": "Builder",
    "section": "Cấu trúc & Quan hệ",
    "kind": "relations",
    "title": "Các bảng kết nối ra sao?",
    "kicker": "CẤU TRÚC / 4 BẢNG CÓ KIỂU",
    "description": "Liên kết để thông tin đi cùng nhau, không cần nhập lại.",
    "action": "Thêm quan hệ",
    "status": "4 bảng · bản nháp",
    "sections": [
      [
        "Customers",
        "name · email · owner",
        "Orders ← reverse(Orders.customer)"
      ],
      [
        "Orders",
        "customer: Ref · state: Enum",
        "total := sum(lines.amount)"
      ],
      [
        "OrderLines",
        "order: Ref · product: Ref",
        "qty · captured_unit_price · amount"
      ],
      [
        "Products",
        "name · price · active",
        "Giá hiện tại không đổi giá đã chốt"
      ]
    ],
    "inspector": [
      [
        "Orders.customer",
        "N:1 → Customers"
      ],
      [
        "Delete",
        "Restrict"
      ],
      [
        "1:1 / Một-một",
        "Mục tiêu sau gate · chưa bật"
      ],
      [
        "N:M / Nhiều-nhiều",
        "Mục tiêu sau gate · chưa bật"
      ],
      [
        "Ref scope",
        "Cùng tenant + app instance"
      ]
    ],
    "note": "Không duy trì hai chiều canonical. Relation không tự sinh policy."
  },
  {
    "id": "formula",
    "pack": "03-structure-formulas",
    "surface": "Builder",
    "section": "Cấu trúc & Quan hệ",
    "kind": "editor",
    "title": "Tính tổng từ các dòng đơn",
    "kicker": "ORDERS / TOTAL",
    "description": "Chọn dữ liệu nguồn, phép tính và kiểm tra kết quả.",
    "action": "Áp dụng công thức",
    "status": "Công thức mẫu hợp lệ",
    "fields": [
      [
        "Tên cột kết quả",
        "Tổng cộng"
      ],
      [
        "Kiểu kết quả",
        "Tiền · VND"
      ],
      [
        "Dữ liệu nguồn",
        "Các dòng đơn → Thành tiền"
      ],
      [
        "Phép tính",
        "Tổng (SUM)"
      ]
    ],
    "sections": [
      [
        "Thử với ORD-1048",
        "3 dòng · 12.480.000 ₫",
        "Giá trong đơn giữ nguyên khi giá sản phẩm đổi"
      ],
      [
        "Khi không được phép xem",
        "Hiển thị Không có quyền",
        "Không giả một tổng chưa đầy đủ"
      ],
      [
        "Xem biểu thức nâng cao",
        "sum(lines.amount)",
        "Tên bảng/cột liên kết bằng mã cố định"
      ]
    ],
    "note": "Syntax minh họa; parser/subset phải được chốt bằng kernel ADR và golden tests."
  },
  {
    "id": "policy",
    "pack": "04-policies-commands",
    "surface": "Builder",
    "section": "Quy tắc & Quyền",
    "kind": "policy",
    "title": "Ai được xem đơn hàng?",
    "kicker": "ORDERS / READ POLICY",
    "description": "Chọn người, điều kiện và kiểm tra bằng tình huống mẫu.",
    "action": "Lưu quy tắc",
    "status": "Chưa áp dụng cho ứng dụng đã xuất bản",
    "fields": [
      [
        "Ai",
        "Quản lý hoặc người phụ trách"
      ],
      [
        "Được làm gì",
        "Xem đơn hàng"
      ],
      [
        "Điều kiện",
        "Đơn do chính mình phụ trách"
      ],
      [
        "Nếu không thỏa điều kiện",
        "Không cho phép"
      ]
    ],
    "sections": [
      [
        "Linh · Quản lý",
        "Được xem trong ví dụ",
        "Quyền tài khoản vẫn cần hợp lệ"
      ],
      [
        "An · Đơn An phụ trách",
        "Được xem trong ví dụ",
        "Chỉ hiện trường được cho phép"
      ],
      [
        "An · Đơn Linh phụ trách",
        "Không được xem",
        "Không mở quyền qua liên kết khách hàng"
      ]
    ],
    "note": "Runtime kiểm tenant boundary + current grants + app policy + restrictions; missing context phải deny."
  },
  {
    "id": "command",
    "pack": "04-policies-commands",
    "surface": "Builder",
    "section": "Quy tắc & Quyền",
    "kind": "flow",
    "title": "Gửi đơn hàng đúng điều kiện",
    "kicker": "HÀNH ĐỘNG / GỬI ĐƠN HÀNG",
    "description": "Chỉ cho gửi khi đơn đủ thông tin và người gửi có quyền.",
    "action": "Kiểm tra command",
    "status": "Hành động mẫu",
    "steps": [
      [
        "Quyền",
        "Người dùng được phép gửi đơn"
      ],
      [
        "Điều kiện",
        "Đơn là Nháp · có ít nhất 1 sản phẩm"
      ],
      [
        "Thay đổi",
        "Nháp → Đã gửi"
      ],
      [
        "Ghi nhận",
        "Lưu đơn và yêu cầu thông báo cùng nhau"
      ],
      [
        "Sau khi lưu",
        "Tiến hành gửi thông báo nếu đủ quyền"
      ]
    ],
    "inspector": [
      [
        "Input",
        "Ref<Orders>"
      ],
      [
        "Idempotency",
        "Request key + immutable intent"
      ],
      [
        "Xung đột",
        "409 / tải lại revision"
      ],
      [
        "Mutation",
        "Kiểm old + new state"
      ],
      [
        "Output",
        "Typed receipt"
      ]
    ],
    "note": "Không giữ transaction qua email, mạng hoặc người duyệt. Chưa có provider thật trong thiết kế. Committed retry trả original receipt dưới current grants dù live revision đã tiến."
  },
  {
    "id": "view-builder",
    "pack": "06-release",
    "surface": "Builder",
    "section": "Chế độ xem",
    "kind": "review",
    "title": "Chỉ hiện điều người dùng cần",
    "kicker": "VIEW / SALES WORKSPACE",
    "description": "Ghép List, Detail và Form từ định nghĩa đã kiểm.",
    "action": "Mở preview Runtime",
    "status": "Preview theo actor mẫu",
    "sections": [
      [
        "Danh sách đơn hàng",
        "Mã đơn · Khách hàng · Tổng · Trạng thái",
        "Sort: cập nhật gần nhất · page size có giới hạn"
      ],
      [
        "Form tạo đơn",
        "Customer picker · các dòng sản phẩm",
        "Quyền và validation từ core"
      ],
      [
        "Hành động",
        "Submit / Approve theo khả năng",
        "Không render nút không có quyền"
      ]
    ],
    "inspector": [
      [
        "Surface",
        "Runtime"
      ],
      [
        "Primary view",
        "Orders / List"
      ],
      [
        "Mobile",
        "List-detail / bottom sheet"
      ],
      [
        "Ẩn field",
        "Transport vẫn enforce policy"
      ]
    ],
    "note": "Ẩn ở UI không phải bảo mật. Các field system receipt/provider không editable qua generic grid."
  },
  {
    "id": "preview",
    "pack": "06-release",
    "surface": "Builder",
    "section": "Xuất bản",
    "kind": "review",
    "title": "Thử trước, không gửi thật",
    "kicker": "PREVIEW / BẢN NHÁP 12",
    "description": "Dùng dữ liệu mẫu để kiểm tra màn hình, quyền và quy trình.",
    "action": "Chạy 6 tình huống mẫu",
    "status": "MÔ PHỎNG · không gửi thông báo",
    "sections": [
      [
        "Tạo và gửi đơn",
        "Typed guards / captured price",
        "Fixture · không ghi tenant live"
      ],
      [
        "Đọc đơn người khác",
        "Deny theo policy",
        "Kiểm relation picker, lookup và export"
      ],
      [
        "Approval quá hạn",
        "Timer và error branch mô phỏng",
        "Không dùng connector live"
      ],
      [
        "Đổi phiên bản",
        "Run v1 giữ plan v1",
        "Kiểm release và binding tách biệt"
      ]
    ],
    "inspector": [
      [
        "Actor",
        "An · SalesRep (fixture)"
      ],
      [
        "Release",
        "Draft 12"
      ],
      [
        "Host effects",
        "Disabled"
      ],
      [
        "Kiểm chứng",
        "Minh họa · chưa có test evidence"
      ],
      [
        "Evidence",
        "Không phải provider test"
      ]
    ],
    "note": "Thiết kế fixture không đồng nghĩa tests đã chạy. Kết quả runtime phải lấy từ test evidence thật."
  },
  {
    "id": "publish",
    "pack": "06-release",
    "surface": "Builder",
    "section": "Xuất bản",
    "kind": "review",
    "title": "Sẵn sàng đưa vào sử dụng?",
    "kicker": "RELEASE / 1.3.0 CANDIDATE",
    "description": "Xem thay đổi và điều kiện còn thiếu trước khi xuất bản.",
    "action": "Yêu cầu xuất bản",
    "status": "Còn 1 điều kiện cần hoàn tất",
    "sections": [
      [
        "Định nghĩa đã kiểm",
        "4 tables · 2 commands · 1 function",
        "Immutable release fingerprint"
      ],
      [
        "Tác động dữ liệu",
        "Thêm optional field · không backfill",
        "Không sinh business SQL table trong fixed-store subset"
      ],
      [
        "Binding của tenant",
        "Email chưa được cấp consent",
        "Tắt function gửi email khi chưa có binding"
      ],
      [
        "Điểm chặn",
        "Storage conformance chưa có evidence",
        "Không báo production-ready từ preview"
      ]
    ],
    "inspector": [
      [
        "AppSpec",
        "1.3.0"
      ],
      [
        "Tenant data revision",
        "Khác model version"
      ],
      [
        "Activation",
        "Atomic / có recovery plan"
      ],
      [
        "Runs đang chạy",
        "Pin release cũ"
      ],
      [
        "Rollback",
        "Không undo effect đã gửi"
      ]
    ],
    "note": "Template update không tự nâng mọi tenant. Từng app instance có release được pin riêng."
  },
  {
    "id": "migration",
    "pack": "06-release",
    "surface": "Builder",
    "section": "Xuất bản",
    "kind": "review",
    "title": "Kiểm tra trước khi đổi kiểu",
    "kicker": "SCHEMA EVOLUTION / IMPACT",
    "description": "Giải quyết dữ liệu bị ảnh hưởng trước khi áp dụng phiên bản mới.",
    "action": "Tạo kế hoạch migration",
    "status": "Chạy thử · chưa áp dụng",
    "sections": [
      [
        "Đổi field required",
        "customer: optional → required",
        "7 record thiếu giá trị cần giải quyết"
      ],
      [
        "Backfill có kiểm",
        "Map stable IDs và policy",
        "Không ghi đè field người dùng vừa sửa"
      ],
      [
        "Atomic activation",
        "Revalidate → cutover → fencing",
        "App instance giữ revision cũ nếu gate lỗi"
      ],
      [
        "Recovery",
        "Export / restore theo scope",
        "Không hồi sinh revoked grants hoặc replay effects"
      ]
    ],
    "inspector": [
      [
        "Ảnh hưởng",
        "7 / 148 orders (fixture)"
      ],
      [
        "Quyền",
        "Migration operator"
      ],
      [
        "Recovery",
        "Bắt buộc xác minh"
      ],
      [
        "Dữ liệu live",
        "Chưa chạm"
      ]
    ],
    "note": "Storage adapter/codec/format extension phải có ADR; không coi đổi đuôi file là migration."
  },
  {
    "id": "runtime-home",
    "pack": "07-runtime",
    "surface": "Runtime",
    "section": "Đơn hàng",
    "kind": "runtime",
    "title": "Hôm nay cần làm gì?",
    "kicker": "KHÔNG GIAN BÁN HÀNG / SALES / PHIÊN BẢN 1.2",
    "description": "Công việc đang chờ bạn và những đơn hàng gần đây.",
    "action": "Tạo đơn hàng",
    "status": "Đã đồng bộ",
    "sections": [
      [
        "Cần bạn xử lý",
        "3 đơn đang chờ duyệt",
        "Mở danh sách công việc"
      ],
      [
        "Đơn hàng của bạn",
        "12 đơn đang hoạt động",
        "Gần nhất: ORD-1047 · An Phú"
      ],
      [
        "Dữ liệu gần đây",
        "Khách hàng và sản phẩm được phép",
        "Đọc theo policy hiện tại"
      ]
    ],
    "note": "Tenant được xác thực từ host context. Selector trên URL không tự cấp quyền.",
    "tasks": [
      [
        "ORD-1048",
        "Minh An",
        "12.480.000 ₫",
        "Hạn 16:30",
        "Linh",
        "Cần duyệt"
      ],
      [
        "ORD-1044",
        "Bình Minh",
        "6.740.000 ₫",
        "Hạn ngày mai",
        "An",
        "Cần duyệt"
      ]
    ],
    "recent": [
      [
        "ORD-1047",
        "An Phú",
        "4.620.000 ₫",
        "10 phút trước",
        "Bạn",
        "Nháp"
      ],
      [
        "ORD-1045",
        "Đông Nam",
        "2.180.000 ₫",
        "25 phút trước",
        "Bạn",
        "Nháp"
      ]
    ]
  },
  {
    "id": "runtime-detail",
    "pack": "07-runtime",
    "surface": "Runtime",
    "section": "Đơn hàng",
    "kind": "detail",
    "title": "ORD-1048",
    "kicker": "ĐƠN HÀNG / MINH AN",
    "description": "Đã gửi · Linh phụ trách · cập nhật 10 phút trước",
    "action": "Mở yêu cầu duyệt",
    "status": "Đã lưu",
    "fields": [
      [
        "Khách hàng",
        "Minh An"
      ],
      [
        "Tổng cộng",
        "12.480.000 ₫"
      ],
      [
        "Trạng thái",
        "Đã gửi"
      ],
      [
        "Phụ trách",
        "Linh"
      ]
    ],
    "sections": [
      [
        "Sản phẩm trong đơn",
        "3 sản phẩm · giá được giữ khi tạo đơn",
        "Xem số lượng và thành tiền"
      ],
      [
        "Lịch sử đơn",
        "Linh gửi đơn lúc 09:12",
        "Thông tin xác nhận chỉ được xem"
      ],
      [
        "Yêu cầu duyệt",
        "Đơn vượt mức cần phê duyệt",
        "Mở yêu cầu để xem quyết định"
      ]
    ],
    "note": "Đổi tên khách hàng không đổi identity; action luôn recheck current grants tại command commit."
  },
  {
    "id": "runtime-form",
    "pack": "07-runtime",
    "surface": "Runtime",
    "section": "Đơn hàng",
    "kind": "form",
    "title": "Tạo đơn hàng",
    "kicker": "ORDERS / NEW DRAFT",
    "description": "Chọn khách hàng rồi thêm sản phẩm. Bạn có thể lưu nháp trước.",
    "action": "Lưu đơn nháp",
    "status": "Chưa gửi",
    "fields": [
      [
        "Khách hàng *",
        "Chọn khách hàng được phép xem"
      ],
      [
        "Phụ trách",
        "Bạn"
      ],
      [
        "Sản phẩm *",
        "Bàn làm việc"
      ],
      [
        "Số lượng *",
        "2"
      ],
      [
        "Đơn giá trong đơn",
        "2.310.000 ₫ · giữ nguyên sau khi lưu"
      ],
      [
        "Ghi chú",
        "Giao trong giờ làm việc"
      ]
    ],
    "sections": [
      [
        "Tổng tạm tính",
        "4.620.000 ₫",
        "Tổng được kiểm tra lại khi lưu"
      ],
      [
        "Lưu trước, gửi sau",
        "Lưu nháp không gửi đơn hàng",
        "Khi mất mạng, hiển thị chưa đồng bộ"
      ]
    ],
    "note": "Lưu nháp và Submit là hai command khác. Không tự gửi đơn khi bấm Save."
  },
  {
    "id": "approval",
    "pack": "07-runtime",
    "surface": "Runtime",
    "section": "Công việc",
    "kind": "detail",
    "title": "Duyệt đơn ORD-1048",
    "kicker": "APPROVAL / ORD-1048",
    "description": "Linh gửi yêu cầu · cần quyết định trước 16:30",
    "action": "Duyệt đơn",
    "status": "Chờ bạn duyệt",
    "fields": [
      [
        "Giá trị đơn",
        "12.480.000 ₫"
      ],
      [
        "Khách hàng",
        "Minh An"
      ],
      [
        "Người yêu cầu",
        "Linh"
      ],
      [
        "Lý do cần duyệt",
        "Vượt ngưỡng 10.000.000 ₫"
      ]
    ],
    "sections": [
      [
        "Xem lại trước khi duyệt",
        "Nếu đơn thay đổi, bạn sẽ được yêu cầu xem lại",
        "Tránh duyệt nhầm phiên bản"
      ],
      [
        "Không đồng ý?",
        "Từ chối và ghi rõ lý do",
        "Đóng màn hình không đồng nghĩa từ chối"
      ],
      [
        "Quyết định được ghi lại",
        "Ai quyết định, khi nào và kết quả",
        "Quyền được kiểm tra khi xác nhận"
      ]
    ],
    "note": "Reject là action riêng. Timeout không tự coi là approval. Mobile giữ CTA dưới safe area."
  },
  {
    "id": "functions",
    "pack": "08-functions",
    "surface": "Builder",
    "section": "Hành động & Quy trình",
    "kind": "flow",
    "title": "Đơn gửi đi, đúng người duyệt",
    "kicker": "QUY TRÌNH / DUYỆT ĐƠN HÀNG",
    "description": "Kết nối các bước, điều kiện và xử lý khi quá hạn.",
    "action": "Mô phỏng quy trình",
    "status": "Quy trình mẫu · chưa chạy thật",
    "steps": [
      [
        "Trigger",
        "OrderSubmitted · typed event"
      ],
      [
        "Điều kiện",
        "total > approval_threshold"
      ],
      [
        "Command",
        "CreateApprovalTask(order)"
      ],
      [
        "Enqueue",
        "Email notification · consent required"
      ],
      [
        "Wait",
        "Approve / Reject / Deadline"
      ],
      [
        "Transition",
        "Checked command + receipt"
      ]
    ],
    "inspector": [
      [
        "Retry",
        "Transient · bounded backoff"
      ],
      [
        "Timeout",
        "24h · branch riêng"
      ],
      [
        "Failure",
        "Quarantine / operator"
      ],
      [
        "Collection",
        "Bounded · no recursion"
      ],
      [
        "Preview",
        "Zero external delivery"
      ]
    ],
    "note": "Core không I/O. Internal at-least-once + idempotency; external exactly-once không được hứa."
  },
  {
    "id": "run-detail",
    "pack": "08-functions",
    "surface": "Runtime",
    "section": "Tự động hóa",
    "kind": "timeline",
    "title": "Cần xác minh trạng thái gửi",
    "kicker": "RUN / RUN-019 · PHIÊN BẢN 1.2",
    "description": "Tin nhắn có thể đã gửi. Kiểm tra kết quả trước khi gửi lại.",
    "action": "Yêu cầu đối soát",
    "status": "Chưa rõ kết quả",
    "steps": [
      [
        "09:12:01",
        "Đơn hàng đã được gửi để duyệt"
      ],
      [
        "09:12:02",
        "Thông báo được đưa vào hàng chờ"
      ],
      [
        "09:12:03",
        "Chưa nhận được xác nhận từ dịch vụ gửi"
      ],
      [
        "Hiện tại",
        "Chưa rõ kết quả · không tự gửi lại"
      ],
      [
        "Tiếp theo",
        "Xác minh trước khi thử lại"
      ]
    ],
    "inspector": [
      [
        "Quy trình",
        "Phiên bản 1.2"
      ],
      [
        "Lần gửi",
        "Lần 1"
      ],
      [
        "Mã đối soát",
        "msg-204"
      ],
      [
        "Dừng gửi",
        "Không thu hồi tin đã gửi"
      ],
      [
        "Nguồn minh họa",
        "Dịch vụ gửi mô phỏng"
      ]
    ],
    "note": "Validation/permanent error không retry vô hạn. Manual retry có quyền/audit và receipt."
  },
  {
    "id": "tenant-ops",
    "pack": "05-hosted-isolation",
    "surface": "Platform",
    "section": "Tenant & Lưu trữ",
    "kind": "ops",
    "title": "Cô lập có thể kiểm chứng",
    "kicker": "HOSTED / ISOLATION SPIKE",
    "description": "Định nghĩa chung, dữ liệu và binding riêng từng tenant.",
    "action": "Mở báo cáo conformance",
    "status": "Thiết kế spike · chưa provision",
    "sections": [
      [
        "Tenant A · Blueprint",
        "Database A / fixed engine layout",
        "Runtime role: no DDL, no BYPASSRLS"
      ],
      [
        "Tenant B · Northstar",
        "Database B / same AppSpec release",
        "Field/policy ở A không thay B"
      ],
      [
        "Resource budgets",
        "Bounded pools · fair scheduling",
        "DB riêng trong cùng cluster vẫn chia CPU / I/O"
      ],
      [
        "Negative evidence",
        "API · refs · subscriptions · cache · blobs",
        "Không quảng cáo hard/physical isolation"
      ]
    ],
    "inspector": [
      [
        "Quyền",
        "Platform operator"
      ],
      [
        "Provisioning",
        "Vai trò riêng runtime"
      ],
      [
        "Adapter",
        "Đề xuất; ADR sau đo"
      ],
      [
        "Backup",
        "Scoped; không hứa tenant PITR"
      ],
      [
        "Secrets",
        "Chỉ handle, không portable"
      ]
    ],
    "note": "Chỉ operator thấy màn này. Tenant ID do host xác thực; không dựa request body để chọn credentials."
  },
  {
    "id": "connectors",
    "pack": "09-connectors",
    "surface": "Builder",
    "section": "Kết nối",
    "kind": "connectors",
    "title": "Kết nối với công cụ bạn dùng",
    "kicker": "TÀI KHOẢN / KẾT NỐI",
    "description": "Chọn tài khoản, quyền gửi và mẫu nội dung.",
    "action": "Cấu hình Email",
    "status": "Chưa có tài khoản được kết nối",
    "sections": [
      [
        "Email",
        "Send / verified webhook / receipts",
        "Test dùng fake provider đến khi có consent"
      ],
      [
        "Webhook",
        "Signature + dedup + tenant correlation",
        "Payload không tự cấp tenant authority"
      ],
      [
        "Zalo",
        "Adapter spike cần permissions",
        "Không coi mọi số điện thoại là recipient hợp lệ"
      ],
      [
        "Field mapping",
        "Order.customer → allowed recipient",
        "Bind secret handle; không ghi token vào AppSpec"
      ]
    ],
    "inspector": [
      [
        "Account",
        "Chưa kết nối"
      ],
      [
        "Consent",
        "Chưa cấp"
      ],
      [
        "Test delivery",
        "Disabled"
      ],
      [
        "Egress",
        "Allowlisted host binding"
      ],
      [
        "Result",
        "Receipt / Unknown / permanent error"
      ]
    ],
    "note": "Preview không gửi thật. Provider capabilities phải phân biệt supported, simulated và unsupported."
  },
  {
    "id": "inbox",
    "pack": "10-messaging",
    "surface": "Runtime",
    "section": "Hộp thư",
    "kind": "chat",
    "title": "Minh An",
    "kicker": "INBOX / MINH AN",
    "description": "Trao đổi về ORD-1048 · Kênh Email",
    "action": "Gửi tin nhắn",
    "status": "Nội dung mẫu",
    "sections": [
      [
        "Minh An",
        "Cho mình xác nhận lịch giao ORD-1048 nhé",
        "Tin nhắn mẫu · 09:18"
      ],
      [
        "Linh",
        "Bên mình sẽ cập nhật sau khi đơn được duyệt",
        "Bản nháp · chưa gửi"
      ],
      [
        "Trạng thái gửi",
        "Chưa gửi",
        "Thông tin xác nhận chỉ được xem"
      ]
    ],
    "inspector": [
      [
        "Đơn liên quan",
        "ORD-1048"
      ],
      [
        "Kênh",
        "Email"
      ],
      [
        "Người tham gia",
        "Minh An · Linh"
      ],
      [
        "Thông tin gửi",
        "Chỉ đọc"
      ],
      [
        "Kênh khác",
        "Chỉ hiện khi đã kết nối"
      ]
    ],
    "note": "Transport-managed collections bảo vệ invariant; full chat/federation/E2EE không là MVP dependency."
  },
  {
    "id": "bot-scope",
    "pack": "11-scoped-bot",
    "surface": "Builder",
    "section": "Kết nối",
    "kind": "policy",
    "title": "Trợ lý được làm những gì?",
    "kicker": "SERVICE ACTOR / SALES ASSISTANT",
    "description": "Chọn dữ liệu được đọc và hành động cần người xác nhận.",
    "action": "Lưu scope của bot",
    "status": "Chưa kết nối mô hình",
    "fields": [
      [
        "Projection được đọc",
        "Products.public · Orders.owned"
      ],
      [
        "Command được gọi",
        "CreateOrderDraft"
      ],
      [
        "Command bị cấm",
        "MarkPaid · UpdatePolicy · credentials"
      ],
      [
        "Rủi ro cao",
        "Human approval theo app policy"
      ]
    ],
    "sections": [
      [
        "Identity khách hàng",
        "Liên kết phải được xác thực",
        "Display name không là identity"
      ],
      [
        "Retrieved content",
        "Untrusted input",
        "Không nâng quyền tool từ nội dung message"
      ],
      [
        "Budgets và consent",
        "Tenant / model / index scope",
        "Current permissions áp trên cache và search"
      ]
    ],
    "note": "Bot không tự sửa inventory hoặc policy. Không dùng toàn VOT monorepo làm dependency."
  },
  {
    "id": "offline",
    "pack": "12-portability",
    "surface": "Runtime",
    "section": "Đồng bộ",
    "kind": "review",
    "title": "Nháp của bạn vẫn được giữ",
    "kicker": "ĐỒNG BỘ / NGOẠI TUYẾN",
    "description": "Bạn đang mất kết nối. Thay đổi chưa được xác nhận trên máy chủ.",
    "action": "Xem 2 thay đổi chờ đồng bộ",
    "status": "Ngoại tuyến · 2 thay đổi chờ đồng bộ",
    "sections": [
      [
        "Đơn nháp chưa đồng bộ",
        "ORD-local-02 · đang lưu trên thiết bị",
        "Chưa gửi đơn hàng"
      ],
      [
        "Thay đổi cần giải quyết",
        "Đơn đã được chỉnh ở nơi khác",
        "Xem hai bản, chọn thay đổi muốn giữ"
      ],
      [
        "Xuất mẫu ứng dụng",
        "Mặc định không kèm dữ liệu riêng",
        "Tính năng sẽ bật khi kiểm định dạng hoàn tất"
      ],
      [
        "Khôi phục",
        "Kiểm tra bản sao và phạm vi",
        "Không tự gửi lại thông báo cũ"
      ]
    ],
    "inspector": [
      [
        "Kết nối",
        "Ngoại tuyến"
      ],
      [
        "Nháp đã giữ",
        "2 thay đổi"
      ],
      [
        "Gửi đơn hàng",
        "Chưa xác nhận"
      ],
      [
        "Tiếp theo",
        "Đăng nhập và kiểm lại khi có mạng"
      ]
    ],
    "note": "Offline/native không được quảng cáo hoàn chỉnh trước conformance và host seam evidence."
  },
  {
    "id": "settings",
    "pack": "01-foundation",
    "surface": "Workspace",
    "section": "Cài đặt",
    "kind": "form",
    "title": "Vừa với cách bạn làm việc",
    "kicker": "TÀI KHOẢN / TRẢI NGHIỆM",
    "description": "Chọn giao diện, ngôn ngữ và mức chuyển động.",
    "action": "Lưu tùy chọn",
    "status": "Tùy chọn trên thiết bị",
    "fields": [
      [
        "Giao diện",
        "Theo hệ thống / Sáng / Tối"
      ],
      [
        "Ngôn ngữ",
        "Tiếng Việt"
      ],
      [
        "Chuyển động",
        "Theo hệ thống / Giảm chuyển động"
      ],
      [
        "Mật độ bảng",
        "Compact desktop / touch-safe mobile"
      ]
    ],
    "sections": [
      [
        "Quyền tài khoản",
        "Do host quản lý",
        "Không tự cấp admin từ AppSpec"
      ],
      [
        "Phiên làm việc",
        "Session expired giữ unsent draft",
        "Reauthenticate rồi kiểm quyền mới"
      ]
    ],
    "note": "Dynamic color tùy chọn về sau; semantic error/warning/success phải giữ nghĩa và contrast."
  },
  {
    "id": "states",
    "pack": "01-foundation",
    "surface": "Workspace",
    "section": "Trạng thái",
    "kind": "states",
    "title": "Mọi điểm dừng đều có lối ra",
    "kicker": "GLOBAL STATES / ERROR & RECOVERY",
    "description": "Lỗi không xóa nháp. Trạng thái không giả thành thành công.",
    "action": "Thử lại trong fixture",
    "status": "Design-only state catalogue",
    "sections": [
      [
        "Không có quyền",
        "Không lộ tên record ẩn",
        "Quay lại view có quyền · không request broad access"
      ],
      [
        "Chưa có dữ liệu",
        "Tạo bản ghi đầu tiên",
        "Một CTA rõ, nội dung giải thích ngắn"
      ],
      [
        "Mất kết nối",
        "Giữ draft · chưa confirmed",
        "Tải lại current revision / retry idempotent"
      ],
      [
        "Phiên hết hạn",
        "Che dữ liệu nhạy cảm",
        "Giữ unsent draft, recheck auth sau sign-in"
      ]
    ],
    "note": "Loading / empty / invalid / denied / saving / saved / conflict / offline / unknown outcome là các trạng thái riêng."
  },
  {
    "id": "components",
    "pack": "01-foundation",
    "surface": "Workspace",
    "section": "Thành phần",
    "kind": "components",
    "title": "Một ngôn ngữ, hai nền tảng",
    "kicker": "M3 EXPRESSIVE / BLUE LEDGER",
    "description": "Tonal grouping, hierarchy rõ, shape có mục đích. Không thêm viền cho mọi card.",
    "action": "Primary action",
    "status": "Canonical token reference",
    "sections": [
      [
        "Actions & shapes",
        "Filled / tonal / text / connected group",
        "48dp touch · pressed shape morph · no hover-only control"
      ],
      [
        "Typed components",
        "RefChip / TypeBadge / FormulaField / Receipt",
        "Data density riêng, không dùng color làm nghĩa duy nhất"
      ],
      [
        "Containment",
        "Surface-low / surface / tonal inspector",
        "0 elevation mặc định · outline chỉ grid/focus/error"
      ],
      [
        "Feedback",
        "Focus 3px · reduced motion · live region",
        "Loading không làm đổi selected identity"
      ]
    ],
    "note": "Leptos adapters + CMP MaterialTheme đọc cùng tokens.json; chưa pin API Expressive experimental như stable."
  }
];
