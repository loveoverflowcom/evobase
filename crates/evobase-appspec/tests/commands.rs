use evobase_appspec::commands::*;
use evobase_appspec::policy::*;
use evobase_appspec::*;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

#[derive(Clone)]
struct Access {
    actor: ActorId,
    scope: Scope,
    active: bool,
    roles: BTreeSet<RoleId>,
    grants: BTreeSet<Grant>,
    now: u64,
}
#[derive(Clone)]
struct FixtureVerifier(Rc<RefCell<Access>>);
impl SessionVerifier for FixtureVerifier {
    fn verify_session(&self, credential: &str) -> Result<SessionFacts, AuthorityError> {
        if credential != "fixture" {
            return Err(AuthorityError::Unverified);
        }
        Ok(SessionFacts {
            actor: self.0.borrow().actor.clone(),
            expires_at: 1000,
            revoked: false,
        })
    }
    fn current_membership(
        &self,
        actor: &ActorId,
        scope: &Scope,
    ) -> Result<MembershipFacts, AuthorityError> {
        let access = self.0.borrow();
        Ok(MembershipFacts {
            actor: actor.clone(),
            scope: scope.clone(),
            active: access.active,
            revision: 1,
            grants: access.grants.clone(),
            roles: access.roles.clone(),
        })
    }
    fn registry_binding(&self, scope: &Scope) -> Result<BindingFacts, AuthorityError> {
        if scope != &self.0.borrow().scope {
            return Err(AuthorityError::WrongScope);
        }
        Ok(BindingFacts {
            scope: scope.clone(),
            binding_id: "store:tenant_demo".to_owned(),
            allowed_mutation_origins: Default::default(),
        })
    }
    fn current_time(&self, _provided_now: u64) -> u64 {
        self.0.borrow().now
    }
}
fn host(scope: &Scope) -> (HostAuthority<FixtureVerifier>, Rc<RefCell<Access>>) {
    let access = Rc::new(RefCell::new(Access {
        actor: ActorId::new("actor_alice").unwrap(),
        scope: scope.clone(),
        active: true,
        roles: Default::default(),
        grants: [Grant::Read, Grant::Write].into_iter().collect(),
        now: 100,
    }));
    (HostAuthority::new(FixtureVerifier(access.clone())), access)
}
fn request(command: &str, record: &str, note: &str) -> RawCommandRequest {
    let prefix = if command == "cmd_resolve_request" {
        "request"
    } else {
        "loan"
    };
    RawCommandRequest {
        command_id: CommandId::new(command).unwrap(),
        record_id: RecordId::new(record).unwrap(),
        idempotency_key: "retry_1".to_owned(),
        inputs: [(
            FieldId::new(format!("fld_{prefix}_note")).unwrap(),
            Value::Text(note.to_owned()),
        )]
        .into_iter()
        .collect(),
    }
}
fn prepare(
    host: &HostAuthority<FixtureVerifier>,
    spec: &CheckedAppSpec,
    facts: &CheckedRecords,
    request: &RawCommandRequest,
) -> Result<PreparedCommand, CommandError> {
    prepare_command(
        host,
        "fixture",
        facts.scope(),
        &RequestChannel::NativeBearer,
        100,
        spec,
        facts,
        request,
    )
}

