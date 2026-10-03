use evobase_appspec::*;
use std::collections::BTreeMap;

fn definition(app: &str, table: &str) -> RawAppSpec {
    RawAppSpec {
        version: FORMAT_VERSION,
        app_id: AppId::new(app).unwrap(),
        name: "Refinements".to_owned(),
        tables: vec![RawTable {
            id: TableId::new(table).unwrap(),
            name: "Records".to_owned(),
            fields: vec![
                RawField {
                    id: FieldId::new("fld_text").unwrap(),
                    name: "Text".to_owned(),
                    required: false,
                    field_type: FieldType::Text,
                },
                RawField {
                    id: FieldId::new("fld_count").unwrap(),
                    name: "Count".to_owned(),
                    required: false,
                    field_type: FieldType::Integer,
                },
                RawField {
                    id: FieldId::new("fld_money").unwrap(),
                    name: "Units".to_owned(),
                    required: false,
                    field_type: FieldType::Money,
                },
            ],
        }],
        capture_rules: vec![],
        policies: vec![],
        submit_rules: vec![],
        constraints: vec![
            constraint(
                table,
                "fld_text",
                "constraint_nonempty",
                Constraint::NonEmpty,
            ),
            constraint(
                table,
                "fld_text",
                "constraint_length",
                Constraint::TextLength { min: 2, max: 4 },
            ),
            constraint(
                table,
                "fld_count",
                "constraint_range",
                Constraint::NumericRange { min: -2, max: 3 },
            ),
            constraint(
                table,
                "fld_money",
                "constraint_money",
                Constraint::NumericRange {
                    min: i64::MIN,
                    max: i64::MAX,
                },
            ),
        ],
    }
}
fn constraint(table: &str, field: &str, id: &str, constraint: Constraint) -> RawFieldConstraint {
    RawFieldConstraint {
        constraint_id: ConstraintId::new(id).unwrap(),
        table_id: TableId::new(table).unwrap(),
        field_id: FieldId::new(field).unwrap(),
        constraint,
    }
}
fn row(scope: &Scope, table: &str, text: Value, count: Value) -> RawRecord {
    RawRecord {
        scope: scope.clone(),
        table_id: TableId::new(table).unwrap(),
        id: RecordId::new("rec_one").unwrap(),
        values: [
            (FieldId::new("fld_text").unwrap(), text),
            (FieldId::new("fld_count").unwrap(), count),
            (FieldId::new("fld_money").unwrap(), Value::Money(i64::MIN)),
        ]
        .into_iter()
        .collect(),
    }
}
fn violation(field: &str, id: &str) -> Error {
    Error::ConstraintViolation {
        record: RecordId::new("rec_one").unwrap(),
        field: FieldId::new(field).unwrap(),
        constraint: ConstraintId::new(id).unwrap(),
    }
}

#[test]
fn unrelated_apps_share_text_unicode_signed_numeric_and_wire_semantics() {
    for (app, table) in [
        ("app_library", "tbl_books"),
        ("app_research", "tbl_samples"),
    ] {
        let checked = CheckedAppSpec::compile(definition(app, table)).unwrap();
        let scope = Scope::new("tenant_local", checked.app_id().clone()).unwrap();
        for text in ["é🙂", "a  ", " ab "] {
            for count in [-2, 0, 3] {
                let rows = vec![row(
                    &scope,
                    table,
                    Value::Text(text.to_owned()),
                    Value::Integer(count),
                )];
                let facts = checked.validate_records(&scope, &rows).unwrap();
                assert_eq!(
                    facts.records()[0]
                        .values()
                        .get(&FieldId::new("fld_text").unwrap()),
                    Some(&Value::Text(text.to_owned()))
                );
                assert_eq!(
                    checked
                        .decode_records(&scope, &facts.encode().unwrap())
                        .unwrap(),
                    facts
                );
            }
        }
        for (text, expected) in [
            ("a", "constraint_length"),
            ("abcde", "constraint_length"),
            ("  ", "constraint_nonempty"),
            ("\u{2003}\u{2003}", "constraint_nonempty"),
        ] {
            let rows = vec![row(
                &scope,
                table,
                Value::Text(text.to_owned()),
                Value::Integer(0),
            )];
            assert_eq!(
                checked.validate_records(&scope, &rows),
                Err(violation("fld_text", expected))
            );
            assert_eq!(
                checked.decode_records(&scope, &serde_json::to_vec(&rows).unwrap()),
                Err(violation("fld_text", expected))
            );
        }
        for count in [-3, 4, i64::MIN, i64::MAX] {
            assert_eq!(
                checked.validate_records(
                    &scope,
                    &[row(
                        &scope,
                        table,
                        Value::Text("ok".to_owned()),
                        Value::Integer(count)
                    )]
                ),
                Err(violation("fld_count", "constraint_range"))
            );
        }
    }
}

