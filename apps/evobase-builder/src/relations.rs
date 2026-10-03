//! Local relation laboratory: the production checked store owns captures and graph edits.
use crate::tr;
use evobase_appspec::relations::{
    BatchChange, CheckedFormulas, Expression, FormulaDefinition, FormulaLimits, LocalPreviewPolicy,
    Numeric, RelationError, RelationStore,
};
use evobase_appspec::{CheckedAppSpec, FieldId, RecordId, Scope, TableId, Value, fixtures};
use leptos::prelude::*;

fn table(id: &str) -> TableId {
    TableId::new(id).expect("fixture table")
}
fn field(id: &str) -> FieldId {
    FieldId::new(id).expect("fixture field")
}
fn record(id: &str) -> RecordId {
    RecordId::new(id).expect("fixture record")
}

struct Session {
    store: RelationStore,
    formulas: CheckedFormulas,
    next_line: u64,
}
fn session() -> Result<Session, RelationError> {
    let spec = CheckedAppSpec::compile(fixtures::example_spec())?;
    let scope = Scope::new("tenant_relation_preview", spec.app_id().clone())?;
    let mut rows = fixtures::example_records(&scope);
    for row in &mut rows {
        for capture in &spec.definition().capture_rules {
            if row.table_id == capture.line_table {
                row.values.remove(&capture.captured_price_field);
            }
        }
    }
    let mut store = RelationStore::new(spec, scope)?;
    store.apply_batch(rows.into_iter().map(BatchChange::Insert).collect())?;
    let line = table("tbl_order_lines");
    let formulas = CheckedFormulas::compile(
        store.spec(),
        vec![
            FormulaDefinition {
                table: line.clone(),
                name: "captured".to_owned(),
                expression: Expression::Field(field("fld_line_price")),
            },
            FormulaDefinition {
                table: line.clone(),
                name: "live".to_owned(),
                expression: Expression::Lookup {
                    ref_field: field("fld_line_product"),
                    target_field: field("fld_product_price"),
                },
            },
            FormulaDefinition {
                table: line.clone(),
                name: "subtotal".to_owned(),
                expression: Expression::Product(
                    Box::new(Expression::Field(field("fld_line_quantity"))),
                    Box::new(Expression::Field(field("fld_line_price"))),
                ),
            },
            FormulaDefinition {
                table: table("tbl_orders"),
                name: "total".to_owned(),
                expression: Expression::Sum {
                    child_table: line,
                    ref_field: field("fld_line_order"),
                    value: Box::new(Expression::Named("subtotal".to_owned())),
                },
            },
        ],
        FormulaLimits::default(),
    )?;
    Ok(Session {
        store,
        formulas,
        next_line: 2,
    })
}

#[derive(Clone)]
enum Notice {
    Ready,
    PriceChanged,
    LineAdded,
    Error(RelationError),
}
fn message(vi: bool, notice: &Notice) -> String {
    match notice {
        Notice::Ready => tr(
            vi,
            "Các tham chiếu dùng ID ổn định. Giá đã chốt được lưu khi tạo dòng.",
            "References use stable IDs. Captured prices are recorded when a line is created.",
        ),
        Notice::PriceChanged => tr(
            vi,
            "Giá hiện tại đã đổi; giá đã chốt của các dòng cũ được giữ nguyên.",
            "The live price changed; older lines keep their captured price.",
        ),
        Notice::LineAdded => tr(
            vi,
            "Đã thêm dòng mới với giá hiện tại được chốt.",
            "A new line captured the current price.",
        ),
        Notice::Error(RelationError::RestrictDelete {
            source_record,
            field,
            ..
        }) => format!(
            "{} {source_record} / {field}",
            tr(
                vi,
                "Không thể xóa: còn dòng tham chiếu.",
                "Cannot delete: a line still references this product."
            )
        ),
        Notice::Error(RelationError::Overflow) => tr(
            vi,
            "Phép tính vượt giới hạn số nguyên 64 bit.",
            "The calculation exceeds the signed 64-bit range.",
        ),
        Notice::Error(RelationError::OutputDenied) => tr(
            vi,
            "Quyền hiện tại không cho phép hiển thị kết quả này.",
            "Current output policy denies this projection.",
        ),
        Notice::Error(RelationError::Kernel(evobase_appspec::Error::InvalidCell { .. })) => tr(
            vi,
            "Nhập giá bằng số nguyên đơn vị nhỏ nhất, ví dụ 2000.",
            "Enter the price as integer minor units, for example 2000.",
        ),
        Notice::Error(RelationError::Kernel(evobase_appspec::Error::Required {
            field, ..
        })) => format!(
            "{} {field}",
            tr(
                vi,
                "Thiếu giá trị bắt buộc:",
                "A required value is missing:"
            )
        ),
        Notice::Error(_) => tr(
            vi,
            "Thao tác không hợp lệ. Dữ liệu trước đó được giữ nguyên.",
            "The operation is invalid. The previous facts were preserved.",
        ),
    }
}
fn numeric(value: Numeric, vi: bool) -> String {
    match value {
        Numeric::Money(value) | Numeric::Integer(value) => value.to_string(),
        Numeric::Null(_) => tr(vi, "Trống", "Null"),
    }
}

