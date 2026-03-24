# Bài Giảng Async Trong EvoBase

Mục tiêu của bài này là giúp bạn hiểu async ở mức "đọc code là đoán được luồng chạy", không chỉ biết định nghĩa chung chung.

## 1. Async Ở Đây Là Gì

Trong EvoBase, async không chỉ là "dùng `async/await`".
Async xuất hiện ở 4 lớp:

- Server nhận request HTTP và chờ I/O.
- Server chờ PostgreSQL qua `sqlx`.
- Server giữ kết nối SSE dài hạn qua `axum::response::sse`.
- Flutter app gọi API, lưu token, và đọc stream sự kiện.

Nói ngắn gọn:

- `async` trong dự án này chủ yếu là để không block thread trong lúc chờ I/O.
- `await` là điểm nhường quyền cho runtime.
- `Future` và `Stream` là hai hình thái async chính ở client.

## 2. Mental Model

Hãy tách async thành 3 câu hỏi:

1. Chỗ nào đang chờ mạng, DB, file, hoặc stream?
2. Trong lúc chờ, code nào vẫn phải giữ state?
3. Khi dữ liệu quay về, ai là người tiếp tục công việc?

Trong EvoBase, câu trả lời thường là:

- `tokio` giữ runtime.
- `axum` giữ request lifecycle.
- `sqlx` giữ DB future.
- `AuthManager` và `AuthBloc` giữ trạng thái auth ở client.
- `InMemoryMessagingHub` giữ connection và queued events.

## 3. Async Trên Server

### 3.1 Điểm khởi động

File `apps/evobase-server/src/main.rs` là entry point bất đồng bộ của hệ thống.

Ở đây:

- `#[tokio::main]` dựng runtime.
- `AppConfig::from_env()` đọc config.
- `PostgresStorage::connect(...).await` mở kết nối DB.
- `axum::serve(...).await` chạy HTTP server.
- `with_graceful_shutdown(...)` đảm bảo tắt server sạch.

Đây là kiểu async phổ biến nhất: khởi động một chuỗi I/O rồi giữ process sống bằng server loop.

### 3.2 Handler async

Trong `crates/evobase-gateway/src/handlers/*.rs`, handler là async vì mỗi request đều có thể chờ:

- parse body
- verify token
- gọi service
- chờ DB
- trả JSON

Ví dụ luồng `/auth/login`:

1. `handlers/auth.rs` nhận `Json<LoginRequest>`.
2. Convert sang domain type.
3. Gọi `state.auth_service.login(...).await`.
4. `JwtAuthService` chờ `storage.find_user_by_username(...).await`.
5. Tạo token bundle.
6. Convert sang `AuthResponseDto`.
7. Wrap bằng `ApiResponse`.

File liên quan:

- `crates/evobase-gateway/src/handlers/auth.rs`
- `crates/evobase-auth/src/service.rs`
- `crates/evobase-core/src/auth.rs`
- `crates/evobase-protocol/src/auth.rs`

### 3.3 Vì sao core dùng `async_trait`

`crates/evobase-core/src/auth.rs`, `storage.rs`, `messaging.rs` khai báo trait cho service.

Rust trait bình thường không hỗ trợ `async fn` trực tiếp theo cách tiện cho kiến trúc này, nên repo dùng `async-trait`.

Lợi ích:

- Core định nghĩa contract.
- Implementation ở crate khác có thể thay thế.
- Gateway chỉ nhìn trait, không phụ thuộc concrete type.

Đây là async gắn với kiến trúc, không chỉ là cú pháp.

### 3.4 DB async với `sqlx`

`crates/evobase-db/src/postgres.rs` dùng `PgPool`, `Transaction`, `QueryBuilder` và các truy vấn async.

Điểm đáng chú ý:

- Mỗi truy vấn là một future.
- `await` xuất hiện ở `connect`, `begin`, `query_as`, `commit`.
- `apply_auth_context` set `request.jwt.claim.sub` và `app.current_user_id` để PostgreSQL RLS biết user hiện tại là ai.

Đây là một pattern rất "real world":

- Authentication được xác thực ở tầng app.
- Authorization tiếp tục được enforced ở tầng DB.

### 3.5 SSE và stream

`crates/evobase-gateway/src/handlers/messaging.rs` là ví dụ async đẹp nhất của repo.

Luồng `/events`:

1. Lấy notification token từ header hoặc query.
2. Verify token qua `AuthService`.
3. `messaging_service.connect(user_id)` trả về `MessagingConnection`.
4. Tạo `stream! { ... }` bằng `async-stream`.
5. `yield` sự kiện `system.ready`.
6. Đọc tiếp từ `receiver.recv().await`.
7. Giữ kết nối bằng `KeepAlive`.
8. Khi stream đóng, `ConnectionGuard::drop()` gọi `disconnect`.

Điểm hay:

