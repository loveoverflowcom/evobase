//! Small synthetic fixtures; no tenant data, host credentials or trusted authority.
use crate::{
    AppId, CaptureRule, FORMAT_VERSION, FieldId, FieldType, RawAppSpec, RawField, RawRecord,
    RawTable, RecordId, RecordRef, Scope, TableId, Value,
};
use std::collections::BTreeMap;

fn field(id: &str, name: &str, required: bool, field_type: FieldType) -> RawField {
    RawField {
        id: FieldId::new(id).expect("fixture identity"),
        name: name.to_owned(),
        required,
        field_type,
    }
}
fn table(id: &str, name: &str, fields: Vec<RawField>) -> RawTable {
    RawTable {
        id: TableId::new(id).expect("fixture identity"),
        name: name.to_owned(),
        fields,
    }
}
fn reference(target: &str) -> FieldType {
    FieldType::Ref {
        target_table: TableId::new(target).expect("fixture identity"),
    }
}

pub fn example_spec() -> RawAppSpec {
    RawAppSpec {
        version: FORMAT_VERSION,
        app_id: AppId::new("app_commerce").expect("fixture identity"),
        name: "Commerce".to_owned(),
        tables: vec![
            table(
                "tbl_customers",
                "Customer",
                vec![
                    field("fld_customer_name", "Name", true, FieldType::Text),
                    field("fld_customer_owner", "Owner", true, FieldType::Text),
                ],
            ),
            table(
                "tbl_orders",
                "Order",
                vec![
                    field(
                        "fld_order_customer",
                        "Customer",
                        true,
                        reference("tbl_customers"),
                    ),
                    field("fld_order_owner", "Owner", true, FieldType::Text),
                    field("fld_order_state", "State", true, FieldType::Text),
                    field("fld_order_notes", "Notes", false, FieldType::Text),
                ],
            ),
            table(
                "tbl_order_lines",
                "OrderLine",
                vec![
                    field("fld_line_order", "Order", true, reference("tbl_orders")),
                    field(
                        "fld_line_product",
                        "Product",
                        true,
                        reference("tbl_products"),
                    ),
                    field("fld_line_quantity", "Quantity", true, FieldType::Integer),
                    field("fld_line_price", "Captured price", true, FieldType::Money),
                ],
            ),
            table(
                "tbl_products",
                "Product",
                vec![
                    field("fld_product_name", "Name", true, FieldType::Text),
                    field("fld_product_price", "Price", true, FieldType::Money),
                ],
            ),
        ],
        capture_rules: vec![CaptureRule {
            line_table: TableId::new("tbl_order_lines").expect("fixture identity"),
            product_ref_field: FieldId::new("fld_line_product").expect("fixture identity"),
            product_price_field: FieldId::new("fld_product_price").expect("fixture identity"),
            captured_price_field: FieldId::new("fld_line_price").expect("fixture identity"),
        }],
    }
}

fn record(scope: &Scope, table: &str, id: &str, values: Vec<(&str, Value)>) -> RawRecord {
    RawRecord {
        scope: scope.clone(),
        id: RecordId::new(id).expect("fixture identity"),
        table_id: TableId::new(table).expect("fixture identity"),
        values: values
            .into_iter()
            .map(|(id, value)| (FieldId::new(id).expect("fixture identity"), value))
            .collect::<BTreeMap<_, _>>(),
    }
}
fn link(scope: &Scope, table: &str, id: &str) -> Value {
    Value::Ref(RecordRef {
        scope: scope.clone(),
        table_id: TableId::new(table).expect("fixture identity"),
        record_id: RecordId::new(id).expect("fixture identity"),
    })
}

pub fn example_records(scope: &Scope) -> Vec<RawRecord> {
    vec![
        record(
            scope,
            "tbl_customers",
            "rec_customer_1",
            vec![
                ("fld_customer_name", Value::Text("Ada".to_owned())),
                ("fld_customer_owner", Value::Text("actor_alice".to_owned())),
            ],
        ),
        record(
            scope,
            "tbl_orders",
            "rec_order_1",
            vec![
                (
                    "fld_order_customer",
                    link(scope, "tbl_customers", "rec_customer_1"),
                ),
                ("fld_order_owner", Value::Text("actor_alice".to_owned())),
                ("fld_order_state", Value::Text("draft".to_owned())),
            ],
        ),
        record(
            scope,
            "tbl_products",
            "rec_product_1",
            vec![
                ("fld_product_name", Value::Text("Widget".to_owned())),
                ("fld_product_price", Value::Money(1250)),
            ],
        ),
        record(
            scope,
            "tbl_order_lines",
            "rec_line_1",
            vec![
                ("fld_line_order", link(scope, "tbl_orders", "rec_order_1")),
                (
                    "fld_line_product",
                    link(scope, "tbl_products", "rec_product_1"),
                ),
                ("fld_line_quantity", Value::Integer(2)),
                ("fld_line_price", Value::Money(1250)),
            ],
        ),
    ]
}
