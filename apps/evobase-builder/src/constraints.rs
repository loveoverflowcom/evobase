//! Structured authoring of canonical constraints; Rust validates the whole dataset.
use crate::{diagnostic, scope_for, tr};
use evobase_appspec::{
    CheckedAppSpec, Constraint, ConstraintId, Error, FORMAT_VERSION, FieldType, RawAppSpec,
    RawField, RawFieldConstraint, RawRecord, TableId,
};
use leptos::prelude::*;

#[derive(Clone)]
enum Notice {
    Ready,
    Applied,
    Removed,
    InvalidBounds,
    Error(Error),
}

fn selected_field(spec: &RawAppSpec, table: &TableId, selected: &str) -> Option<RawField> {
    let fields = &spec.tables.iter().find(|t| &t.id == table)?.fields;
    let eligible = |field: &&RawField| {
        matches!(
            field.field_type,
            FieldType::Text | FieldType::Integer | FieldType::Money
        )
    };
    fields
        .iter()
        .filter(eligible)
        .find(|field| field.id.as_str() == selected)
        .or_else(|| fields.iter().find(eligible))
        .cloned()
}

fn validate_source(next: RawAppSpec, rows: &[RawRecord]) -> Result<RawAppSpec, Error> {
    let checked = CheckedAppSpec::compile(next)?;
    checked.validate_records(&scope_for(checked.definition()), rows)?;
    Ok(checked.definition().clone())
}

fn constraint_label(rule: &Constraint, vi: bool) -> String {
    match rule {
        Constraint::NonEmpty => tr(
            vi,
            "Có nội dung ngoài khoảng trắng",
            "Contains non-whitespace text",
        ),
        Constraint::TextLength { min, max } => format!(
            "{} {min}–{max}",
            tr(vi, "Độ dài ký tự:", "Character length:")
        ),
        Constraint::NumericRange { min, max } => {
            format!("{} {min}–{max}", tr(vi, "Khoảng giá trị:", "Value range:"))
        }
    }
}