#[test]
fn absent_blank_null_are_optional_but_required_has_precedence() {
    let mut raw = definition("app_absent", "tbl_items");
    let scope = Scope::new("tenant_local", raw.app_id.clone()).unwrap();
    let checked = CheckedAppSpec::compile(raw.clone()).unwrap();
    for values in [
        BTreeMap::new(),
        [(FieldId::new("fld_text").unwrap(), Value::Blank)]
            .into_iter()
            .collect(),
        [(FieldId::new("fld_text").unwrap(), Value::Null)]
            .into_iter()
            .collect(),
    ] {
        let rows = vec![RawRecord {
            values,
            ..row(&scope, "tbl_items", Value::Blank, Value::Blank)
        }];
        assert!(checked.validate_records(&scope, &rows).is_ok());
        raw.tables[0].fields[0].required = true;
        assert_eq!(
            CheckedAppSpec::compile(raw.clone())
                .unwrap()
                .validate_records(&scope, &rows),
            Err(Error::Required {
                record: RecordId::new("rec_one").unwrap(),
                field: FieldId::new("fld_text").unwrap()
            })
        );
    }
}

#[test]
fn invalid_declarations_and_version_one_extensions_reject_precisely() {
    let mut raw = definition("app_errors", "tbl_items");
    raw.version = 1;
    assert_eq!(
        CheckedAppSpec::compile(raw.clone()),
        Err(Error::UnsupportedDeclaration {
            version: 1,
            declaration: "constraints"
        })
    );
    raw.version = FORMAT_VERSION;
    raw.constraints[0].constraint = Constraint::NumericRange { min: 0, max: 1 };
    assert_eq!(
        CheckedAppSpec::compile(raw.clone()),
        Err(Error::InvalidConstraint {
            constraint: ConstraintId::new("constraint_nonempty").unwrap(),
            reason: "constraint does not support the field type"
        })
    );
    raw.constraints[0].constraint = Constraint::NonEmpty;
    raw.constraints[2].constraint = Constraint::NumericRange { min: 2, max: 1 };
    assert_eq!(
        CheckedAppSpec::compile(raw.clone()),
        Err(Error::InvalidConstraint {
            constraint: ConstraintId::new("constraint_range").unwrap(),
            reason: "invalid inclusive bounds"
        })
    );
    raw.constraints[2].constraint = Constraint::NumericRange { min: -2, max: 3 };
    raw.constraints.push(raw.constraints[0].clone());
    assert_eq!(
        CheckedAppSpec::compile(raw.clone()),
        Err(Error::InvalidConstraint {
            constraint: ConstraintId::new("constraint_nonempty").unwrap(),
            reason: "duplicate constraint identity"
        })
    );
    raw.constraints = vec![raw.constraints[0].clone(); MAX_CONSTRAINTS + 1];
    assert_eq!(
        CheckedAppSpec::compile(raw),
        Err(Error::LimitExceeded {
            resource: "constraints",
            limit: MAX_CONSTRAINTS
        })
    );
}

#[test]
fn canonical_bindings_survive_rename_reorder_and_historical_v1_stays_identical() {
    let mut raw = definition("app_portable", "tbl_items");
    let checked = CheckedAppSpec::compile(raw.clone()).unwrap();
    assert_eq!(
        CheckedAppSpec::decode(&checked.encode().unwrap()).unwrap(),
        checked
    );
    raw.constraints.reverse();
    raw.tables[0].fields.reverse();
    assert_eq!(
        CheckedAppSpec::compile(raw.clone())
            .unwrap()
            .encode()
            .unwrap(),
        checked.encode().unwrap()
    );
    raw.tables[0].name = "Tên mới".to_owned();
    raw.tables[0].fields[0].name = "Đổi tên".to_owned();
    let renamed = CheckedAppSpec::compile(raw).unwrap();
    let scope = Scope::new("tenant_local", checked.app_id().clone()).unwrap();
    let rows = vec![row(
        &scope,
        "tbl_items",
        Value::Text("ok".to_owned()),
        Value::Integer(2),
    )];
    assert_eq!(
        checked.validate_records(&scope, &rows).unwrap(),
        renamed.validate_records(&scope, &rows).unwrap()
    );
    let historical = include_bytes!("fixtures/commerce-v1.json");
    assert_eq!(
        CheckedAppSpec::decode(historical)
            .unwrap()
            .encode()
            .unwrap(),
        historical
    );
    assert_eq!(
        CheckedAppSpec::decode(include_bytes!("fixtures/commerce-v2.json"))
            .unwrap()
            .definition()
            .version,
        FORMAT_VERSION
    );
}
