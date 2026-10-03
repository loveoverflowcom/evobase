//! A deliberately small owner/role policy and SubmitOrder decision boundary.
//!
//! `SessionVerifier` is a trusted host adapter, not an implementation of authentication.
//! Every public query/command re-reads session and current membership before inspecting facts
//! or receipts. A browser must never instantiate this adapter from client supplied actor/grants.
//! Checked outputs have private fields and cannot be deserialized. Hosts must still commit the
//! expected revision, writes, audit and receipt together; this module does not implement storage.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AppId, CheckedAppSpec, CheckedRecord, CheckedRecords, FieldId, FieldType, RawRecord, RecordId,
    Scope, TableId, Value,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActorId(String);

impl ActorId {
    pub fn new(value: &str) -> Result<Self, AuthorityError> {
        bounded_id(value, "actor_").map(Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct RoleId(String);

impl RoleId {
    pub fn new(value: &str) -> Result<Self, AuthorityError> {
        bounded_id(value, "role_").map(Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RoleId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(&value).map_err(serde::de::Error::custom)
    }
}

fn bounded_id(value: &str, prefix: &str) -> Result<String, AuthorityError> {
    if value.len() > 96
        || value.len() <= prefix.len()
        || !value.starts_with(prefix)
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(AuthorityError::InvalidIdentity);
    }
    Ok(value.to_owned())
}

/// Host capabilities are independent: design/publish/manage never imply business access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grant {
    Design,
    Publish,
    Manage,
    Read,
    Write,
    Submit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AuthorityError {
    #[error("missing session")]
    MissingSession,
    #[error("unverified session")]
    Unverified,
    #[error("expired session")]
    Expired,
    #[error("revoked session or membership")]
    Revoked,
    #[error("invalid canonical identity")]
    InvalidIdentity,
    #[error("scope denied")]
    WrongScope,
    #[error("capability denied")]
    MissingGrant,
    #[error("origin denied")]
    OriginDenied,
    #[error("trusted host binding unavailable")]
    BindingUnavailable,
}

/// These are facts returned by a trusted adapter. They are never a portable AppSpec field.
#[derive(Debug, Clone)]
pub struct SessionFacts {
    pub actor: ActorId,
    pub expires_at: u64,
    pub revoked: bool,
}

#[derive(Debug, Clone)]
pub struct MembershipFacts {
    pub actor: ActorId,
    pub scope: Scope,
    pub active: bool,
    pub revision: u64,
    pub grants: BTreeSet<Grant>,
    pub roles: BTreeSet<RoleId>,
}

#[derive(Debug, Clone)]
pub struct BindingFacts {
    pub scope: Scope,
    /// Opaque registry key only. Never a client-selected database URL or credential.
    pub binding_id: String,
    pub allowed_mutation_origins: BTreeSet<String>,
}

/// Implement only in the trusted host. A fixture implementation establishes no real identity.
pub trait SessionVerifier {
    fn verify_session(&self, credential: &str) -> Result<SessionFacts, AuthorityError>;
    fn current_membership(
        &self,
        actor: &ActorId,
        scope: &Scope,
    ) -> Result<MembershipFacts, AuthorityError>;
    fn registry_binding(&self, scope: &Scope) -> Result<BindingFacts, AuthorityError>;
}

/// The trusted transport chooses the channel and supplies the actual request Origin header.
#[derive(Debug, Clone)]
pub enum RequestChannel {
    NativeBearer,
    BrowserMutation { origin: Option<String> },
}

/// Checked authority cannot cross a wire boundary as trusted input.
/// ```compile_fail
/// use evobase_appspec::policy::TrustedContext;
/// let _: TrustedContext = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug)]
pub struct TrustedContext {
    actor: ActorId,
    scope: Scope,
    roles: BTreeSet<RoleId>,
    membership_revision: u64,
    binding_id: String,
}

impl TrustedContext {
    pub fn actor(&self) -> &ActorId {
        &self.actor
    }
    pub fn scope(&self) -> &Scope {
        &self.scope
    }
    pub fn membership_revision(&self) -> u64 {
        self.membership_revision
    }
    pub fn binding_id(&self) -> &str {
        &self.binding_id
    }
}

pub struct HostAuthority<V> {
    verifier: V,
}

impl<V: SessionVerifier> HostAuthority<V> {
    pub fn new(verifier: V) -> Self {
        Self { verifier }
    }

    /// Re-resolve for each operation; a cached context is not current authorization.
    pub fn resolve(
        &self,
        credential: &str,
        scope: &Scope,
        channel: &RequestChannel,
        now: u64,
        grant: Grant,
    ) -> Result<TrustedContext, AuthorityError> {
        if credential.is_empty() {
            return Err(AuthorityError::MissingSession);
        }
        let session = self.verifier.verify_session(credential)?;
        if session.revoked {
            return Err(AuthorityError::Revoked);
        }
        if now >= session.expires_at {
            return Err(AuthorityError::Expired);
        }
        let membership = self.verifier.current_membership(&session.actor, scope)?;
        if membership.actor != session.actor || membership.scope != *scope {
            return Err(AuthorityError::WrongScope);
        }
        if !membership.active || membership.revision == 0 {
            return Err(AuthorityError::Revoked);
        }
        if !membership.grants.contains(&grant) {
            return Err(AuthorityError::MissingGrant);
        }
        let binding = self.verifier.registry_binding(scope)?;
        if binding.scope != *scope {
            return Err(AuthorityError::WrongScope);
        }
        if binding.binding_id.is_empty() || binding.binding_id.len() > 96 {
            return Err(AuthorityError::BindingUnavailable);
        }
        if let RequestChannel::BrowserMutation { origin } = channel
            && !origin
                .as_ref()
                .is_some_and(|origin| binding.allowed_mutation_origins.contains(origin))
        {
            return Err(AuthorityError::OriginDenied);
        }
        Ok(TrustedContext {
            actor: session.actor,
            scope: scope.clone(),
            roles: membership.roles,
            membership_revision: membership.revision,
            binding_id: binding.binding_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    #[error(transparent)]
    Authority(#[from] AuthorityError),
    #[error("unsupported or invalid policy definition")]
    InvalidPolicy,
    #[error("facts outside selected scope")]
    ScopeMismatch,
    #[error("row or field access denied")]
    Denied,
    #[error("invalid bounded command input")]
    InvalidInput,
    #[error("field is immutable for this command")]
    ImmutableField,
    #[error("order must be in draft state")]
    NotDraft,
    #[error("order must contain lines")]
    EmptyOrder,
    #[error("quantity must be between 1 and 10000")]
    InvalidQuantity,
    #[error("idempotency key already used for a different intent")]
    IntentConflict,
    #[error("numeric output overflow")]
    Overflow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawOwnerRolePolicy {
    pub rule_id: String,
    pub revision: u64,
    pub table_id: TableId,
    pub owner_field: FieldId,
    pub read_fields: BTreeSet<FieldId>,
    pub read_roles: BTreeSet<RoleId>,
    pub write_roles: BTreeSet<RoleId>,
    pub submit_roles: BTreeSet<RoleId>,
}

/// Only equality to the current actor OR one explicitly allowed current host role is supported.
#[derive(Debug)]
pub struct OwnerRolePolicy {
    definition: CheckedAppSpec,
    app_id: AppId,
    rule: RawOwnerRolePolicy,
}

impl OwnerRolePolicy {
    pub fn check(spec: &CheckedAppSpec, rule: RawOwnerRolePolicy) -> Result<Self, PolicyError> {
        if bounded_id(&rule.rule_id, "rule_").is_err()
            || !spec.definition().policies.contains(&rule)
            || rule.revision == 0
            || rule.read_fields.len() > 256
            || rule.read_roles.len() > 64
            || rule.write_roles.len() > 64
            || rule.submit_roles.len() > 64
            || !matches!(
                spec.field(&rule.table_id, &rule.owner_field)
                    .map(|f| &f.field_type),
                Some(FieldType::Text)
            )
            || rule.read_fields.iter().any(|field| {
                spec.field(&rule.table_id, field)
                    .is_none_or(|field| matches!(field.field_type, FieldType::Ref { .. }))
            })
        {
            return Err(PolicyError::InvalidPolicy);
        }
        Ok(Self {
            definition: spec.clone(),
            app_id: spec.app_id().clone(),
            rule,
        })
    }
    pub fn rule_id(&self) -> &str {
        &self.rule.rule_id
    }
    pub fn revision(&self) -> u64 {
        self.rule.revision
    }
    pub fn table_id(&self) -> &TableId {
        &self.rule.table_id
    }
    pub fn owner_field(&self) -> &FieldId {
        &self.rule.owner_field
    }
    pub fn sentence(&self, grant: Grant) -> String {
        let roles = self
            .roles(grant)
            .iter()
            .map(RoleId::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{}: {} equals the current verified actor or current role is one of [{}]",
            self.rule.rule_id, self.rule.owner_field, roles
        )
    }
    fn roles(&self, grant: Grant) -> &BTreeSet<RoleId> {
        match grant {
            Grant::Read => &self.rule.read_roles,
            Grant::Submit => &self.rule.submit_roles,
            _ => &self.rule.write_roles,
        }
    }
    fn allows(&self, context: &TrustedContext, row: &CheckedRecord, grant: Grant) -> bool {
        row.scope() == context.scope()
            && row.table_id() == &self.rule.table_id
            && (matches!(row.values().get(&self.rule.owner_field), Some(Value::Text(owner)) if owner == context.actor.as_str())
                || self
                    .roles(grant)
                    .iter()
                    .any(|role| context.roles.contains(role)))
    }
    fn check_scope(&self, scope: &Scope, facts: &CheckedRecords) -> Result<(), PolicyError> {
        if scope.app_id() != &self.app_id || facts.scope() != scope {
            return Err(PolicyError::ScopeMismatch);
        }
        self.definition
            .validate_records(scope, &facts.to_raw())
            .map_err(|_| PolicyError::ScopeMismatch)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionPurpose {
    Query,
    Picker,
    Lookup,
    Export,
}

#[derive(Debug)]
pub struct VisibleRecord {
    id: RecordId,
    values: BTreeMap<FieldId, Value>,
}
impl VisibleRecord {
    pub fn id(&self) -> &RecordId {
        &self.id
    }
    pub fn values(&self) -> &BTreeMap<FieldId, Value> {
        &self.values
    }
}

/// A projection cannot be forged by deserializing a response.
/// ```compile_fail
/// use evobase_appspec::policy::CheckedProjection;
/// let _: CheckedProjection = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug)]
pub struct CheckedProjection {
    rows: Vec<VisibleRecord>,
    purpose: ProjectionPurpose,
    membership_revision: u64,
    policy_revision: u64,
}

/// Current output-policy seam for relation/formula adapters. Every predicate checks the current
/// host grant again. A Ref grants no access, and complete-table scans are deliberately denied
/// because this small row-policy profile cannot attest unrestricted visibility of every row.
pub struct CurrentOutputPolicy<'a, V> {
    host: &'a HostAuthority<V>,
    credential: &'a str,
    scope: &'a Scope,
    channel: &'a RequestChannel,
    now: u64,
    facts: &'a CheckedRecords,
    policies: &'a [&'a OwnerRolePolicy],
}
impl<'a, V: SessionVerifier> CurrentOutputPolicy<'a, V> {
    pub fn new(
        host: &'a HostAuthority<V>,
        credential: &'a str,
        scope: &'a Scope,
        channel: &'a RequestChannel,
        now: u64,
        facts: &'a CheckedRecords,
        policies: &'a [&'a OwnerRolePolicy],
    ) -> Self {
        Self {
            host,
            credential,
            scope,
            channel,
            now,
            facts,
            policies,
        }
    }
    fn current(
        &self,
        scope: &Scope,
        table: &TableId,
    ) -> Option<(TrustedContext, &OwnerRolePolicy)> {
        if scope != self.scope {
            return None;
        }
        let context = self
            .host
            .resolve(self.credential, scope, self.channel, self.now, Grant::Read)
            .ok()?;
        let policy = self
            .policies
            .iter()
            .find(|policy| policy.table_id() == table)?;
        policy.check_scope(scope, self.facts).ok()?;
        Some((context, policy))
    }
}
impl<V: SessionVerifier> crate::relations::OutputPolicy for CurrentOutputPolicy<'_, V> {
    fn allow_snapshot(&self, spec: &CheckedAppSpec, facts: &CheckedRecords) -> bool {
        facts == self.facts
            && !self.policies.is_empty()
            && self
                .policies
                .iter()
                .all(|policy| policy.definition == *spec)
            && self
                .host
                .resolve(
                    self.credential,
                    self.scope,
                    self.channel,
                    self.now,
                    Grant::Read,
                )
                .is_ok()
    }
    fn allow_record(&self, scope: &Scope, table: &TableId, record: &RecordId) -> bool {
        let Some((context, policy)) = self.current(scope, table) else {
            return false;
        };
        self.facts
            .find(table, record)
            .is_some_and(|row| policy.allows(&context, row, Grant::Read))
    }
    fn allow_field(&self, scope: &Scope, table: &TableId, field: &FieldId) -> bool {
        self.current(scope, table)
            .is_some_and(|(_, policy)| policy.rule.read_fields.contains(field))
    }
    // allow_complete_scan defaults to false, independent of whether the table is empty.
}
impl CheckedProjection {
    pub fn rows(&self) -> &[VisibleRecord] {
        &self.rows
    }
    pub fn purpose(&self) -> ProjectionPurpose {
        self.purpose
    }
    pub fn membership_revision(&self) -> u64 {
        self.membership_revision
    }
    pub fn policy_revision(&self) -> u64 {
        self.policy_revision
    }
}

/// Filtering is performed before projecting fields, counting, lookup or aggregation.
// Explicit host/session/scope/clock and checked facts remain separate trust-boundary inputs.
#[allow(clippy::too_many_arguments)]
pub fn project<V: SessionVerifier>(
    host: &HostAuthority<V>,
    credential: &str,
    scope: &Scope,
    channel: &RequestChannel,
    now: u64,
    facts: &CheckedRecords,
    policy: &OwnerRolePolicy,
    purpose: ProjectionPurpose,
) -> Result<CheckedProjection, PolicyError> {
    let context = host.resolve(credential, scope, channel, now, Grant::Read)?;
    policy.check_scope(scope, facts)?;
    let rows = facts
        .records()
        .iter()
        .filter(|row| policy.allows(&context, row, Grant::Read))
        .map(|row| VisibleRecord {
            id: row.id().clone(),
            values: row
                .values()
                .iter()
                .filter(|(field, _)| policy.rule.read_fields.contains(*field))
                .map(|(field, value)| (field.clone(), value.clone()))
                .collect(),
        })
        .collect();
    Ok(CheckedProjection {
        rows,
        purpose,
        membership_revision: context.membership_revision,
        policy_revision: policy.revision(),
    })
}

/// A hidden record and an absent record both return None; callers must not add raw existence hints.
#[allow(clippy::too_many_arguments)] // Same explicit authorization inputs as project.
pub fn lookup<V: SessionVerifier>(
    host: &HostAuthority<V>,
    credential: &str,
    scope: &Scope,
    channel: &RequestChannel,
    now: u64,
    facts: &CheckedRecords,
    policy: &OwnerRolePolicy,
    id: &RecordId,
) -> Result<Option<VisibleRecord>, PolicyError> {
    Ok(project(
        host,
        credential,
        scope,
        channel,
        now,
        facts,
        policy,
        ProjectionPurpose::Lookup,
    )?
    .rows
    .into_iter()
    .find(|row| row.id() == id))
}

pub fn visible_count<V: SessionVerifier>(
    host: &HostAuthority<V>,
    credential: &str,
    scope: &Scope,
    channel: &RequestChannel,
    now: u64,
    facts: &CheckedRecords,
    policy: &OwnerRolePolicy,
) -> Result<usize, PolicyError> {
    Ok(project(
        host,
        credential,
        scope,
        channel,
        now,
        facts,
        policy,
        ProjectionPurpose::Query,
    )?
    .rows
    .len())
}

/// The numeric field must itself be visible. Unsupported values reject rather than drop policy.
#[allow(clippy::too_many_arguments)] // Same explicit authorization inputs as project.
pub fn visible_sum<V: SessionVerifier>(
    host: &HostAuthority<V>,
    credential: &str,
    scope: &Scope,
    channel: &RequestChannel,
    now: u64,
    facts: &CheckedRecords,
    policy: &OwnerRolePolicy,
    field: &FieldId,
) -> Result<i64, PolicyError> {
    let projection = project(
        host,
        credential,
        scope,
        channel,
        now,
        facts,
        policy,
        ProjectionPurpose::Query,
    )?;
    if !policy.rule.read_fields.contains(field) {
        return Err(PolicyError::Denied);
    }
    projection
        .rows
        .iter()
        .try_fold(0i64, |sum, row| match row.values.get(field) {
            Some(Value::Integer(value) | Value::Money(value)) => {
                sum.checked_add(*value).ok_or(PolicyError::Overflow)
            }
            Some(Value::Blank | Value::Null) | None => Ok(sum),
            _ => Err(PolicyError::InvalidInput),
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSubmitOrderRule {
    pub rule_id: String,
    pub revision: u64,
    pub policy_rule_id: String,
    pub order_table: TableId,
    pub state_field: FieldId,
    pub notes_field: FieldId,
    pub line_table: TableId,
    pub line_order_field: FieldId,
    pub quantity_field: FieldId,
    pub captured_price_field: FieldId,
}

#[derive(Debug)]
pub struct SubmitOrderRule {
    definition: CheckedAppSpec,
    app_id: AppId,
    rule: RawSubmitOrderRule,
}
impl SubmitOrderRule {
    pub fn check(
        spec: &CheckedAppSpec,
        policy: &OwnerRolePolicy,
        rule: RawSubmitOrderRule,
    ) -> Result<Self, PolicyError> {
        if bounded_id(&rule.rule_id, "rule_").is_err()
            || !spec.definition().submit_rules.contains(&rule)
            || rule.revision == 0
            || rule.policy_rule_id != policy.rule_id()
            || rule.order_table != *policy.table_id()
            || rule.line_table == rule.order_table
            || policy.app_id != *spec.app_id()
            || policy.definition != *spec
            || !matches!(
                spec.field(&rule.order_table, &rule.state_field)
                    .map(|f| &f.field_type),
                Some(FieldType::Text)
            )
            || !matches!(
                spec.field(&rule.order_table, &rule.notes_field)
                    .map(|f| &f.field_type),
                Some(FieldType::Text)
            )
            || !matches!(
                spec.field(&rule.line_table, &rule.quantity_field)
                    .map(|f| &f.field_type),
                Some(FieldType::Integer)
            )
            || !matches!(
                spec.field(&rule.line_table, &rule.captured_price_field)
                    .map(|f| &f.field_type),
                Some(FieldType::Money)
            )
            || !matches!(spec.field(&rule.line_table, &rule.line_order_field).map(|f| &f.field_type), Some(FieldType::Ref { target_table }) if target_table == &rule.order_table)
            || !spec.definition().capture_rules.iter().any(|capture| {
                capture.line_table == rule.line_table
                    && capture.captured_price_field == rule.captured_price_field
            })
            || rule.state_field == rule.notes_field
            || rule.notes_field == *policy.owner_field()
            || rule.state_field == *policy.owner_field()
            || rule.line_order_field == rule.quantity_field
            || rule.captured_price_field == rule.quantity_field
        {
            return Err(PolicyError::InvalidPolicy);
        }
        Ok(Self {
            definition: spec.clone(),
            app_id: spec.app_id().clone(),
            rule,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinePatch {
    pub line_id: RecordId,
    pub changes: BTreeMap<FieldId, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSubmitOrder {
    pub order_id: RecordId,
    pub idempotency_key: String,
    pub changes: BTreeMap<FieldId, Value>,
    pub line_patches: Vec<LinePatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedIntent {
    command: RawSubmitOrderRule,
    order_id: RecordId,
    changes: BTreeMap<FieldId, Value>,
    line_patches: Vec<LinePatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionReceipt {
    order_id: RecordId,
    state: String,
}
impl SubmissionReceipt {
    pub fn order_id(&self) -> &RecordId {
        &self.order_id
    }
    pub fn state(&self) -> &str {
        &self.state
    }
}

/// An example host receipt adapter. No persistence, transaction or delivery guarantee is implied.
#[derive(Debug, Default)]
pub struct ReceiptBook {
    entries: BTreeMap<(Scope, ActorId, String), (NormalizedIntent, SubmissionReceipt)>,
}
impl ReceiptBook {
    /// Call only after a host atomically commits the checked writes. This in-memory helper cannot do that.
    pub fn record_applied(&mut self, batch: &CheckedCommandBatch) -> Result<(), PolicyError> {
        let key = (
            batch.scope.clone(),
            batch.actor.clone(),
            batch.idempotency_key.clone(),
        );
        if let Some((intent, _)) = self.entries.get(&key) {
            if intent != &batch.intent {
                return Err(PolicyError::IntentConflict);
            }
            return Ok(());
        }
        self.entries
            .insert(key, (batch.intent.clone(), batch.receipt.clone()));
        Ok(())
    }
}

/// Checked writes are constructed only by the deterministic command decision.
/// ```compile_fail
/// use evobase_appspec::policy::CheckedCommandBatch;
/// let _: CheckedCommandBatch = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug)]
pub struct CheckedCommandBatch {
    scope: Scope,
    actor: ActorId,
    binding_id: String,
    expected_revision: u64,
    membership_revision: u64,
    policy_revision: u64,
    idempotency_key: String,
    intent: NormalizedIntent,
    writes: Vec<RawRecord>,
    receipt: SubmissionReceipt,
}
impl CheckedCommandBatch {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }
    pub fn actor(&self) -> &ActorId {
        &self.actor
    }
    pub fn binding_id(&self) -> &str {
        &self.binding_id
    }
    pub fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    pub fn membership_revision(&self) -> u64 {
        self.membership_revision
    }
    pub fn policy_revision(&self) -> u64 {
        self.policy_revision
    }
    pub fn writes(&self) -> &[RawRecord] {
        &self.writes
    }
    pub fn receipt(&self) -> &SubmissionReceipt {
        &self.receipt
    }
}

#[derive(Debug)]
pub enum CommandDecision {
    Apply(Box<CheckedCommandBatch>),
    Replay(SubmissionReceipt),
}

fn normalized_intent(
    request: &RawSubmitOrder,
    rule: &RawSubmitOrderRule,
) -> Result<NormalizedIntent, PolicyError> {
    if request.idempotency_key.is_empty()
        || request.idempotency_key.len() > 96
        || !request
            .idempotency_key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        || request.changes.len() > 1
        || request.line_patches.len() > 256
    {
        return Err(PolicyError::InvalidInput);
    }
    for (field, value) in &request.changes {
        if field != &rule.notes_field {
            return Err(PolicyError::ImmutableField);
        }
        match value {
            Value::Text(text) if text.len() <= 1000 => {}
            _ => return Err(PolicyError::InvalidInput),
        }
    }
    for patch in &request.line_patches {
        if patch.changes.len() > 1 {
            return Err(PolicyError::InvalidInput);
        }
        for (field, value) in &patch.changes {
            if field != &rule.quantity_field {
                return Err(PolicyError::ImmutableField);
            }
            if !matches!(value, Value::Integer(1..=10_000)) {
                return Err(PolicyError::InvalidQuantity);
            }
        }
    }
    let changes = request
        .changes
        .iter()
        .map(|(field, value)| {
            let Value::Text(text) = value else {
                unreachable!("validated bounded note")
            };
            (field.clone(), Value::Text(text.trim().to_owned()))
        })
        .collect();
    let mut line_patches = request.line_patches.clone();
    line_patches.sort_by(|a, b| a.line_id.cmp(&b.line_id));
    if line_patches
        .windows(2)
        .any(|pair| pair[0].line_id == pair[1].line_id)
    {
        return Err(PolicyError::InvalidInput);
    }
    Ok(NormalizedIntent {
        command: rule.clone(),
        order_id: request.order_id.clone(),
        changes,
        line_patches,
    })
}

fn raw_record(row: &CheckedRecord) -> RawRecord {
    RawRecord {
        scope: row.scope().clone(),
        id: row.id().clone(),
        table_id: row.table_id().clone(),
        values: row.values().clone(),
    }
}

/// Decision only. The host must load current facts, close TOCTOU with the expected revisions and
/// atomically commit rows/audit/receipt. Old captures and selectors are copied from checked facts.
#[allow(clippy::too_many_arguments)] // Keep request, trusted host facts, snapshot and revisions explicit.
pub fn decide_submit_order<V: SessionVerifier>(
    host: &HostAuthority<V>,
    credential: &str,
    scope: &Scope,
    channel: &RequestChannel,
    now: u64,
    spec: &CheckedAppSpec,
    facts: &CheckedRecords,
    data_revision: u64,
    policy: &OwnerRolePolicy,
    command: &SubmitOrderRule,
    receipts: &ReceiptBook,
    request: &RawSubmitOrder,
) -> Result<CommandDecision, PolicyError> {
    // Current grants are deliberately checked before policy, facts, input details and receipt lookup.
    let context = host.resolve(credential, scope, channel, now, Grant::Submit)?;
    policy.check_scope(scope, facts)?;
    if command.app_id != *scope.app_id()
        || spec.app_id() != scope.app_id()
        || command.definition != *spec
        || policy.definition != *spec
        || command.rule.policy_rule_id != policy.rule_id()
        || data_revision == 0
    {
        return Err(PolicyError::ScopeMismatch);
    }
    let order = facts
        .find(&command.rule.order_table, &request.order_id)
        .ok_or(PolicyError::Denied)?;
    if !policy.allows(&context, order, Grant::Submit) {
        return Err(PolicyError::Denied);
    }
    let intent = normalized_intent(request, &command.rule)?;
    let receipt_key = (
        scope.clone(),
        context.actor.clone(),
        request.idempotency_key.clone(),
    );
    if let Some((previous, receipt)) = receipts.entries.get(&receipt_key) {
        return if previous == &intent {
            Ok(CommandDecision::Replay(receipt.clone()))
        } else {
            Err(PolicyError::IntentConflict)
        };
    }
    if !matches!(order.values().get(&command.rule.state_field), Some(Value::Text(state)) if state == "draft")
    {
        return Err(PolicyError::NotDraft);
    }
    let mut updated_order = raw_record(order);
    updated_order.values.extend(intent.changes.clone());
    updated_order.values.insert(
        command.rule.state_field.clone(),
        Value::Text("submitted".to_owned()),
    );
    let lines = facts.records().iter().filter(|row| row.table_id() == &command.rule.line_table
        && matches!(row.values().get(&command.rule.line_order_field), Some(Value::Ref(reference))
            if reference.scope == *scope && reference.table_id == command.rule.order_table && reference.record_id == request.order_id)).collect::<Vec<_>>();
    if lines.is_empty() {
        return Err(PolicyError::EmptyOrder);
    }
    if intent
        .line_patches
        .iter()
        .any(|patch| !lines.iter().any(|row| row.id() == &patch.line_id))
    {
        return Err(PolicyError::InvalidInput);
    }
    let mut writes = vec![updated_order];
    for line in lines {
        let mut updated_line = raw_record(line);
        if let Some(patch) = intent
            .line_patches
            .iter()
            .find(|patch| &patch.line_id == line.id())
        {
            updated_line.values.extend(patch.changes.clone());
        }
        if !matches!(
            updated_line.values.get(&command.rule.quantity_field),
            Some(Value::Integer(1..=10_000))
        ) {
            return Err(PolicyError::InvalidQuantity);
        }
        if !matches!(updated_line.values.get(&command.rule.captured_price_field), Some(Value::Money(price)) if *price >= 0)
        {
            return Err(PolicyError::InvalidInput);
        }
        writes.push(updated_line);
    }
    // Validate final facts independently, then check proposed owner policy again.
    let mut final_rows = facts.records().iter().map(raw_record).collect::<Vec<_>>();
    for write in &writes {
        let target = final_rows
            .iter_mut()
            .find(|row| row.table_id == write.table_id && row.id == write.id)
            .ok_or(PolicyError::InvalidInput)?;
        *target = write.clone();
    }
    let checked_final = spec
        .validate_records(scope, &final_rows)
        .map_err(|_| PolicyError::InvalidInput)?;
    let proposed_order = checked_final
        .find(&command.rule.order_table, &request.order_id)
        .ok_or(PolicyError::InvalidInput)?;
    if !policy.allows(&context, proposed_order, Grant::Submit) {
        return Err(PolicyError::Denied);
    }
    Ok(CommandDecision::Apply(Box::new(CheckedCommandBatch {
        scope: scope.clone(),
        actor: context.actor,
        binding_id: context.binding_id,
        expected_revision: data_revision,
        membership_revision: context.membership_revision,
        policy_revision: policy.revision(),
        idempotency_key: request.idempotency_key.clone(),
        intent,
        writes,
        receipt: SubmissionReceipt {
            order_id: request.order_id.clone(),
            state: "submitted".to_owned(),
        },
    })))
}
