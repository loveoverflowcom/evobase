//! All identity in these tests is explicitly simulated. No token/provider/API is verified here.
use evobase_appspec::policy::*;
use evobase_appspec::{
    AppId, CheckedAppSpec, CheckedRecords, FieldId, FieldType, RawField, RawRecord, RecordId,
    Scope, TableId, Value, fixtures,
};
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

fn field(value: &str) -> FieldId {
    FieldId::new(value).unwrap()
}
fn table(value: &str) -> TableId {
    TableId::new(value).unwrap()
}
fn record(value: &str) -> RecordId {
    RecordId::new(value).unwrap()
}
fn actor(value: &str) -> ActorId {
    ActorId::new(value).unwrap()
}
fn role(value: &str) -> RoleId {
    RoleId::new(value).unwrap()
}

#[derive(Clone)]
struct FixtureState {
    session: SessionFacts,
    membership: MembershipFacts,
    binding: BindingFacts,
}
struct SimulatedVerifier(Rc<RefCell<FixtureState>>);
impl SessionVerifier for SimulatedVerifier {
    fn verify_session(&self, credential: &str) -> Result<SessionFacts, AuthorityError> {
        if credential != "fixture_session" {
            return Err(AuthorityError::Unverified);
        }
        Ok(self.0.borrow().session.clone())
    }
    fn current_membership(
        &self,
        _: &ActorId,
        _: &Scope,
    ) -> Result<MembershipFacts, AuthorityError> {
        Ok(self.0.borrow().membership.clone())
    }
    fn registry_binding(&self, _: &Scope) -> Result<BindingFacts, AuthorityError> {
        Ok(self.0.borrow().binding.clone())
    }
}

fn raw_policy() -> RawOwnerRolePolicy {
    RawOwnerRolePolicy {
        rule_id: "rule_order_owner".to_owned(),
        revision: 1,
        table_id: table("tbl_orders"),
        owner_field: field("fld_order_owner"),
        read_fields: [field("fld_order_state"), field("fld_order_notes")]
            .into_iter()
            .collect(),
        read_roles: [role("role_auditor")].into_iter().collect(),
        write_roles: [role("role_editor")].into_iter().collect(),
        submit_roles: [role("role_operator")].into_iter().collect(),
    }
}
fn raw_command() -> RawSubmitOrderRule {
    RawSubmitOrderRule {
        rule_id: "rule_submit_order".to_owned(),
        revision: 1,
        policy_rule_id: "rule_order_owner".to_owned(),
        order_table: table("tbl_orders"),
        state_field: field("fld_order_state"),
        notes_field: field("fld_order_notes"),
        line_table: table("tbl_order_lines"),
        line_order_field: field("fld_line_order"),
        quantity_field: field("fld_line_quantity"),
        captured_price_field: field("fld_line_price"),
    }
}
struct Harness {
    spec: CheckedAppSpec,
    scope: Scope,
    rows: Vec<RawRecord>,
    policy: OwnerRolePolicy,
    command: SubmitOrderRule,
    host: HostAuthority<SimulatedVerifier>,
    state: Rc<RefCell<FixtureState>>,
}
impl Harness {
    fn new() -> Self {
        let spec = CheckedAppSpec::compile(fixtures::example_policy_spec()).unwrap();
        let scope = Scope::new("tenant_fixture", spec.app_id().clone()).unwrap();
        let policy = OwnerRolePolicy::check(&spec, raw_policy()).unwrap();
        let command = SubmitOrderRule::check(&spec, &policy, raw_command()).unwrap();
        let rows = fixtures::example_records(&scope);
        let state = Rc::new(RefCell::new(FixtureState {
            session: SessionFacts {
                actor: actor("actor_alice"),
                expires_at: 1000,
                revoked: false,
            },
            membership: MembershipFacts {
                actor: actor("actor_alice"),
                scope: scope.clone(),
                active: true,
                revision: 7,
                grants: [Grant::Read, Grant::Submit].into_iter().collect(),
                roles: BTreeSet::new(),
            },
            binding: BindingFacts {
                scope: scope.clone(),
                binding_id: "binding_fixture".to_owned(),
                allowed_mutation_origins: ["https://builder.example".to_owned()]
                    .into_iter()
                    .collect(),
            },
        }));
        Self {
            spec,
            scope,
            rows,
            policy,
            command,
            host: HostAuthority::new(SimulatedVerifier(state.clone())),
            state,
        }
    }
    fn facts(&self) -> CheckedRecords {
        self.spec.validate_records(&self.scope, &self.rows).unwrap()
    }
    fn request(&self) -> RawSubmitOrder {
        RawSubmitOrder {
            order_id: record("rec_order_1"),
            idempotency_key: "submit-1".to_owned(),
            changes: [(
                field("fld_order_notes"),
                Value::Text("  urgent  ".to_owned()),
            )]
            .into_iter()
            .collect(),
            line_patches: vec![LinePatch {
                line_id: record("rec_line_1"),
                changes: [(field("fld_line_quantity"), Value::Integer(3))]
                    .into_iter()
                    .collect(),
            }],
        }
    }
    fn submit(
        &self,
        receipts: &ReceiptBook,
        request: &RawSubmitOrder,
    ) -> Result<CommandDecision, PolicyError> {
        decide_submit_order(
            &self.host,
            "fixture_session",
            &self.scope,
            &RequestChannel::NativeBearer,
            100,
            &self.spec,
            &self.facts(),
            12,
            &self.policy,
            &self.command,
            receipts,
            request,
        )
    }
    fn projection(&self, purpose: ProjectionPurpose) -> Result<CheckedProjection, PolicyError> {
        project(
            &self.host,
            "fixture_session",
            &self.scope,
            &RequestChannel::NativeBearer,
            100,
            &self.facts(),
            &self.policy,
            purpose,
        )
    }
}