#[test]
fn unrelated_app_transitions_write_only_allowed_fields_and_checked_event_projections() {
    for (raw, rows, command, record, state_field, final_state) in [
        (
            fixtures::support_workflow_spec(),
            fixtures::support_workflow_records as fn(&Scope) -> Vec<RawRecord>,
            "cmd_resolve_request",
            "rec_request_1",
            "fld_request_state",
            "resolved",
        ),
        (
            fixtures::library_workflow_spec(),
            fixtures::library_workflow_records as fn(&Scope) -> Vec<RawRecord>,
            "cmd_return_loan",
            "rec_loan_1",
            "fld_loan_state",
            "returned",
        ),
    ] {
        let spec = CheckedAppSpec::compile(raw).unwrap();
        let scope = Scope::new("tenant_demo", spec.app_id().clone()).unwrap();
        let facts = spec.validate_records(&scope, &rows(&scope)).unwrap();
        let (host, _) = host(&scope);
        let request = request(command, record, "  completed  ");
        let eligible = available_command_records(
            &host,
            "fixture",
            &scope,
            &RequestChannel::NativeBearer,
            100,
            &spec,
            &facts,
        )
        .unwrap();
        assert_eq!(eligible.len(), 1);
        assert!(eligible[0].1.contains(&request.record_id));
        let prepared = prepare(&host, &spec, &facts, &request).unwrap();
        let original_intent = prepared.intent_bytes().to_vec();
        let batch = decide_command(prepared, 7).unwrap();
        assert_eq!(batch.expected_revision(), 7);
        assert_eq!(batch.writes().len(), 1);
        assert_eq!(batch.receipt().state(), final_state);
        assert_eq!(batch.writes()[0].scope, scope);
        let old = facts
            .find(&batch.writes()[0].table_id, &request.record_id)
            .unwrap();
        for (field, value) in old.values() {
            if field.as_str() != state_field && !request.inputs.contains_key(field) {
                assert_eq!(batch.writes()[0].values.get(field), Some(value));
            }
        }
        assert_eq!(batch.events().len(), 1);
        assert_eq!(batch.events()[0].payload().len(), 2);
        assert_eq!(
            batch.events()[0]
                .payload()
                .get(&FieldId::new(state_field).unwrap()),
            Some(&Value::Text(final_state.to_owned()))
        );
        let mut final_rows = facts.to_raw();
        *final_rows
            .iter_mut()
            .find(|row| row.id == request.record_id)
            .unwrap() = batch.writes()[0].clone();
        let committed = spec.validate_records(&scope, &final_rows).unwrap();
        // Persistent adapters can replay from this authorization step despite terminal state.
        let retry = prepare(&host, &spec, &committed, &request).unwrap();
        assert_eq!(retry.intent_bytes(), original_intent);
        assert_eq!(
            decide_command(retry, 8).unwrap_err(),
            CommandError::InvalidTransition {
                from: final_state.to_owned(),
                to: final_state.to_owned()
            }
        );
        assert_eq!(facts, spec.validate_records(&scope, &rows(&scope)).unwrap());
        let receipt: TransitionReceipt =
            serde_json::from_slice(&serde_json::to_vec(batch.receipt()).unwrap()).unwrap();
        assert_eq!(receipt, *batch.receipt());
    }
}

#[test]
fn authority_precedes_malformed_input_replay_and_scope_diagnostics() {
    let spec = CheckedAppSpec::compile(fixtures::support_workflow_spec()).unwrap();
    let scope = Scope::new("tenant_demo", spec.app_id().clone()).unwrap();
    let facts = spec
        .validate_records(&scope, &fixtures::support_workflow_records(&scope))
        .unwrap();
    let (host, access) = host(&scope);
    let mut request = request("cmd_resolve_request", "rec_request_1", "ok");
    request.idempotency_key = "bad key".to_owned();
    access.borrow_mut().active = false;
    assert_eq!(
        prepare(&host, &spec, &facts, &request).unwrap_err(),
        CommandError::Authority(AuthorityError::Revoked)
    );
    access.borrow_mut().active = true;
    access.borrow_mut().now = 1000;
    assert_eq!(
        prepare(&host, &spec, &facts, &request).unwrap_err(),
        CommandError::Authority(AuthorityError::Expired)
    );
    access.borrow_mut().now = 100;
    access.borrow_mut().grants.remove(&Grant::Write);
    assert_eq!(
        prepare(&host, &spec, &facts, &request).unwrap_err(),
        CommandError::Authority(AuthorityError::MissingGrant)
    );
}

