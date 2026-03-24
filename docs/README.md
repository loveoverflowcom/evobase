# EvoBase Docs Map

Đây là bản đồ đọc nhanh cho người muốn hiểu dự án theo đúng luồng thực tế của codebase.

## Nên đọc theo thứ tự nào

1. `QUICK_START.md` để biết cách chạy dự án và gọi API.
2. `ARCHITECTURE.md` để nắm khung kiến trúc hiện tại.
3. `LESSON_ARCHITECTURE.md` để hiểu vì sao dự án được tách như vậy.
4. `LESSON_ASYNC.md` để hiểu các điểm bất đồng bộ trong backend và Flutter.
5. `diagrams/README.md` để chọn sơ đồ phù hợp khi cần tra cứu nhanh.

## Tài liệu nền

- `ARCHITECTURE.md`: mô tả kiến trúc tổng quan của backend.
- `QUICK_START.md`: chạy thử server và gọi API bằng `curl`.
- `REFACTORING_NOTES.md`: các ghi chú chuyển đổi và cải tổ kiến trúc.

## Bài giảng

- `LESSON_ASYNC.md`: async trong `tokio`, `axum`, `sqlx`, SSE, và luồng async ở Flutter.
- `LESSON_ARCHITECTURE.md`: kiến trúc theo lớp, boundary giữa crate, DTO, gateway, storage, messaging, và workbench.

## Sơ đồ nên xem cùng bài giảng

- `diagrams/01_component_overview.puml`
- `diagrams/02_auth_rls_sequence.puml`
- `diagrams/03_messaging_sse_flow.puml`
- `diagrams/04_sse_connection_state.puml`
- `diagrams/05_bootstrap_sequence.puml`
- `diagrams/06_runtime_deployment.puml`
- `diagrams/07_core_contracts_class.puml`
- `diagrams/08_auth_token_lifecycle.puml`
- `diagrams/09_rest_gateway_flow.puml`
- `diagrams/10_data_model_erd.puml`

## Cách dùng tài liệu này

Nếu bạn đang debug một request, hãy bắt đầu từ:

1. `apps/evobase-server/src/main.rs`
2. `crates/evobase-gateway/src/routes.rs`
3. `crates/evobase-gateway/src/handlers/*.rs`
4. `crates/evobase-core/src/*.rs`
5. `crates/evobase-db/src/postgres.rs`
6. `crates/evobase-auth/src/service.rs`
7. `crates/evobase-messaging/src/hub.rs`

Nếu bạn đang đọc phía Flutter, hãy đi theo:

1. `apps/evobase_workbench/lib/main.dart`
2. `apps/evobase_workbench/lib/client/core/api_client.dart`
3. `apps/evobase_workbench/lib/client/auth/auth_manager.dart`
4. `apps/evobase_workbench/lib/app/blocs/auth/auth_bloc.dart`
5. `apps/evobase_workbench/lib/app/pages/*.dart`