#[test]
fn each_host_grant_is_independent_of_every_other_grant() {
    let h = Harness::new();
    let grants = [
        Grant::Design,
        Grant::Publish,
        Grant::Manage,
        Grant::Read,
        Grant::Write,
        Grant::Submit,
    ];
    for held in grants {
        h.state.borrow_mut().membership.grants = [held].into_iter().collect();
        for requested in grants {
            let actual = h.host.resolve(
                "fixture_session",
                &h.scope,
                &RequestChannel::NativeBearer,
                100,
                requested,
            );
            if held == requested {
                assert_eq!(actual.unwrap().actor().as_str(), "actor_alice");
            } else {
                assert_eq!(actual.unwrap_err(), AuthorityError::MissingGrant);
            }
        }
    }
}

#[test]
fn malformed_forged_expired_revoked_and_wrong_scope_authority_deny_precisely() {
    let h = Harness::new();
    let resolve = |credential: &str| {
        h.host.resolve(
            credential,
            &h.scope,
            &RequestChannel::NativeBearer,
            100,
            Grant::Read,
        )
    };
    assert_eq!(resolve("").unwrap_err(), AuthorityError::MissingSession);
    assert_eq!(
        resolve("actor_alice").unwrap_err(),
        AuthorityError::Unverified
    );
    h.state.borrow_mut().session.expires_at = 100;
    assert_eq!(
        resolve("fixture_session").unwrap_err(),
        AuthorityError::Expired
    );
    h.state.borrow_mut().session.expires_at = 1000;
    h.state.borrow_mut().session.revoked = true;
    assert_eq!(
        resolve("fixture_session").unwrap_err(),
        AuthorityError::Revoked
    );
    h.state.borrow_mut().session.revoked = false;
    h.state.borrow_mut().membership.active = false;
    assert_eq!(
        resolve("fixture_session").unwrap_err(),
        AuthorityError::Revoked
    );
    h.state.borrow_mut().membership.active = true;
    h.state.borrow_mut().membership.actor = actor("actor_mallory");
    assert_eq!(
        resolve("fixture_session").unwrap_err(),
        AuthorityError::WrongScope
    );
    h.state.borrow_mut().membership.actor = actor("actor_alice");
    h.state.borrow_mut().membership.scope =
        Scope::new("tenant_other", h.spec.app_id().clone()).unwrap();
    assert_eq!(
        resolve("fixture_session").unwrap_err(),
        AuthorityError::WrongScope
    );
    h.state.borrow_mut().membership.scope =
        Scope::new("tenant_fixture", AppId::new("app_other").unwrap()).unwrap();
    assert_eq!(
        resolve("fixture_session").unwrap_err(),
        AuthorityError::WrongScope
    );
    h.state.borrow_mut().membership.scope = h.scope.clone();
    h.state.borrow_mut().binding.scope =
        Scope::new("tenant_other", h.spec.app_id().clone()).unwrap();
    assert_eq!(
        resolve("fixture_session").unwrap_err(),
        AuthorityError::WrongScope
    );
}