#[component]
pub fn RelationInspector(locale: RwSignal<bool>) -> impl IntoView {
    let session = StoredValue::new(session());
    let revision = RwSignal::new(0u64);
    let price = RwSignal::new("2000".to_owned());
    let notice = RwSignal::new(Notice::Ready);

    let apply_price = move |_| {
        let Some(result) = session.try_update_value(|session| -> Result<(), RelationError> {
            let session = session.as_mut().map_err(|error| error.clone())?;
            let product_table = table("tbl_products");
            let product_id = record("rec_product_1");
            let price_field = field("fld_product_price");
            let value = session.store.spec().parse_cell(
                session.store.records().scope(),
                &product_table,
                &price_field,
                &price.get_untracked(),
                &session.store.records().to_raw(),
            )?;
            let mut product = session
                .store
                .records()
                .find(&product_table, &product_id)
                .expect("fixture product")
                .as_raw()
                .clone();
            product.values.insert(price_field, value);
            session
                .store
                .apply_batch(vec![BatchChange::Replace(product)])
        }) else {
            return;
        };
        match result {
            Ok(()) => {
                revision.update(|r| *r += 1);
                notice.set(Notice::PriceChanged);
            }
            Err(error) => notice.set(Notice::Error(error)),
        }
    };
    let add_line = move |_| {
        let Some(result) = session.try_update_value(|session| -> Result<(), RelationError> {
            let session = session.as_mut().map_err(|error| error.clone())?;
            let mut line = session
                .store
                .records()
                .find(&table("tbl_order_lines"), &record("rec_line_1"))
                .expect("fixture line")
                .as_raw()
                .clone();
            line.id = RecordId::new(format!("rec_line_{}", session.next_line))?;
            line.values.remove(&field("fld_line_price"));
            line.values
                .insert(field("fld_line_quantity"), Value::Integer(1));
            session.store.apply_batch(vec![BatchChange::Insert(line)])?;
            session.next_line += 1;
            Ok(())
        }) else {
            return;
        };
        match result {
            Ok(()) => {
                revision.update(|r| *r += 1);
                notice.set(Notice::LineAdded);
            }
            Err(error) => notice.set(Notice::Error(error)),
        }
    };
    let restrict_delete = move |_| {
        let Some(result) = session.try_update_value(|session| -> Result<(), RelationError> {
            let session = session.as_mut().map_err(|error| error.clone())?;
            session.store.apply_batch(vec![BatchChange::Delete {
                table_id: table("tbl_products"),
                record_id: record("rec_product_1"),
            }])
        }) else {
            return;
        };
        match result {
            Ok(()) => {
                revision.update(|r| *r += 1);
            }
            Err(error) => notice.set(Notice::Error(error)),
        }
    };

    view! {
        <section class="surface relation-inspector" aria-labelledby="relation-heading" data-testid="relation-inspector">
            <h2 id="relation-heading">{move || tr(locale.get(), "Quan hệ và giá đã chốt", "Relations and captured prices")}</h2>
            <p>{move || tr(locale.get(), "Ví dụ độc lập để thử liên kết: Khách hàng → Đơn hàng → Dòng hàng → Sản phẩm. Đơn vị tiền là số nguyên nhỏ nhất.", "Independent local example for relation checks: Customer → Order → OrderLine → Product. Money uses integer minor units.")}</p>
            <p class="support">{move || tr(locale.get(), "Hỗ trợ N:1, chiều ngược được tính và chặn xóa. Quyền truy cập trên máy chủ chưa được kết nối với bản xem trước này.", "Supports N:1, derived reverse edges and restrict delete. Hosted authorization is not connected to this preview.")}</p>
            <div class="toolbar">
                <label for="relation-price">{move || tr(locale.get(), "Giá sản phẩm mới", "New product price")}</label>
                <input id="relation-price" data-testid="relation-price" inputmode="numeric" aria-describedby="relation-status" aria-invalid=move || if matches!(notice.get(), Notice::Error(RelationError::Kernel(evobase_appspec::Error::InvalidCell { .. } | evobase_appspec::Error::Required { .. } | evobase_appspec::Error::Overflow { .. }))) { "true" } else { "false" } prop:value=move || price.get() on:input=move |event| price.set(event_target_value(&event)) />
                <button class="primary" data-testid="relation-change-price" on:click=apply_price>{move || tr(locale.get(), "Đổi giá", "Change price")}</button>
                <button data-testid="relation-add-line" on:click=add_line>{move || tr(locale.get(), "Thêm dòng (×1)", "Add line (×1)")}</button>
                <button data-testid="relation-delete-product" on:click=restrict_delete>{move || tr(locale.get(), "Thử xóa sản phẩm", "Try deleting product")}</button>
            </div>
            <p class="status" data-state=move || if matches!(notice.get(), Notice::Error(_)) { "error" } else { "ready" } role="status" aria-live="polite" data-testid="relation-status">{move || message(locale.get(), &notice.get())}</p>
            <p class="support" id="relation-scroll-hint">{move || tr(locale.get(), "Cuộn ngang để xem giá đã chốt và thành tiền. Dùng phím mũi tên khi vùng bảng được chọn.", "Scroll horizontally to review captured prices and subtotals. Use arrow keys when the table region is focused.")}</p>
            <div class="table-scroll" tabindex="0" aria-label=move || tr(locale.get(), "Bảng so sánh giá và thành tiền", "Price and subtotal comparison") aria-describedby="relation-scroll-hint">
                <table data-testid="relation-lines">
                    <caption>{move || tr(locale.get(), "Dòng hàng tham chiếu đơn rec_order_1", "Lines referencing order rec_order_1")}</caption>
                    <thead><tr>
                        <th scope="col">{move || tr(locale.get(), "ID dòng", "Line ID")}</th>
                        <th scope="col">{move || tr(locale.get(), "Giá hiện tại (lookup)", "Live price (lookup)")}</th>
                        <th scope="col">{move || tr(locale.get(), "Giá đã chốt", "Captured price")}</th>
                        <th scope="col">{move || tr(locale.get(), "Thành tiền lịch sử", "Historical subtotal")}</th>
                    </tr></thead>
                    <tbody>{move || {
                        revision.get();
                        let vi = locale.get();
                        session.with_value(|session| match session {
                            Ok(session) => {
                                let line_table = table("tbl_order_lines");
                                let ids = session.store.reverse_refs(&table("tbl_orders"), &record("rec_order_1"), &line_table, &field("fld_line_order"), &LocalPreviewPolicy);
                                match ids {
                                    Ok(ids) => ids.into_iter().map(|id| {
                                        let values = ["live", "captured", "subtotal"].map(|name| session.formulas.evaluate(&session.store, &line_table, &id, name, &LocalPreviewPolicy));
                                        let display = values.map(|result| result.map(|value| numeric(value, vi)).unwrap_or_else(|error| message(vi, &Notice::Error(error))));
                                        view! { <tr data-record-id=id.as_str().to_owned()><th scope="row">{id.to_string()}</th><td>{display[0].clone()}</td><td>{display[1].clone()}</td><td>{display[2].clone()}</td></tr> }.into_any()
                                    }).collect_view().into_any(),
                                    Err(error) => view! { <tr><td colspan="4">{message(vi, &Notice::Error(error))}</td></tr> }.into_any(),
                                }
                            }
                            Err(error) => view! { <tr><td colspan="4">{message(vi, &Notice::Error(error.clone()))}</td></tr> }.into_any(),
                        })
                    }}</tbody>
                </table>
            </div>
            <p data-testid="relation-total">{move || {
                revision.get();
                let vi = locale.get();
                session.with_value(|session| match session {
                    Ok(session) => match session.formulas.evaluate(&session.store, &table("tbl_orders"), &record("rec_order_1"), "total", &LocalPreviewPolicy) {
                        Ok(value) => format!("{} {}", tr(vi, "Tổng lịch sử:", "Historical total:"), numeric(value, vi)),
                        Err(error) => message(vi, &Notice::Error(error)),
                    },
                    Err(error) => message(vi, &Notice::Error(error.clone())),
                })
            }}</p>
        </section>
    }
}
