use evobase_appspec::relations::*;
use evobase_appspec::*;
use std::collections::BTreeMap;

fn table(s: &str) -> TableId {
    TableId::new(format!("tbl_{s}")).unwrap()
}
fn field(s: &str) -> FieldId {
    FieldId::new(format!("fld_{s}")).unwrap()
}
fn record(s: &str) -> RecordId {
    RecordId::new(format!("rec_{s}")).unwrap()
}
fn scope() -> Scope {
    Scope::new("tenant_demo", AppId::new("app_shop").unwrap()).unwrap()
}
fn schema() -> CheckedAppSpec {
    let field_def = |name: &str, required, field_type| RawField {
        id: field(name),
        name: name.to_owned(),
        required,
        field_type,
    };
    let definition = serde_json::json!({
        "version": FORMAT_VERSION,
        "app_id": scope().app_id().clone(),
        "name": "Shop",
        "tables": vec![
            RawTable {
                id: table("customer"),
                name: "Customer".to_owned(),
                fields: vec![field_def("customer_name", true, FieldType::Text)],
            },
            RawTable {
                id: table("order"),
                name: "Order".to_owned(),
                fields: vec![field_def(
                    "order_customer",
                    true,
                    FieldType::Ref {
                        target_table: table("customer"),
                    },
                )],
            },
            RawTable {
                id: table("product"),
                name: "Product".to_owned(),
                fields: vec![
                    field_def("product_name", true, FieldType::Text),
                    field_def("product_price", true, FieldType::Money),
                ],
            },
            RawTable {
                id: table("line"),
                name: "OrderLine".to_owned(),
                fields: vec![
                    field_def(
                        "line_order",
                        true,
                        FieldType::Ref {
                            target_table: table("order"),
                        },
                    ),
                    field_def(
                        "line_product",
                        true,
                        FieldType::Ref {
                            target_table: table("product"),
                        },
                    ),
                    field_def("line_quantity", false, FieldType::Integer),
                    field_def("line_captured", true, FieldType::Money),
                ],
            },
        ],
        "capture_rules": vec![CaptureRule {
            line_table: table("line"),
            product_ref_field: field("line_product"),
            product_price_field: field("product_price"),
            captured_price_field: field("line_captured"),
        }],
    });
    CheckedAppSpec::compile(serde_json::from_value(definition).unwrap()).unwrap()
}
fn row(t: &str, r: &str, values: Vec<(&str, Value)>) -> RawRecord {
    RawRecord {
        scope: scope(),
        id: record(r),
        table_id: table(t),
        values: values.into_iter().map(|(f, v)| (field(f), v)).collect(),
    }
}
fn reference(t: &str, r: &str) -> Value {
    Value::Ref(RecordRef {
        scope: scope(),
        table_id: table(t),
        record_id: record(r),
    })
}
fn line(id: &str, quantity: Value) -> RawRecord {
    row(
        "line",
        id,
        vec![
            ("line_order", reference("order", "order")),
            ("line_product", reference("product", "product")),
            ("line_quantity", quantity),
        ],
    )
}
fn seed(price: i64, quantity: Value) -> Vec<RawRecord> {
    vec![
        line("line", quantity),
        row(
            "product",
            "product",
            vec![
                ("product_name", Value::Text("Coffee".to_owned())),
                ("product_price", Value::Money(price)),
            ],
        ),
        row(
            "order",
            "order",
            vec![("order_customer", reference("customer", "customer"))],
        ),
        row(
            "customer",
            "customer",
            vec![("customer_name", Value::Text("An".to_owned()))],
        ),
    ]
}
fn store(price: i64, quantity: Value) -> RelationStore {
    let mut store = RelationStore::new(schema(), scope()).unwrap();
    store
        .apply_batch(
            seed(price, quantity)
                .into_iter()
                .map(BatchChange::Insert)
                .collect(),
        )
        .unwrap();
    store
}
fn raw(store: &RelationStore, t: &str, id: &str) -> RawRecord {
    store
        .records()
        .find(&table(t), &record(id))
        .unwrap()
        .as_raw()
        .clone()
}
fn subtotal() -> Expression {
    Expression::Product(
        Box::new(Expression::Field(field("line_quantity"))),
        Box::new(Expression::Field(field("line_captured"))),
    )
}
fn definitions() -> Vec<FormulaDefinition> {
    vec![
        FormulaDefinition {
            table: table("line"),
            name: "subtotal".to_owned(),
            expression: subtotal(),
        },
        FormulaDefinition {
            table: table("line"),
            name: "live".to_owned(),
            expression: Expression::Lookup {
                ref_field: field("line_product"),
                target_field: field("product_price"),
            },
        },
        FormulaDefinition {
            table: table("order"),
            name: "total".to_owned(),
            expression: Expression::Sum {
                child_table: table("line"),
                ref_field: field("line_order"),
                value: Box::new(Expression::Named("subtotal".to_owned())),
            },
        },
    ]
}
fn evaluate(
    formulas: &CheckedFormulas,
    store: &RelationStore,
    table_name: &str,
    record_name: &str,
    name: &str,
) -> Result<Numeric, RelationError> {
    formulas.evaluate(
        store,
        &table(table_name),
        &record(record_name),
        name,
        &LocalPreviewPolicy,
    )
}