#[test]
fn browser_mutation_requires_exact_origin_while_native_is_a_trusted_transport_choice() {
    let h = Harness::new();
    for origin in [
        None,
        Some("https://builder.example.attacker.test".to_owned()),
        Some("null".to_owned()),
    ] {
        assert_eq!(
            h.host
                .resolve(
                    "fixture_session",
                    &h.scope,
                    &RequestChannel::BrowserMutation { origin },
                    100,
                    Grant::Submit
                )
                .unwrap_err(),
            AuthorityError::OriginDenied
        );
    }
    assert_eq!(
        h.host
            .resolve(
                "fixture_session",
                &h.scope,
                &RequestChannel::BrowserMutation {
                    origin: Some("https://builder.example".to_owned())
                },
                100,
                Grant::Submit
            )
            .unwrap()
            .binding_id(),
        "binding_fixture"
    );
}

#[test]
fn owner_and_roles_require_current_host_business_grants() {
    let h = Harness::new();
    h.state.borrow_mut().session.actor = actor("actor_bob");
    h.state.borrow_mut().membership.actor = actor("actor_bob");
    assert_eq!(
        h.projection(ProjectionPurpose::Query).unwrap().rows().len(),
        0
    );
    h.state
        .borrow_mut()
        .membership
        .roles
        .insert(role("role_auditor"));
    assert_eq!(
        h.projection(ProjectionPurpose::Query).unwrap().rows().len(),
        1
    );
    assert_eq!(
        h.submit(&ReceiptBook::default(), &h.request()).unwrap_err(),
        PolicyError::Denied
    );
    h.state
        .borrow_mut()
        .membership
        .roles
        .insert(role("role_operator"));
    assert!(matches!(
        h.submit(&ReceiptBook::default(), &h.request()).unwrap(),
        CommandDecision::Apply(_)
    ));
    h.state.borrow_mut().membership.grants = [Grant::Manage, Grant::Design, Grant::Publish]
        .into_iter()
        .collect();
    assert_eq!(
        h.projection(ProjectionPurpose::Query).unwrap_err(),
        PolicyError::Authority(AuthorityError::MissingGrant)
    );
    assert_eq!(
        h.submit(&ReceiptBook::default(), &h.request()).unwrap_err(),
        PolicyError::Authority(AuthorityError::MissingGrant)
    );
}

