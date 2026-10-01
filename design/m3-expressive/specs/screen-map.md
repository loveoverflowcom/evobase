# Screen map and flows

Target design only. 27 authored screens; each has desktop and compact static frames. Compact Builder is read-only summary; compact Runtime is intended native CMP target, not native evidence.

## Primary journey

Workspace → Create draft → Typed grid / fields / import resolve → Relations / formula → Policy / command → Preview → Release review / evolution → Authorized Runtime → Form / approval → Function run / connector / inbox.

Provider sign-in is deferred to identity ADR; denied and expired-session states are covered. General1:1/N:M and offline/provider capabilities remain gated.

## 01-foundation · Foundation / M3 Expressive shells

Thống nhất AppSpec/host boundaries, source tokens, adaptive shells và trạng thái phiên. Provider sign-in journey chờ identity ADR.

### workspace · Một nơi cho công việc
- Surface: Workspace; primary intent: Tạo ứng dụng
- Flow: Ứng dụng; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Định nghĩa ứng dụng, dữ liệu vận hành và kết nối tài khoản có vòng đời riêng.
- Editable frame: frames/workspace-desktop.svg; compact: frames/workspace-mobile.svg

### new-app · Bắt đầu với một bảng
- Surface: Workspace; primary intent: Tạo bản nháp
- Flow: Ứng dụng; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Tạo draft không tạo tenant database và không gửi thông báo.
- Editable frame: frames/new-app-desktop.svg; compact: frames/new-app-mobile.svg

### settings · Vừa với cách bạn làm việc
- Surface: Workspace; primary intent: Lưu tùy chọn
- Flow: Cài đặt; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Dynamic color tùy chọn về sau; semantic error/warning/success phải giữ nghĩa và contrast.
- Editable frame: frames/settings-desktop.svg; compact: frames/settings-mobile.svg

### states · Mọi điểm dừng đều có lối ra
- Surface: Workspace; primary intent: Thử lại trong fixture
- Flow: Trạng thái; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Loading / empty / invalid / denied / saving / saved / conflict / offline / unknown outcome là các trạng thái riêng.
- Editable frame: frames/states-desktop.svg; compact: frames/states-mobile.svg

### components · Một ngôn ngữ, hai nền tảng
- Surface: Workspace; primary intent: Primary action
- Flow: Thành phần; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Leptos adapters + CMP MaterialTheme đọc cùng tokens.json; chưa pin API Expressive experimental như stable.
- Editable frame: frames/components-desktop.svg; compact: frames/components-mobile.svg

- Acceptance: Token source→generated output→mounted consumer trace; không fork palette
- Acceptance: Workspace/Builder/Runtime navigation separate; compact Builder read-only summary
- Acceptance: Light/dark, focus, VI/EN, reduced motion, zoom/text-scale evidence
- Acceptance: Denied/session-expired giữ unsent draft; không coi settings là auth journey hoàn chỉnh

## 02-data-grid · Typed data / grid / import

Kernel semantics trước grid. Bảng có kiểu và Ref hiển thị nhãn, lưu identity; import cần resolve.

### builder-data · Đơn hàng
- Surface: Builder; primary intent: Thêm bản ghi mẫu
- Flow: Bảng dữ liệu; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Ô hiển thị nhãn, lưu record identity. Không mở quyền đọc chỉ vì có quan hệ.
- Editable frame: frames/builder-data-desktop.svg; compact: frames/builder-data-mobile.svg

### field-inspector · Một cột, một ý nghĩa
- Surface: Builder; primary intent: Lưu cấu hình cột
- Flow: Bảng dữ liệu; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Chuyển kiểu có dữ liệu cần impact preview; không đổi thầm sau một lần click.
- Editable frame: frames/field-inspector-desktop.svg; compact: frames/field-inspector-mobile.svg

### import-resolve · Khớp đúng trước khi nhập
- Surface: Builder; primary intent: Tiếp tục 28 dòng hợp lệ
- Flow: Bảng dữ liệu; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Paste và IME phải giữ text đến khi commit; lỗi theo ô/dòng, undo có ranh giới rõ.
- Editable frame: frames/import-resolve-desktop.svg; compact: frames/import-resolve-mobile.svg

