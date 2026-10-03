//! Reviewed shared conformance vectors executed natively and in the WASI artifact.
use evobase_appspec::*;
use std::collections::BTreeMap;

const GOLDEN: &[u8] = include_bytes!("fixtures/commerce-v1.json");
fn spec() -> CheckedAppSpec {
    CheckedAppSpec::compile(fixtures::example_spec()).unwrap()
}
fn scope() -> Scope {
    Scope::new("tenant_demo", AppId::new("app_commerce").unwrap()).unwrap()
}
fn field(id: &str) -> FieldId {
    FieldId::new(id).unwrap()
}
fn table(id: &str) -> TableId {
    TableId::new(id).unwrap()
}
fn record(id: &str) -> RecordId {
    RecordId::new(id).unwrap()
}
fn json_error(error: Error, contains: &str) {
    match error {
        Error::InvalidJson { message } => assert!(
            message.contains(contains),
            "expected {contains:?}; received {message:?}"
        ),
        other => panic!("expected InvalidJson, received {other:?}"),
    }
}

pub fn golden_vectors() {
    let checked = spec();
    assert_eq!(checked.encode().unwrap(), GOLDEN);
    let decoded = CheckedAppSpec::decode(GOLDEN).unwrap();
    assert_eq!(decoded, checked);
    assert_eq!(decoded.encode().unwrap(), GOLDEN);
    assert_eq!(
        CheckedAppSpec::decode(include_bytes!("fixtures/commerce-v0.json")),
        Err(Error::UnsupportedVersion {
            found: 0,
            supported: 1
        })
    );
    assert_eq!(
        CheckedAppSpec::decode(include_bytes!("fixtures/commerce-v2.json")),
        Err(Error::UnsupportedVersion {
            found: 2,
            supported: 1
        })
    );
    json_error(
        CheckedAppSpec::decode(include_bytes!("fixtures/unknown-binding.json")).unwrap_err(),
        "unknown field `database_url`",
    );
    json_error(
        CheckedAppSpec::decode(include_bytes!("fixtures/duplicate-version.json")).unwrap_err(),
        "duplicate field `version`",
    );
    let unknown_tag = String::from_utf8(GOLDEN.to_vec())
        .unwrap()
        .replace("\"type\":\"money\"", "\"type\":\"decimal\"");
    json_error(
        CheckedAppSpec::decode(unknown_tag.as_bytes()).unwrap_err(),
        "unknown variant `decimal`",
    );
}

pub fn identity_vectors() {
    assert_eq!(
        TableId::new("app_wrong"),
        Err(Error::InvalidId {
            kind: "table",
            value: "app_wrong".to_owned()
        })
    );
    assert_eq!(
        RecordId::new("rec_"),
        Err(Error::InvalidId {
            kind: "record",
            value: "rec_".to_owned()
        })
    );
    assert_eq!(
        FieldId::new("fld_é"),
        Err(Error::InvalidId {
            kind: "field",
            value: "fld_é".to_owned()
        })
    );
    assert!(
        serde_json::from_str::<TableId>("\"app_wrong\"")
            .unwrap_err()
            .to_string()
            .contains("invalid table identity")
    );
    let baseline = spec();
    let mut raw = fixtures::example_spec();
    raw.name = "Bán hàng".to_owned();
    raw.tables.reverse();
    for table in &mut raw.tables {
        table.name = format!("Vietnamese {}", table.name);
        table.fields.reverse();
    }
    let renamed = CheckedAppSpec::compile(raw).unwrap();
    assert_eq!(renamed.app_id(), baseline.app_id());
    assert_eq!(
        renamed.tables().iter().map(|t| &t.id).collect::<Vec<_>>(),
        baseline.tables().iter().map(|t| &t.id).collect::<Vec<_>>()
    );
    let facts = fixtures::example_records(&scope());
    assert_eq!(
        renamed.validate_records(&scope(), &facts).unwrap(),
        baseline.validate_records(&scope(), &facts).unwrap()
    );
    let mut duplicate = fixtures::example_spec();
    duplicate.tables.push(duplicate.tables[0].clone());
    assert_eq!(
        CheckedAppSpec::compile(duplicate),
        Err(Error::DuplicateTable(table("tbl_customers")))
    );
    let mut duplicate = fixtures::example_spec();
    let duplicated_field = duplicate.tables[0].fields[0].clone();
    duplicate.tables[0].fields.push(duplicated_field);
    assert_eq!(
        CheckedAppSpec::compile(duplicate),
        Err(Error::DuplicateField {
            table: table("tbl_customers"),
            field: field("fld_customer_name")
        })
    );
    let mut broken = fixtures::example_spec();
    broken.tables.retain(|t| t.id != table("tbl_products"));
    assert_eq!(
        CheckedAppSpec::compile(broken),
        Err(Error::UnknownTable(table("tbl_products")))
    );
}