#[test]
fn final_state_batch_capture_and_historical_price_are_atomic() {
    let mut store = store(1_200, Value::Integer(2));
    let formulas =
        CheckedFormulas::compile(store.spec(), definitions(), FormulaLimits::default()).unwrap();
    let before = store.records().to_raw();
    assert_eq!(
        evaluate(&formulas, &store, "order", "order", "total"),
        Ok(Numeric::Money(2_400))
    );
    assert_eq!(store.records().to_raw(), before, "query has no effects");
    let mut product = raw(&store, "product", "product");
    product
        .values
        .insert(field("product_price"), Value::Money(2_000));
    store
        .apply_batch(vec![
            BatchChange::Replace(product),
            BatchChange::Insert(line("second", Value::Integer(3))),
        ])
        .unwrap();
    assert_eq!(
        evaluate(&formulas, &store, "line", "line", "live"),
        Ok(Numeric::Money(2_000))
    );
    assert_eq!(
        raw(&store, "line", "line").values[&field("line_captured")],
        Value::Money(1_200)
    );
    assert_eq!(
        raw(&store, "line", "second").values[&field("line_captured")],
        Value::Money(2_000)
    );
    assert_eq!(
        evaluate(&formulas, &store, "order", "order", "total"),
        Ok(Numeric::Money(8_400))
    );
    let snapshot = store.records().to_raw();
    let mut forged = raw(&store, "line", "line");
    forged
        .values
        .insert(field("line_captured"), Value::Money(1));
    assert_eq!(
        store.apply_batch(vec![BatchChange::Replace(forged)]),
        Err(RelationError::CapturedValueImmutable {
            field: field("line_captured")
        })
    );
    let mut forged = line("forged", Value::Integer(1));
    forged
        .values
        .insert(field("line_captured"), Value::Money(2_000));
    assert_eq!(
        store.apply_batch(vec![BatchChange::Insert(forged)]),
        Err(RelationError::CaptureInputForbidden {
            field: field("line_captured")
        })
    );
    assert_eq!(store.records().to_raw(), snapshot);
    let mut retargeted = raw(&store, "line", "line");
    retargeted
        .values
        .insert(field("line_product"), reference("product", "different"));
    assert_eq!(
        store.apply_batch(vec![BatchChange::Replace(retargeted)]),
        Err(RelationError::CapturedReferenceImmutable {
            field: field("line_product")
        })
    );
}

#[test]
fn trusted_snapshot_restore_preserves_history_but_future_edits_still_validate() {
    let mut source = store(100, Value::Integer(2));
    let mut product = raw(&source, "product", "product");
    product
        .values
        .insert(field("product_price"), Value::Money(300));
    source
        .apply_batch(vec![BatchChange::Replace(product)])
        .unwrap();
    let checked_history = source
        .spec()
        .validate_records(&scope(), &source.records().to_raw())
        .unwrap();
    let mut restored =
        RelationStore::from_checked_snapshot(schema(), checked_history, &LocalPreviewPolicy)
            .unwrap();
    assert_eq!(
        raw(&restored, "line", "line").values[&field("line_captured")],
        Value::Money(100)
    );
    let mut attempted = raw(&restored, "line", "line");
    attempted
        .values
        .insert(field("line_captured"), Value::Money(300));
    assert_eq!(
        restored.apply_batch(vec![BatchChange::Replace(attempted)]),
        Err(RelationError::CapturedValueImmutable {
            field: field("line_captured")
        })
    );
    restored
        .apply_batch(vec![BatchChange::Insert(line("new", Value::Integer(1)))])
        .unwrap();
    assert_eq!(
        raw(&restored, "line", "new").values[&field("line_captured")],
        Value::Money(300)
    );
}