- Acceptance: Golden codec/ID/type/Ref round-trip vectors độc lập GUI/SQL
- Acceptance: Ref wrong table/missing/ambiguous/cross-scope bị chặn; reverse derived
- Acceptance: Grid keyboard/IME/paste/sort/rename không selection drift; currency unit rõ
- Acceptance: Cần hiện receipt/revision và lỗi theo ô, không giả save success

## 03-structure-formulas · Relations / formulas

Quan hệ và biểu thức thuần cùng stable-ID binding; captured price khác live lookup.

### relations · Các bảng kết nối ra sao?
- Surface: Builder; primary intent: Thêm quan hệ
- Flow: Cấu trúc & Quan hệ; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Không duy trì hai chiều canonical. Relation không tự sinh policy.
- Editable frame: frames/relations-desktop.svg; compact: frames/relations-mobile.svg

### formula · Tính tổng từ các dòng đơn
- Surface: Builder; primary intent: Áp dụng công thức
- Flow: Cấu trúc & Quan hệ; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Syntax minh họa; parser/subset phải được chốt bằng kernel ADR và golden tests.
- Editable frame: frames/formula-desktop.svg; compact: frames/formula-mobile.svg

- Acceptance: Baseline N:1 + restrict có conformance; 1:1/N:M disabled đến khi test/gate đạt
- Acceptance: Không hai canonical directions; edges/indexes projection có consistency proof
- Acceptance: Formula type/resource bounds; computed output policy, không partial visible-child total giả
- Acceptance: Rename/reorder/label đổi vẫn giữ nghĩa; targeted differential/golden tests

## 04-policies-commands · Authority / policy / commands

Câu cấu trúc dễ hiểu với current authority, guards và typed receipt.

### policy · Ai được xem đơn hàng?
- Surface: Builder; primary intent: Lưu quy tắc
- Flow: Quy tắc & Quyền; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Runtime kiểm tenant boundary + current grants + app policy + restrictions; missing context phải deny.
- Editable frame: frames/policy-desktop.svg; compact: frames/policy-mobile.svg

### command · Gửi đơn hàng đúng điều kiện
- Surface: Builder; primary intent: Kiểm tra command
- Flow: Quy tắc & Quyền; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Không giữ transaction qua email, mạng hoặc người duyệt. Chưa có provider thật trong thiết kế. Committed retry trả original receipt dưới current grants dù live revision đã tiến.
- Editable frame: frames/command-desktop.svg; compact: frames/command-mobile.svg

- Acceptance: Host boundary AND current grants AND app policy AND restrictions; missing context deny
- Acceptance: Relation/picker/join/export/aggregate không ngầm mở quyền
- Acceptance: Old+new state và field mutability kiểm tại command commit
- Acceptance: Request key binds immutable intent; exact retry returns original receipt under current grants; no effect duplication

## 05-hosted-isolation · Isolated fixed-store spike

Đo fixed-layout engine store và database-per-tenant, giữ lựa chọn ADR chưa settled production.

### tenant-ops · Cô lập có thể kiểm chứng
- Surface: Platform; primary intent: Mở báo cáo conformance
- Flow: Tenant & Lưu trữ; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Chỉ operator thấy màn này. Tenant ID do host xác thực; không dựa request body để chọn credentials.
- Editable frame: frames/tenant-ops-desktop.svg; compact: frames/tenant-ops-mobile.svg

- Acceptance: Hai tenant DB + provisioner/runtime role separation; runtime no DDL/BYPASSRLS
- Acceptance: Thêm logical table/optional field/Ref/publish không business SQL DDL; actual catalog evidence
- Acceptance: Records/edges/index/unique/revision/outbox atomic; concurrency/restart/failure vectors
- Acceptance: Tenant negative tests gồm API/Ref/cache/subscription/blob/log/bot; resource/latency measurements + support envelope ADR

## 06-release · Preview / release / evolution

Preview zero delivery, immutable checked release, impact/backfill/recovery trước activation.