#[test]
fn hidden_owner_state_inputs_blank_type_and_guard_bypasses_reject_exactly() {
    let spec = CheckedAppSpec::compile(fixtures::support_workflow_spec()).unwrap();
    let scope = Scope::new("tenant_demo", spec.app_id().clone()).unwrap();
    let rows = fixtures::support_workflow_records(&scope);
    let facts = spec.validate_records(&scope, &rows).unwrap();
    let (host, _) = host(&scope);
    for field in ["fld_request_owner", "fld_request_state"] {
        let mut request = request("cmd_resolve_request", "rec_request_1", "ok");
        request.inputs.insert(
            FieldId::new(field).unwrap(),
            Value::Text("forged".to_owned()),
        );
        assert_eq!(
            prepare(&host, &spec, &facts, &request).unwrap_err(),
            CommandError::ImmutableField(FieldId::new(field).unwrap())
        );
    }
    for (value, reason) in [
        (Value::Blank, "required input is blank"),
        (Value::Null, "required input is blank"),
        (Value::Text("  ".to_owned()), "input constraint failed"),
        (Value::Integer(2), "wrong type or text limit"),
    ] {
        let mut request = request("cmd_resolve_request", "rec_request_1", "ok");
        request
            .inputs
            .insert(FieldId::new("fld_request_note").unwrap(), value);
        assert_eq!(
            prepare(&host, &spec, &facts, &request).unwrap_err(),
            CommandError::InvalidInput {
                field: Some(FieldId::new("fld_request_note").unwrap()),
                reason
            }
        );
    }
    let mut missing = request("cmd_resolve_request", "rec_request_1", "ok");
    missing.inputs.clear();
    assert_eq!(
        prepare(&host, &spec, &facts, &missing).unwrap_err(),
        CommandError::InvalidInput {
            field: Some(FieldId::new("fld_request_note").unwrap()),
            reason: "required input is absent"
        }
    );
    let mut changed = rows.clone();
    changed[0].values.insert(
        FieldId::new("fld_request_priority").unwrap(),
        Value::Integer(3),
    );
    let guarded = spec.validate_records(&scope, &changed).unwrap();
    let prepared = prepare(
        &host,
        &spec,
        &guarded,
        &request("cmd_resolve_request", "rec_request_1", "ok"),
    )
    .unwrap();
    assert_eq!(
        decide_command(prepared, 1).unwrap_err(),
        CommandError::GuardFailed {
            field: FieldId::new("fld_request_priority").unwrap()
        }
    );
    assert_eq!(
        facts.to_raw(),
        spec.validate_records(&scope, &rows).unwrap().to_raw()
    );
}

#[test]
fn final_field_constraint_is_enforced_and_partial_inputs_preserve_hidden_facts() {
    let mut raw = fixtures::support_workflow_spec();
    raw.constraints
        .iter_mut()
        .find(|rule| rule.field_id.as_str() == "fld_request_priority")
        .unwrap()
        .constraint = Constraint::NumericRange { min: 1, max: 5 };
    let spec = CheckedAppSpec::compile(raw).unwrap();
    let scope = Scope::new("tenant_demo", spec.app_id().clone()).unwrap();
    let facts = spec
        .validate_records(&scope, &fixtures::support_workflow_records(&scope))
        .unwrap();
    let (host, _) = host(&scope);
    let mut request = request("cmd_resolve_request", "rec_request_1", "ok");
    request.inputs.insert(
        FieldId::new("fld_request_priority").unwrap(),
        Value::Integer(10),
    );
    let prepared = prepare(&host, &spec, &facts, &request).unwrap();
    assert_eq!(
        decide_command(prepared, 1).unwrap_err(),
        CommandError::Kernel(Error::ConstraintViolation {
            record: RecordId::new("rec_request_1").unwrap(),
            field: FieldId::new("fld_request_priority").unwrap(),
            constraint: ConstraintId::new("constraint_request_count").unwrap()
        })
    );
}