#[test]
fn restrict_delete_uses_final_state_and_rolls_back_the_entire_batch() {
    let mut store = store(5, Value::Integer(1));
    let snapshot = store.records().to_raw();
    let mut product = raw(&store, "product", "product");
    product
        .values
        .insert(field("product_price"), Value::Money(77));
    assert_eq!(
        store.apply_batch(vec![
            BatchChange::Replace(product),
            BatchChange::Delete {
                table_id: table("order"),
                record_id: record("order")
            }
        ]),
        Err(RelationError::RestrictDelete {
            table: table("order"),
            record: record("order"),
            source_table: table("line"),
            source_record: record("line"),
            field: field("line_order")
        })
    );
    assert_eq!(store.records().to_raw(), snapshot);
    store
        .apply_batch(vec![
            BatchChange::Delete {
                table_id: table("product"),
                record_id: record("product"),
            },
            BatchChange::Delete {
                table_id: table("line"),
                record_id: record("line"),
            },
        ])
        .unwrap();
    assert_eq!(
        store.reverse_refs(
            &table("order"),
            &record("order"),
            &table("line"),
            &field("line_order"),
            &LocalPreviewPolicy
        ),
        Ok(vec![])
    );
}

#[test]
fn retargeting_an_ordinary_ref_updates_only_derived_edges_and_rollups() {
    let mut store = store(20, Value::Integer(3));
    let formulas =
        CheckedFormulas::compile(store.spec(), definitions(), FormulaLimits::default()).unwrap();
    let mut moved = raw(&store, "line", "line");
    moved
        .values
        .insert(field("line_order"), reference("order", "second_order"));
    let second_order = row(
        "order",
        "second_order",
        vec![("order_customer", reference("customer", "customer"))],
    );
    store
        .apply_batch(vec![
            BatchChange::Replace(moved),
            BatchChange::Insert(second_order),
        ])
        .unwrap();
    assert_eq!(
        store.reverse_refs(
            &table("order"),
            &record("order"),
            &table("line"),
            &field("line_order"),
            &LocalPreviewPolicy
        ),
        Ok(vec![])
    );
    assert_eq!(
        store.reverse_refs(
            &table("order"),
            &record("second_order"),
            &table("line"),
            &field("line_order"),
            &LocalPreviewPolicy
        ),
        Ok(vec![record("line")])
    );
    assert_eq!(
        evaluate(&formulas, &store, "order", "order", "total"),
        Ok(Numeric::Money(0))
    );
    assert_eq!(
        evaluate(&formulas, &store, "order", "second_order", "total"),
        Ok(Numeric::Money(60))
    );
    assert_eq!(
        raw(&store, "line", "line").values[&field("line_captured")],
        Value::Money(20)
    );
}

#[test]
fn malformed_references_return_exact_diagnostics_without_partial_facts() {
    for (bad_ref, expected) in [
        (
            RecordRef {
                scope: scope(),
                table_id: table("customer"),
                record_id: record("customer"),
            },
            Error::WrongReferenceTable {
                record: record("customer"),
                expected: table("product"),
                actual: table("customer"),
            },
        ),
        (
            RecordRef {
                scope: scope(),
                table_id: table("product"),
                record_id: record("missing"),
            },
            Error::MissingReference {
                table: table("product"),
                record: record("missing"),
            },
        ),
        (
            RecordRef {
                scope: Scope::new("other_tenant", scope().app_id().clone()).unwrap(),
                table_id: table("product"),
                record_id: record("product"),
            },
            Error::CrossScope {
                record: record("product"),
            },
        ),
        (
            RecordRef {
                scope: Scope::new("tenant_demo", AppId::new("app_other").unwrap()).unwrap(),
                table_id: table("product"),
                record_id: record("product"),
            },
            Error::CrossScope {
                record: record("product"),
            },
        ),
    ] {
        let mut store = RelationStore::new(schema(), scope()).unwrap();
        let mut rows = seed(5, Value::Integer(1));
        rows[0]
            .values
            .insert(field("line_product"), Value::Ref(bad_ref));
        assert_eq!(
            store.apply_batch(rows.into_iter().map(BatchChange::Insert).collect()),
            Err(RelationError::Kernel(expected))
        );
        assert_eq!(store.records().records().len(), 0);
    }
}