### view-builder · Chỉ hiện điều người dùng cần
- Surface: Builder; primary intent: Mở preview Runtime
- Flow: Chế độ xem; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Ẩn ở UI không phải bảo mật. Các field system receipt/provider không editable qua generic grid.
- Editable frame: frames/view-builder-desktop.svg; compact: frames/view-builder-mobile.svg

### preview · Thử trước, không gửi thật
- Surface: Builder; primary intent: Chạy 6 tình huống mẫu
- Flow: Xuất bản; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Thiết kế fixture không đồng nghĩa tests đã chạy. Kết quả runtime phải lấy từ test evidence thật.
- Editable frame: frames/preview-desktop.svg; compact: frames/preview-mobile.svg

### publish · Sẵn sàng đưa vào sử dụng?
- Surface: Builder; primary intent: Yêu cầu xuất bản
- Flow: Xuất bản; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Template update không tự nâng mọi tenant. Từng app instance có release được pin riêng.
- Editable frame: frames/publish-desktop.svg; compact: frames/publish-mobile.svg

### migration · Kiểm tra trước khi đổi kiểu
- Surface: Builder; primary intent: Tạo kế hoạch migration
- Flow: Xuất bản; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Storage adapter/codec/format extension phải có ADR; không coi đổi đuôi file là migration.
- Editable frame: frames/migration-desktop.svg; compact: frames/migration-mobile.svg

- Acceptance: Draft edits never activate; preview no live writes/no external dispatch
- Acceptance: AppSpec/model/data/binding/policy revisions separate; app instance pins release
- Acceptance: Type/required/backfill impact preview + atomic cutover/recovery proof; no-DDL still migration
- Acceptance: Live runs keep pinned version; rollback/restore does not unsend/replay effects or revive revoked grants

## 07-runtime · Runtime web / CMP

List-detail/form/approval có phép; mobile ưu tiên việc thực cần làm, không modeling trên phone.

### runtime-home · Hôm nay cần làm gì?
- Surface: Runtime; primary intent: Tạo đơn hàng
- Flow: Đơn hàng; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Tenant được xác thực từ host context. Selector trên URL không tự cấp quyền.
- Editable frame: frames/runtime-home-desktop.svg; compact: frames/runtime-home-mobile.svg

### runtime-detail · ORD-1048
- Surface: Runtime; primary intent: Mở yêu cầu duyệt
- Flow: Đơn hàng; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Đổi tên khách hàng không đổi identity; action luôn recheck current grants tại command commit.
- Editable frame: frames/runtime-detail-desktop.svg; compact: frames/runtime-detail-mobile.svg

### runtime-form · Tạo đơn hàng
- Surface: Runtime; primary intent: Lưu đơn nháp
- Flow: Đơn hàng; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Lưu nháp và Submit là hai command khác. Không tự gửi đơn khi bấm Save.
- Editable frame: frames/runtime-form-desktop.svg; compact: frames/runtime-form-mobile.svg

### approval · Duyệt đơn ORD-1048
- Surface: Runtime; primary intent: Duyệt đơn
- Flow: Công việc; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Reject là action riêng. Timeout không tự coi là approval. Mobile giữ CTA dưới safe area.
- Editable frame: frames/approval-desktop.svg; compact: frames/approval-mobile.svg

- Acceptance: Only authorized projection/actions render; trusted runtime still enforces
- Acceptance: Save draft vs Submit distinct; receipt/revision/conflict/no-false-sent semantics
- Acceptance: Approval old+new/current grants and reviewed revision; reject explicit; Close/Back not decision
- Acceptance: Visible navigation-up, Android Back/iOS edge/browser history/IME/safe-area + native CMP evidence separate from mobile Web

## 08-functions · Durable declarative functions

Finite typed steps, durable waits và trạng thái gửi chưa rõ kết quả.

### functions · Đơn gửi đi, đúng người duyệt
- Surface: Builder; primary intent: Mô phỏng quy trình
- Flow: Hành động & Quy trình; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Core không I/O. Internal at-least-once + idempotency; external exactly-once không được hứa.
- Editable frame: frames/functions-desktop.svg; compact: frames/functions-mobile.svg