pub fn hostile_vectors() {
    assert_eq!(
        CheckedAppSpec::decode(&vec![b' '; MAX_BYTES + 1]),
        Err(Error::LimitExceeded {
            resource: "bytes",
            limit: MAX_BYTES
        })
    );
    let nested = format!(
        "{}0{}",
        "[".repeat(MAX_DEPTH + 1),
        "]".repeat(MAX_DEPTH + 1)
    );
    assert_eq!(
        CheckedAppSpec::decode(nested.as_bytes()),
        Err(Error::LimitExceeded {
            resource: "depth",
            limit: MAX_DEPTH
        })
    );
    let nodes = format!("[{}]", vec!["0"; MAX_NODES + 1].join(","));
    assert_eq!(
        CheckedAppSpec::decode(nodes.as_bytes()),
        Err(Error::LimitExceeded {
            resource: "nodes",
            limit: MAX_NODES
        })
    );
    json_error(
        CheckedAppSpec::decode(b"\xff").unwrap_err(),
        "expected value",
    );
    let mut excessive = fixtures::example_spec();
    excessive.tables = vec![excessive.tables[0].clone(); MAX_TABLES + 1];
    assert_eq!(
        CheckedAppSpec::compile(excessive),
        Err(Error::LimitExceeded {
            resource: "tables",
            limit: MAX_TABLES
        })
    );
    let mut excessive = fixtures::example_spec();
    excessive.tables[0].fields =
        vec![excessive.tables[0].fields[0].clone(); MAX_FIELDS_PER_TABLE + 1];
    assert_eq!(
        CheckedAppSpec::compile(excessive),
        Err(Error::LimitExceeded {
            resource: "fields per table",
            limit: MAX_FIELDS_PER_TABLE
        })
    );
    // A near-field-bound raw definition must not compile a canonical format decode would reject.
    let mut near_bound = fixtures::example_spec();
    near_bound.capture_rules.clear();
    near_bound.tables = (0..8)
        .map(|t| RawTable {
            id: table(&format!("tbl_{t}")),
            name: "T".to_owned(),
            fields: (0..250)
                .map(|f| RawField {
                    id: field(&format!("fld_{t}_{f}")),
                    name: "F".to_owned(),
                    required: false,
                    field_type: FieldType::Text,
                })
                .collect(),
        })
        .collect();
    assert_eq!(
        CheckedAppSpec::compile(near_bound),
        Err(Error::LimitExceeded {
            resource: "nodes",
            limit: MAX_NODES
        })
    );
}

