//! Generated single-tenant HTTP Runtime. Credentials and unresolved intents stay in memory.
use crate::{tr, type_label};
use evobase_appspec::{AppId, FieldId, FieldType, RecordId, TableId};
use evobase_protocol::appspec::{
    API_VERSION, ApiErrorDto, CommandMetadataDto, CommandRequestDto, ErrorCodeDto,
    FieldMetadataDto, ListResponseDto, MetadataResponseDto, ReceiptResponseDto, RefDto, ValueDto,
};
use leptos::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
const runtimeControllers = new Set();
export function runtimeAbortAll() { for (const controller of runtimeControllers) controller.abort(); runtimeControllers.clear(); }
export function runtimeValidateBase(base) {
  try {
    const origin = new URL(base);
    const local = ['localhost', '127.0.0.1', '[::1]'].includes(origin.hostname);
    return (origin.protocol === 'https:' || (origin.protocol === 'http:' && local)) && !origin.username && !origin.password && !origin.search && !origin.hash && (origin.pathname === '/' || origin.pathname === '');
  } catch { return false; }
}
export async function runtimeHttp(base, path, credential, method, body) {
  const origin = new URL(base);
  if (!runtimeValidateBase(base)) throw new Error('configuration');
  const controller = new AbortController();
  runtimeControllers.add(controller);
  const timeout = setTimeout(() => controller.abort(), 10000);
  try {
    const response = await fetch(new URL(path, origin), {
      method, credentials: 'omit', cache: 'no-store', signal: controller.signal,
      headers: { Authorization: `Bearer ${credential}`, ...(method === 'POST' ? { 'Content-Type': 'application/json' } : {}) },
      ...(method === 'POST' ? { body } : {})
    });
    const reader = response.body.getReader();
    const decoder = new TextDecoder('utf-8', { fatal: true });
    let bytes = 0;
    let payload = '';
    try {
      for (;;) {
        const chunk = await reader.read();
        if (chunk.done) break;
        bytes += chunk.value.byteLength;
        if (bytes > 1048576) { await reader.cancel(); throw new Error('limit'); }
        payload += decoder.decode(chunk.value, { stream: true });
      }
      payload += decoder.decode();
    } finally { reader.releaseLock(); }
    // Domain values remain an opaque string until the exact Rust decoder runs.
    return JSON.stringify({ status: response.status, payload });
  } finally { clearTimeout(timeout); runtimeControllers.delete(controller); }
}
export function runtimeRequestKey() { return `req_${crypto.randomUUID()}`; }
"#)]
extern "C" {
    #[wasm_bindgen(js_name = runtimeAbortAll)]
    fn runtime_abort_all();
    #[wasm_bindgen(js_name = runtimeValidateBase)]
    fn runtime_validate_base(base: &str) -> bool;
    #[wasm_bindgen(catch, js_name = runtimeHttp)]
    async fn runtime_http(
        base: &str,
        path: &str,
        credential: &str,
        method: &str,
        body: &str,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = runtimeRequestKey)]
    fn runtime_request_key() -> Result<String, JsValue>;
}