#[test]
fn stable_identity_and_derived_reverse_match_independent_adjacency_oracle() {
    let mut original = schema().definition().clone();
    original.name = "Đổi tên".to_owned();
    original.tables.reverse();
    for table in &mut original.tables {
        table.name = "Duplicate label".to_owned();
        table.fields.reverse();
        for field in &mut table.fields {
            field.name = "Same".to_owned();
        }
    }
    let renamed = CheckedAppSpec::compile(original).unwrap();
    let mut renamed_store = RelationStore::new(renamed, scope()).unwrap();
    let mut rows = seed(12, Value::Integer(2));
    for n in 0..19 {
        rows.push(line(&format!("child_{n:02}"), Value::Integer(n)));
    }
    let mut adjacency: BTreeMap<(TableId, RecordId), Vec<RecordId>> = BTreeMap::new();
    for row in &rows {
        if let Some(Value::Ref(r)) = row.values.get(&field("line_order")) {
            adjacency
                .entry((r.table_id.clone(), r.record_id.clone()))
                .or_default()
                .push(row.id.clone());
        }
    }
    let expected = adjacency
        .get_mut(&(table("order"), record("order")))
        .unwrap();
    expected.sort();
    rows.reverse();
    renamed_store
        .apply_batch(rows.into_iter().map(BatchChange::Insert).collect())
        .unwrap();
    assert_eq!(
        renamed_store.reverse_refs(
            &table("order"),
            &record("order"),
            &table("line"),
            &field("line_order"),
            &LocalPreviewPolicy
        ),
        Ok(expected.clone())
    );
    let formulas =
        CheckedFormulas::compile(&schema(), definitions(), FormulaLimits::default()).unwrap();
    assert_eq!(
        evaluate(&formulas, &renamed_store, "line", "line", "subtotal"),
        Ok(Numeric::Money(24)),
        "label/order changes do not change bindings"
    );
    let rows = renamed_store.records().to_raw();
    let mut duplicate = rows
        .iter()
        .find(|r| r.table_id == table("product"))
        .unwrap()
        .clone();
    duplicate.id = record("duplicate");
    renamed_store
        .apply_batch(vec![BatchChange::Insert(duplicate)])
        .unwrap();
    // Both target display labels match; import must not guess an identity.
    assert_eq!(
        renamed_store.spec().parse_cell(
            &scope(),
            &table("line"),
            &field("line_product"),
            "Coffee",
            &renamed_store.records().to_raw()
        ),
        Err(Error::AmbiguousReference {
            table: table("product"),
            label: "Coffee".to_owned()
        })
    );
}

#[test]
fn checked_formula_semantics_reject_definition_drift() {
    let first = store(12, Value::Integer(1));
    let formulas =
        CheckedFormulas::compile(first.spec(), definitions(), FormulaLimits::default()).unwrap();
    let mut changed = schema().definition().clone();
    changed
        .tables
        .iter_mut()
        .find(|t| t.id == table("line"))
        .unwrap()
        .fields
        .iter_mut()
        .find(|f| f.id == field("line_quantity"))
        .unwrap()
        .required = true;
    let mut second =
        RelationStore::new(CheckedAppSpec::compile(changed).unwrap(), scope()).unwrap();
    second
        .apply_batch(
            seed(12, Value::Integer(1))
                .into_iter()
                .map(BatchChange::Insert)
                .collect(),
        )
        .unwrap();
    assert_eq!(
        evaluate(&formulas, &second, "line", "line", "subtotal"),
        Err(RelationError::DefinitionMismatch)
    );
}