### run-detail · Cần xác minh trạng thái gửi
- Surface: Runtime; primary intent: Yêu cầu đối soát
- Flow: Tự động hóa; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Validation/permanent error không retry vô hạn. Manual retry có quyền/audit và receipt.
- Editable frame: frames/run-detail-desktop.svg; compact: frames/run-detail-mobile.svg

- Acceptance: Pinned plan/run IDs/input snapshot/checkpoint/lease fencing and per-tenant budgets
- Acceptance: Transaction + event/outbox atomic; no network wait while holding DB transaction
- Acceptance: Lost ACK/duplicate/timer/restart/late-worker tests; bounded transient retries/permanent quarantine
- Acceptance: OutcomeUnknown + reconcile/correlation; current consent at dispatch; no exactly-once external promise

## 09-connectors · Email / webhook capability bindings

Contract capabilities thực, host account/consent/secret handles riêng definition.

### connectors · Kết nối với công cụ bạn dùng
- Surface: Builder; primary intent: Cấu hình Email
- Flow: Kết nối; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Preview không gửi thật. Provider capabilities phải phân biệt supported, simulated và unsupported.
- Editable frame: frames/connectors-desktop.svg; compact: frames/connectors-mobile.svg

- Acceptance: Email fake provider labeled simulated; real-target evidence only after authorized test
- Acceptance: Verified webhook/dedup/tenant correlation; payload cannot grant actor or tenant
- Acceptance: Secret handles/accounts chỉ ở host bindings; portable definition chỉ abstract connector contracts, không host-secret binding; egress/consent recheck
- Acceptance: Zalo separate adapter spike/permissions; unsupported disabled; phone number not automatic valid recipient

## 10-messaging · Business inbox / messaging

Hộp thư có context với protected transport metadata.

### inbox · Minh An
- Surface: Runtime; primary intent: Gửi tin nhắn
- Flow: Hộp thư; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Transport-managed collections bảo vệ invariant; full chat/federation/E2EE không là MVP dependency.
- Editable frame: frames/inbox-desktop.svg; compact: frames/inbox-mobile.svg

- Acceptance: Conversation/message/participant/delivery typed managed projections
- Acceptance: Provider IDs/receipts read-only; generic edit cannot fake delivered/received
- Acceptance: Scoped inbox/attachment/search + current output policy tests
- Acceptance: Draft message separate dispatch; unknown delivery clearly shown; no federation/E2EE requirement for MVP

## 11-scoped-bot · Scoped service-actor bot

Allowed projections/commands, budgets, consent và human approval.

### bot-scope · Trợ lý được làm những gì?
- Surface: Builder; primary intent: Lưu scope của bot
- Flow: Kết nối; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Bot không tự sửa inventory hoặc policy. Không dùng toàn VOT monorepo làm dependency.
- Editable frame: frames/bot-scope-desktop.svg; compact: frames/bot-scope-mobile.svg

- Acceptance: Verified identity linking; display name not authority
- Acceptance: Retrieved content untrusted; no permission elevation/tool instruction from messages
- Acceptance: Bot cannot MarkPaid/change inventory/policy/credentials without app policy and authorized command
- Acceptance: Index/search/cache/history tenant+current-grant scope, prompt injection/denied/no-model-binding tests

## 12-portability · Local profile / recovery

ADR support envelope cho portable execution, local unsent draft và restore.

### offline · Nháp của bạn vẫn được giữ
- Surface: Runtime; primary intent: Xem 2 thay đổi chờ đồng bộ
- Flow: Đồng bộ; inspect/cancel/back preserve draft; no live effects in fixture
- Required states: loading / empty / denied / invalid / pending / confirmed / conflict / unsupported as applicable
- Developer invariant: Offline/native không được quảng cáo hoàn chỉnh trước conformance và host seam evidence.
- Editable frame: frames/offline-desktop.svg; compact: frames/offline-mobile.svg

- Acceptance: Bounded codec/version/provenance and stable identity compatibility, .evobase extension ADR
- Acceptance: Local snapshot does not grant hosted authority; no falsely confirmed offline send
- Acceptance: Conflict needs review; reauth/revalidation/current permissions at sync
- Acceptance: Restore/export/receipts test: no revoked-grant resurrection or arbitrary external replay