#[derive(Clone)]
struct Config {
    base: String,
    app: AppId,
    credential: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpReply {
    status: u16,
    payload: String,
}

#[derive(Clone)]
enum Failure {
    Configuration,
    Transport,
    Protocol,
    Api(ErrorCodeDto),
}

#[derive(Clone)]
enum Notice {
    Ready,
    Loading,
    Live,
    Sending,
    Unknown,
    NoReceipt,
    Failure(Failure),
}

#[derive(Clone)]
struct Attempt {
    config: Config,
    request: CommandRequestDto,
    body: String,
    uncertain: bool,
}

#[derive(Clone)]
enum InputIssue {
    Required,
    Type(FieldType),
}

#[derive(Clone, Copy)]
struct State {
    metadata: RwSignal<Option<MetadataResponseDto>>,
    list: RwSignal<Option<ListResponseDto>>,
    table: RwSignal<String>,
    record: RwSignal<String>,
    command: RwSignal<String>,
    params: RwSignal<BTreeMap<FieldId, String>>,
    errors: RwSignal<BTreeMap<FieldId, InputIssue>>,
    attempt: RwSignal<Option<Attempt>>,
    receipt: RwSignal<Option<ReceiptResponseDto>>,
    notice: RwSignal<Notice>,
    busy: RwSignal<bool>,
    generation: RwSignal<u64>,
}

impl State {
    fn select_record(self, record: String) {
        if self.record.get_untracked() != record {
            self.params.set(BTreeMap::new());
            self.errors.set(BTreeMap::new());
            self.command.set(String::new());
            self.record.set(record);
        }
    }
    fn show_failure(self, error: Failure) {
        if matches!(error, Failure::Api(ErrorCodeDto::Denied)) {
            self.metadata.set(None);
            self.list.set(None);
            self.receipt.set(None);
        }
        self.notice.set(Notice::Failure(error));
        self.busy.set(false);
    }
    fn invalidate(self) {
        runtime_abort_all();
        self.generation
            .update(|generation| *generation = generation.wrapping_add(1));
        self.metadata.set(None);
        self.list.set(None);
        self.receipt.set(None);
        self.busy.set(false);
        if let Some(mut attempt) = self.attempt.get_untracked() {
            attempt.uncertain = true;
            self.attempt.set(Some(attempt));
            self.notice.set(Notice::Unknown);
        } else {
            self.notice.set(Notice::Ready);
        }
    }
    fn command(self) -> Option<CommandMetadataDto> {
        let metadata = self.metadata.get()?;
        let selected = self.command.get();
        metadata
            .commands
            .iter()
            .filter(|command| {
                command.table_id.as_str() == self.table.get()
                    && command
                        .eligible_record_ids
                        .iter()
                        .any(|record| record.as_str() == self.record.get())
            })
            .find(|command| command.command_id == selected)
            .or_else(|| {
                metadata.commands.iter().find(|command| {
                    command.table_id.as_str() == self.table.get()
                        && command
                            .eligible_record_ids
                            .iter()
                            .any(|record| record.as_str() == self.record.get())
                })
            })
            .cloned()
    }
}

async fn request(config: &Config, path: &str, method: &str, body: &str) -> Result<String, Failure> {
    let raw = runtime_http(&config.base, path, &config.credential, method, body)
        .await
        .map_err(|_| Failure::Transport)?;
    let text = raw.as_string().ok_or(Failure::Protocol)?;
    // The bounded payload was wrapped by our own helper; only this envelope uses ordinary serde.
    let reply: HttpReply = serde_json::from_str(&text).map_err(|_| Failure::Protocol)?;
    if (200..300).contains(&reply.status) {
        Ok(reply.payload)
    } else {
        Err(Failure::Api(
            ApiErrorDto::decode(reply.payload.as_bytes())
                .map_err(|_| Failure::Protocol)?
                .code,
        ))
    }
}

fn display_value(value: Option<&ValueDto>, vi: bool) -> String {
    match value {
        None => tr(vi, "Không khả dụng", "Unavailable"),
        Some(ValueDto::Blank) => tr(vi, "Trống", "Blank"),
        Some(ValueDto::Null) => "null".to_owned(),
        Some(ValueDto::Text(text)) => text.clone(),
        Some(ValueDto::Integer(number) | ValueDto::Money(number)) => number.to_string(),
        Some(ValueDto::Bool(value)) => tr(
            vi,
            if *value { "Đúng" } else { "Sai" },
            if *value { "True" } else { "False" },
        ),
        Some(ValueDto::Ref(reference)) => reference.record_id.to_string(),
    }
}

fn parse_param(field: &FieldMetadataDto, input: &str) -> Result<ValueDto, ()> {
    if input.is_empty() {
        return Ok(ValueDto::Blank);
    }
    if input == "null" {
        return Ok(ValueDto::Null);
    }
    match &field.field_type {
        FieldType::Text => Ok(ValueDto::Text(
            input.strip_prefix("text:").unwrap_or(input).to_owned(),
        )),
        FieldType::Integer => input.parse().map(ValueDto::Integer).map_err(|_| ()),
        FieldType::Money => input.parse().map(ValueDto::Money).map_err(|_| ()),
        FieldType::Bool => match input {
            "true" => Ok(ValueDto::Bool(true)),
            "false" => Ok(ValueDto::Bool(false)),
            _ => Err(()),
        },
        FieldType::Ref { target_table } => Ok(ValueDto::Ref(RefDto {
            table_id: target_table.clone(),
            record_id: RecordId::new(input).map_err(|_| ())?,
        })),
    }
}

async fn load_table(state: State, config: Config, table: TableId, generation: u64) {
    let path = format!("/v1/apps/{}/tables/{table}?limit=100", config.app);
    let result = request(&config, &path, "GET", "")
        .await
        .and_then(|raw| ListResponseDto::decode(raw.as_bytes()).map_err(|_| Failure::Protocol));
    if state.generation.try_get_untracked() != Some(generation)
        || state.table.get_untracked() != table.as_str()
    {
        return;
    }
    match result {
        Ok(list)
            if list.app_id == config.app
                && list.table_id == table
                && state
                    .metadata
                    .get_untracked()
                    .is_some_and(|metadata| metadata.release_id == list.release_id) =>
        {
            let selected = state.record.get_untracked();
            if !list
                .records
                .iter()
                .any(|record| record.record_id.as_str() == selected)
            {
                state.select_record(
                    list.records
                        .first()
                        .map(|record| record.record_id.to_string())
                        .unwrap_or_default(),
                );
            }
            state.list.set(Some(list));
            state.notice.set(Notice::Live);
        }
        Ok(_) => state.notice.set(Notice::Failure(Failure::Protocol)),
        Err(error) => state.show_failure(error),
    }
    state.busy.set(false);
}

fn receipt_matches(receipt: &ReceiptResponseDto, attempt: &Attempt) -> bool {
    receipt.app_id == attempt.config.app
        && receipt.command_id == attempt.request.command_id
        && receipt.record_id == attempt.request.record_id
        && receipt.release_id == attempt.request.release_id
        && receipt.request_key == attempt.request.request_key
}

async fn load_application(state: State, config: Config, generation: u64) {
    let path = format!("/v1/apps/{}/runtime-metadata", config.app);
    let result = request(&config, &path, "GET", "")
        .await
        .and_then(|raw| MetadataResponseDto::decode(raw.as_bytes()).map_err(|_| Failure::Protocol));
    if state.generation.try_get_untracked() != Some(generation) {
        return;
    }
    match result {
        Ok(metadata) if metadata.app_id == config.app && metadata.canonical_appspec.is_none() => {
            let selected = state.table.get_untracked();
            let table = metadata
                .tables
                .iter()
                .find(|table| table.table_id.as_str() == selected)
                .or_else(|| metadata.tables.first())
                .map(|table| table.table_id.clone());
            state.metadata.set(Some(metadata));
            if let Some(table) = table {
                state.table.set(table.to_string());
                load_table(state, config, table, generation).await;
            } else {
                state.busy.set(false);
                state.notice.set(Notice::Live);
            }
        }
        Ok(_) => {
            state.notice.set(Notice::Failure(Failure::Protocol));
            state.busy.set(false);
        }
        Err(error) => state.show_failure(error),
    }
}

async fn finish_receipt(
    state: State,
    attempt: &Attempt,
    receipt: ReceiptResponseDto,
    generation: u64,
    credential: String,
) {
    if !receipt_matches(&receipt, attempt) {
        let mut unresolved = attempt.clone();
        unresolved.uncertain = true;
        state.attempt.set(Some(unresolved));
        state.notice.set(Notice::Unknown);
        state.busy.set(false);
        return;
    }
    state.receipt.set(Some(receipt));
    state.attempt.set(None);
    state.errors.set(BTreeMap::new());
    let mut config = attempt.config.clone();
    config.credential = credential;
    load_application(state, config, generation).await;
}

fn dispatch(state: State, mut attempt: Attempt, credential: String, recover: bool) {
    if state.busy.get_untracked() {
        return;
    }
    state.busy.set(true);
    state.notice.set(Notice::Sending);
    let generation = state.generation.get_untracked();
    let mut config = attempt.config.clone();
    config.credential = credential.clone();
    leptos::task::spawn_local(async move {
        let path = if recover {
            format!(
                "/v1/apps/{}/receipts/{}",
                config.app, attempt.request.request_key
            )
        } else {
            format!("/v1/apps/{}/commands", config.app)
        };
        let result = request(
            &config,
            &path,
            if recover { "GET" } else { "POST" },
            if recover { "" } else { &attempt.body },
        )
        .await;
        if state.generation.try_get_untracked() != Some(generation) {
            return;
        }
        match result {
            Ok(raw) => match ReceiptResponseDto::decode(raw.as_bytes()) {
                Ok(receipt) => {
                    finish_receipt(state, &attempt, receipt, generation, credential).await
                }
                Err(_) => {
                    attempt.uncertain = true;
                    state.attempt.set(Some(attempt));
                    state.notice.set(Notice::Unknown);
                    state.busy.set(false);
                }
            },
            Err(Failure::Api(ErrorCodeDto::NotFound)) if recover => {
                state.notice.set(Notice::NoReceipt);
                state.busy.set(false);
            }
            Err(Failure::Transport | Failure::Protocol | Failure::Api(ErrorCodeDto::Internal)) => {
                attempt.uncertain = true;
                state.attempt.set(Some(attempt));
                state.notice.set(Notice::Unknown);
                state.busy.set(false);
            }
            Err(error) => {
                if !recover && !attempt.uncertain {
                    state.attempt.set(None);
                }
                state.show_failure(error);
            }
        }
    });
}

fn notice_text(notice: Notice, vi: bool) -> String {
    match notice {
        Notice::Ready => tr(
            vi,
            "Kết nối để mở ứng dụng đã phát hành.",
            "Connect to open a released application.",
        ),
        Notice::Loading => tr(vi, "Đang đọc ứng dụng…", "Loading application…"),
        Notice::Live => tr(vi, "Đã đọc dữ liệu từ máy chủ.", "Server data loaded."),
        Notice::Sending => tr(
            vi,
            "Đang chờ kết quả từ máy chủ…",
            "Waiting for the server result…",
        ),
        Notice::Unknown => tr(
            vi,
            "Chưa biết thao tác đã được lưu hay chưa. Khôi phục biên nhận hoặc thử lại cùng yêu cầu; nội dung được giữ nguyên.",
            "The operation outcome is unknown. Recover the receipt or retry the same request; your input is preserved.",
        ),
        Notice::NoReceipt => tr(
            vi,
            "Chưa có biên nhận cho yêu cầu này. Có thể thử lại cùng yêu cầu đã giữ.",
            "No receipt is available for this request. You can retry the retained request.",
        ),
        Notice::Failure(Failure::Configuration) => tr(
            vi,
            "Kiểm tra địa chỉ máy chủ, ứng dụng và thông tin đăng nhập.",
            "Check the server address, application and credential.",
        ),
        Notice::Failure(Failure::Transport) => tr(
            vi,
            "Không thể đọc máy chủ. Nội dung nhập được giữ nguyên.",
            "The server could not be reached. Your input is preserved.",
        ),
        Notice::Failure(Failure::Protocol) => tr(
            vi,
            "Máy chủ trả về dữ liệu không được hỗ trợ. Nội dung nhập được giữ nguyên.",
            "The server returned unsupported data. Your input is preserved.",
        ),
        Notice::Failure(Failure::Api(ErrorCodeDto::Denied)) => tr(
            vi,
            "Từ chối: phiên hoặc quyền hiện tại không cho phép thao tác. Nội dung nhập được giữ nguyên.",
            "Denied: the current session or permission does not allow this operation. Your input is preserved.",
        ),
        Notice::Failure(Failure::Api(ErrorCodeDto::Conflict)) => tr(
            vi,
            "Dữ liệu đã thay đổi. Đọc lại để xem trước khi gửi một yêu cầu mới.",
            "The data changed. Refresh and review it before submitting a new request.",
        ),
        Notice::Failure(Failure::Api(ErrorCodeDto::Validation)) => tr(
            vi,
            "Dữ liệu không đáp ứng quy tắc của ứng dụng. Sửa nội dung và thử lại.",
            "The input does not satisfy the application's rules. Correct it and retry.",
        ),
        Notice::Failure(Failure::Api(ErrorCodeDto::Unsupported)) => tr(
            vi,
            "Thao tác này chưa được máy chủ hỗ trợ.",
            "This operation is not supported by the server.",
        ),
        Notice::Failure(Failure::Api(ErrorCodeDto::NotFound)) => tr(
            vi,
            "Không tìm thấy nội dung được phép đọc.",
            "The permitted content was not found.",
        ),
        Notice::Failure(Failure::Api(ErrorCodeDto::Internal)) => tr(
            vi,
            "Máy chủ chưa thể hoàn tất thao tác. Nội dung nhập được giữ nguyên.",
            "The server could not complete the operation. Your input is preserved.",
        ),
    }
}

fn command_label(command: &CommandMetadataDto) -> String {
    if command.name != command.command_id {
        return command.name.clone();
    }
    let mut label = command
        .command_id
        .strip_prefix("cmd_")
        .unwrap_or(&command.command_id)
        .replace('_', " ");
    if let Some(first) = label.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    label
}

#[component]
pub fn RuntimeApp() -> impl IntoView {
    on_cleanup(runtime_abort_all);
    let locale = RwSignal::new(true);
    let dark = RwSignal::new(false);
    let base = RwSignal::new("http://127.0.0.1:8080".to_owned());
    let app = RwSignal::new(String::new());
    let credential = RwSignal::new(String::new());
    let state = State {
        metadata: RwSignal::new(None),
        list: RwSignal::new(None),
        table: RwSignal::new(String::new()),
        record: RwSignal::new(String::new()),
        command: RwSignal::new(String::new()),
        params: RwSignal::new(BTreeMap::new()),
        errors: RwSignal::new(BTreeMap::new()),
        attempt: RwSignal::new(None),
        receipt: RwSignal::new(None),
        notice: RwSignal::new(Notice::Ready),
        busy: RwSignal::new(false),
        generation: RwSignal::new(0),
    };
    let config = move || -> Result<Config, Failure> {
        let credential = credential.get_untracked();
        if credential.is_empty()
            || credential.len() > 4096
            || base.get_untracked().len() > 2048
            || !runtime_validate_base(&base.get_untracked())
        {
            return Err(Failure::Configuration);
        }
        Ok(Config {
            base: base.get_untracked(),
            app: AppId::new(app.get_untracked()).map_err(|_| Failure::Configuration)?,
            credential,
        })
    };
    let connect = move |_| {
        if state.busy.get_untracked() {
            return;
        }
        let config = match config() {
            Ok(config) => config,
            Err(error) => {
                state.notice.set(Notice::Failure(error));
                return;
            }
        };
        if let Some(attempt) = state.attempt.get_untracked()
            && (attempt.config.base != config.base || attempt.config.app != config.app)
        {
            state.notice.set(Notice::Unknown);
            return;
        }
        state.invalidate();
        state.busy.set(true);
        state.notice.set(Notice::Loading);
        let generation = state.generation.get_untracked();
        leptos::task::spawn_local(load_application(state, config, generation));
    };
    let refresh = move |_| {
        if state.busy.get_untracked() || state.attempt.get_untracked().is_some() {
            return;
        }
        if let Ok(config) = config() {
            state.busy.set(true);
            state.notice.set(Notice::Loading);
            let generation = state.generation.get_untracked();
            state.metadata.set(None);
            state.list.set(None);
            leptos::task::spawn_local(load_application(state, config, generation));
        }
    };
    let submit = move |_| {
        if state.busy.get_untracked() || state.attempt.get_untracked().is_some() {
            return;
        }
        let (Ok(config), Some(metadata), Some(list), Some(command)) = (
            config(),
            state.metadata.get_untracked(),
            state.list.get_untracked(),
            state.command(),
        ) else {
            return;
        };
        let Ok(record_id) = RecordId::new(state.record.get_untracked()) else {
            return;
        };
        if !list
            .records
            .iter()
            .any(|record| record.record_id == record_id)
        {
            return;
        }
        let buffers = state.params.get_untracked();
        let mut params = BTreeMap::new();
        let mut errors = BTreeMap::new();
        for field in &command.input_fields {
            let Some(input) = buffers.get(&field.field_id) else {
                if field.required {
                    errors.insert(field.field_id.clone(), InputIssue::Required);
                }
                continue;
            };
            match parse_param(field, input) {
                Ok(ValueDto::Blank | ValueDto::Null) if field.required => {
                    errors.insert(field.field_id.clone(), InputIssue::Required);
                }
                Ok(value) => {
                    params.insert(field.field_id.clone(), value);
                }
                Err(()) => {
                    errors.insert(
                        field.field_id.clone(),
                        InputIssue::Type(field.field_type.clone()),
                    );
                }
            }
        }
        state.errors.set(errors.clone());
        if !errors.is_empty() {
            state
                .notice
                .set(Notice::Failure(Failure::Api(ErrorCodeDto::Validation)));
            crate::focus_first_error();
            return;
        }
        let Ok(request_key) = runtime_request_key() else {
            state.notice.set(Notice::Failure(Failure::Configuration));
            return;
        };
        let request = CommandRequestDto {
            api_version: API_VERSION,
            command_id: command.command_id,
            record_id,
            params,
            request_key,
            expected_revision: list.revision,
            release_id: metadata.release_id,
        };
        let Ok(body) = request.encode().and_then(|bytes| {
            String::from_utf8(bytes).map_err(|_| evobase_protocol::appspec::WireError::Validation {
                field: "body",
                message: "encoding".to_owned(),
            })
        }) else {
            state.notice.set(Notice::Failure(Failure::Protocol));
            return;
        };
        let mut retained_config = config;
        retained_config.credential.clear();
        let attempt = Attempt {
            config: retained_config,
            request,
            body,
            uncertain: false,
        };
        state.receipt.set(None);
        state.attempt.set(Some(attempt.clone()));
        dispatch(state, attempt, credential.get_untracked(), false);
    };
    let locked = move || state.busy.get() || state.attempt.get().is_some();
    view! {
        <div class="shell runtime-shell">
            <header class="topbar"><div class="brand"><span class="brand-mark" aria-hidden="true">"E"</span>"EvoBase" <span class="local-tag">"Runtime"</span></div><div class="top-controls">
                <a href="?">{move || tr(locale.get(), "Mở Builder", "Open Builder")}</a>
                <button data-testid="runtime-locale" on:click=move |_| { locale.update(|vi| *vi = !*vi); if let Some(root) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.document_element()) { let _ = root.set_attribute("lang", if locale.get_untracked() { "vi" } else { "en" }); } }>{move || tr(locale.get(), "English", "Tiếng Việt")}</button>
                <button data-testid="runtime-theme" aria-pressed=move || dark.get().to_string() on:click=move |_| { dark.update(|dark| *dark = !*dark); if let Some(root) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.document_element()) { let _ = root.set_attribute("data-theme", if dark.get_untracked() { "dark" } else { "light" }); } }>{move || tr(locale.get(), if dark.get() { "Giao diện sáng" } else { "Giao diện tối" }, if dark.get() { "Light theme" } else { "Dark theme" })}</button>
            </div></header>
            <main class="runtime-main workspace">
                <section class="overview"><span class="eyebrow">"Runtime"</span><h1>{move || tr(locale.get(), "Công việc trong ứng dụng", "Work in your application")}</h1><p>{move || tr(locale.get(), "Đọc dữ liệu và thực hiện thao tác của ứng dụng đã phát hành.", "Read records and perform actions in a released application.")}</p></section>
                <section class="surface"><details open=move || state.metadata.get().is_none()><summary>{move || tr(locale.get(), "Kết nối ứng dụng", "Application connection")}</summary><div class="preview-form">
                    <label>{move || tr(locale.get(), "Máy chủ", "Server")}<input data-testid="runtime-server" disabled=move || state.attempt.get().is_some() prop:value=move || base.get() on:input=move |event| { base.set(event_target_value(&event)); state.params.set(BTreeMap::new()); state.errors.set(BTreeMap::new()); state.table.set(String::new()); state.command.set(String::new()); state.record.set(String::new()); state.invalidate(); }/></label>
                    <label>{move || tr(locale.get(), "Định danh ứng dụng", "Application identity")}<input data-testid="runtime-app" disabled=move || state.attempt.get().is_some() prop:value=move || app.get() on:input=move |event| { app.set(event_target_value(&event)); state.params.set(BTreeMap::new()); state.errors.set(BTreeMap::new()); state.table.set(String::new()); state.command.set(String::new()); state.record.set(String::new()); state.invalidate(); }/></label>
                    <label>{move || tr(locale.get(), "Thông tin đăng nhập", "Credential")}<input data-testid="runtime-credential" type="password" autocomplete="off" prop:value=move || credential.get() on:input=move |event| { credential.set(event_target_value(&event)); state.invalidate(); }/></label>
                </div><p class="support">{move || tr(locale.get(), "Thông tin đăng nhập chỉ được giữ trong cửa sổ này. Đóng hoặc tải lại sẽ mất nội dung chưa gửi và yêu cầu đang chờ khôi phục.", "The credential stays in this window. Closing or reloading loses unsent input and any request awaiting recovery.")}</p><div class="actions"><button data-testid="runtime-connect" disabled=move || state.busy.get() on:click=connect>{move || tr(locale.get(), "Kết nối", "Connect")}</button><button data-testid="runtime-disconnect" on:click=move |_| { credential.set(String::new()); state.invalidate(); }>{move || tr(locale.get(), "Ngắt kết nối", "Disconnect")}</button></div></details></section>
                <p data-testid="runtime-status" role="status" class="status" data-state=move || if matches!(state.notice.get(), Notice::Unknown | Notice::NoReceipt | Notice::Failure(_)) { "error" } else { "normal" }>{move || notice_text(state.notice.get(), locale.get())}</p>
                <Show when=move || state.receipt.get().is_some()><p data-testid="runtime-receipt" role="status" class="surface">{move || state.receipt.get().map(|receipt| format!("{} {}{}", tr(locale.get(), "Máy chủ đã xác nhận · phiên bản dữ liệu", "Server confirmed · data revision"), receipt.revision.get(), if receipt.replayed { tr(locale.get(), " · biên nhận được khôi phục", " · recovered receipt") } else { String::new() })).unwrap_or_default()}</p></Show>
                <Show when=move || state.attempt.get().is_some() && !state.busy.get()><section class="surface"><p>{move || tr(locale.get(), "Giữ nguyên yêu cầu đang chờ để tránh gửi trùng thao tác.", "Keep the pending request intact to avoid duplicating the operation.")}</p><div class="actions">
                    <button data-testid="runtime-recover" on:click=move |_| { if let Some(attempt) = state.attempt.get_untracked() { dispatch(state, attempt, credential.get_untracked(), true); } }>{move || tr(locale.get(), "Khôi phục biên nhận", "Recover receipt")}</button>
                    <button data-testid="runtime-retry" on:click=move |_| { if let Some(attempt) = state.attempt.get_untracked() { dispatch(state, attempt, credential.get_untracked(), false); } }>{move || tr(locale.get(), "Thử lại cùng yêu cầu", "Retry same request")}</button>
                </div></section></Show>
                <Show when=move || state.metadata.get().is_some()><section class="surface"><div class="toolbar"><h2>{move || tr(locale.get(), "Bản ghi", "Records")}</h2><button data-testid="runtime-refresh" disabled=locked on:click=refresh>{move || tr(locale.get(), "Đọc lại dữ liệu", "Refresh records")}</button></div>
                    <label>{move || tr(locale.get(), "Bảng", "Table")}<select data-testid="runtime-table" disabled=locked prop:value=move || state.table.get() on:change=move |event| {
                        state.table.set(event_target_value(&event)); state.list.set(None); state.record.set(String::new()); state.command.set(String::new()); state.params.set(BTreeMap::new()); state.errors.set(BTreeMap::new());
                        if let (Ok(config), Ok(table)) = (config(), TableId::new(state.table.get_untracked())) { state.busy.set(true); state.notice.set(Notice::Loading); let generation = state.generation.get_untracked(); leptos::task::spawn_local(load_table(state, config, table, generation)); }
                    }>{move || state.metadata.get().map(|metadata| metadata.tables.into_iter().map(|table| view! { <option value=table.table_id.to_string()>{table.name}</option> }).collect_view())}</select></label>
                    <Show when=move || state.list.get().is_some_and(|list| list.has_more)><p data-testid="runtime-truncated" role="status">{move || tr(locale.get(), "Đang hiển thị 100 bản ghi đầu tiên; còn bản ghi khác chưa hiển thị.", "Showing the first 100 records; additional records are not displayed.")}</p></Show>
                    <Show when=move || state.list.get().is_some_and(|list| !list.records.is_empty()) fallback=move || view! { <p data-testid="runtime-empty">{move || tr(locale.get(), "Chưa có bản ghi để hiển thị.", "No records to display.")}</p> }>
                        <div class="runtime-records" data-testid="runtime-records">{move || {
                            let table = state.metadata.get().and_then(|metadata| metadata.tables.into_iter().find(|table| table.table_id.as_str() == state.table.get()));
                            state.list.get().map(|list| list.records.into_iter().map(|record| {
                                let id = record.record_id.to_string(); let selected_id = id.clone(); let click_id = id.clone();
                                let fields = table.as_ref().map(|table| table.fields.iter().map(|field| (field.name.clone(), record.values.get(&field.field_id).cloned())).collect::<Vec<_>>()).unwrap_or_default();
                                view! { <article class="runtime-record" data-record-id=id><button data-testid=format!("runtime-record-{selected_id}") aria-pressed=move || (state.record.get() == selected_id).to_string() class:selected=move || state.record.get() == click_id disabled=locked on:click={let id = record.record_id.to_string(); move |_| state.select_record(id.clone()) }>{move || tr(locale.get(), "Chọn bản ghi", "Select record")}</button><dl>{fields.into_iter().map(|(name, value)| view! { <div><dt>{name}</dt><dd>{move || display_value(value.as_ref(), locale.get())}</dd></div> }).collect_view()}</dl></article> }
                            }).collect_view())
                        }}</div>
                    </Show>
                </section>
                <section class="surface" data-testid="runtime-actions"><h2>{move || tr(locale.get(), "Thao tác với bản ghi đã chọn", "Actions for the selected record")}</h2>
                    <Show when=move || state.command().is_some() && !state.record.get().is_empty() fallback=move || view! { <p>{move || tr(locale.get(), "Chưa có thao tác được cấp cho bảng này.", "No action is available for this table.")}</p> }>
                        <label>{move || tr(locale.get(), "Thao tác", "Action")}<select data-testid="runtime-command" disabled=locked prop:value=move || state.command().map(|command| command.command_id).unwrap_or_default() on:change=move |event| { state.command.set(event_target_value(&event)); state.params.set(BTreeMap::new()); state.errors.set(BTreeMap::new()); }>{move || state.metadata.get().map(|metadata| metadata.commands.into_iter().filter(|command| command.table_id.as_str() == state.table.get() && command.eligible_record_ids.iter().any(|record| record.as_str() == state.record.get())).map(|command| { let label = command_label(&command); view! { <option value=command.command_id>{label}</option> } }).collect_view())}</select></label>
                        <div class="preview-form"><For each=move || state.command().map(|command| command.input_fields).unwrap_or_default() key=|field| field.field_id.clone() children=move |field| {
                            let id = field.field_id; let value_id = id.clone(); let input_id = id.clone(); let error_id = id.clone(); let text_id = id.clone(); let label_type = field.field_type; let required = field.required;
                            let described = format!("runtime-error-{id}");
                            view! { <label>{field.name}<span class="support">{move || type_label(&label_type, locale.get())}</span><span class="support">{move || tr(locale.get(), if required { "Bắt buộc" } else { "Tùy chọn · để nguyên để giữ giá trị hiện có" }, if required { "Required" } else { "Optional · leave untouched to keep the existing value" })}</span><input data-testid=format!("runtime-param-{id}") disabled=locked aria-invalid=move || state.errors.get().contains_key(&error_id).to_string() aria-describedby=described.clone() prop:value=move || state.params.get().get(&value_id).cloned().unwrap_or_default() on:input=move |event| state.params.update(|params| { params.insert(input_id.clone(), event_target_value(&event)); })/><span class="cell-help" id=described>{move || state.errors.get().get(&text_id).map(|issue| match issue { InputIssue::Required => tr(locale.get(), "Nhập giá trị bắt buộc.", "Enter the required value."), InputIssue::Type(kind) => format!("{} {}", tr(locale.get(), "Kiểu cần nhập:", "Expected type:"), type_label(kind, locale.get())) }).unwrap_or_default()}</span></label> }
                        }/></div>
                        <button class="primary" data-testid="runtime-submit" disabled=locked on:click=submit>{move || state.command().map(|command| command_label(&command)).unwrap_or_default()}</button>
                    </Show>
                </section></Show>
            </main>
        </div>
    }
}
