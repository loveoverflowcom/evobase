//! A mounted local-only Builder. Checked Rust definitions own validation;
//! browser state owns unsaved input, selection, focus and local persistence.
pub mod constraints;
pub mod policy;
pub mod relations;

use evobase_appspec::relations::{BatchChange, LocalPreviewPolicy, RelationError, RelationStore};
use evobase_appspec::{
    CheckedAppSpec, Error, FieldId, FieldType, RawAppSpec, RawField, RawRecord, Scope, TableId,
    Value,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

const STORAGE_KEY: &str = "evobase.builder.local.v1";
const MAX_IMPORT_BYTES: usize = 65_536;
const MAX_IMPORT_ROWS: usize = 100;
type Buffers = BTreeMap<String, String>;

#[derive(Clone)]
struct Snapshot {
    revision: u64,
    spec: RawAppSpec,
    records: Vec<RawRecord>,
}

#[derive(Clone, Copy, PartialEq)]
enum Status {
    Clean,
    Dirty,
    Saving,
    Saved,
    Invalid,
    Conflict,
    Expired,
    StorageError,
    Unsupported,
}

pub fn tr(vi: bool, vi_text: &str, en: &str) -> String {
    if vi { vi_text } else { en }.to_owned()
}

fn storage() -> Result<web_sys::Storage, String> {
    web_sys::window()
        .ok_or("browser unavailable")?
        .local_storage()
        .map_err(|_| "storage access blocked")?
        .ok_or_else(|| "storage unavailable".to_owned())
}

fn scope_for(spec: &RawAppSpec) -> Scope {
    Scope::new("tenant_local_fixture", spec.app_id.clone()).expect("fixed synthetic scope")
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredSnapshot {
    format: u32,
    // String keeps the envelope exact even when a browser helper inspects it.
    revision: String,
    definition_json: String,
    records_json: String,
}

fn decode_saved(raw: &str) -> Result<Snapshot, String> {
    if raw.len() > evobase_appspec::MAX_BYTES {
        return Err("local snapshot exceeds limit".to_owned());
    }
    let stored: StoredSnapshot =
        serde_json::from_str(raw).map_err(|_| "local snapshot is invalid")?;
    if stored.format != 1 {
        return Err("unsupported local snapshot".to_owned());
    }
    let checked =
        CheckedAppSpec::decode(stored.definition_json.as_bytes()).map_err(|e| e.to_string())?;
    let spec = checked.definition().clone();
    if spec.tables.is_empty() {
        return Err("local Builder needs a table".to_owned());
    }
    let records = checked
        .decode_records(&scope_for(&spec), stored.records_json.as_bytes())
        .map_err(|e| e.to_string())?
        .to_raw();
    let revision = stored
        .revision
        .parse::<u64>()
        .map_err(|_| "invalid local revision")?;
    Ok(Snapshot {
        revision,
        spec,
        records,
    })
}

fn encode_saved(snapshot: &Snapshot) -> Result<String, String> {
    let checked = CheckedAppSpec::compile(snapshot.spec.clone()).map_err(|e| e.to_string())?;
    let rows = checked
        .validate_records(&scope_for(&snapshot.spec), &snapshot.records)
        .map_err(|e| e.to_string())?;
    let stored = StoredSnapshot {
        format: 1,
        revision: snapshot.revision.to_string(),
        definition_json: String::from_utf8(checked.encode().map_err(|e| e.to_string())?)
            .map_err(|_| "definition encoding")?,
        records_json: String::from_utf8(rows.encode().map_err(|e| e.to_string())?)
            .map_err(|_| "record encoding")?,
    };
    let raw = serde_json::to_string(&stored).map_err(|_| "snapshot encoding")?;
    if raw.len() > evobase_appspec::MAX_BYTES {
        return Err("snapshot exceeds limit".to_owned());
    }
    Ok(raw)
}

// Web Locks serializes this app's cross-tab local save intents. Unsupported
// browsers decline saving explicitly instead of promising a non-atomic receipt.
// The browser lock compares exact snapshot bytes; domain JSON stays opaque Rust strings.
#[wasm_bindgen(inline_js = r#"
export async function writeLocalSnapshot(key, expected, serialized) {
  if (!navigator.locks) throw new Error('unsupported');
  return navigator.locks.request('evobase-builder-local-v1', () => {
    const current = localStorage.getItem(key);
    if (current !== (expected === '' ? null : expected)) throw new Error('conflict');
    localStorage.setItem(key, serialized);
    const receipt = localStorage.getItem(key);
    if (receipt !== serialized) throw new Error('conflict');
    return receipt;
  });
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = writeLocalSnapshot)]
    async fn write_local_snapshot(
        key: &str,
        revision: &str,
        serialized: &str,
    ) -> Result<JsValue, JsValue>;
}