#[component]
pub fn ConstraintInspector(
    spec: RwSignal<RawAppSpec>,
    rows: RwSignal<Vec<RawRecord>>,
    table_id: RwSignal<TableId>,
    locale: RwSignal<bool>,
    saving: RwSignal<bool>,
    paused: RwSignal<bool>,
    on_source_change: Callback<()>,
) -> impl IntoView {
    let selected = RwSignal::new(String::new());
    let kind = RwSignal::new("non_empty".to_owned());
    let lower = RwSignal::new("0".to_owned());
    let upper = RwSignal::new("256".to_owned());
    let notice = RwSignal::new(Notice::Ready);
    let field = move || selected_field(&spec.get(), &table_id.get(), &selected.get());
    let disabled = move || saving.get() || paused.get();
    let add = move |_| {
        if disabled() {
            return;
        }
        let mut next = spec.get_untracked();
        let table = table_id.get_untracked();
        let Some(field) = selected_field(&next, &table, &selected.get_untracked()) else {
            return;
        };
        let constraint = match field.field_type {
            FieldType::Text if kind.get_untracked() != "text_length" => Constraint::NonEmpty,
            FieldType::Text => match (
                lower.get_untracked().parse::<usize>(),
                upper.get_untracked().parse::<usize>(),
            ) {
                (Ok(min), Ok(max)) => Constraint::TextLength { min, max },
                _ => {
                    notice.set(Notice::InvalidBounds);
                    return;
                }
            },
            FieldType::Integer | FieldType::Money => match (
                lower.get_untracked().parse::<i64>(),
                upper.get_untracked().parse::<i64>(),
            ) {
                (Ok(min), Ok(max)) => Constraint::NumericRange { min, max },
                _ => {
                    notice.set(Notice::InvalidBounds);
                    return;
                }
            },
            _ => return,
        };
        let mut suffix = 1;
        while next
            .constraints
            .iter()
            .any(|rule| rule.constraint_id.as_str() == format!("constraint_local_{suffix}"))
        {
            suffix += 1;
        }
        next.version = FORMAT_VERSION;
        next.constraints.push(RawFieldConstraint {
            constraint_id: ConstraintId::new(format!("constraint_local_{suffix}"))
                .expect("generated constraint identity"),
            table_id: table,
            field_id: field.id,
            constraint,
        });
        match validate_source(next, &rows.get_untracked()) {
            Ok(next) => {
                spec.set(next);
                notice.set(Notice::Applied);
                on_source_change.run(());
            }
            Err(error) => notice.set(Notice::Error(error)),
        }
    };
    view! {
        <section class="surface constraint-panel" data-testid="constraint-inspector">
            <h3>{move || tr(locale.get(), "Quy tắc dữ liệu", "Data rules")}</h3>
            <p class="support">{move || tr(locale.get(), "Quy tắc áp dụng cho ô nhập, biểu mẫu và dữ liệu dán. Dữ liệu hiện có phải hợp lệ trước khi thêm quy tắc. Ô tùy chọn trống hoặc null được bỏ qua.", "Rules apply to cells, forms and pasted data. Existing data must satisfy a new rule. Optional blank or null values are skipped.")}</p>
            <ul class="constraint-list" data-testid="constraint-source">
                {move || {
                    let definition = spec.get();
                    let table = table_id.get();
                    definition.constraints.iter().filter(|rule| rule.table_id == table).map(|rule| {
                        let id = rule.constraint_id.clone();
                        let title = definition.tables.iter().find(|t| t.id == table).and_then(|t| t.fields.iter().find(|f| f.id == rule.field_id)).map(|f| f.name.clone()).unwrap_or_default();
                        let rule_kind = rule.constraint.clone();
                        view! { <li><strong>{title}</strong><span>{move || constraint_label(&rule_kind, locale.get())}</span><button data-testid=format!("constraint-remove-{id}") disabled=disabled on:click=move |_| {
                            if disabled() { return; }
                            let mut next = spec.get_untracked(); next.constraints.retain(|rule| rule.constraint_id != id);
                            match validate_source(next, &rows.get_untracked()) { Ok(next) => { spec.set(next); notice.set(Notice::Removed); on_source_change.run(()); }, Err(error) => notice.set(Notice::Error(error)) }
                        }>{move || tr(locale.get(), "Bỏ quy tắc", "Remove rule")}</button></li> }
                    }).collect_view()
                }}
            </ul>
            <Show when=move || field().is_some() fallback=move || view! { <p>{move || tr(locale.get(), "Chọn bảng có trường văn bản hoặc số để thêm quy tắc.", "Choose a table with a text or numeric field to add rules.")}</p> }>
                <label>{move || tr(locale.get(), "Trường", "Field")}<select data-testid="constraint-field" disabled=disabled prop:value=move || field().map(|f| f.id.to_string()).unwrap_or_default() on:change=move |event| {
                    selected.set(event_target_value(&event)); notice.set(Notice::Ready);
                    if field().is_some_and(|f| matches!(f.field_type, FieldType::Text)) { kind.set("non_empty".to_owned()); lower.set("0".to_owned()); upper.set("256".to_owned()); }
                    else { kind.set("numeric_range".to_owned()); lower.set("0".to_owned()); upper.set(i64::MAX.to_string()); }
                }>{move || spec.get().tables.into_iter().find(|t| t.id == table_id.get()).map(|table| table.fields.into_iter().filter(|f| matches!(f.field_type, FieldType::Text | FieldType::Integer | FieldType::Money)).map(|f| view! { <option value=f.id.to_string()>{f.name}</option> }).collect_view())}</select></label>
                <label>{move || tr(locale.get(), "Điều kiện", "Condition")}<select data-testid="constraint-kind" disabled=disabled prop:value=move || if field().is_some_and(|f| matches!(f.field_type, FieldType::Text)) { if kind.get() == "text_length" { "text_length".to_owned() } else { "non_empty".to_owned() } } else { "numeric_range".to_owned() } on:change=move |event| { kind.set(event_target_value(&event)); notice.set(Notice::Ready); }>
                    {move || if field().is_some_and(|f| matches!(f.field_type, FieldType::Text)) { view! { <option value="non_empty">{move || tr(locale.get(), "Có nội dung", "Non-empty text")}</option><option value="text_length">{move || tr(locale.get(), "Độ dài ký tự", "Character length")}</option> }.into_any() } else { view! { <option value="numeric_range">{move || tr(locale.get(), "Khoảng giá trị", "Value range")}</option> }.into_any() }}
                </select></label>
                <Show when=move || kind.get() != "non_empty" || field().is_some_and(|f| !matches!(f.field_type, FieldType::Text))>
                    <label>{move || tr(locale.get(), "Tối thiểu (bao gồm)", "Minimum (inclusive)")}<input data-testid="constraint-min" disabled=disabled inputmode="numeric" prop:value=move || lower.get() on:input=move |event| lower.set(event_target_value(&event))/></label>
                    <label>{move || tr(locale.get(), "Tối đa (bao gồm)", "Maximum (inclusive)")}<input data-testid="constraint-max" disabled=disabled inputmode="numeric" prop:value=move || upper.get() on:input=move |event| upper.set(event_target_value(&event))/></label>
                </Show>
                <button data-testid="constraint-add" disabled=disabled on:click=add>{move || tr(locale.get(), "Kiểm tra và thêm quy tắc", "Validate and add rule")}</button>
            </Show>
            <p data-testid="constraint-status" role="status" class:error-panel=move || matches!(notice.get(), Notice::Error(_) | Notice::InvalidBounds)>{move || match notice.get() {
                Notice::Ready => tr(locale.get(), "Thêm quy tắc cho trường đã chọn.", "Add a rule for the selected field."),
                Notice::Applied => tr(locale.get(), "Đã thêm quy tắc vào bản nháp. Dữ liệu hiện có hợp lệ.", "Rule added to the draft. Existing data satisfies it."),
                Notice::Removed => tr(locale.get(), "Đã bỏ quy tắc khỏi bản nháp.", "Rule removed from the draft."),
                Notice::InvalidBounds => tr(locale.get(), "Nhập giới hạn bằng số nguyên trong phạm vi hỗ trợ.", "Enter integer bounds within the supported range."),
                Notice::Error(error) => format!("{} {}", tr(locale.get(), "Chưa thay đổi quy tắc.", "Rules were not changed."), diagnostic(&error, locale.get())),
            }}</p>
        </section>
    }
}