#[test]
fn whole_money_product_agrees_with_independent_i128_oracle_at_boundaries() {
    for quantity in [i64::MIN, -9, -1, 0, 1, 2, i64::MAX] {
        for price in [i64::MIN, -13, -1, 0, 1, 10, i64::MAX] {
            let store = store(price, Value::Integer(quantity));
            let formulas =
                CheckedFormulas::compile(store.spec(), definitions(), FormulaLimits::default())
                    .unwrap();
            let oracle = i128::from(quantity) * i128::from(price);
            let expected = i64::try_from(oracle)
                .map(Numeric::Money)
                .map_err(|_| RelationError::Overflow);
            assert_eq!(
                evaluate(&formulas, &store, "line", "line", "subtotal"),
                expected,
                "quantity={quantity},price={price}"
            );
        }
    }
}

#[test]
fn null_empty_total_and_sum_overflow_have_explicit_semantics() {
    let mut store = store(i64::MAX, Value::Null);
    let formulas =
        CheckedFormulas::compile(store.spec(), definitions(), FormulaLimits::default()).unwrap();
    assert_eq!(
        evaluate(&formulas, &store, "line", "line", "subtotal"),
        Ok(Numeric::Null(NumericKind::Money))
    );
    assert_eq!(
        evaluate(&formulas, &store, "order", "order", "total"),
        Ok(Numeric::Money(0))
    );
    store
        .apply_batch(vec![BatchChange::Delete {
            table_id: table("line"),
            record_id: record("line"),
        }])
        .unwrap();
    assert_eq!(
        evaluate(&formulas, &store, "order", "order", "total"),
        Ok(Numeric::Money(0))
    );
    store
        .apply_batch(vec![
            BatchChange::Insert(line("a", Value::Integer(1))),
            BatchChange::Insert(line("b", Value::Integer(1))),
        ])
        .unwrap();
    assert!(i128::from(i64::MAX) * 2 > i128::from(i64::MAX));
    assert_eq!(
        evaluate(&formulas, &store, "order", "order", "total"),
        Err(RelationError::Overflow)
    );
    let total = vec![FormulaDefinition {
        table: table("line"),
        name: "bad".to_owned(),
        expression: Expression::Total(vec![]),
    }];
    assert_eq!(
        CheckedFormulas::compile(store.spec(), total, FormulaLimits::default()).err(),
        Some(RelationError::EmptyTotal)
    );
    let mixed = vec![FormulaDefinition {
        table: table("line"),
        name: "mixed".to_owned(),
        expression: Expression::Total(vec![
            Expression::Field(field("line_quantity")),
            Expression::Field(field("line_captured")),
        ]),
    }];
    assert_eq!(
        CheckedFormulas::compile(store.spec(), mixed, FormulaLimits::default()).err(),
        Some(RelationError::NumericTypeMismatch)
    );
}