fn same_content(a: &Snapshot, b: &Snapshot) -> bool {
    fn canonical(snapshot: &Snapshot) -> Result<(Vec<u8>, Vec<u8>), Error> {
        let definition = CheckedAppSpec::compile(snapshot.spec.clone())?;
        let records = definition.validate_records(&scope_for(&snapshot.spec), &snapshot.records)?;
        Ok((definition.encode()?, records.encode()?))
    }
    match (canonical(a), canonical(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn cell_key(table: &TableId, record: &evobase_appspec::RecordId, field: &FieldId) -> String {
    format!("{table}/{record}/{field}")
}

fn value_input(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Blank) => String::new(),
        Some(Value::Null) => "null".to_owned(),
        Some(Value::Text(s)) => {
            if s.is_empty() || s == "null" || s.starts_with("text:") {
                format!("text:{s}")
            } else {
                s.clone()
            }
        }
        Some(Value::Integer(n)) | Some(Value::Money(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Ref(r)) => r.record_id.to_string(),
    }
}

fn type_label(kind: &FieldType, vi: bool) -> String {
    match kind {
        FieldType::Text => tr(vi, "Văn bản", "Text"),
        FieldType::Integer => tr(vi, "Số nguyên", "Integer"),
        FieldType::Bool => tr(vi, "Đúng / sai", "Boolean"),
        FieldType::Money => tr(vi, "Tiền · đơn vị nhỏ nhất", "Money · minor units"),
        FieldType::Ref { .. } => tr(vi, "Liên kết · định danh", "Reference · identity"),
    }
}

fn diagnostic(error: &Error, vi: bool) -> String {
    match error {
        Error::Required { .. } => tr(
            vi,
            "Trường bắt buộc không được để trống hoặc null.",
            "Required fields cannot be blank or null.",
        ),
        Error::ConstraintViolation { .. } => tr(
            vi,
            "Giá trị không đáp ứng quy tắc dữ liệu của trường.",
            "Value does not satisfy this field's data rule.",
        ),
        Error::InvalidConstraint { .. } => tr(
            vi,
            "Quy tắc không tương thích với kiểu trường hoặc giới hạn không hợp lệ.",
            "The rule is incompatible with the field type or its bounds are invalid.",
        ),
        Error::AmbiguousReference { .. } => tr(
            vi,
            "Tên liên kết trùng lặp. Chọn định danh bản ghi rõ ràng.",
            "Reference label is ambiguous. Enter an explicit record identity.",
        ),
        Error::MissingReference { .. } => tr(
            vi,
            "Không tìm thấy bản ghi liên kết trong bảng đích.",
            "No matching record in the target table.",
        ),
        Error::InvalidCell { expected, .. } | Error::WrongType { expected, .. } => format!(
            "{} {expected}.",
            tr(vi, "Kiểu dữ liệu cần nhập:", "Expected input type:")
        ),
        Error::Overflow { .. } => tr(
            vi,
            "Giá trị vượt giới hạn số nguyên 64 bit.",
            "Value exceeds the signed 64-bit range.",
        ),
        Error::LimitExceeded { limit, .. } => {
            format!("{} {limit}.", tr(vi, "Vượt giới hạn:", "Limit exceeded:"))
        }
        _ => tr(
            vi,
            "Dữ liệu không hợp lệ theo AppSpec. Xem chi tiết kỹ thuật.",
            "Data does not satisfy the AppSpec. See technical details.",
        ),
    }
}

/// This restoration seam is consciously local preview history. It grants no
/// hosted authority or provenance. Every new/changed authoring fact goes through
/// the same relation batch commands that own captures and the final graph.
fn apply_draft_changes(
    checked: &CheckedAppSpec,
    scope: &Scope,
    before: &[RawRecord],
    next: Vec<RawRecord>,
) -> Result<Vec<RawRecord>, RelationError> {
    let history = checked.validate_records(scope, before)?;
    let mut store =
        RelationStore::from_checked_snapshot(checked.clone(), history, &LocalPreviewPolicy)?;
    let original: BTreeMap<_, _> = before
        .iter()
        .map(|row| ((row.table_id.clone(), row.id.clone()), row))
        .collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut changes = Vec::new();
    for row in next {
        let key = (row.table_id.clone(), row.id.clone());
        if !seen.insert(key.clone()) {
            return Err(RelationError::DuplicateMutation {
                table: key.0,
                record: key.1,
            });
        }
        match original.get(&key) {
            Some(old) if **old == row => {}
            Some(_) => changes.push(BatchChange::Replace(row)),
            None => changes.push(BatchChange::Insert(row)),
        }
    }
    for key in original.keys().filter(|key| !seen.contains(*key)) {
        changes.push(BatchChange::Delete {
            table_id: key.0.clone(),
            record_id: key.1.clone(),
        });
    }
    store.apply_batch(changes)?;
    Ok(store.records().to_raw())
}

fn relation_diagnostic(error: &RelationError, vi: bool) -> String {
    match error {
        RelationError::Kernel(error) => diagnostic(error, vi),
        RelationError::CaptureInputForbidden { .. } => tr(
            vi,
            "Giá đã chốt được tính khi tạo dòng; để trống cột này.",
            "Captured prices are derived when creating a line; leave this column empty.",
        ),
        RelationError::CapturedValueImmutable { .. } => tr(
            vi,
            "Không thể sửa giá lịch sử đã chốt.",
            "Captured historical prices cannot be edited.",
        ),
        RelationError::CapturedReferenceImmutable { .. } => tr(
            vi,
            "Tạo dòng mới để chọn sản phẩm khác.",
            "Create a new line to choose another product.",
        ),
        RelationError::CaptureSourceInvalid { .. } => tr(
            vi,
            "Nguồn giá không có giá trị tiền hợp lệ.",
            "The capture source has no valid money value.",
        ),
        RelationError::BatchLimit => tr(
            vi,
            "Số thay đổi vượt giới hạn của một lượt kiểm tra.",
            "Too many changes in one checked batch.",
        ),
        _ => tr(
            vi,
            "Thay đổi không hợp lệ với các liên kết của bản nháp. Dữ liệu trước đó được giữ nguyên.",
            "This change does not satisfy the draft relation rules. Prior data is preserved.",
        ),
    }
}

/// Pending edits are parsed together and the entire candidate dataset is validated
/// before any row is changed. A diagnostic preserves every original input.
fn candidate(
    spec: &RawAppSpec,
    records: &[RawRecord],
    buffers: &Buffers,
    vi: bool,
) -> Result<Vec<RawRecord>, Buffers> {
    let checked = CheckedAppSpec::compile(spec.clone())
        .map_err(|e| BTreeMap::from([("definition".to_owned(), diagnostic(&e, vi))]))?;
    let scope = scope_for(spec);
    let mut output = records.to_vec();
    let mut errors = BTreeMap::new();
    for row in &mut output {
        let Some(table) = spec.tables.iter().find(|t| t.id == row.table_id) else {
            continue;
        };
        for field in &table.fields {
            let key = cell_key(&row.table_id, &row.id, &field.id);
            if let Some(text) = buffers.get(&key) {
                match checked.parse_cell(&scope, &row.table_id, &field.id, text, records) {
                    Ok(value) => {
                        row.values.insert(field.id.clone(), value);
                    }
                    Err(error) => {
                        errors.insert(key, diagnostic(&error, vi));
                    }
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    apply_draft_changes(&checked, &scope, records, output).map_err(|error| {
        let key = match &error {
            RelationError::Kernel(
                Error::Required { record, field }
                | Error::WrongType { record, field, .. }
                | Error::ConstraintViolation { record, field, .. },
            ) => records
                .iter()
                .find(|r| &r.id == record)
                .map(|r| cell_key(&r.table_id, record, field))
                .unwrap_or_else(|| "dataset".to_owned()),
            RelationError::CapturedValueImmutable { field }
            | RelationError::CapturedReferenceImmutable { field } => buffers
                .keys()
                .find(|key| key.ends_with(&format!("/{field}")))
                .cloned()
                .unwrap_or_else(|| "dataset".to_owned()),
            _ => "dataset".to_owned(),
        };
        BTreeMap::from([(key, relation_diagnostic(&error, vi))])
    })
}

fn prepare_import(
    current: &RawAppSpec,
    records: &[RawRecord],
    table_id: &TableId,
    source: &str,
    vi: bool,
) -> Result<Vec<RawRecord>, Vec<String>> {
    let Some(table) = current.tables.iter().find(|t| &t.id == table_id) else {
        return Err(vec![tr(vi, "Không tìm thấy bảng.", "Unknown table.")]);
    };
    if source.len() > MAX_IMPORT_BYTES {
        return Err(vec![tr(
            vi,
            "Dữ liệu vượt giới hạn 64 KiB.",
            "Input exceeds the 64 KiB limit.",
        )]);
    }
    let lines: Vec<_> = source.lines().collect();
    if lines.len() < 2 || lines.len() > MAX_IMPORT_ROWS + 1 {
        return Err(vec![tr(
            vi,
            "Cần một hàng tiêu đề và tối đa 100 hàng dữ liệu.",
            "Use a header and at most 100 data rows.",
        )]);
    }
    let headers: Vec<_> = lines[0].split('\t').collect();
    let mut fields = Vec::new();
    let mut diagnostics = Vec::new();
    for (column, header) in headers.iter().enumerate() {
        match table.fields.iter().find(|f| f.id.as_str() == *header) {
            Some(field)
                if !fields
                    .iter()
                    .any(|existing: &&RawField| existing.id == field.id) =>
            {
                fields.push(field)
            }
            _ => diagnostics.push(format!(
                "{} {}: {}",
                tr(vi, "Cột", "Column"),
                column + 1,
                tr(
                    vi,
                    "định danh trường không tồn tại hoặc bị lặp.",
                    "unknown or duplicate field identity."
                )
            )),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let checked = match CheckedAppSpec::compile(current.clone()) {
        Ok(spec) => spec,
        Err(error) => {
            return Err(vec![diagnostic(&error, vi)]);
        }
    };
    let scope = scope_for(current);
    let baseline = records;
    let mut next = baseline.to_vec();
    let start_index = next.len();
    for (index, line) in lines.iter().skip(1).enumerate() {
        let cells: Vec<_> = line.split('\t').collect();
        if cells.len() != fields.len() {
            diagnostics.push(format!(
                "{} {}: {}",
                tr(vi, "Hàng", "Row"),
                index + 2,
                tr(
                    vi,
                    "số cột không khớp tiêu đề.",
                    "column count differs from the header."
                )
            ));
            continue;
        }
        let mut values = BTreeMap::new();
        for (column, (field, text)) in fields.iter().zip(cells.iter()).enumerate() {
            match checked.parse_cell(&scope, &table.id, &field.id, text, baseline) {
                Ok(value) => {
                    values.insert(field.id.clone(), value);
                }
                Err(error) => diagnostics.push(format!(
                    "{} {}, {} {} ({}): {}",
                    tr(vi, "Hàng", "Row"),
                    index + 2,
                    tr(vi, "cột", "column"),
                    column + 1,
                    field.name,
                    diagnostic(&error, vi)
                )),
            }
        }
        let mut suffix = 1;
        while next
            .iter()
            .any(|r| r.id.as_str() == format!("rec_import_{suffix}"))
        {
            suffix += 1;
        }
        next.push(RawRecord {
            scope: scope.clone(),
            id: evobase_appspec::RecordId::new(format!("rec_import_{suffix}"))
                .expect("generated identity"),
            table_id: table.id.clone(),
            values,
        });
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let output = apply_draft_changes(&checked, &scope, baseline, next.clone());
    output.map_err(|error| {
        let (record, field) = match &error {
            RelationError::Kernel(
                Error::Required { record, field }
                | Error::WrongType { record, field, .. }
                | Error::ConstraintViolation { record, field, .. },
            ) => (Some(record), Some(field)),
            RelationError::CaptureInputForbidden { field }
            | RelationError::CaptureSourceInvalid { field } => (
                next[start_index..]
                    .iter()
                    .find(|r| {
                        r.values
                            .get(field)
                            .is_some_and(|v| !matches!(v, Value::Blank | Value::Null))
                    })
                    .map(|r| &r.id),
                Some(field),
            ),
            _ => (None, None),
        };
        let location = field
            .map(|field| {
                let row = record
                    .and_then(|record| next[start_index..].iter().position(|r| &r.id == record))
                    .unwrap_or(0);
                let name = table
                    .fields
                    .iter()
                    .find(|f| &f.id == field)
                    .map(|f| f.name.as_str())
                    .unwrap_or(field.as_str());
                let column = headers
                    .iter()
                    .position(|h| *h == field.as_str())
                    .map(|c| (c + 1).to_string())
                    .unwrap_or_else(|| tr(vi, "thiếu", "missing"));
                format!(
                    "{} {}, {} {} ({}): ",
                    tr(vi, "Hàng", "Row"),
                    row + 2,
                    tr(vi, "cột", "column"),
                    column,
                    name
                )
            })
            .unwrap_or_default();
        vec![format!("{location}{}", relation_diagnostic(&error, vi))]
    })
}

#[component]
fn FieldEditor(
    spec: RwSignal<RawAppSpec>,
    records: RwSignal<Vec<RawRecord>>,
    buffers: RwSignal<Buffers>,
    errors: RwSignal<Buffers>,
    status: RwSignal<Status>,
    composing: RwSignal<bool>,
    table: TableId,
    record: evobase_appspec::RecordId,
    field: RawField,
    locale: RwSignal<bool>,
    saving: RwSignal<bool>,
    paused: RwSignal<bool>,
    #[prop(default = "cell")] context: &'static str,
) -> impl IntoView {
    let key = cell_key(&table, &record, &field.id);
    let name = field.name.clone();
    let field_id = field.id.clone();
    let numeric = matches!(field.field_type, FieldType::Integer | FieldType::Money);
    let key_value = key.clone();
    let table_value = table.clone();
    let record_value = record.clone();
    let field_value = field_id.clone();
    let current = move || {
        buffers.get().get(&key_value).cloned().unwrap_or_else(|| {
            records
                .get()
                .iter()
                .find(|r| r.table_id == table_value && r.id == record_value)
                .map(|r| value_input(r.values.get(&field_value)))
                .unwrap_or_default()
        })
    };
    let key_input = key.clone();
    let key_error = key.clone();
    let key_description = key.clone();
    let described = format!("error-{context}-{}-{}", record.as_str(), field.id.as_str());
    let described_input = described.clone();
    let managed_id = format!(
        "managed-{context}-{}-{}",
        record.as_str(),
        field.id.as_str()
    );
    let managed_input = managed_id.clone();
    let identity = StoredValue::new((table.clone(), field.id.clone()));
    let managed = move || {
        identity.with_value(|(table, field)| {
            spec.get().capture_rules.iter().any(|rule| {
                &rule.line_table == table
                    && (&rule.captured_price_field == field || &rule.product_ref_field == field)
            })
        })
    };
    let label_name = name.clone();
    let reference = matches!(field.field_type, FieldType::Ref { .. });
    let ref_table = table.clone();
    let ref_record = record.clone();
    let ref_field = field_id.clone();
    let commit = move || {
        if composing.get_untracked() {
            return false;
        }
        if buffers.get_untracked().is_empty() {
            return true;
        }
        match candidate(
            &spec.get_untracked(),
            &records.get_untracked(),
            &buffers.get_untracked(),
            locale.get_untracked(),
        ) {
            Ok(next) => {
                records.set(next);
                buffers.set(BTreeMap::new());
                errors.set(BTreeMap::new());
                status.set(Status::Dirty);
                true
            }
            Err(next) => {
                errors.set(next);
                status.set(Status::Invalid);
                false
            }
        }
    };
    let key_escape = key.clone();
    view! {
        <input data-testid=format!("{context}-{}-{}", record.as_str(), field.id.as_str())
            data-cell-key=key class:numeric=numeric class:managed=managed readonly=managed disabled=move || saving.get() || paused.get()
            aria-label=move || format!("{} · {}", label_name, tr(locale.get(), "Bản ghi", "Record"))
            aria-invalid=move || errors.get().contains_key(&key_error).to_string()
            aria-describedby=move || if managed() { format!("{described_input} {managed_input}") } else { described_input.clone() } prop:value=current
            on:input=move |event| {
                let text = event_target_value(&event);
                buffers.update(|b| { b.insert(key_input.clone(), text); });
                status.set(Status::Dirty);
            }
            on:compositionstart=move |_| composing.set(true)
            on:compositionend=move |_| composing.set(false)
            on:change=move |_| { commit(); }
            on:keydown=move |event: web_sys::KeyboardEvent| {
                if event.is_composing() || composing.get_untracked() { return; }
                match event.key().as_str() {
                    "Enter" => { event.prevent_default(); commit(); }
                    "Tab" => { if !commit() { event.prevent_default(); } }
                    "Escape" => {
                        event.prevent_default();
                        buffers.update(|b| { b.remove(&key_escape); });
                        errors.update(|b| { b.remove(&key_escape); });
                        status.set(Status::Dirty);
                    }
                    _ => {}
                }
            }/>
        <span class="ref-label" hidden=!reference>{move || {
            let dataset = records.get();
            dataset.iter().find(|r| r.table_id == ref_table && r.id == ref_record).and_then(|r| r.values.get(&ref_field)).and_then(|value| {
                if let Value::Ref(target) = value {
                    dataset.iter().find(|r| r.table_id == target.table_id && r.id == target.record_id).and_then(|r| r.values.values().find_map(|v| if let Value::Text(label) = v { Some(label.clone()) } else { None }))
                } else { None }
            }).unwrap_or_else(|| tr(locale.get(), "Chưa chọn liên kết", "No reference selected"))
        }}</span>
        <span id=managed_id class="managed-note" hidden=move || !managed()>{move || tr(locale.get(), "Giữ nguyên sản phẩm và giá đã chốt. Tạo dòng mới để thay đổi.", "Historical product and price are fixed. Create a new line to change them.")}</span>
        <span id=described class="cell-help">{move || errors.get().get(&key_description).cloned().unwrap_or_default()}</span>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let initial_spec = evobase_appspec::fixtures::example_spec();
    let default = Snapshot {
        revision: 0,
        records: evobase_appspec::fixtures::example_records(&scope_for(&initial_spec)),
        spec: initial_spec,
    };
    let stored_bytes = storage().and_then(|storage| {
        storage
            .get_item(STORAGE_KEY)
            .map_err(|_| "storage read failed".to_owned())
    });
    let (initial, load_failed) = match &stored_bytes {
        Ok(Some(raw)) => match decode_saved(raw) {
            Ok(snapshot) => (snapshot, false),
            Err(_) => (default, true),
        },
        Ok(None) => (default, false),
        Err(_) => (default, true),
    };
    let persisted_bytes = RwSignal::new(stored_bytes.ok().flatten());
    // Failed restoration cannot overwrite the unreadable source. Recovery is an
    // explicit storage operation outside this local editing surface.
    let restoration_failed = load_failed;
    let saved = RwSignal::new(initial.clone());
    let spec = RwSignal::new(initial.spec.clone());
    let records = RwSignal::new(initial.records.clone());
    let selected =
        table_from_hash(&initial.spec).unwrap_or_else(|| initial.spec.tables[0].id.clone());
    let table_id = RwSignal::new(selected);
    let locale = RwSignal::new(true);
    let dark = RwSignal::new(false);
    let buffers = RwSignal::new(BTreeMap::<String, String>::new());
    let errors = RwSignal::new(BTreeMap::<String, String>::new());
    let status = RwSignal::new(if load_failed {
        Status::StorageError
    } else {
        Status::Clean
    });
    let paused = RwSignal::new(false);
    let composing = RwSignal::new(false);
    let saving = RwSignal::new(false);
    let app_name_input = RwSignal::new(initial.spec.name.clone());
    let field_name = RwSignal::new(String::new());
    let field_type = RwSignal::new("text".to_owned());
    let import_source = RwSignal::new(String::new());
    let import_errors = RwSignal::new(Vec::<String>::new());
    let form_mode = RwSignal::new(false);

    let select_table = move |id: TableId| {
        if table_id.get_untracked() != id {
            if let Some(window) = web_sys::window()
                && let Ok(history) = window.history()
            {
                let _ = history.push_state_with_url(&JsValue::NULL, "", Some(&format!("#{id}")));
            }
            table_id.set(id);
        }
    };
    let history_listener =
        leptos::leptos_dom::helpers::window_event_listener(leptos::ev::popstate, move |_| {
            let definition = spec.get_untracked();
            let selected =
                table_from_hash(&definition).unwrap_or_else(|| definition.tables[0].id.clone());
            table_id.set(selected);
        });
    on_cleanup(move || history_listener.remove());
    let save = move |_| {
        if restoration_failed
            || paused.get_untracked()
            || composing.get_untracked()
            || saving.get_untracked()
        {
            return;
        }
        let mut definition = spec.get_untracked();
        definition.name = app_name_input.get_untracked();
        let next = match candidate(
            &definition,
            &records.get_untracked(),
            &buffers.get_untracked(),
            locale.get_untracked(),
        ) {
            Ok(rows) => rows,
            Err(diagnostics) => {
                if let Some(id) = diagnostics
                    .keys()
                    .find_map(|key| key.split('/').next().and_then(|id| TableId::new(id).ok()))
                {
                    select_table(id);
                    form_mode.set(false);
                }
                errors.set(diagnostics);
                status.set(Status::Invalid);
                focus_first_error();
                return;
            }
        };
        let baseline = saved.get_untracked();
        let mut draft = Snapshot {
            revision: baseline.revision,
            spec: definition,
            records: next,
        };
        if !same_content(&draft, &baseline) {
            let Some(revision) = draft.revision.checked_add(1) else {
                status.set(Status::StorageError);
                return;
            };
            draft.revision = revision;
        }
        let raw = match encode_saved(&draft) {
            Ok(raw) => raw,
            Err(_) => {
                status.set(Status::StorageError);
                return;
            }
        };
        let expected_storage = persisted_bytes.get_untracked().unwrap_or_default();
        saving.set(true);
        status.set(Status::Saving);
        leptos::task::spawn_local(async move {
            let result = write_local_snapshot(STORAGE_KEY, &expected_storage, &raw).await;
            match result {
                Ok(value) => match value.as_string().and_then(|raw| decode_saved(&raw).ok()) {
                    Some(receipt)
                        if receipt.revision == draft.revision && same_content(&receipt, &draft) =>
                    {
                        persisted_bytes.set(Some(raw.clone()));
                        spec.set(receipt.spec.clone());
                        records.set(receipt.records.clone());
                        saved.set(receipt);
                        buffers.set(BTreeMap::new());
                        errors.set(BTreeMap::new());
                        status.set(Status::Saved);
                    }
                    _ => status.set(Status::StorageError),
                },
                Err(error) => {
                    let message = js_sys::Reflect::get(&error, &JsValue::from_str("message"))
                        .ok()
                        .and_then(|v| v.as_string());
                    status.set(match message.as_deref() {
                        Some("conflict") => Status::Conflict,
                        Some("unsupported") => Status::Unsupported,
                        _ => Status::StorageError,
                    });
                }
            }
            saving.set(false);
        });
    };
    let cancel = move |_| {
        let snapshot = saved.get_untracked();
        app_name_input.set(snapshot.spec.name.clone());
        spec.set(snapshot.spec);
        records.set(snapshot.records);
        buffers.set(BTreeMap::new());
        errors.set(BTreeMap::new());
        import_errors.set(Vec::new());
        status.set(Status::Clean);
    };
    let add_field = move |_| {
        let mut next = spec.get_untracked();
        let mut index = 1;
        while next.tables.iter().any(|t| {
            t.fields
                .iter()
                .any(|f| f.id.as_str() == format!("fld_local_{index}"))
        }) {
            index += 1;
        }
        let Some(table) = next
            .tables
            .iter_mut()
            .find(|t| t.id == table_id.get_untracked())
        else {
            return;
        };
        let name = field_name.get_untracked();
        if name.trim().is_empty() {
            errors.set(BTreeMap::from([(
                "field".to_owned(),
                tr(
                    locale.get_untracked(),
                    "Nhập tên trường.",
                    "Enter a field name.",
                ),
            )]));
            return;
        }
        let kind = match field_type.get_untracked().as_str() {
            "integer" => FieldType::Integer,
            "bool" => FieldType::Bool,
            "money" => FieldType::Money,
            _ => FieldType::Text,
        };
        table.fields.push(RawField {
            id: FieldId::new(format!("fld_local_{index}")).expect("generated stable identity"),
            name,
            required: false,
            field_type: kind,
        });
        match CheckedAppSpec::compile(next.clone()) {
            Ok(_) => {
                spec.set(next);
                field_name.set(String::new());
                errors.set(BTreeMap::new());
                status.set(Status::Dirty);
            }
            Err(error) => {
                errors.set(BTreeMap::from([(
                    "field".to_owned(),
                    diagnostic(&error, locale.get_untracked()),
                )]));
            }
        }
    };
    let import = move |_| match prepare_import(
        &spec.get_untracked(),
        &records.get_untracked(),
        &table_id.get_untracked(),
        &import_source.get_untracked(),
        locale.get_untracked(),
    ) {
        Ok(next) => {
            records.set(next);
            import_errors.set(Vec::new());
            status.set(Status::Dirty);
        }
        Err(diagnostics) => {
            import_errors.set(diagnostics);
            status.set(Status::Invalid);
        }
    };

    view! {
        <div class="shell">
            <header class="topbar">
                <div class="brand"><span class="brand-mark" aria-hidden="true">"E"</span>"EvoBase" <span class="local-tag">{move || tr(locale.get(), "Nháp trên thiết bị", "Local draft")}</span></div>
                <div class="top-controls">
                    <button data-testid="locale" on:click=move |_| {
                        locale.update(|v| *v = !*v);
                        if !errors.get_untracked().is_empty() {
                            let mut definition = spec.get_untracked(); definition.name = app_name_input.get_untracked();
                            if let Err(diagnostics) = candidate(&definition, &records.get_untracked(), &buffers.get_untracked(), locale.get_untracked()) { errors.set(diagnostics); }
                            else if errors.get_untracked().contains_key("field") && field_name.get_untracked().trim().is_empty() { errors.set(BTreeMap::from([("field".to_owned(), tr(locale.get_untracked(), "Nhập tên trường.", "Enter a field name."))])); }
                        }
                        if !import_errors.get_untracked().is_empty() {
                            import_errors.set(prepare_import(&spec.get_untracked(), &records.get_untracked(), &table_id.get_untracked(), &import_source.get_untracked(), locale.get_untracked()).err().unwrap_or_default());
                        }
                        if let Some(document) = web_sys::window().and_then(|w| w.document()) && let Some(root) = document.document_element() { let _ = root.set_attribute("lang", if locale.get_untracked() { "vi" } else { "en" }); }
                    }>{move || tr(locale.get(), "English", "Tiếng Việt")}</button>
                    <button data-testid="theme" aria-pressed=move || dark.get().to_string() on:click=move |_| {
                        dark.update(|v| *v = !*v);
                        if let Some(document) = web_sys::window().and_then(|w| w.document()) && let Some(root) = document.document_element() { let _ = root.set_attribute("data-theme", if dark.get_untracked() { "dark" } else { "light" }); }
                    }>{move || tr(locale.get(), if dark.get() { "Giao diện sáng" } else { "Giao diện tối" }, if dark.get() { "Light theme" } else { "Dark theme" })}</button>
                </div>
            </header>
            <div class="layout">
                <nav class="rail" aria-label=move || tr(locale.get(), "Bảng của ứng dụng", "Application tables")>
                    <span class="eyebrow">{move || tr(locale.get(), "Ứng dụng của bạn", "Your application")}</span>
                    <strong data-testid="app-name">{move || spec.get().name}</strong>
                    {move || spec.get().tables.into_iter().map(|table| {
                        let id = table.id.clone(); let selected_id = id.clone();
                        view! { <button data-testid=format!("table-{}", id.as_str()) class:selected=move || table_id.get() == selected_id aria-pressed=move || (table_id.get() == table.id).to_string() on:click=move |_| select_table(id.clone())>{table.name}</button> }
                    }).collect_view()}
                    <small>{move || tr(locale.get(), "Dữ liệu mẫu · không kết nối máy chủ", "Synthetic data · no server connection")}</small>
                </nav>
                <main class="workspace">
                    <section class="overview">
                        <span class="eyebrow">{move || tr(locale.get(), "Builder / Bản nháp", "Builder / Draft")}</span>
                        <h1>{move || tr(locale.get(), "Định hình dữ liệu của bạn", "Shape your application data")}</h1>
                        <p>{move || tr(locale.get(), "Chỉnh sửa bản nháp cục bộ, kiểm tra kiểu dữ liệu và giữ lại trên trình duyệt. Bản nháp chưa được phát hành.", "Edit a local draft, check typed data, and keep it in this browser. This draft is not published.")}</p>
                    </section>
                    <div class="toolbar">
                        <div><h2>{move || spec.get().tables.iter().find(|t| t.id == table_id.get()).map(|t| t.name.clone()).unwrap_or_default()}</h2>
                            <p class="support">{move || format!("{} {} · {} {}", records.get().iter().filter(|r| r.table_id == table_id.get()).count(), tr(locale.get(), "bản ghi", "records"), tr(locale.get(), "Nháp cục bộ phiên bản", "Local draft revision"), saved.get().revision)}</p>
                        </div>
                        <div class="desktop-authoring"><div class="actions">
                            <button data-testid="cancel" disabled=move || saving.get() on:click=cancel>{move || tr(locale.get(), "Hủy chỉnh sửa", "Cancel edits")}</button>
                            <button class="primary" data-testid="save" disabled=move || restoration_failed || paused.get() || composing.get() || saving.get() on:click=save>{move || tr(locale.get(), "Giữ nháp trên thiết bị", "Save local draft")}</button>
                        </div></div>
                    </div>
                    <p class="status" data-testid="draft-status" role="status" data-state=move || match status.get() { Status::Invalid | Status::Conflict | Status::Expired | Status::StorageError | Status::Unsupported => "error", Status::Saved => "saved", _ => "normal" }>
                        {move || match status.get() {
                            Status::Saving => tr(locale.get(), "Đang giữ nháp trên thiết bị…", "Saving local draft…"),
                            Status::Clean => tr(locale.get(), "Nháp cục bộ · chưa phát hành", "Local draft · not published"),
                            Status::Dirty => tr(locale.get(), "Có thay đổi chưa được giữ trên thiết bị", "Changes are not yet saved on this device"),
                            Status::Saved => tr(locale.get(), "Đã giữ nháp trên thiết bị · chưa đồng bộ máy chủ", "Saved on this device · not synchronized to a server"),
                            Status::Invalid => tr(locale.get(), "Kiểm tra trường được đánh dấu. Nội dung nhập được giữ nguyên.", "Check the marked fields. Your input is preserved."),
                            Status::Conflict => tr(locale.get(), "Nháp đã thay đổi ở cửa sổ khác. Nội dung hiện tại còn trong cửa sổ này. Tải lại sẽ bỏ thay đổi chưa lưu để xem nháp đã lưu.", "The draft changed in another tab. Your input remains in this window. Reload discards unsaved changes to review the saved draft."),
                            Status::Expired => tr(locale.get(), "Mô phỏng phiên hết hạn · nháp giữ nguyên", "Simulated expired session · draft preserved"),
                            Status::Unsupported => tr(locale.get(), "Trình duyệt chưa hỗ trợ khóa ghi giữa các cửa sổ. Nháp được giữ trong cửa sổ này; dùng trình duyệt mới để lưu an toàn.", "This browser does not support cross-tab write locks. Your draft remains in this window; use a current browser to save safely."),
                            Status::StorageError => tr(locale.get(), "Không thể đọc hoặc giữ nháp trên thiết bị. Nội dung hiện tại vẫn còn trong cửa sổ này.", "Local storage could not be read or written. Current input remains in this window."),
                        }}
                    </p>
                    <Show when=move || restoration_failed><p class="error-panel" role="alert" data-testid="restore-error">{move || tr(locale.get(), "Bản nháp đã lưu không thể đọc an toàn. Dữ liệu gốc được giữ nguyên và nút lưu bị khóa trong cửa sổ này.", "The stored draft could not be restored safely. Original bytes are preserved and saving is disabled in this window.")}</p></Show>
                    <section class="compact-handoff surface" data-testid="compact-handoff">
                        <h2>{move || tr(locale.get(), "Xem tóm tắt bản nháp", "Draft summary")}</h2>
                        <p>{move || tr(locale.get(), "Mở Builder trên màn hình rộng để chỉnh sửa bảng và trường.", "Open Builder on a wider screen to edit tables and fields.")}</p>
                        {move || spec.get().tables.iter().find(|t| t.id == table_id.get()).map(|table| table.fields.iter().map(|field| view! { <div class="field-row"><strong>{field.name.clone()}</strong><span>{type_label(&field.field_type, locale.get())}</span></div> }).collect_view())}
                    </section>
                    <div class="desktop-authoring">
                        <div class="content-columns">
                            <div class="workspace">
                                <section class="surface grid-surface">
                                    <div class="grid-header"><h3>{move || tr(locale.get(), "Dữ liệu bản nháp", "Draft data")}</h3><div class="actions">
                                        <button data-testid="view-table" class:selected=move || !form_mode.get() aria-pressed=move || (!form_mode.get()).to_string() on:click=move |_| form_mode.set(false)>{move || tr(locale.get(), "Bảng", "Table")}</button>
                                        <button data-testid="view-form" class:selected=move || form_mode.get() aria-pressed=move || form_mode.get().to_string() on:click=move |_| form_mode.set(true)>{move || tr(locale.get(), "Biểu mẫu", "Form")}</button>
                                    </div></div>
                                    <Show when=move || !form_mode.get() fallback=move || view! {
                                        <div class="surface"><p class="support">{move || tr(locale.get(), "Biểu mẫu từ cùng AppSpec · bản ghi đầu tiên", "Form from the same AppSpec · first record")}</p><div class="preview-form">
                                            <For each=move || {
                                                let selected = table_id.get(); let definition = spec.get();
                                                let row = records.get().into_iter().find(|r| r.table_id == selected);
                                                definition.tables.into_iter().find(|t| t.id == selected).and_then(|table| row.map(|row| table.fields.into_iter().map(|field| (selected.clone(), row.id.clone(), field)).collect::<Vec<_>>())).unwrap_or_default()
                                            } key=|(table, record, field)| (table.clone(), record.clone(), field.id.clone())
                                            children=move |(table, record, field)| view! { <label>{field.name.clone()}<FieldEditor spec records buffers errors status composing table record field locale saving paused context="form"/></label> }/>

                                        </div></div>
                                    }>
                                        <div class="table-scroll"><table data-testid="typed-table">
                                            <caption>{move || tr(locale.get(), "Nhập trực tiếp · Enter/Tab kiểm tra · Escape hủy ô · null khác ô trống", "Edit directly · Enter/Tab validate · Escape cancels a cell · null differs from blank")}</caption>
                                            <thead><tr><th scope="col">"#"</th>{move || spec.get().tables.into_iter().find(|t| t.id == table_id.get()).map(|table| table.fields.into_iter().map(|field| view! { <th scope="col">{field.name}<small class="support">" · "{type_label(&field.field_type, locale.get())}</small></th> }).collect_view())}</tr></thead>
                                            <tbody><For
                                                each=move || { let selected = table_id.get(); records.get().into_iter().filter(|r| r.table_id == selected).collect::<Vec<_>>() }
                                                key=|row| (row.table_id.clone(), row.id.clone())
                                                children=move |row| {
                                                    let row_table = row.table_id.clone(); let row_id = row.id.clone();
                                                    let table_fields = row.table_id.clone(); let row_index = row.id.clone(); let table_index = row.table_id.clone();
                                                    view! { <tr data-record-id=row.id.to_string()><td>{move || records.get().iter().filter(|r| r.table_id == table_index).position(|r| r.id == row_index).map(|i| i+1).unwrap_or(0)}</td>
                                                        <For each=move || spec.get().tables.into_iter().find(|t| t.id == table_fields).map(|t| t.fields).unwrap_or_default()
                                                            key=|field| field.id.clone()
                                                            children=move |field| view! { <td><FieldEditor spec records buffers errors status composing table=row_table.clone() record=row_id.clone() field locale saving paused/></td> }/>
                                                    </tr> }
                                                }/>
                                            </tbody>
                                        </table></div>
                                    </Show>
                                </section>
                                <section class="surface">
                                    <h3>{move || tr(locale.get(), "Dán dữ liệu có kiểm tra", "Checked paste / import")}</h3>
                                    <p class="support">{move || tr(locale.get(), "TSV: hàng đầu dùng định danh trường, tối đa 100 hàng / 64 KiB. Một lỗi sẽ giữ toàn bộ dữ liệu gốc, không thêm từng phần. Liên kết dùng định danh hoặc tên khớp duy nhất.", "TSV: header uses field identities, at most 100 rows / 64 KiB. An error preserves the source and adds no partial rows. References require an identity or a unique exact label.")}</p>
                                    <details><summary>{move || tr(locale.get(), "Tiêu đề để sao chép", "Header to copy")}</summary><code data-testid="import-header">{move || spec.get().tables.iter().find(|t| t.id == table_id.get()).map(|t| t.fields.iter().map(|f| f.id.as_str()).collect::<Vec<_>>().join("\t")).unwrap_or_default()}</code></details>
                                    <label>{move || tr(locale.get(), "Dữ liệu TSV", "TSV input")}<textarea data-testid="import-source" prop:value=move || import_source.get() on:input=move |event| import_source.set(event_target_value(&event))></textarea></label>
                                    <div class="actions"><button data-testid="import-apply" disabled=move || paused.get() || saving.get() on:click=import>{move || tr(locale.get(), "Kiểm tra và thêm toàn bộ", "Validate and add all rows")}</button></div>
                                    <Show when=move || !import_errors.get().is_empty()><div class="error-panel" role="alert" data-testid="import-errors"><strong>{move || tr(locale.get(), "Chưa thêm dữ liệu. Sửa nguồn và kiểm tra lại.", "No rows added. Correct the source and retry.")}</strong><ul>{move || import_errors.get().into_iter().map(|error| view!{<li>{error}</li>}).collect_view()}</ul></div></Show>
                                </section>
                            </div>
                            <aside class="inspector workspace" aria-label=move || tr(locale.get(), "Cấu trúc bản nháp", "Draft structure")>
                                <section class="surface">
                                    <h3>{move || tr(locale.get(), "Ứng dụng và trường", "Application and fields")}</h3>
                                    <label>{move || tr(locale.get(), "Tên ứng dụng", "Application name")}<input data-testid="rename-app" aria-invalid=move || errors.get().contains_key("definition").to_string() aria-describedby="app-name-error" disabled=move || saving.get() || paused.get() prop:value=move || app_name_input.get() on:input=move |event| {
                                        app_name_input.set(event_target_value(&event)); status.set(Status::Dirty);
                                    } on:change=move |_| {
                                        let mut next = spec.get_untracked(); next.name = app_name_input.get_untracked();
                                        match CheckedAppSpec::compile(next.clone()) { Ok(_) => { spec.set(next); errors.update(|e| { e.remove("definition"); }); status.set(Status::Dirty); }, Err(error) => { errors.set(BTreeMap::from([("definition".to_owned(), diagnostic(&error, locale.get_untracked()))])); status.set(Status::Invalid); } }
                                    }/></label>
                                    <div class="field-list">{move || spec.get().tables.into_iter().find(|t| t.id == table_id.get()).map(|table| table.fields.into_iter().map(|field| view! { <div class="field-row"><div><strong>{field.name}</strong><small>{type_label(&field.field_type, locale.get())}</small></div><span class="support">{tr(locale.get(), if field.required { "Bắt buộc" } else { "Tùy chọn" }, if field.required { "Required" } else { "Optional" })}</span></div> }).collect_view())}</div>
                                    <div class="field-form">
                                        <label>{move || tr(locale.get(), "Tên trường tùy chọn mới", "New optional field name")}<input data-testid="field-name" prop:value=move || field_name.get() on:input=move |event| field_name.set(event_target_value(&event))/></label>
                                        <label>{move || tr(locale.get(), "Kiểu dữ liệu", "Field type")}<select data-testid="field-type" on:change=move |event| field_type.set(event_target_value(&event))>
                                            <option value="text">{move || tr(locale.get(), "Văn bản", "Text")}</option><option value="integer">{move || tr(locale.get(), "Số nguyên", "Integer")}</option><option value="bool">{move || tr(locale.get(), "Đúng / sai", "Boolean")}</option><option value="money">{move || tr(locale.get(), "Tiền · đơn vị nhỏ nhất", "Money · minor units")}</option>
                                        </select></label>
                                        <button data-testid="add-field" disabled=move || paused.get() || saving.get() on:click=add_field>{move || tr(locale.get(), "Thêm vào bản nháp", "Add to draft")}</button>
                                    </div>
                                    <Show when=move || errors.get().contains_key("field") || errors.get().contains_key("definition")><p class="error-panel" role="alert" id="app-name-error">{move || { let diagnostics = errors.get(); diagnostics.get("field").or_else(|| diagnostics.get("definition")).cloned().unwrap_or_default() }}</p></Show>
                                </section>
                                <constraints::ConstraintInspector spec rows=records table_id locale saving paused on_source_change=Callback::new(move |_| status.set(Status::Dirty))/>
                                <relations::RelationInspector locale/>
                                <policy::PolicyInspector spec rows=records table_id locale saving=saving paused=paused on_source_change=Callback::new(move |_| status.set(Status::Dirty))/>
                                <section class="surface"><details><summary>{move || tr(locale.get(), "Mô phỏng gián đoạn", "Interruption simulation")}</summary><p class="support">{move || tr(locale.get(), "Chỉ thử trạng thái cục bộ. Không có đăng nhập hay phiên máy chủ.", "Local state exercise only. There is no sign-in or server session.")}</p><button data-testid="session-toggle" disabled=move || saving.get() on:click=move |_| { paused.update(|v| *v = !*v); status.set(if paused.get_untracked() { Status::Expired } else { Status::Dirty }); }>{move || tr(locale.get(), if paused.get() { "Tiếp tục chỉnh nháp" } else { "Mô phỏng phiên hết hạn" }, if paused.get() { "Resume draft editing" } else { "Simulate session expiry" })}</button></details></section>
                            </aside>
                        </div>
                    </div>
                    <footer class="footer">{move || tr(locale.get(), "Kiểm tra dữ liệu · giữ nháp trong trình duyệt · chưa phát hành", "Checked data · draft kept in this browser · not published")}</footer>
                </main>
            </div>
        </div>
    }
}

fn table_from_hash(spec: &RawAppSpec) -> Option<TableId> {
    let hash = web_sys::window()?.location().hash().ok()?;
    let id = TableId::new(hash.trim_start_matches('#')).ok()?;
    spec.tables.iter().any(|t| t.id == id).then_some(id)
}

fn focus_first_error() {
    leptos::task::spawn_local(async {
        let _ = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
            .await;
        if let Some(document) = web_sys::window().and_then(|window| window.document())
            && let Ok(Some(element)) = document.query_selector("input[aria-invalid='true']")
            && let Some(element) = element.dyn_ref::<web_sys::HtmlElement>()
        {
            let _ = element.focus();
        }
    });
}

#[wasm_bindgen(start)]
pub fn start() {
    leptos::mount::mount_to_body(App);
}
