//! Bounded declarative single-record transitions. Persistence owns replay and atomic commit.
use crate::policy::{
    ActorId, AuthorityError, Grant, HostAuthority, OwnerRolePolicy, PolicyError, RequestChannel,
    SessionVerifier, TrustedContext,
};
use crate::{
    CheckedAppSpec, CheckedRecords, CommandId, Constraint, Error, EventId, FieldId, FieldType,
    RawAppSpec, RawRecord, RecordId, Scope, StateMachineId, TableId, Value,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_COMMANDS: usize = 64;
pub const MAX_STATE_MACHINES: usize = 64;
pub const MAX_STATES: usize = 64;
pub const MAX_COMMAND_INPUTS: usize = 64;
pub const MAX_GUARDS: usize = 32;
pub const MAX_EVENTS: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawStateMachine {
    pub machine_id: StateMachineId,
    pub table_id: TableId,
    pub state_field: FieldId,
    pub states: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub terminal_states: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandInput {
    /// Input identity is the writable target field identity in this profile.
    pub field_id: FieldId,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandGuard {
    pub field_id: FieldId,
    pub equals: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEventIntent {
    pub event_id: EventId,
    /// Payload is an allowlisted projection of the proposed new record, never raw input.
    pub fields: BTreeSet<FieldId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCommand {
    pub command_id: CommandId,
    pub revision: u64,
    pub policy_rule_id: String,
    pub state_machine_id: StateMachineId,
    pub from_state: String,
    pub to_state: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<CommandInput>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guards: Vec<CommandGuard>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<RawEventIntent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawCommandRequest {
    pub command_id: CommandId,
    pub record_id: RecordId,
    pub idempotency_key: String,
    #[serde(deserialize_with = "crate::value::unique_values")]
    pub inputs: BTreeMap<FieldId, Value>,
}
impl RawCommandRequest {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        crate::codec::preflight(bytes)?;
        serde_json::from_slice(bytes).map_err(crate::codec::json_error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    Authority(#[from] AuthorityError),
    #[error(transparent)]
    Policy(#[from] PolicyError),
    #[error(transparent)]
    Kernel(#[from] Error),
    #[error("unknown command {0}")]
    UnknownCommand(CommandId),
    #[error("invalid command input {field:?}: {reason}")]
    InvalidInput {
        field: Option<FieldId>,
        reason: &'static str,
    },
    #[error("field {0} is immutable for this command")]
    ImmutableField(FieldId),
    #[error("transition from {from} to {to} is not available")]
    InvalidTransition { from: String, to: String },
    #[error("command guard failed for field {field}")]
    GuardFailed { field: FieldId },
    #[error("expected data revision must be positive")]
    RevisionRequired,
}

pub(crate) fn check_limits(raw: &RawAppSpec) -> Result<(), Error> {
    for (resource, count, limit) in [
        (
            "state machines",
            raw.state_machines.len(),
            MAX_STATE_MACHINES,
        ),
        ("commands", raw.commands.len(), MAX_COMMANDS),
    ] {
        if count > limit {
            return Err(Error::LimitExceeded { resource, limit });
        }
    }
    for machine in &raw.state_machines {
        if machine.states.len() > MAX_STATES || machine.terminal_states.len() > MAX_STATES {
            return Err(Error::LimitExceeded {
                resource: "states",
                limit: MAX_STATES,
            });
        }
    }
    for command in &raw.commands {
        for (resource, count, limit) in [
            ("command inputs", command.inputs.len(), MAX_COMMAND_INPUTS),
            ("command guards", command.guards.len(), MAX_GUARDS),
            ("command events", command.events.len(), MAX_EVENTS),
        ] {
            if count > limit {
                return Err(Error::LimitExceeded { resource, limit });
            }
        }
        if command
            .inputs
            .iter()
            .any(|input| input.constraints.len() > 8)
        {
            return Err(Error::LimitExceeded {
                resource: "input constraints",
                limit: 8,
            });
        }
        if command
            .events
            .iter()
            .any(|event| event.fields.len() > MAX_COMMAND_INPUTS)
        {
            return Err(Error::LimitExceeded {
                resource: "event fields",
                limit: MAX_COMMAND_INPUTS,
            });
        }
    }
    Ok(())
}

fn value_matches(field_type: &FieldType, value: &Value) -> bool {
    matches!(
        (field_type, value),
        (_, Value::Blank | Value::Null)
            | (FieldType::Text, Value::Text(_))
            | (FieldType::Integer, Value::Integer(_))
            | (FieldType::Money, Value::Money(_))
            | (FieldType::Bool, Value::Bool(_))
    )
}

pub(crate) fn check_declarations(spec: &CheckedAppSpec) -> Result<(), Error> {
    let mut machine_ids = BTreeSet::new();
    let mut machine_tables = BTreeSet::new();
    for machine in &spec.definition().state_machines {
        let invalid = |reason| Error::InvalidStateMachine {
            machine: machine.machine_id.clone(),
            reason,
        };
        if !machine_ids.insert(&machine.machine_id) || !machine_tables.insert(&machine.table_id) {
            return Err(invalid("duplicate machine identity or table binding"));
        }
        if machine.states.is_empty()
            || machine.states.iter().any(|state| {
                state.is_empty() || state.len() > 64 || state.chars().any(char::is_control)
            })
            || !machine.terminal_states.is_subset(&machine.states)
        {
            return Err(invalid("invalid finite states or terminal subset"));
        }
        if !spec
            .field(&machine.table_id, &machine.state_field)
            .is_some_and(|field| field.field_type == FieldType::Text && field.required)
        {
            return Err(invalid("state field must be required text"));
        }
        for state in &machine.states {
            if spec.definition().constraints.iter().any(|rule| {
                rule.table_id == machine.table_id
                    && rule.field_id == machine.state_field
                    && !rule.constraint.accepts(&Value::Text(state.clone()))
            }) {
                return Err(invalid("declared state violates a field constraint"));
            }
        }
    }
    let mut ids = BTreeSet::new();
    for command in &spec.definition().commands {
        let invalid = |reason| Error::InvalidCommand {
            command: command.command_id.clone(),
            reason,
        };
        if !ids.insert(&command.command_id) || command.revision == 0 {
            return Err(invalid("duplicate command identity or zero revision"));
        }
        let machine = spec
            .definition()
            .state_machines
            .iter()
            .find(|machine| machine.machine_id == command.state_machine_id)
            .ok_or_else(|| invalid("unknown state machine"))?;
        let policy = spec
            .definition()
            .policies
            .iter()
            .find(|policy| policy.rule_id == command.policy_rule_id)
            .ok_or_else(|| invalid("unknown policy"))?;
        if policy.table_id != machine.table_id || policy.owner_field == machine.state_field {
            return Err(invalid("policy and state bindings do not match"));
        }
        if !machine.states.contains(&command.from_state)
            || !machine.states.contains(&command.to_state)
            || machine.terminal_states.contains(&command.from_state)
            || command.from_state == command.to_state
        {
            return Err(invalid("unknown, terminal, or identical transition states"));
        }
        let mut fields = BTreeSet::new();
        for input in &command.inputs {
            let field = spec
                .field(&machine.table_id, &input.field_id)
                .ok_or_else(|| invalid("unknown input field"))?;
            if !fields.insert(&input.field_id)
                || input.field_id == machine.state_field
                || input.field_id == policy.owner_field
                || matches!(field.field_type, FieldType::Ref { .. })
                || spec.definition().capture_rules.iter().any(|capture| {
                    capture.line_table == machine.table_id
                        && (capture.captured_price_field == input.field_id
                            || capture.product_ref_field == input.field_id)
                })
            {
                return Err(invalid("duplicate or immutable input field"));
            }
            for constraint in &input.constraints {
                constraint
                    .check_type(&field.field_type)
                    .map_err(|_| invalid("invalid input constraint"))?;
            }
        }
        let mut guards = BTreeSet::new();
        for guard in &command.guards {
            let field = spec
                .field(&machine.table_id, &guard.field_id)
                .ok_or_else(|| invalid("unknown guard field"))?;
            if !guards.insert(&guard.field_id)
                || matches!(field.field_type, FieldType::Ref { .. })
                || !value_matches(&field.field_type, &guard.equals)
                || matches!(&guard.equals,Value::Text(text) if text.len()>crate::MAX_TEXT_BYTES)
            {
                return Err(invalid("duplicate or incompatible equality guard"));
            }
        }
        let mut events = BTreeSet::new();
        for event in &command.events {
            if !events.insert(&event.event_id)
                || event
                    .fields
                    .iter()
                    .any(|field| !policy.read_fields.contains(field))
            {
                return Err(invalid("duplicate event or non-readable payload field"));
            }
        }
    }
    Ok(())
}

/// Authorized normalized intent. No state or guard decision occurs before persistent replay.
/// ```compile_fail
/// use evobase_appspec::commands::PreparedCommand;
/// let _: PreparedCommand = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug)]
pub struct PreparedCommand {
    spec: CheckedAppSpec,
    facts: CheckedRecords,
    context: TrustedContext,
    policy: OwnerRolePolicy,
    command: RawCommand,
    machine: RawStateMachine,
    request: RawCommandRequest,
    intent_bytes: Vec<u8>,
}
impl PreparedCommand {
    pub fn authority(&self) -> &TrustedContext {
        &self.context
    }
    pub fn scope(&self) -> &Scope {
        self.context.scope()
    }
    pub fn actor(&self) -> &ActorId {
        self.context.actor()
    }
    pub fn binding_id(&self) -> &str {
        self.context.binding_id()
    }
    pub fn membership_revision(&self) -> u64 {
        self.context.membership_revision()
    }
    pub fn policy_revision(&self) -> u64 {
        self.policy.revision()
    }
    pub fn idempotency_key(&self) -> &str {
        &self.request.idempotency_key
    }
    pub fn intent_bytes(&self) -> &[u8] {
        &self.intent_bytes
    }
    pub fn command_id(&self) -> &CommandId {
        &self.command.command_id
    }
    pub fn record_id(&self) -> &RecordId {
        &self.request.record_id
    }
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_command<V: SessionVerifier>(
    host: &HostAuthority<V>,
    credential: &str,
    scope: &Scope,
    channel: &RequestChannel,
    now: u64,
    spec: &CheckedAppSpec,
    facts: &CheckedRecords,
    request: &RawCommandRequest,
) -> Result<PreparedCommand, CommandError> {
    // Resolve before reading the requested command, row, input detail or persisted receipt.
    let context = host.resolve(credential, scope, channel, now, Grant::Write)?;
    if scope.app_id() != spec.app_id() || facts.scope() != scope {
        return Err(PolicyError::ScopeMismatch.into());
    }
    let command = spec
        .definition()
        .commands
        .iter()
        .find(|command| command.command_id == request.command_id)
        .ok_or_else(|| CommandError::UnknownCommand(request.command_id.clone()))?;
    let machine = spec
        .definition()
        .state_machines
        .iter()
        .find(|machine| machine.machine_id == command.state_machine_id)
        .expect("checked machine binding");
    let raw_policy = spec
        .definition()
        .policies
        .iter()
        .find(|policy| policy.rule_id == command.policy_rule_id)
        .expect("checked policy binding");
    let policy = OwnerRolePolicy::check(spec, raw_policy.clone())?;
    policy.check_scope(scope, facts)?;
    let row = facts
        .find(&machine.table_id, &request.record_id)
        .ok_or(PolicyError::Denied)?;
    if !policy.allows(&context, row, Grant::Write) {
        return Err(PolicyError::Denied.into());
    }
    let invalid = |field, reason| CommandError::InvalidInput { field, reason };
    if request.idempotency_key.is_empty()
        || request.idempotency_key.len() > 96
        || !request
            .idempotency_key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(invalid(None, "invalid bounded request key"));
    }
    if request.inputs.len() > MAX_COMMAND_INPUTS {
        return Err(invalid(None, "too many inputs"));
    }
    crate::codec::bounded_encode(request)?;
    for (field, value) in &request.inputs {
        let input = command
            .inputs
            .iter()
            .find(|input| &input.field_id == field)
            .ok_or_else(|| CommandError::ImmutableField(field.clone()))?;
        let definition = spec
            .field(&machine.table_id, field)
            .expect("checked input binding");
        if !value_matches(&definition.field_type, value)
            || matches!(value,Value::Text(text) if text.len()>crate::MAX_TEXT_BYTES)
        {
            return Err(invalid(Some(field.clone()), "wrong type or text limit"));
        }
        if input.required && matches!(value, Value::Blank | Value::Null) {
            return Err(invalid(Some(field.clone()), "required input is blank"));
        }
        if input
            .constraints
            .iter()
            .any(|constraint| !constraint.accepts(value))
        {
            return Err(invalid(Some(field.clone()), "input constraint failed"));
        }
    }
    for input in &command.inputs {
        if input.required && !request.inputs.contains_key(&input.field_id) {
            return Err(invalid(
                Some(input.field_id.clone()),
                "required input is absent",
            ));
        }
    }
    #[derive(Serialize)]
    struct Intent<'a> {
        definition: &'a RawAppSpec,
        command_id: &'a CommandId,
        record_id: &'a RecordId,
        inputs: &'a BTreeMap<FieldId, Value>,
    }
    let intent_bytes = crate::codec::bounded_encode(&Intent {
        definition: spec.definition(),
        command_id: &request.command_id,
        record_id: &request.record_id,
        inputs: &request.inputs,
    })?;
    Ok(PreparedCommand {
        spec: spec.clone(),
        facts: facts.clone(),
        context,
        policy,
        command: command.clone(),
        machine: machine.clone(),
        request: request.clone(),
        intent_bytes,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionReceipt {
    command_id: CommandId,
    record_id: RecordId,
    state: String,
}
impl TransitionReceipt {
    pub fn command_id(&self) -> &CommandId {
        &self.command_id
    }
    pub fn record_id(&self) -> &RecordId {
        &self.record_id
    }
    pub fn state(&self) -> &str {
        &self.state
    }
}

#[derive(Debug)]
pub struct CheckedEventIntent {
    event_id: EventId,
    payload: BTreeMap<FieldId, Value>,
}
impl CheckedEventIntent {
    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }
    pub fn payload(&self) -> &BTreeMap<FieldId, Value> {
        &self.payload
    }
}

/// Opaque decision; an adapter must commit these writes, events, audit and receipt atomically.
/// ```compile_fail
/// use evobase_appspec::commands::CheckedTransitionBatch;
/// let _: CheckedTransitionBatch = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug)]
pub struct CheckedTransitionBatch {
    prepared: PreparedCommand,
    expected_revision: u64,
    writes: Vec<RawRecord>,
    events: Vec<CheckedEventIntent>,
    receipt: TransitionReceipt,
}
impl CheckedTransitionBatch {
    pub fn authority(&self) -> &TrustedContext {
        self.prepared.authority()
    }
    pub fn scope(&self) -> &Scope {
        self.prepared.scope()
    }
    pub fn actor(&self) -> &ActorId {
        self.prepared.actor()
    }
    pub fn binding_id(&self) -> &str {
        self.prepared.binding_id()
    }
    pub fn membership_revision(&self) -> u64 {
        self.prepared.membership_revision()
    }
    pub fn policy_revision(&self) -> u64 {
        self.prepared.policy_revision()
    }
    pub fn idempotency_key(&self) -> &str {
        self.prepared.idempotency_key()
    }
    pub fn intent_bytes(&self) -> &[u8] {
        self.prepared.intent_bytes()
    }
    pub fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    pub fn writes(&self) -> &[RawRecord] {
        &self.writes
    }
    pub fn events(&self) -> &[CheckedEventIntent] {
        &self.events
    }
    pub fn receipt(&self) -> &TransitionReceipt {
        &self.receipt
    }
}

fn check_transition(
    command: &RawCommand,
    machine: &RawStateMachine,
    row: &crate::CheckedRecord,
) -> Result<(), CommandError> {
    let state = match row.values().get(&machine.state_field) {
        Some(Value::Text(state)) => state.clone(),
        _ => String::new(),
    };
    if state != command.from_state || machine.terminal_states.contains(&state) {
        return Err(CommandError::InvalidTransition {
            from: state,
            to: command.to_state.clone(),
        });
    }
    for guard in &command.guards {
        if row.values().get(&guard.field_id) != Some(&guard.equals) {
            return Err(CommandError::GuardFailed {
                field: guard.field_id.clone(),
            });
        }
    }
    Ok(())
}

pub fn decide_command(
    prepared: PreparedCommand,
    data_revision: u64,
) -> Result<CheckedTransitionBatch, CommandError> {
    if data_revision == 0 {
        return Err(CommandError::RevisionRequired);
    }
    let row = prepared
        .facts
        .find(&prepared.machine.table_id, &prepared.request.record_id)
        .expect("prepared row");
    check_transition(&prepared.command, &prepared.machine, row)?;
    let mut proposed = row.as_raw().clone();
    proposed.values.extend(prepared.request.inputs.clone());
    proposed.values.insert(
        prepared.machine.state_field.clone(),
        Value::Text(prepared.command.to_state.clone()),
    );
    let mut final_rows = prepared.facts.to_raw();
    let target = final_rows
        .iter_mut()
        .find(|row| row.table_id == proposed.table_id && row.id == proposed.id)
        .expect("prepared row");
    *target = proposed;
    let final_facts = prepared
        .spec
        .validate_records(prepared.scope(), &final_rows)?;
    let proposed = final_facts
        .find(&prepared.machine.table_id, &prepared.request.record_id)
        .expect("validated row");
    if !prepared
        .policy
        .allows(&prepared.context, proposed, Grant::Write)
    {
        return Err(PolicyError::Denied.into());
    }
    let events = prepared
        .command
        .events
        .iter()
        .map(|event| CheckedEventIntent {
            event_id: event.event_id.clone(),
            payload: event
                .fields
                .iter()
                .map(|field| (field.clone(), proposed.values()[field].clone()))
                .collect(),
        })
        .collect();
    let writes = vec![proposed.as_raw().clone()];
    let receipt = TransitionReceipt {
        command_id: prepared.command.command_id.clone(),
        record_id: prepared.request.record_id.clone(),
        state: prepared.command.to_state.clone(),
    };
    Ok(CheckedTransitionBatch {
        prepared,
        expected_revision: data_revision,
        writes,
        events,
        receipt,
    })
}

/// Read-only action metadata from the same current policy, state and equality guards as execution.
#[allow(clippy::too_many_arguments)]
pub fn available_commands<V: SessionVerifier>(
    host: &HostAuthority<V>,
    credential: &str,
    scope: &Scope,
    channel: &RequestChannel,
    now: u64,
    spec: &CheckedAppSpec,
    facts: &CheckedRecords,
) -> Result<Vec<RawCommand>, CommandError> {
    Ok(
        available_command_records(host, credential, scope, channel, now, spec, facts)?
            .into_iter()
            .map(|(command, _)| command)
            .collect(),
    )
}

/// Eligible record IDs are revealed only through current Read and Write grants and row policies.
#[allow(clippy::too_many_arguments)]
pub fn available_command_records<V: SessionVerifier>(
    host: &HostAuthority<V>,
    credential: &str,
    scope: &Scope,
    channel: &RequestChannel,
    now: u64,
    spec: &CheckedAppSpec,
    facts: &CheckedRecords,
) -> Result<Vec<(RawCommand, Vec<RecordId>)>, CommandError> {
    let read_context = host.resolve(credential, scope, channel, now, Grant::Read)?;
    let context = host.resolve(credential, scope, channel, now, Grant::Write)?;
    if read_context != context {
        return Err(AuthorityError::Revoked.into());
    }
    if scope.app_id() != spec.app_id() || facts.scope() != scope {
        return Err(PolicyError::ScopeMismatch.into());
    }
    let mut available = Vec::new();
    for command in &spec.definition().commands {
        let machine = spec
            .definition()
            .state_machines
            .iter()
            .find(|machine| machine.machine_id == command.state_machine_id)
            .expect("checked machine");
        let policy = OwnerRolePolicy::check(
            spec,
            spec.definition()
                .policies
                .iter()
                .find(|policy| policy.rule_id == command.policy_rule_id)
                .expect("checked policy")
                .clone(),
        )?;
        policy.check_scope(scope, facts)?;
        let eligible = facts
            .records()
            .iter()
            .filter(|row| {
                policy.allows(&read_context, row, Grant::Read)
                    && policy.allows(&context, row, Grant::Write)
                    && check_transition(command, machine, row).is_ok()
            })
            .map(|row| row.id().clone())
            .collect::<Vec<_>>();
        if !eligible.is_empty() {
            available.push((command.clone(), eligible));
        }
    }
    Ok(available)
}