#[test]
fn query_picker_lookup_export_count_and_aggregate_filter_before_observing_output() {
    let mut h = Harness::new();
    let mut definition = fixtures::example_policy_spec();
    definition
        .tables
        .iter_mut()
        .find(|t| t.id == table("tbl_orders"))
        .unwrap()
        .fields
        .push(RawField {
            id: field("fld_order_total"),
            name: "Total".to_owned(),
            required: false,
            field_type: FieldType::Money,
        });
    let mut policy = raw_policy();
    policy.read_fields.insert(field("fld_order_total"));
    policy.read_fields.remove(&field("fld_order_notes"));
    definition.policies = vec![policy.clone()];
    h.spec = CheckedAppSpec::compile(definition).unwrap();
    h.policy = OwnerRolePolicy::check(&h.spec, policy).unwrap();
    let order = h
        .rows
        .iter_mut()
        .find(|row| row.id == record("rec_order_1"))
        .unwrap();
    order
        .values
        .insert(field("fld_order_total"), Value::Money(10));
    order.values.insert(
        field("fld_order_notes"),
        Value::Text("private note".to_owned()),
    );
    let mut hidden = order.clone();
    hidden.id = record("rec_order_hidden");
    hidden.values.insert(
        field("fld_order_owner"),
        Value::Text("actor_bob".to_owned()),
    );
    hidden
        .values
        .insert(field("fld_order_total"), Value::Money(999));
    h.rows.push(hidden);
    for purpose in [
        ProjectionPurpose::Query,
        ProjectionPurpose::Picker,
        ProjectionPurpose::Lookup,
        ProjectionPurpose::Export,
    ] {
        let result = h.projection(purpose).unwrap();
        assert_eq!(
            result
                .rows()
                .iter()
                .map(|row| row.id().as_str())
                .collect::<Vec<_>>(),
            ["rec_order_1"]
        );
        assert!(
            !result.rows()[0]
                .values()
                .contains_key(&field("fld_order_notes"))
        );
        assert!(
            !result.rows()[0]
                .values()
                .contains_key(&field("fld_order_owner"))
        );
        assert!(
            !result.rows()[0]
                .values()
                .contains_key(&field("fld_order_customer"))
        );
    }
    assert_eq!(
        visible_count(
            &h.host,
            "fixture_session",
            &h.scope,
            &RequestChannel::NativeBearer,
            100,
            &h.facts(),
            &h.policy
        )
        .unwrap(),
        1
    );
    assert_eq!(
        visible_sum(
            &h.host,
            "fixture_session",
            &h.scope,
            &RequestChannel::NativeBearer,
            100,
            &h.facts(),
            &h.policy,
            &field("fld_order_total")
        )
        .unwrap(),
        10
    );
    for id in [record("rec_order_hidden"), record("rec_order_absent")] {
        assert!(
            lookup(
                &h.host,
                "fixture_session",
                &h.scope,
                &RequestChannel::NativeBearer,
                100,
                &h.facts(),
                &h.policy,
                &id
            )
            .unwrap()
            .is_none()
        );
    }
    assert_eq!(
        visible_sum(
            &h.host,
            "fixture_session",
            &h.scope,
            &RequestChannel::NativeBearer,
            100,
            &h.facts(),
            &h.policy,
            &field("fld_order_notes")
        )
        .unwrap_err(),
        PolicyError::Denied
    );
}

#[test]
fn ref_output_and_ambiguous_owner_or_reference_command_definitions_are_rejected() {
    let h = Harness::new();
    let mut policy = raw_policy();
    policy.read_fields.insert(field("fld_order_customer"));
    assert_eq!(
        OwnerRolePolicy::check(&h.spec, policy).unwrap_err(),
        PolicyError::InvalidPolicy
    );
    let mut command = raw_command();
    command.state_field = field("fld_order_owner");
    assert_eq!(
        SubmitOrderRule::check(&h.spec, &h.policy, command).unwrap_err(),
        PolicyError::InvalidPolicy
    );
    let mut command = raw_command();
    command.line_order_field = field("fld_line_product");
    assert_eq!(
        SubmitOrderRule::check(&h.spec, &h.policy, command).unwrap_err(),
        PolicyError::InvalidPolicy
    );
    let mut command = raw_command();
    command.captured_price_field = field("fld_product_price");
    assert_eq!(
        SubmitOrderRule::check(&h.spec, &h.policy, command).unwrap_err(),
        PolicyError::InvalidPolicy
    );
    for mutate in [0, 1, 2, 3] {
        let mut definition = fixtures::example_policy_spec();
        let command = &mut definition.submit_rules[0];
        match mutate {
            0 => command.state_field = field("fld_order_owner"),
            1 => command.line_order_field = field("fld_line_product"),
            2 => command.captured_price_field = field("fld_product_price"),
            _ => command.line_table = table("tbl_orders"),
        }
        assert_eq!(
            CheckedAppSpec::compile(definition).unwrap_err(),
            evobase_appspec::Error::InvalidPolicy {
                rule_id: "rule_submit_order".to_owned()
            }
        );
    }
}