#[test]
fn executable_metadata_intersects_read_write_scope_and_compares_all_current_roles() {
    let spec = CheckedAppSpec::compile(fixtures::support_workflow_spec()).unwrap();
    let scope = Scope::new("tenant_demo", spec.app_id().clone()).unwrap();
    let facts = spec
        .validate_records(&scope, &fixtures::support_workflow_records(&scope))
        .unwrap();
    let (host, access) = host(&scope);
    let request = request("cmd_resolve_request", "rec_request_1", "ok");
    access.borrow_mut().actor = ActorId::new("actor_bob").unwrap();
    access
        .borrow_mut()
        .roles
        .insert(RoleId::new("role_editor").unwrap());
    let prepared = prepare(&host, &spec, &facts, &request).unwrap();
    assert!(
        available_commands(
            &host,
            "fixture",
            &scope,
            &RequestChannel::NativeBearer,
            100,
            &spec,
            &facts
        )
        .unwrap()
        .is_empty()
    );
    access.borrow_mut().roles.clear();
    let current = host
        .resolve(
            "fixture",
            &scope,
            &RequestChannel::NativeBearer,
            100,
            Grant::Write,
        )
        .unwrap();
    assert_ne!(prepared.authority(), &current); // Revision remains 1: roles still participate.
    assert_eq!(
        prepare(&host, &spec, &facts, &request).unwrap_err(),
        CommandError::Policy(PolicyError::Denied)
    );
    access.borrow_mut().actor = ActorId::new("actor_alice").unwrap();
    access.borrow_mut().grants.remove(&Grant::Read);
    assert!(prepare(&host, &spec, &facts, &request).is_ok());
    assert_eq!(
        available_commands(
            &host,
            "fixture",
            &scope,
            &RequestChannel::NativeBearer,
            100,
            &spec,
            &facts
        )
        .unwrap_err(),
        CommandError::Authority(AuthorityError::MissingGrant)
    );
}

#[test]
fn definitions_terminal_and_unknown_states_and_input_binding_errors_are_checked() {
    let mut raw = fixtures::support_workflow_spec();
    raw.commands[0].from_state = "resolved".to_owned();
    assert_eq!(
        CheckedAppSpec::compile(raw),
        Err(Error::InvalidCommand {
            command: CommandId::new("cmd_resolve_request").unwrap(),
            reason: "unknown, terminal, or identical transition states"
        })
    );
    let mut raw = fixtures::support_workflow_spec();
    raw.commands[0].inputs[0].field_id = FieldId::new("fld_request_owner").unwrap();
    assert_eq!(
        CheckedAppSpec::compile(raw),
        Err(Error::InvalidCommand {
            command: CommandId::new("cmd_resolve_request").unwrap(),
            reason: "duplicate or immutable input field"
        })
    );
    let spec = CheckedAppSpec::compile(fixtures::support_workflow_spec()).unwrap();
    let scope = Scope::new("tenant_demo", spec.app_id().clone()).unwrap();
    let mut rows = fixtures::support_workflow_records(&scope);
    rows[0].values.insert(
        FieldId::new("fld_request_state").unwrap(),
        Value::Text("unpublished".to_owned()),
    );
    let expected = Error::InvalidStateValue {
        record: RecordId::new("rec_request_1").unwrap(),
        field: FieldId::new("fld_request_state").unwrap(),
        machine: StateMachineId::new("machine_request").unwrap(),
    };
    assert_eq!(spec.validate_records(&scope, &rows), Err(expected.clone()));
    assert_eq!(
        spec.decode_records(&scope, &serde_json::to_vec(&rows).unwrap()),
        Err(expected)
    );
}