pub fn value_vectors() {
    let checked = spec();
    let scope = scope();
    let mut facts = fixtures::example_records(&scope);
    facts.reverse();
    let baseline = checked.validate_records(&scope, &facts).unwrap();
    let encoded = baseline.encode().unwrap();
    assert_eq!(checked.decode_records(&scope, &encoded).unwrap(), baseline);
    let order = baseline
        .find(&table("tbl_orders"), &record("rec_order_1"))
        .unwrap();
    assert_eq!(
        order.values().get(&field("fld_order_notes")),
        Some(&Value::Blank)
    );
    let order = facts
        .iter_mut()
        .find(|r| r.id == record("rec_order_1"))
        .unwrap();
    order.values.insert(field("fld_order_notes"), Value::Null);
    assert_eq!(
        checked
            .validate_records(&scope, &facts)
            .unwrap()
            .find(&table("tbl_orders"), &record("rec_order_1"))
            .unwrap()
            .values()
            .get(&field("fld_order_notes")),
        Some(&Value::Null)
    );
    for value in [Value::Blank, Value::Null] {
        let mut facts = fixtures::example_records(&scope);
        facts[0].values.insert(field("fld_customer_name"), value);
        assert_eq!(
            checked.validate_records(&scope, &facts),
            Err(Error::Required {
                record: record("rec_customer_1"),
                field: field("fld_customer_name")
            })
        );
    }
    let mut wrong = fixtures::example_records(&scope);
    wrong[2]
        .values
        .insert(field("fld_product_price"), Value::Integer(1250));
    assert_eq!(
        checked.validate_records(&scope, &wrong),
        Err(Error::WrongType {
            record: record("rec_product_1"),
            field: field("fld_product_price"),
            expected: "money"
        })
    );
    let mut huge = fixtures::example_records(&scope);
    huge[0].values.insert(
        field("fld_customer_name"),
        Value::Text("x".repeat(MAX_TEXT_BYTES + 1)),
    );
    assert_eq!(
        checked.validate_records(&scope, &huge),
        Err(Error::LimitExceeded {
            resource: "text bytes",
            limit: MAX_TEXT_BYTES
        })
    );
    let mut duplicate = fixtures::example_records(&scope);
    duplicate.push(duplicate[0].clone());
    assert_eq!(
        checked.validate_records(&scope, &duplicate),
        Err(Error::DuplicateRecord {
            table: table("tbl_customers"),
            record: record("rec_customer_1")
        })
    );
    let runtime = r#"[{"scope":{"tenant_id":"tenant_demo","app_id":"app_commerce"},"id":"rec_customer_1","table_id":"tbl_customers","values":{"fld_customer_name":{"type":"text","value":"A"},"fld_customer_name":{"type":"text","value":"B"}}}]"#;
    json_error(
        checked
            .decode_records(&scope, runtime.as_bytes())
            .unwrap_err(),
        "duplicate field value fld_customer_name",
    );
    for wire in [
        r#"{"type":"money","value":1.2}"#,
        r#"{"type":"money","value":1e2}"#,
        r#"{"type":"money","value":9223372036854775808}"#,
    ] {
        assert!(
            serde_json::from_str::<Value>(wire).is_err(),
            "must reject non-exact money {wire}"
        );
    }
    assert_eq!(
        serde_json::from_str::<Value>(r#"{"type":"money","value":9223372036854775807}"#).unwrap(),
        Value::Money(i64::MAX)
    );
    // Sparse raw facts expand optional fields. Checked facts must fit the wire envelope too.
    let sparse_spec = CheckedAppSpec::compile(RawAppSpec {
        version: 1,
        app_id: scope.app_id().clone(),
        name: "Sparse".to_owned(),
        tables: vec![RawTable {
            id: table("tbl_sparse"),
            name: "Sparse".to_owned(),
            fields: (0..256)
                .map(|i| RawField {
                    id: field(&format!("fld_sparse_{i}")),
                    name: "Optional".to_owned(),
                    required: false,
                    field_type: FieldType::Text,
                })
                .collect(),
        }],
        capture_rules: vec![],
    })
    .unwrap();
    let sparse: Vec<_> = (0..100)
        .map(|i| RawRecord {
            scope: scope.clone(),
            id: record(&format!("rec_sparse_{i}")),
            table_id: table("tbl_sparse"),
            values: BTreeMap::new(),
        })
        .collect();
    assert_eq!(
        sparse_spec.validate_records(&scope, &sparse),
        Err(Error::LimitExceeded {
            resource: "nodes",
            limit: MAX_NODES
        })
    );
}

pub fn reference_vectors() {
    let checked = spec();
    let scope = scope();
    let mut facts = fixtures::example_records(&scope);
    let Value::Ref(reference) = facts[3].values.get_mut(&field("fld_line_product")).unwrap() else {
        panic!()
    };
    reference.record_id = record("rec_missing");
    assert_eq!(
        checked.validate_records(&scope, &facts),
        Err(Error::MissingReference {
            table: table("tbl_products"),
            record: record("rec_missing")
        })
    );
    reference_vectors_wrong_table();
    let mut facts = fixtures::example_records(&scope);
    let Value::Ref(reference) = facts[3].values.get_mut(&field("fld_line_product")).unwrap() else {
        panic!()
    };
    reference.scope = Scope::new("other_tenant", scope.app_id().clone()).unwrap();
    assert_eq!(
        checked.validate_records(&scope, &facts),
        Err(Error::CrossScope {
            record: record("rec_product_1")
        })
    );
    let wrong_app = Scope::new("tenant_demo", AppId::new("app_other").unwrap()).unwrap();
    assert_eq!(
        checked.validate_records(&wrong_app, &[]),
        Err(Error::WrongApp {
            expected: AppId::new("app_commerce").unwrap(),
            actual: AppId::new("app_other").unwrap()
        })
    );
    let mut facts = fixtures::example_records(&scope);
    facts[0].scope = Scope::new("other_tenant", scope.app_id().clone()).unwrap();
    assert_eq!(
        checked.validate_records(&scope, &facts),
        Err(Error::CrossScope {
            record: record("rec_customer_1")
        })
    );
}
fn reference_vectors_wrong_table() {
    let checked = spec();
    let scope = scope();
    let mut facts = fixtures::example_records(&scope);
    let Value::Ref(reference) = facts[3].values.get_mut(&field("fld_line_product")).unwrap() else {
        panic!()
    };
    reference.table_id = table("tbl_customers");
    reference.record_id = record("rec_customer_1");
    assert_eq!(
        checked.validate_records(&scope, &facts),
        Err(Error::WrongReferenceTable {
            record: record("rec_customer_1"),
            expected: table("tbl_products"),
            actual: table("tbl_customers")
        })
    );
}

pub fn authoring_vectors() {
    let checked = spec();
    let scope = scope();
    let mut facts = fixtures::example_records(&scope);
    let price = field("fld_product_price");
    let products = table("tbl_products");
    assert_eq!(
        checked.parse_cell(&scope, &products, &price, "9223372036854775807", &facts),
        Ok(Value::Money(i64::MAX))
    );
    assert_eq!(
        checked.parse_cell(&scope, &products, &price, "9223372036854775808", &facts),
        Err(Error::Overflow { kind: "money" })
    );
    assert_eq!(
        checked.parse_cell(&scope, &products, &price, "-9223372036854775809", &facts),
        Err(Error::Overflow { kind: "money" })
    );
    for input in ["1.0", "1e3", "+1", "1,250"] {
        assert_eq!(
            checked.parse_cell(&scope, &products, &price, input, &facts),
            Err(Error::InvalidCell {
                expected: "money",
                input: input.to_owned()
            })
        );
    }
    let name = field("fld_product_name");
    assert_eq!(
        checked.parse_cell(&scope, &products, &name, "", &facts),
        Ok(Value::Blank)
    );
    assert_eq!(
        checked.parse_cell(&scope, &products, &name, "null", &facts),
        Ok(Value::Null)
    );
    assert_eq!(
        checked.parse_cell(&scope, &products, &name, "text:null", &facts),
        Ok(Value::Text("null".to_owned()))
    );
    assert_eq!(
        checked.parse_cell(&scope, &products, &name, "text:", &facts),
        Ok(Value::Text(String::new()))
    );
    assert_eq!(
        checked.parse_cell(&scope, &products, &name, "text:text:literal", &facts),
        Ok(Value::Text("text:literal".to_owned()))
    );
    let product_ref = field("fld_line_product");
    let lines = table("tbl_order_lines");
    let expected = Value::Ref(RecordRef {
        scope: scope.clone(),
        table_id: products.clone(),
        record_id: record("rec_product_1"),
    });
    assert_eq!(
        checked.parse_cell(&scope, &lines, &product_ref, "rec_product_1", &facts),
        Ok(expected.clone())
    );
    assert_eq!(
        checked.parse_cell(&scope, &lines, &product_ref, "Widget", &facts),
        Ok(expected)
    );
    let mut other = facts[2].clone();
    other.id = record("rec_product_2");
    facts.push(other);
    assert_eq!(
        checked.parse_cell(&scope, &lines, &product_ref, "Widget", &facts),
        Err(Error::AmbiguousReference {
            table: products,
            label: "Widget".to_owned()
        })
    );
}

pub fn capture_schema_vectors() {
    let mut raw = fixtures::example_spec();
    raw.tables[2]
        .fields
        .iter_mut()
        .find(|f| f.id == field("fld_line_price"))
        .unwrap()
        .field_type = FieldType::Text;
    assert_eq!(
        CheckedAppSpec::compile(raw),
        Err(Error::InvalidCapture {
            reason: "capture source and destination must be money".to_owned()
        })
    );
    let mut raw = fixtures::example_spec();
    raw.capture_rules.push(raw.capture_rules[0].clone());
    assert_eq!(
        CheckedAppSpec::compile(raw),
        Err(Error::InvalidCapture {
            reason: "duplicate capture tbl_order_lines.fld_line_price".to_owned()
        })
    );
}

pub fn run_vectors() -> String {
    golden_vectors();
    identity_vectors();
    hostile_vectors();
    value_vectors();
    reference_vectors();
    authoring_vectors();
    capture_schema_vectors();
    let mut report = BTreeMap::new();
    report.insert("profile", "evobase-appspec-v1-json".to_owned());
    report.insert(
        "groups",
        "golden,identity,hostile,value,reference,authoring,capture-schema".to_owned(),
    );
    report.insert(
        "canonical",
        String::from_utf8(spec().encode().unwrap()).unwrap(),
    );
    report.insert("money_max", i64::MAX.to_string());
    report.insert(
        "future_error",
        Error::UnsupportedVersion {
            found: 2,
            supported: 1,
        }
        .to_string(),
    );
    serde_json::to_string(&report).unwrap()
}

#[test]
fn golden() {
    golden_vectors();
}
#[test]
fn identities() {
    identity_vectors();
}
#[test]
fn hostile() {
    hostile_vectors();
}
#[test]
fn values() {
    value_vectors();
}
#[test]
fn references() {
    reference_vectors();
}
#[test]
fn authoring() {
    authoring_vectors();
}
#[test]
fn capture_schema() {
    capture_schema_vectors();
}