#[test]
fn only_exact_canonical_appspec_policy_and_command_rules_can_be_checked() {
    let h = Harness::new();
    let mut broader = raw_policy();
    broader.read_roles.insert(role("role_operator"));
    assert_eq!(
        OwnerRolePolicy::check(&h.spec, broader).unwrap_err(),
        PolicyError::InvalidPolicy
    );
    let mut other_command = raw_command();
    other_command.revision += 1;
    assert_eq!(
        SubmitOrderRule::check(&h.spec, &h.policy, other_command).unwrap_err(),
        PolicyError::InvalidPolicy
    );
    let encoded = h.spec.encode().unwrap();
    let decoded = CheckedAppSpec::decode(&encoded).unwrap();
    assert_eq!(decoded.definition().policies, h.spec.definition().policies);
    assert_eq!(
        decoded.definition().submit_rules,
        h.spec.definition().submit_rules
    );
    let mut duplicate = fixtures::example_policy_spec();
    duplicate.policies.push(duplicate.policies[0].clone());
    assert_eq!(
        CheckedAppSpec::compile(duplicate).unwrap_err(),
        evobase_appspec::Error::InvalidPolicy {
            rule_id: "rule_order_owner".to_owned()
        }
    );
    let mut unsupported = fixtures::example_policy_spec();
    unsupported.policies[0]
        .read_fields
        .insert(field("fld_order_customer"));
    assert_eq!(
        CheckedAppSpec::compile(unsupported).unwrap_err(),
        evobase_appspec::Error::InvalidPolicy {
            rule_id: "rule_order_owner".to_owned()
        }
    );
}

#[test]
fn nullable_or_missing_owner_denies_without_a_current_explicit_role() {
    let mut h = Harness::new();
    let mut definition = fixtures::example_policy_spec();
    definition
        .tables
        .iter_mut()
        .find(|t| t.id == table("tbl_orders"))
        .unwrap()
        .fields
        .iter_mut()
        .find(|f| f.id == field("fld_order_owner"))
        .unwrap()
        .required = false;
    h.spec = CheckedAppSpec::compile(definition).unwrap();
    h.policy = OwnerRolePolicy::check(&h.spec, raw_policy()).unwrap();
    for owner in [Value::Null, Value::Blank] {
        h.rows
            .iter_mut()
            .find(|r| r.id == record("rec_order_1"))
            .unwrap()
            .values
            .insert(field("fld_order_owner"), owner);
        assert_eq!(
            h.projection(ProjectionPurpose::Query).unwrap().rows().len(),
            0
        );
    }
}

#[test]
fn bounded_inputs_reject_oversize_duplicate_and_foreign_line_patches() {
    let h = Harness::new();
    let mut request = h.request();
    request
        .changes
        .insert(field("fld_order_notes"), Value::Text("x".repeat(1001)));
    assert_eq!(
        h.submit(&ReceiptBook::default(), &request).unwrap_err(),
        PolicyError::InvalidInput
    );
    request = h.request();
    request.line_patches.push(request.line_patches[0].clone());
    assert_eq!(
        h.submit(&ReceiptBook::default(), &request).unwrap_err(),
        PolicyError::InvalidInput
    );
    request = h.request();
    request.line_patches[0].line_id = record("rec_other_or_hidden_line");
    assert_eq!(
        h.submit(&ReceiptBook::default(), &request).unwrap_err(),
        PolicyError::InvalidInput
    );
}

#[test]
fn relation_policy_is_bound_to_exact_snapshot_and_pinned_definition() {
    use evobase_appspec::relations::OutputPolicy;
    let h = Harness::new();
    let facts = h.facts();
    let channel = RequestChannel::NativeBearer;
    let policies = [&h.policy];
    let output = CurrentOutputPolicy::new(
        &h.host,
        "fixture_session",
        &h.scope,
        &channel,
        100,
        &facts,
        &policies,
    );
    assert!(output.allow_snapshot(&h.spec, &facts));
    let mut other_rows = facts.to_raw();
    let order = other_rows
        .iter_mut()
        .find(|r| r.id == record("rec_order_1"))
        .unwrap();
    order.values.insert(
        field("fld_order_owner"),
        Value::Text("actor_bob".to_owned()),
    );
    order.values.insert(
        field("fld_order_notes"),
        Value::Text("other tenant private note".to_owned()),
    );
    let changed = h.spec.validate_records(&h.scope, &other_rows).unwrap();
    assert!(!output.allow_snapshot(&h.spec, &changed));
    let mut other_definition = fixtures::example_policy_spec();
    other_definition.policies[0].revision += 1;
    let changed_definition = CheckedAppSpec::compile(other_definition).unwrap();
    assert!(!output.allow_snapshot(&changed_definition, &facts));
    h.state.borrow_mut().membership.active = false;
    assert!(!output.allow_snapshot(&h.spec, &facts));
}