#[test]
fn request_codec_rejects_duplicate_fields_and_semantic_intent_binds_exact_definition() {
    let duplicate=br#"{"command_id":"cmd_resolve_request","record_id":"rec_request_1","idempotency_key":"key","inputs":{"fld_request_note":{"type":"text","value":"one"},"fld_request_note":{"type":"text","value":"two"}}}"#;
    match RawCommandRequest::decode(duplicate).unwrap_err() {
        Error::InvalidJson { message } => {
            assert!(message.contains("duplicate field value fld_request_note"))
        }
        other => panic!("{other:?}"),
    }
    let raw = fixtures::support_workflow_spec();
    let spec = CheckedAppSpec::compile(raw.clone()).unwrap();
    let scope = Scope::new("tenant_demo", spec.app_id().clone()).unwrap();
    let facts = spec
        .validate_records(&scope, &fixtures::support_workflow_records(&scope))
        .unwrap();
    let (host, _) = host(&scope);
    let request = request("cmd_resolve_request", "rec_request_1", "ok");
    let original = prepare(&host, &spec, &facts, &request).unwrap();
    let mut changed = raw;
    changed.commands[0].revision = 2;
    let changed = CheckedAppSpec::compile(changed).unwrap();
    assert_ne!(
        original.intent_bytes(),
        prepare(&host, &changed, &facts, &request)
            .unwrap()
            .intent_bytes()
    );
    let mut alternate = request.clone();
    alternate.inputs.insert(
        FieldId::new("fld_request_note").unwrap(),
        Value::Text("different".to_owned()),
    );
    assert_ne!(
        original.intent_bytes(),
        prepare(&host, &spec, &facts, &alternate)
            .unwrap()
            .intent_bytes()
    );
    alternate = request.clone();
    alternate.idempotency_key = "different_key".to_owned();
    assert_eq!(
        original.intent_bytes(),
        prepare(&host, &spec, &facts, &alternate)
            .unwrap()
            .intent_bytes()
    );
    assert_eq!(
        decide_command(original, 0).unwrap_err(),
        CommandError::RevisionRequired
    );
}

#[test]
fn workflow_declarations_roundtrip_reorder_and_renamed_labels_keep_decisions() {
    let mut raw = fixtures::support_workflow_spec();
    let spec = CheckedAppSpec::compile(raw.clone()).unwrap();
    assert_eq!(
        CheckedAppSpec::decode(&spec.encode().unwrap()).unwrap(),
        spec
    );
    raw.tables[0].fields.reverse();
    raw.commands[0].inputs.reverse();
    raw.commands[0].guards.reverse();
    raw.constraints.reverse();
    assert_eq!(
        CheckedAppSpec::compile(raw.clone())
            .unwrap()
            .encode()
            .unwrap(),
        spec.encode().unwrap()
    );
    raw.name = "Yêu cầu".to_owned();
    raw.tables[0].name = "Đổi tên".to_owned();
    let renamed = CheckedAppSpec::compile(raw.clone()).unwrap();
    let scope = Scope::new("tenant_demo", spec.app_id().clone()).unwrap();
    let facts = spec
        .validate_records(&scope, &fixtures::support_workflow_records(&scope))
        .unwrap();
    let (host, _) = host(&scope);
    let request = request("cmd_resolve_request", "rec_request_1", "ok");
    assert_eq!(
        decide_command(prepare(&host, &spec, &facts, &request).unwrap(), 1)
            .unwrap()
            .writes(),
        decide_command(prepare(&host, &renamed, &facts, &request).unwrap(), 1)
            .unwrap()
            .writes()
    );
    raw.version = 1;
    raw.constraints.clear();
    assert_eq!(
        CheckedAppSpec::compile(raw),
        Err(Error::UnsupportedDeclaration {
            version: 1,
            declaration: "state_machines"
        })
    );
}

#[test]
fn reviewed_workflow_and_records_fixtures_match_the_portable_checked_contract() {
    for (raw, records, definition_json, records_json) in [
        (
            fixtures::support_workflow_spec(),
            fixtures::support_workflow_records as fn(&Scope) -> Vec<RawRecord>,
            include_bytes!("fixtures/support-v2.json").as_slice(),
            include_bytes!("fixtures/support-records.json").as_slice(),
        ),
        (
            fixtures::library_workflow_spec(),
            fixtures::library_workflow_records as fn(&Scope) -> Vec<RawRecord>,
            include_bytes!("fixtures/library-v2.json").as_slice(),
            include_bytes!("fixtures/library-records.json").as_slice(),
        ),
    ] {
        let spec = CheckedAppSpec::compile(raw).unwrap();
        assert_eq!(spec.encode().unwrap(), definition_json);
        assert_eq!(CheckedAppSpec::decode(definition_json).unwrap(), spec);
        let scope = Scope::new("tenant_demo", spec.app_id().clone()).unwrap();
        let facts = spec.validate_records(&scope, &records(&scope)).unwrap();
        assert_eq!(facts.encode().unwrap(), records_json);
        assert_eq!(spec.decode_records(&scope, records_json).unwrap(), facts);
    }
}
