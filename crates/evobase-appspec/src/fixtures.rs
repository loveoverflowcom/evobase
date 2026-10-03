//! Small synthetic fixtures; no tenant data, host credentials or trusted authority.
use crate::{
    AppId, CaptureRule, FieldId, FieldType, RawAppSpec, RawField, RawRecord, RawTable, RecordId,
    RecordRef, Scope, TableId, Value,
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
        version: 1,
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
        policies: vec![],
        submit_rules: vec![],
        constraints: vec![],
        state_machines: vec![],
        commands: vec![],
    }
}

/// Explicit owner-policy/command demonstration. Host capabilities and identities are absent.
pub fn example_policy_spec() -> RawAppSpec {
    use crate::policy::{RawOwnerRolePolicy, RawSubmitOrderRule, RoleId};
    let mut definition = example_spec();
    definition.policies.push(RawOwnerRolePolicy {
        rule_id: "rule_order_owner".to_owned(),
        revision: 1,
        table_id: TableId::new("tbl_orders").expect("fixture identity"),
        owner_field: FieldId::new("fld_order_owner").expect("fixture identity"),
        read_fields: ["fld_order_state", "fld_order_notes"]
            .into_iter()
            .map(|id| FieldId::new(id).expect("fixture identity"))
            .collect(),
        read_roles: [RoleId::new("role_auditor").expect("fixture role")]
            .into_iter()
            .collect(),
        write_roles: [RoleId::new("role_editor").expect("fixture role")]
            .into_iter()
            .collect(),
        submit_roles: [RoleId::new("role_operator").expect("fixture role")]
            .into_iter()
            .collect(),
    });
    definition.submit_rules.push(RawSubmitOrderRule {
        rule_id: "rule_submit_order".to_owned(),
        revision: 1,
        policy_rule_id: "rule_order_owner".to_owned(),
        order_table: TableId::new("tbl_orders").expect("fixture identity"),
        state_field: FieldId::new("fld_order_state").expect("fixture identity"),
        notes_field: FieldId::new("fld_order_notes").expect("fixture identity"),
        line_table: TableId::new("tbl_order_lines").expect("fixture identity"),
        line_order_field: FieldId::new("fld_line_order").expect("fixture identity"),
        quantity_field: FieldId::new("fld_line_quantity").expect("fixture identity"),
        captured_price_field: FieldId::new("fld_line_price").expect("fixture identity"),
    });
    definition
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

/// A synthetic request-resolution app, using the same declared executor as the unrelated library.
pub fn support_workflow_spec() -> RawAppSpec {
    workflow_spec(
        "support",
        "requests",
        "request",
        "Support requests",
        ("open", "resolved"),
        "cmd_resolve_request",
        2,
    )
}

/// A synthetic library-return app; these business labels remain fixture data.
pub fn library_workflow_spec() -> RawAppSpec {
    workflow_spec(
        "library",
        "loans",
        "loan",
        "Library loans",
        ("borrowed", "returned"),
        "cmd_return_loan",
        14,
    )
}

fn workflow_spec(
    app: &str,
    table_suffix: &str,
    prefix: &str,
    name: &str,
    states: (&str, &str),
    command_id: &str,
    count: i64,
) -> RawAppSpec {
    let (from, to) = states;
    use crate::commands::{
        CommandGuard, CommandInput, RawCommand, RawEventIntent, RawStateMachine,
    };
    use crate::policy::{RawOwnerRolePolicy, RoleId};
    use crate::{CommandId, Constraint, ConstraintId, EventId, RawFieldConstraint, StateMachineId};
    let table_id = TableId::new(format!("tbl_{table_suffix}")).unwrap();
    let owner = FieldId::new(format!("fld_{prefix}_owner")).unwrap();
    let state = FieldId::new(format!("fld_{prefix}_state")).unwrap();
    let note = FieldId::new(format!("fld_{prefix}_note")).unwrap();
    let count_field = FieldId::new(if prefix == "request" {
        "fld_request_priority"
    } else {
        "fld_loan_days"
    })
    .unwrap();
    let machine_id = StateMachineId::new(format!("machine_{prefix}")).unwrap();
    let policy_id = format!("rule_{prefix}_owner");
    RawAppSpec {
        version: crate::FORMAT_VERSION,
        app_id: AppId::new(format!("app_{app}")).unwrap(),
        name: name.to_owned(),
        tables: vec![RawTable {
            id: table_id.clone(),
            name: name.to_owned(),
            fields: vec![
                RawField {
                    id: owner.clone(),
                    name: "Owner".to_owned(),
                    required: true,
                    field_type: FieldType::Text,
                },
                RawField {
                    id: state.clone(),
                    name: "State".to_owned(),
                    required: true,
                    field_type: FieldType::Text,
                },
                RawField {
                    id: note.clone(),
                    name: "Note".to_owned(),
                    required: false,
                    field_type: FieldType::Text,
                },
                RawField {
                    id: count_field.clone(),
                    name: if prefix == "request" {
                        "Priority"
                    } else {
                        "Days"
                    }
                    .to_owned(),
                    required: true,
                    field_type: FieldType::Integer,
                },
            ],
        }],
        capture_rules: vec![],
        submit_rules: vec![],
        policies: vec![RawOwnerRolePolicy {
            rule_id: policy_id.clone(),
            revision: 1,
            table_id: table_id.clone(),
            owner_field: owner,
            read_fields: [state.clone(), note.clone(), count_field.clone()]
                .into_iter()
                .collect(),
            read_roles: [RoleId::new("role_auditor").unwrap()].into_iter().collect(),
            write_roles: [RoleId::new("role_editor").unwrap()].into_iter().collect(),
            submit_roles: Default::default(),
        }],
        constraints: vec![
            RawFieldConstraint {
                constraint_id: ConstraintId::new(format!("constraint_{prefix}_note")).unwrap(),
                table_id: table_id.clone(),
                field_id: note.clone(),
                constraint: Constraint::TextLength { min: 0, max: 200 },
            },
            RawFieldConstraint {
                constraint_id: ConstraintId::new(format!("constraint_{prefix}_count")).unwrap(),
                table_id: table_id.clone(),
                field_id: count_field.clone(),
                constraint: Constraint::NumericRange { min: 1, max: 30 },
            },
        ],
        state_machines: vec![RawStateMachine {
            machine_id: machine_id.clone(),
            table_id,
            state_field: state.clone(),
            states: [from.to_owned(), to.to_owned()].into_iter().collect(),
            terminal_states: [to.to_owned()].into_iter().collect(),
        }],
        commands: vec![RawCommand {
            command_id: CommandId::new(command_id).unwrap(),
            revision: 1,
            policy_rule_id: policy_id,
            state_machine_id: machine_id,
            from_state: from.to_owned(),
            to_state: to.to_owned(),
            inputs: vec![
                CommandInput {
                    field_id: note.clone(),
                    required: true,
                    constraints: vec![
                        Constraint::NonEmpty,
                        Constraint::TextLength { min: 1, max: 200 },
                    ],
                },
                CommandInput {
                    field_id: count_field.clone(),
                    required: false,
                    constraints: vec![Constraint::NumericRange { min: 1, max: 30 }],
                },
            ],
            guards: vec![CommandGuard {
                field_id: count_field,
                equals: Value::Integer(count),
            }],
            events: vec![RawEventIntent {
                event_id: EventId::new(format!("event_{prefix}_completed")).unwrap(),
                fields: [state, note].into_iter().collect(),
            }],
        }],
    }
}

pub fn support_workflow_records(scope: &Scope) -> Vec<RawRecord> {
    let first = record(
        scope,
        "tbl_requests",
        "rec_request_1",
        vec![
            ("fld_request_owner", Value::Text("actor_alice".to_owned())),
            ("fld_request_state", Value::Text("open".to_owned())),
            ("fld_request_priority", Value::Integer(2)),
        ],
    );
    vec![
        first.clone(),
        RawRecord {
            id: RecordId::new("rec_request_2").unwrap(),
            ..first
        },
    ]
}

pub fn library_workflow_records(scope: &Scope) -> Vec<RawRecord> {
    vec![record(
        scope,
        "tbl_loans",
        "rec_loan_1",
        vec![
            ("fld_loan_owner", Value::Text("actor_alice".to_owned())),
            ("fld_loan_state", Value::Text("borrowed".to_owned())),
            ("fld_loan_days", Value::Integer(14)),
        ],
    )]
}