#[test]
fn checked_command_normalizes_mutable_values_and_preserves_owner_scope_and_capture() {
    let h = Harness::new();
    let CommandDecision::Apply(batch) = h.submit(&ReceiptBook::default(), &h.request()).unwrap()
    else {
        panic!("fresh apply required")
    };
    assert_eq!(batch.expected_revision(), 12);
    assert_eq!(batch.membership_revision(), 7);
    assert_eq!(batch.policy_revision(), 1);
    assert_eq!(batch.binding_id(), "binding_fixture");
    assert_eq!(batch.receipt().state(), "submitted");
    let order = batch
        .writes()
        .iter()
        .find(|row| row.id == record("rec_order_1"))
        .unwrap();
    assert_eq!(order.scope, h.scope);
    assert_eq!(
        order.values[&field("fld_order_owner")],
        Value::Text("actor_alice".to_owned())
    );
    assert_eq!(
        order.values[&field("fld_order_notes")],
        Value::Text("urgent".to_owned())
    );
    assert_eq!(
        order.values[&field("fld_order_state")],
        Value::Text("submitted".to_owned())
    );
    let line = batch
        .writes()
        .iter()
        .find(|row| row.id == record("rec_line_1"))
        .unwrap();
    assert_eq!(line.values[&field("fld_line_quantity")], Value::Integer(3));
    assert_eq!(line.values[&field("fld_line_price")], Value::Money(1250));
    assert_eq!(
        line.values[&field("fld_line_product")],
        h.rows
            .iter()
            .find(|r| r.id == record("rec_line_1"))
            .unwrap()
            .values[&field("fld_line_product")]
    );
}

#[test]
fn owner_tenant_references_captured_price_and_invalid_quantities_cannot_be_patched() {
    let h = Harness::new();
    let receipts = ReceiptBook::default();
    for immutable in [
        "fld_order_owner",
        "fld_order_customer",
        "fld_order_state",
        "fld_tenant",
    ] {
        let mut request = h.request();
        request.changes = [(field(immutable), Value::Text("actor_mallory".to_owned()))]
            .into_iter()
            .collect();
        assert_eq!(
            h.submit(&receipts, &request).unwrap_err(),
            PolicyError::ImmutableField
        );
    }
    for immutable in ["fld_line_price", "fld_line_product", "fld_line_order"] {
        let mut request = h.request();
        request.line_patches[0].changes =
            [(field(immutable), Value::Money(1))].into_iter().collect();
        assert_eq!(
            h.submit(&receipts, &request).unwrap_err(),
            PolicyError::ImmutableField
        );
    }
    for quantity in [
        Value::Integer(0),
        Value::Integer(-1),
        Value::Integer(10_001),
        Value::Text("3".to_owned()),
        Value::Null,
    ] {
        let mut request = h.request();
        request.line_patches[0]
            .changes
            .insert(field("fld_line_quantity"), quantity);
        assert_eq!(
            h.submit(&receipts, &request).unwrap_err(),
            PolicyError::InvalidQuantity
        );
    }
}

#[test]
fn old_state_and_complete_order_lines_are_business_guards() {
    let mut h = Harness::new();
    h.rows
        .iter_mut()
        .find(|r| r.id == record("rec_order_1"))
        .unwrap()
        .values
        .insert(
            field("fld_order_state"),
            Value::Text("cancelled".to_owned()),
        );
    assert_eq!(
        h.submit(&ReceiptBook::default(), &h.request()).unwrap_err(),
        PolicyError::NotDraft
    );
    h.rows
        .iter_mut()
        .find(|r| r.id == record("rec_order_1"))
        .unwrap()
        .values
        .insert(field("fld_order_state"), Value::Text("draft".to_owned()));
    h.rows
        .retain(|row| row.table_id != table("tbl_order_lines"));
    let mut request = h.request();
    request.line_patches.clear();
    assert_eq!(
        h.submit(&ReceiptBook::default(), &request).unwrap_err(),
        PolicyError::EmptyOrder
    );
}