- `Stream` phù hợp hơn `Future` cho data đến nhiều lần.
- `Drop` được dùng như cơ chế dọn connection.
- `async-stream` làm code SSE dễ đọc hơn so với tự dựng manual stream.

## 4. Async Trên Client Flutter

### 4.1 `Future` ở tầng API

`apps/evobase_workbench/lib/client/core/api_client.dart` dùng `dio` để gọi HTTP.

Ở đây async được đóng gói thành:

- `get`
- `post`
- `patch`
- `delete`

Mỗi method trả về `Future<ApiResponse<T>>`.

Điều đáng học từ file này:

- interceptor chèn token trước request.
- lỗi từ network được map sang exception có nghĩa nghiệp vụ.
- response envelope được parse nhất quán.

### 4.2 Auth flow là async có state

`apps/evobase_workbench/lib/client/auth/auth_manager.dart` là nơi kết hợp:

- HTTP async
- lưu token local
- auto-refresh
- đồng bộ token vào `ApiClient`

Điểm đáng chú ý:

- `_isRefreshing` ngăn refresh chồng nhau.
- `_saveAuthResponse()` dùng `Future.wait(...)` để ghi nhiều key local cùng lúc.
- `isTokenExpired()` quyết định có cần refresh không.

Đây là async có "state machine" chứ không chỉ là gọi API rồi xong.

### 4.3 `flutter_bloc` biến async thành UI state

`apps/evobase_workbench/lib/app/blocs/auth/auth_bloc.dart` là lớp điều phối:

- event từ UI
- gọi `AuthManager`
- emit loading/authenticated/error

`LoginPage` và `HomePage` chỉ việc phản ứng theo state.

Mẫu tư duy:

- UI không gọi API trực tiếp.
- UI bắn event.
- Bloc quản lý async lifecycle.

### 4.4 SSE ở Flutter

`apps/evobase_workbench/lib/lab/event_monitor/sse_listener.dart` cho thấy cách client đọc `ResponseType.stream`.

Async ở đây khác với HTTP thường:

- kết nối mở lâu
- parse chunk liên tục
- có thể nhận event từng phần
- cần `close()` để dọn tài nguyên

Đây là điểm thường bị bỏ sót khi mới học async: không phải async nào cũng kết thúc sau một response.

## 5. Những Chỗ Dễ Sai

### 5.1 Giữ lock quá lâu

`InMemoryMessagingHub` dùng `RwLock`.

Nguyên tắc cần nhớ:

- lock nên giữ ngắn.
- không giữ lock qua `await`.
- việc replay queued message nên xảy ra sau khi đã lấy dữ liệu cần thiết ra khỏi state.

### 5.2 Dùng unbounded queue mà không nghĩ về backpressure

`tokio::sync::mpsc::unbounded_channel` tiện cho SSE, nhưng nếu consumer chậm thì memory có thể tăng.

Trong dự án này, nó hợp lý vì message volume kỳ vọng không quá lớn, nhưng đây là chỗ nên quan sát khi scale.

### 5.3 Đồng bộ token không nhất quán

Client có 2 loại token local:

- access/refresh/notification token
- admin token

Nếu chỉ clear một phần, state sẽ lệch.

Repo xử lý qua `TokenStorage` và `AuthManager`, nên khi thêm flow mới bạn nên đi qua 2 lớp này trước.

### 5.4 Async nhưng CPU-bound

Nếu sau này thêm tác vụ nặng như:

- encode file lớn
- transform JSON cực lớn
- compute phức tạp

thì đừng để chạy trực tiếp trong handler async lâu quá. Hãy cân nhắc tách worker hoặc `spawn_blocking`.

## 6. Cách Đọc Async Trong Repo

Khi gặp một function async, hãy tự hỏi:

1. Nó chờ cái gì?
2. Nó có giữ state nào qua `await` không?
3. Nếu fail thì lỗi được map ở đâu?
4. Ai sở hữu retry / refresh / cleanup?

Áp dụng vào repo:

- HTTP entry point: `crates/evobase-gateway/src/handlers/*.rs`
- domain async: `crates/evobase-auth/src/service.rs`
- storage async: `crates/evobase-db/src/postgres.rs`
- messaging async: `crates/evobase-messaging/src/hub.rs`
- client async: `apps/evobase_workbench/lib/client/*.dart`

## 7. Bài Tập Tự Học

1. Trace luồng `POST /auth/login` từ Flutter đến DB và quay về UI.
2. Trace luồng `GET /events` và xác định chỗ nào giữ connection sống.
3. So sánh `Future` ở Dart với `async fn` ở Rust.
4. Tìm một chỗ trong repo nơi `await` đi kèm với lock hoặc state, rồi đánh giá xem nó có an toàn không.

## 8. Tóm Tắt Một Câu

Async trong EvoBase là cơ chế để:

- giữ server không block khi chờ I/O
- giữ token/state nhất quán
- giữ SSE sống lâu
- giúp UI Flutter phản ứng đúng với lifecycle bất đồng bộ