#[test]
fn cycles_depth_nodes_records_and_unsupported_money_product_reject_exactly() {
    let store = store(10, Value::Integer(2));
    let cycle = vec![
        FormulaDefinition {
            table: table("line"),
            name: "a".to_owned(),
            expression: Expression::Named("b".to_owned()),
        },
        FormulaDefinition {
            table: table("line"),
            name: "b".to_owned(),
            expression: Expression::Named("a".to_owned()),
        },
    ];
    assert_eq!(
        CheckedFormulas::compile(store.spec(), cycle, FormulaLimits::default()).err(),
        Some(RelationError::FormulaCycle {
            table: table("line"),
            name: "a".to_owned()
        })
    );
    let mut nested = Expression::Field(field("line_quantity"));
    for _ in 0..4 {
        nested = Expression::Total(vec![nested]);
    }
    assert_eq!(
        CheckedFormulas::compile(
            store.spec(),
            vec![FormulaDefinition {
                table: table("line"),
                name: "deep".to_owned(),
                expression: nested
            }],
            FormulaLimits {
                max_depth: 3,
                ..FormulaLimits::default()
            }
        )
        .err(),
        Some(RelationError::DepthExceeded)
    );
    assert_eq!(
        CheckedFormulas::compile(
            store.spec(),
            definitions(),
            FormulaLimits {
                max_nodes: 2,
                ..FormulaLimits::default()
            }
        )
        .err(),
        Some(RelationError::NodeBudgetExceeded)
    );
    let money_product = Expression::Product(
        Box::new(Expression::Field(field("line_captured"))),
        Box::new(Expression::Field(field("line_captured"))),
    );
    assert_eq!(
        CheckedFormulas::compile(
            store.spec(),
            vec![FormulaDefinition {
                table: table("line"),
                name: "bad".to_owned(),
                expression: money_product
            }],
            FormulaLimits::default()
        )
        .err(),
        Some(RelationError::MoneyProductUnsupported)
    );
    let mut multiple = RelationStore::new(schema(), scope()).unwrap();
    let mut rows = seed(10, Value::Integer(2));
    rows.push(line("second", Value::Integer(3)));
    multiple
        .apply_batch(rows.into_iter().map(BatchChange::Insert).collect())
        .unwrap();
    let low = CheckedFormulas::compile(
        multiple.spec(),
        definitions(),
        FormulaLimits {
            max_records: 1,
            ..FormulaLimits::default()
        },
    )
    .unwrap();
    assert_eq!(
        evaluate(&low, &multiple, "order", "order", "total"),
        Err(RelationError::RecordBudgetExceeded)
    );
    // Compile visits five nodes, while evaluating both matching children uses seven.
    let low = CheckedFormulas::compile(
        multiple.spec(),
        definitions(),
        FormulaLimits {
            max_nodes: 5,
            ..FormulaLimits::default()
        },
    )
    .unwrap();
    assert_eq!(
        evaluate(&low, &multiple, "order", "order", "total"),
        Err(RelationError::NodeBudgetExceeded)
    );
    let low = CheckedFormulas::compile(
        store.spec(),
        definitions(),
        FormulaLimits {
            max_records: 1,
            ..FormulaLimits::default()
        },
    )
    .unwrap();
    assert_eq!(
        evaluate(&low, &store, "line", "line", "live"),
        Err(RelationError::RecordBudgetExceeded),
        "a lookup must count its target row as work"
    );
    assert_eq!(
        CheckedFormulas::compile(
            store.spec(),
            definitions(),
            FormulaLimits {
                max_nodes: 0,
                ..FormulaLimits::default()
            }
        )
        .err(),
        Some(RelationError::InvalidLimits)
    );
}

struct PartialPolicy;
impl OutputPolicy for PartialPolicy {
    fn allow_snapshot(&self, _: &CheckedAppSpec, _: &CheckedRecords) -> bool {
        true
    }
    fn allow_record(&self, _: &Scope, _: &TableId, _: &RecordId) -> bool {
        true
    }
    fn allow_field(&self, _: &Scope, _: &TableId, _: &FieldId) -> bool {
        true
    }
    // No complete scan grant: deny before learning whether hidden rows exist.
}
struct HiddenPrice;
impl OutputPolicy for HiddenPrice {
    fn allow_snapshot(&self, _: &CheckedAppSpec, _: &CheckedRecords) -> bool {
        true
    }
    fn allow_record(&self, _: &Scope, _: &TableId, _: &RecordId) -> bool {
        true
    }
    fn allow_field(&self, _: &Scope, _: &TableId, f: &FieldId) -> bool {
        f != &field("product_price")
    }
    fn allow_complete_scan(&self, _: &Scope, _: &TableId, _: &[FieldId]) -> bool {
        true
    }
}

struct IdOnlyPolicy;
impl OutputPolicy for IdOnlyPolicy {
    fn allow_record(&self, _: &Scope, _: &TableId, _: &RecordId) -> bool {
        true
    }
    fn allow_field(&self, _: &Scope, _: &TableId, _: &FieldId) -> bool {
        true
    }
    fn allow_complete_scan(&self, _: &Scope, _: &TableId, _: &[FieldId]) -> bool {
        true
    }
}

struct SnapshotPolicy {
    spec: CheckedAppSpec,
    records: CheckedRecords,
}
impl OutputPolicy for SnapshotPolicy {
    fn allow_snapshot(&self, spec: &CheckedAppSpec, records: &CheckedRecords) -> bool {
        spec == &self.spec && records == &self.records
    }
    fn allow_record(&self, _: &Scope, _: &TableId, _: &RecordId) -> bool {
        true
    }
    fn allow_field(&self, _: &Scope, _: &TableId, _: &FieldId) -> bool {
        true
    }
    fn allow_complete_scan(&self, _: &Scope, _: &TableId, _: &[FieldId]) -> bool {
        true
    }
}