#[test]
fn exact_normalized_retry_replays_changed_intent_conflicts_and_revocation_precedes_replay() {
    let mut h = Harness::new();
    let mut receipts = ReceiptBook::default();
    let request = h.request();
    let CommandDecision::Apply(batch) = h.submit(&receipts, &request).unwrap() else {
        panic!("fresh apply required")
    };
    for write in batch.writes() {
        *h.rows
            .iter_mut()
            .find(|row| row.id == write.id && row.table_id == write.table_id)
            .unwrap() = write.clone();
    }
    receipts.record_applied(&batch).unwrap();
    let mut normalized_retry = request.clone();
    normalized_retry
        .changes
        .insert(field("fld_order_notes"), Value::Text("urgent".to_owned()));
    let CommandDecision::Replay(receipt) = h.submit(&receipts, &normalized_retry).unwrap() else {
        panic!("exact replay required")
    };
    assert_eq!(receipt, *batch.receipt());
    normalized_retry.changes.insert(
        field("fld_order_notes"),
        Value::Text("different".to_owned()),
    );
    assert_eq!(
        h.submit(&receipts, &normalized_retry).unwrap_err(),
        PolicyError::IntentConflict
    );
    h.state.borrow_mut().membership.active = false;
    assert_eq!(
        h.submit(&receipts, &request).unwrap_err(),
        PolicyError::Authority(AuthorityError::Revoked)
    );
    assert_eq!(
        h.projection(ProjectionPurpose::Export).unwrap_err(),
        PolicyError::Authority(AuthorityError::Revoked)
    );
}

#[test]
fn authorization_is_checked_before_any_bad_input_or_scope_diagnostic() {
    let h = Harness::new();
    h.state.borrow_mut().membership.grants.clear();
    let mut request = h.request();
    request.idempotency_key.clear();
    request.order_id = record("rec_secret_absent");
    assert_eq!(
        h.submit(&ReceiptBook::default(), &request).unwrap_err(),
        PolicyError::Authority(AuthorityError::MissingGrant)
    );
}

#[test]
fn policy_dto_has_fallible_role_codec_and_cannot_contain_host_capabilities() {
    assert_eq!(
        RoleId::new("admin").unwrap_err(),
        AuthorityError::InvalidIdentity
    );
    assert!(serde_json::from_str::<RoleId>("\"role_/../../admin\"").is_err());
    let mut json = serde_json::to_value(raw_policy()).unwrap();
    json["platform_admin"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<RawOwnerRolePolicy>(json).is_err());
}

#[test]
fn relation_output_adapter_rechecks_grants_for_fields_and_denies_complete_scans_even_when_empty() {
    use evobase_appspec::relations::OutputPolicy;
    let h = Harness::new();
    let facts = h.facts();
    let channel = RequestChannel::NativeBearer;
    let policies = [&h.policy];
    let output = CurrentOutputPolicy::new(
        &h.host,
        "fixture_session",
        &h.scope,
        &channel,
        100,
        &facts,
        &policies,
    );
    assert!(output.allow_record(&h.scope, &table("tbl_orders"), &record("rec_order_1")));
    assert!(output.allow_field(&h.scope, &table("tbl_orders"), &field("fld_order_state")));
    assert!(!output.allow_field(&h.scope, &table("tbl_orders"), &field("fld_order_customer")));
    assert!(!output.allow_record(&h.scope, &table("tbl_orders"), &record("rec_order_absent")));
    assert!(!output.allow_complete_scan(&h.scope, &table("tbl_orders"), &[]));
    h.state.borrow_mut().membership.grants.clear();
    assert!(!output.allow_record(&h.scope, &table("tbl_orders"), &record("rec_order_1")));
    assert!(!output.allow_field(&h.scope, &table("tbl_orders"), &field("fld_order_state")));
    let empty = h.spec.validate_records(&h.scope, &[]).unwrap();
    let empty_output = CurrentOutputPolicy::new(
        &h.host,
        "fixture_session",
        &h.scope,
        &channel,
        100,
        &empty,
        &policies,
    );
    assert!(!empty_output.allow_complete_scan(&h.scope, &table("tbl_orders"), &[]));
}