#[test]
fn id_grants_without_current_snapshot_attestation_cannot_read_projections() {
    let mut store = store(10, Value::Integer(2));
    let formulas =
        CheckedFormulas::compile(store.spec(), definitions(), FormulaLimits::default()).unwrap();
    let policy = SnapshotPolicy {
        spec: store.spec().clone(),
        records: store.records().clone(),
    };
    assert_eq!(
        formulas.evaluate(&store, &table("order"), &record("order"), "total", &policy),
        Ok(Numeric::Money(20))
    );
    assert_eq!(
        formulas.evaluate(
            &store,
            &table("order"),
            &record("order"),
            "total",
            &IdOnlyPolicy
        ),
        Err(RelationError::OutputDenied)
    );
    let mut product = raw(&store, "product", "product");
    product
        .values
        .insert(field("product_price"), Value::Money(50));
    store
        .apply_batch(vec![BatchChange::Replace(product)])
        .unwrap();
    assert_eq!(
        store.lookup(
            &table("line"),
            &record("line"),
            &field("line_product"),
            &field("product_price"),
            &policy
        ),
        Err(RelationError::OutputDenied)
    );
    assert_eq!(
        store.reverse_refs(
            &table("order"),
            &record("order"),
            &table("line"),
            &field("line_order"),
            &policy
        ),
        Err(RelationError::OutputDenied)
    );
    assert_eq!(
        formulas.evaluate(&store, &table("order"), &record("order"), "total", &policy),
        Err(RelationError::OutputDenied)
    );
}

#[test]
fn references_never_grant_visibility_and_empty_projections_also_require_policy() {
    let mut store = store(10, Value::Integer(2));
    let formulas =
        CheckedFormulas::compile(store.spec(), definitions(), FormulaLimits::default()).unwrap();
    assert_eq!(
        formulas.evaluate(&store, &table("line"), &record("line"), "live", &DenyOutput),
        Err(RelationError::OutputDenied)
    );
    assert_eq!(
        formulas.evaluate(
            &store,
            &table("line"),
            &record("line"),
            "live",
            &HiddenPrice
        ),
        Err(RelationError::OutputDenied)
    );
    for populated in [true, false] {
        if !populated {
            store
                .apply_batch(vec![BatchChange::Delete {
                    table_id: table("line"),
                    record_id: record("line"),
                }])
                .unwrap();
        }
        assert_eq!(
            store.reverse_refs(
                &table("order"),
                &record("order"),
                &table("line"),
                &field("line_order"),
                &PartialPolicy
            ),
            Err(RelationError::OutputDenied)
        );
        assert_eq!(
            formulas.evaluate(
                &store,
                &table("order"),
                &record("order"),
                "total",
                &PartialPolicy
            ),
            Err(RelationError::OutputDenied)
        );
    }
    let lookup_sum = vec![FormulaDefinition {
        table: table("order"),
        name: "prices".to_owned(),
        expression: Expression::Sum {
            child_table: table("line"),
            ref_field: field("line_order"),
            value: Box::new(Expression::Lookup {
                ref_field: field("line_product"),
                target_field: field("product_price"),
            }),
        },
    }];
    let formulas =
        CheckedFormulas::compile(store.spec(), lookup_sum, FormulaLimits::default()).unwrap();
    assert_eq!(
        formulas.evaluate(
            &store,
            &table("order"),
            &record("order"),
            "prices",
            &HiddenPrice
        ),
        Err(RelationError::OutputDenied),
        "hidden field is denied even with no child rows"
    );
    let mut definition = schema().definition().clone();
    definition.capture_rules.clear();
    definition
        .tables
        .iter_mut()
        .find(|t| t.id == table("line"))
        .unwrap()
        .fields
        .iter_mut()
        .find(|f| f.id == field("line_product"))
        .unwrap()
        .required = false;
    let mut nullable =
        RelationStore::new(CheckedAppSpec::compile(definition).unwrap(), scope()).unwrap();
    let mut rows = seed(10, Value::Integer(2));
    rows[0].values.insert(field("line_product"), Value::Null);
    rows[0]
        .values
        .insert(field("line_captured"), Value::Money(10));
    nullable
        .apply_batch(rows.into_iter().map(BatchChange::Insert).collect())
        .unwrap();
    assert_eq!(
        nullable.lookup(
            &table("line"),
            &record("line"),
            &field("line_product"),
            &field("product_price"),
            &HiddenPrice
        ),
        Err(RelationError::OutputDenied),
        "a null reference cannot bypass target-field restrictions"
    );
}
