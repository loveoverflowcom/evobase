//! Canonical N:1 facts, insertion-time captures, and bounded pure numeric projections.
//!
//! This in-memory preview boundary has no persistence or host authorization. Every output
//! projection requires an explicit policy; a reference never supplies a grant.
use crate::{
    AppId, CaptureRule, CheckedAppSpec, CheckedRecord, CheckedRecords, Error, FieldId, FieldType,
    RawRecord, RecordId, Scope, TableId, Value,
};
use std::collections::{BTreeMap, BTreeSet};

const MAX_BATCH: usize = 1_024;
const MAX_FORMULAS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationError {
    Kernel(Error),
    BatchLimit,
    DuplicateMutation {
        table: TableId,
        record: RecordId,
    },
    AlreadyExists {
        table: TableId,
        record: RecordId,
    },
    MissingRecord {
        table: TableId,
        record: RecordId,
    },
    RestrictDelete {
        table: TableId,
        record: RecordId,
        source_table: TableId,
        source_record: RecordId,
        field: FieldId,
    },
    CaptureInputForbidden {
        field: FieldId,
    },
    CapturedValueImmutable {
        field: FieldId,
    },
    CapturedReferenceImmutable {
        field: FieldId,
    },
    CaptureSourceInvalid {
        field: FieldId,
    },
    OutputDenied,
    InvalidLimits,
    FormulaLimit,
    InvalidFormulaName,
    DuplicateFormula {
        table: TableId,
        name: String,
    },
    MissingFormula {
        table: TableId,
        name: String,
    },
    FormulaCycle {
        table: TableId,
        name: String,
    },
    UnknownTable {
        table: TableId,
    },
    UnknownField {
        table: TableId,
        field: FieldId,
    },
    NumericFieldRequired {
        table: TableId,
        field: FieldId,
    },
    ReferenceFieldRequired {
        table: TableId,
        field: FieldId,
    },
    ReferenceTargetMismatch {
        table: TableId,
        field: FieldId,
    },
    NumericTypeMismatch,
    MoneyProductUnsupported,
    EmptyTotal,
    DepthExceeded,
    NodeBudgetExceeded,
    RecordBudgetExceeded,
    DefinitionMismatch,
    Overflow,
}

impl std::fmt::Display for RelationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RelationError {}
impl From<Error> for RelationError {
    fn from(value: Error) -> Self {
        Self::Kernel(value)
    }
}

/// Explicit output authorization adapter. Missing authority must deny. A hosted adapter
/// must check current tenant, host grant, app policy and field restrictions here.
pub trait OutputPolicy {
    /// Attest that authorization facts, definition and revisions match this exact
    /// output snapshot. ID-only decisions from a different snapshot are insufficient.
    fn allow_snapshot(&self, _: &CheckedAppSpec, _: &CheckedRecords) -> bool {
        false
    }
    fn allow_record(&self, scope: &Scope, table: &TableId, record: &RecordId) -> bool;
    fn allow_field(&self, scope: &Scope, table: &TableId, field: &FieldId) -> bool;
    /// Authorize the complete table projection independently of data existence. Returning
    /// true promises that every row and each requested field can be read. Hosts with row
    /// restrictions must deny this initial profile; a partial scan is unsupported.
    fn allow_complete_scan(&self, _: &Scope, _: &TableId, _: &[FieldId]) -> bool {
        false
    }
}

/// Synthetic local Builder preview only. Never use this as hosted authority.
pub struct LocalPreviewPolicy;
impl OutputPolicy for LocalPreviewPolicy {
    fn allow_snapshot(&self, _: &CheckedAppSpec, _: &CheckedRecords) -> bool {
        true
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

pub struct DenyOutput;
impl OutputPolicy for DenyOutput {
    fn allow_record(&self, _: &Scope, _: &TableId, _: &RecordId) -> bool {
        false
    }
    fn allow_field(&self, _: &Scope, _: &TableId, _: &FieldId) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub enum BatchChange {
    Insert(RawRecord),
    Replace(RawRecord),
    Delete {
        table_id: TableId,
        record_id: RecordId,
    },
}

/// Only successfully checked final-state batches can alter these canonical facts.
pub struct RelationStore {
    spec: CheckedAppSpec,
    records: CheckedRecords,
}

impl RelationStore {
    pub fn new(spec: CheckedAppSpec, scope: Scope) -> Result<Self, RelationError> {
        let records = spec.validate_records(&scope, &[])?;
        Ok(Self { spec, records })
    }
    /// Restore already persisted historical facts supplied by a trusted synthetic local
    /// adapter, with an explicit local-preview marker. This checks the schema and final graph,
    /// but cannot establish the provenance of a historical price. Never treat an untrusted request body as history;
    /// new authoring inputs must pass Insert and derive their captures instead.
    pub fn from_checked_snapshot(
        spec: CheckedAppSpec,
        records: CheckedRecords,
        _preview: &LocalPreviewPolicy,
    ) -> Result<Self, RelationError> {
        let records = spec.validate_records(records.scope(), &records.to_raw())?;
        Ok(Self { spec, records })
    }
    pub fn spec(&self) -> &CheckedAppSpec {
        &self.spec
    }
    pub fn records(&self) -> &CheckedRecords {
        &self.records
    }

    pub fn apply_batch(&mut self, changes: Vec<BatchChange>) -> Result<(), RelationError> {
        if changes.len() > MAX_BATCH {
            return Err(RelationError::BatchLimit);
        }
        // Bound borrowed input before copying either it or the current snapshot. The
        // reference list is bounded by MAX_BATCH; the encoder never exceeds MAX_BYTES.
        let incoming: Vec<&RawRecord> = changes
            .iter()
            .filter_map(|change| match change {
                BatchChange::Insert(row) | BatchChange::Replace(row) => Some(row),
                BatchChange::Delete { .. } => None,
            })
            .collect();
        crate::codec::bounded_encode(&incoming)?;
        for row in incoming {
            for value in row.values.values() {
                if let Value::Text(text) = value
                    && text.len() > crate::MAX_TEXT_BYTES
                {
                    return Err(Error::LimitExceeded {
                        resource: "text bytes",
                        limit: crate::MAX_TEXT_BYTES,
                    }
                    .into());
                }
            }
        }
        let mut candidate: BTreeMap<(TableId, RecordId), RawRecord> = self
            .records
            .records()
            .iter()
            .map(|r| ((r.table_id().clone(), r.id().clone()), raw_record(r)))
            .collect();
        let mut touched = BTreeSet::new();
        let mut inserted = Vec::new();
        let mut deleted = Vec::new();
        for change in changes {
            let key = match &change {
                BatchChange::Insert(r) | BatchChange::Replace(r) => {
                    (r.table_id.clone(), r.id.clone())
                }
                BatchChange::Delete {
                    table_id,
                    record_id,
                } => (table_id.clone(), record_id.clone()),
            };
            if !touched.insert(key.clone()) {
                return Err(RelationError::DuplicateMutation {
                    table: key.0,
                    record: key.1,
                });
            }
            match change {
                BatchChange::Insert(r) => {
                    if candidate.contains_key(&key) {
                        return Err(RelationError::AlreadyExists {
                            table: key.0,
                            record: key.1,
                        });
                    }
                    for capture in &self.spec.definition().capture_rules {
                        if capture.line_table == r.table_id
                            && r.values
                                .get(&capture.captured_price_field)
                                .is_some_and(|v| !matches!(v, Value::Blank | Value::Null))
                        {
                            return Err(RelationError::CaptureInputForbidden {
                                field: capture.captured_price_field.clone(),
                            });
                        }
                    }
                    inserted.push(key.clone());
                    candidate.insert(key, r);
                }
                BatchChange::Replace(r) => {
                    let old = candidate
                        .get(&key)
                        .ok_or_else(|| RelationError::MissingRecord {
                            table: key.0.clone(),
                            record: key.1.clone(),
                        })?;
                    for capture in &self.spec.definition().capture_rules {
                        if capture.line_table == r.table_id {
                            if old.values.get(&capture.captured_price_field)
                                != r.values.get(&capture.captured_price_field)
                            {
                                return Err(RelationError::CapturedValueImmutable {
                                    field: capture.captured_price_field.clone(),
                                });
                            }
                            if old.values.get(&capture.product_ref_field)
                                != r.values.get(&capture.product_ref_field)
                            {
                                return Err(RelationError::CapturedReferenceImmutable {
                                    field: capture.product_ref_field.clone(),
                                });
                            }
                        }
                    }
                    candidate.insert(key, r);
                }
                BatchChange::Delete { .. } => {
                    if candidate.remove(&key).is_none() {
                        return Err(RelationError::MissingRecord {
                            table: key.0,
                            record: key.1,
                        });
                    }
                    deleted.push(key);
                }
            }
        }
        // Validate scope/type/the whole final graph before reading any capture source. A
        // temporary value fills only insertion-derived destinations; it is never published.
        let mut graph_rows = candidate.clone();
        for key in &inserted {
            let row = graph_rows.get_mut(key).expect("inserted candidate");
            for capture in &self.spec.definition().capture_rules {
                if capture.line_table == row.table_id {
                    row.values
                        .insert(capture.captured_price_field.clone(), Value::Money(0));
                }
            }
        }
        // Restrict checks use only surviving references; deleting both ends in one batch is legal.
        for (table, record) in &deleted {
            for row in candidate.values() {
                for (field, value) in &row.values {
                    if matches!(value, Value::Ref(r) if &r.table_id == table && &r.record_id == record && &r.scope == self.records.scope())
                    {
                        return Err(RelationError::RestrictDelete {
                            table: table.clone(),
                            record: record.clone(),
                            source_table: row.table_id.clone(),
                            source_record: row.id.clone(),
                            field: field.clone(),
                        });
                    }
                }
            }
        }
        let graph_rows: Vec<_> = graph_rows.into_values().collect();
        self.spec
            .validate_records(self.records.scope(), &graph_rows)?;
        // Capture from the completed batch, including Product rows inserted in the same batch.
        for key in inserted {
            let mut row = candidate.get(&key).expect("inserted candidate").clone();
            for capture in &self.spec.definition().capture_rules {
                if capture.line_table != row.table_id {
                    continue;
                }
                let source = match row.values.get(&capture.product_ref_field) {
                    Some(Value::Ref(reference)) if &reference.scope == self.records.scope() => {
                        let target = match &self
                            .spec
                            .field(&row.table_id, &capture.product_ref_field)
                            .expect("checked capture field")
                            .field_type
                        {
                            FieldType::Ref { target_table } => target_table,
                            _ => unreachable!("checked capture reference"),
                        };
                        if &reference.table_id != target {
                            return Err(RelationError::CaptureSourceInvalid {
                                field: capture.product_ref_field.clone(),
                            });
                        }
                        candidate.get(&(reference.table_id.clone(), reference.record_id.clone()))
                    }
                    _ => None,
                };
                let price = match source.and_then(|r| r.values.get(&capture.product_price_field)) {
                    Some(Value::Money(price)) => *price,
                    _ => {
                        return Err(RelationError::CaptureSourceInvalid {
                            field: capture.product_price_field.clone(),
                        });
                    }
                };
                row.values
                    .insert(capture.captured_price_field.clone(), Value::Money(price));
            }
            candidate.insert(key, row);
        }
        let rows: Vec<_> = candidate.into_values().collect();
        let checked = self.spec.validate_records(self.records.scope(), &rows)?;
        self.records = checked;
        Ok(())
    }

    /// Reverse edges are computed, never editable. Denied rows/fields reject the whole
    /// projection rather than returning a filtered count that could leak existence.
    pub fn reverse_refs(
        &self,
        table: &TableId,
        record: &RecordId,
        child_table: &TableId,
        ref_field: &FieldId,
        policy: &dyn OutputPolicy,
    ) -> Result<Vec<RecordId>, RelationError> {
        self.authorize_snapshot(policy)?;
        self.authorized_record(table, record, policy)?;
        self.reference_target(child_table, ref_field, table)?;
        self.authorize_scan(child_table, std::slice::from_ref(ref_field), policy)?;
        let mut result = Vec::new();
        for child in self
            .records
            .records()
            .iter()
            .filter(|r| r.table_id() == child_table)
        {
            self.authorize_field(child, ref_field, policy)?;
            if matches!(child.values().get(ref_field), Some(Value::Ref(r)) if &r.table_id == table && &r.record_id == record)
            {
                result.push(child.id().clone());
            }
        }
        Ok(result)
    }

    pub fn lookup(
        &self,
        table: &TableId,
        record: &RecordId,
        ref_field: &FieldId,
        target_field: &FieldId,
        policy: &dyn OutputPolicy,
    ) -> Result<Value, RelationError> {
        self.authorize_snapshot(policy)?;
        let row = self.authorized_record(table, record, policy)?;
        self.authorize_field(row, ref_field, policy)?;
        let target = self.reference_target_any(table, ref_field)?;
        if self.spec.field(target, target_field).is_none() {
            return Err(RelationError::UnknownField {
                table: target.clone(),
                field: target_field.clone(),
            });
        }
        if !policy.allow_field(self.records.scope(), target, target_field) {
            return Err(RelationError::OutputDenied);
        }
        match row.values().get(ref_field) {
            Some(Value::Ref(reference)) => {
                let target_row =
                    self.authorized_record(&reference.table_id, &reference.record_id, policy)?;
                self.authorize_field(target_row, target_field, policy)?;
                Ok(target_row
                    .values()
                    .get(target_field)
                    .cloned()
                    .unwrap_or(Value::Null))
            }
            _ => Ok(Value::Null),
        }
    }

    fn authorized_record(
        &self,
        table: &TableId,
        record: &RecordId,
        policy: &dyn OutputPolicy,
    ) -> Result<&CheckedRecord, RelationError> {
        if !policy.allow_record(self.records.scope(), table, record) {
            return Err(RelationError::OutputDenied);
        }
        self.records
            .find(table, record)
            .ok_or_else(|| RelationError::MissingRecord {
                table: table.clone(),
                record: record.clone(),
            })
    }
    fn authorize_snapshot(&self, policy: &dyn OutputPolicy) -> Result<(), RelationError> {
        if !policy.allow_snapshot(&self.spec, &self.records) {
            return Err(RelationError::OutputDenied);
        }
        Ok(())
    }
    fn authorize_field(
        &self,
        row: &CheckedRecord,
        field: &FieldId,
        policy: &dyn OutputPolicy,
    ) -> Result<(), RelationError> {
        if !policy.allow_record(row.scope(), row.table_id(), row.id())
            || !policy.allow_field(row.scope(), row.table_id(), field)
        {
            return Err(RelationError::OutputDenied);
        }
        Ok(())
    }
    fn authorize_scan(
        &self,
        table: &TableId,
        fields: &[FieldId],
        policy: &dyn OutputPolicy,
    ) -> Result<(), RelationError> {
        if !policy.allow_complete_scan(self.records.scope(), table, fields)
            || fields
                .iter()
                .any(|field| !policy.allow_field(self.records.scope(), table, field))
        {
            return Err(RelationError::OutputDenied);
        }
        Ok(())
    }
    fn reference_target_any(
        &self,
        table: &TableId,
        field: &FieldId,
    ) -> Result<&TableId, RelationError> {
        reference_target(&self.spec, table, field)
    }
    fn reference_target(
        &self,
        table: &TableId,
        field: &FieldId,
        target: &TableId,
    ) -> Result<(), RelationError> {
        if self.reference_target_any(table, field)? != target {
            return Err(RelationError::ReferenceTargetMismatch {
                table: table.clone(),
                field: field.clone(),
            });
        }
        Ok(())
    }
}

fn raw_record(row: &CheckedRecord) -> RawRecord {
    RawRecord {
        scope: row.scope().clone(),
        table_id: row.table_id().clone(),
        id: row.id().clone(),
        values: row.values().clone(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericKind {
    Integer,
    Money,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Numeric {
    Integer(i64),
    Money(i64),
    Null(NumericKind),
}
impl Numeric {
    fn kind(self) -> NumericKind {
        match self {
            Self::Integer(_) => NumericKind::Integer,
            Self::Money(_) => NumericKind::Money,
            Self::Null(k) => k,
        }
    }
    fn value(self) -> Option<i64> {
        match self {
            Self::Integer(v) | Self::Money(v) => Some(v),
            Self::Null(_) => None,
        }
    }
    fn from(kind: NumericKind, value: i64) -> Self {
        match kind {
            NumericKind::Integer => Self::Integer(value),
            NumericKind::Money => Self::Money(value),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expression {
    Field(FieldId),
    Lookup {
        ref_field: FieldId,
        target_field: FieldId,
    },
    Sum {
        child_table: TableId,
        ref_field: FieldId,
        value: Box<Expression>,
    },
    Product(Box<Expression>, Box<Expression>),
    Total(Vec<Expression>),
    Named(String),
}
#[derive(Clone, Debug)]
pub struct FormulaDefinition {
    pub table: TableId,
    pub name: String,
    pub expression: Expression,
}

#[derive(Clone, Copy, Debug)]
pub struct FormulaLimits {
    pub max_nodes: usize,
    pub max_depth: usize,
    /// Count the root row, every scanned child candidate and every dereferenced lookup.
    pub max_records: usize,
}
impl Default for FormulaLimits {
    fn default() -> Self {
        Self {
            max_nodes: 256,
            max_depth: 24,
            max_records: 1_024,
        }
    }
}
impl FormulaLimits {
    fn validate(self) -> Result<(), RelationError> {
        if self.max_nodes == 0
            || self.max_nodes > 16_384
            || self.max_depth == 0
            || self.max_depth > 64
            || self.max_records == 0
            || self.max_records > 10_000
        {
            return Err(RelationError::InvalidLimits);
        }
        Ok(())
    }
}

/// No Deserialize or public constructor bypass: raw expressions must compile against the
/// canonical checked definition. Named expressions permit acyclic reuse, never recursion.
pub struct CheckedFormulas {
    definitions: BTreeMap<(TableId, String), Expression>,
    kinds: BTreeMap<(TableId, String), NumericKind>,
    limits: FormulaLimits,
    shape: SemanticShape,
}
#[derive(PartialEq, Eq)]
struct SemanticShape {
    app: AppId,
    tables: BTreeSet<TableId>,
    fields: BTreeMap<(TableId, FieldId), (FieldType, bool)>,
    captures: Vec<CaptureRule>,
}
impl SemanticShape {
    fn from(spec: &CheckedAppSpec) -> Self {
        Self {
            app: spec.app_id().clone(),
            tables: spec.tables().iter().map(|t| t.id.clone()).collect(),
            fields: spec
                .tables()
                .iter()
                .flat_map(|t| {
                    t.fields.iter().map(|f| {
                        (
                            (t.id.clone(), f.id.clone()),
                            (f.field_type.clone(), f.required),
                        )
                    })
                })
                .collect(),
            captures: spec.definition().capture_rules.clone(),
        }
    }
}
impl CheckedFormulas {
    pub fn compile(
        spec: &CheckedAppSpec,
        definitions: Vec<FormulaDefinition>,
        limits: FormulaLimits,
    ) -> Result<Self, RelationError> {
        limits.validate()?;
        if definitions.len() > MAX_FORMULAS {
            return Err(RelationError::FormulaLimit);
        }
        let mut formulas = Self {
            definitions: BTreeMap::new(),
            kinds: BTreeMap::new(),
            limits,
            shape: SemanticShape::from(spec),
        };
        for definition in definitions {
            if definition.name.is_empty()
                || definition.name.len() > 64
                || !definition
                    .name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            {
                return Err(RelationError::InvalidFormulaName);
            }
            if spec.table(&definition.table).is_none() {
                return Err(RelationError::UnknownTable {
                    table: definition.table,
                });
            }
            let key = (definition.table, definition.name);
            if formulas
                .definitions
                .insert(key.clone(), definition.expression)
                .is_some()
            {
                return Err(RelationError::DuplicateFormula {
                    table: key.0,
                    name: key.1,
                });
            }
        }
        for (key, expression) in &formulas.definitions {
            let mut budget = Budget::new(limits);
            let mut visiting = BTreeSet::from([key.clone()]);
            let kind = formulas.check_expression(
                spec,
                &key.0,
                expression,
                1,
                &mut budget,
                &mut visiting,
            )?;
            formulas.kinds.insert(key.clone(), kind);
        }
        Ok(formulas)
    }

    fn check_expression(
        &self,
        spec: &CheckedAppSpec,
        table: &TableId,
        expression: &Expression,
        depth: usize,
        budget: &mut Budget,
        visiting: &mut BTreeSet<(TableId, String)>,
    ) -> Result<NumericKind, RelationError> {
        budget.node(depth)?;
        match expression {
            Expression::Field(field) => numeric_kind(spec, table, field),
            Expression::Lookup {
                ref_field,
                target_field,
            } => numeric_kind(
                spec,
                reference_target(spec, table, ref_field)?,
                target_field,
            ),
            Expression::Sum {
                child_table,
                ref_field,
                value,
            } => {
                if reference_target(spec, child_table, ref_field)? != table {
                    return Err(RelationError::ReferenceTargetMismatch {
                        table: child_table.clone(),
                        field: ref_field.clone(),
                    });
                }
                self.check_expression(spec, child_table, value, depth + 1, budget, visiting)
            }
            Expression::Product(left, right) => product_kind(
                self.check_expression(spec, table, left, depth + 1, budget, visiting)?,
                self.check_expression(spec, table, right, depth + 1, budget, visiting)?,
            ),
            Expression::Total(values) => {
                let mut kind = None;
                for value in values {
                    let current =
                        self.check_expression(spec, table, value, depth + 1, budget, visiting)?;
                    if kind.is_some_and(|k| k != current) {
                        return Err(RelationError::NumericTypeMismatch);
                    }
                    kind = Some(current);
                }
                kind.ok_or(RelationError::EmptyTotal)
            }
            Expression::Named(name) => {
                let key = (table.clone(), name.clone());
                if !visiting.insert(key.clone()) {
                    return Err(RelationError::FormulaCycle {
                        table: key.0,
                        name: key.1,
                    });
                }
                let expression =
                    self.definitions
                        .get(&key)
                        .ok_or_else(|| RelationError::MissingFormula {
                            table: table.clone(),
                            name: name.clone(),
                        })?;
                let kind =
                    self.check_expression(spec, table, expression, depth + 1, budget, visiting)?;
                visiting.remove(&key);
                Ok(kind)
            }
        }
    }

    pub fn evaluate(
        &self,
        store: &RelationStore,
        table: &TableId,
        record: &RecordId,
        name: &str,
        policy: &dyn OutputPolicy,
    ) -> Result<Numeric, RelationError> {
        store.authorize_snapshot(policy)?;
        let key = (table.clone(), name.to_owned());
        let expression =
            self.definitions
                .get(&key)
                .ok_or_else(|| RelationError::MissingFormula {
                    table: table.clone(),
                    name: name.to_owned(),
                })?;
        let row = store.authorized_record(table, record, policy)?;
        if self.shape != SemanticShape::from(&store.spec) {
            return Err(RelationError::DefinitionMismatch);
        }
        self.authorize_expression(store, table, expression, policy)?;
        let mut budget = Budget::new(self.limits);
        budget.record()?;
        let result = self.evaluate_expression(store, row, expression, 1, &mut budget, policy)?;
        if self.kinds.get(&key) != Some(&result.kind()) {
            return Err(RelationError::NumericTypeMismatch);
        }
        Ok(result)
    }

    fn authorize_expression(
        &self,
        store: &RelationStore,
        table: &TableId,
        expression: &Expression,
        policy: &dyn OutputPolicy,
    ) -> Result<(), RelationError> {
        match expression {
            Expression::Field(field) => {
                if !policy.allow_field(store.records.scope(), table, field) {
                    return Err(RelationError::OutputDenied);
                }
            }
            Expression::Lookup {
                ref_field,
                target_field,
            } => {
                let target = reference_target(&store.spec, table, ref_field)?;
                if !policy.allow_field(store.records.scope(), table, ref_field)
                    || !policy.allow_field(store.records.scope(), target, target_field)
                {
                    return Err(RelationError::OutputDenied);
                }
            }
            Expression::Sum {
                child_table,
                ref_field,
                value,
            } => {
                let mut fields = BTreeSet::from([ref_field.clone()]);
                collect_fields(value, &self.definitions, child_table, &mut fields);
                store.authorize_scan(
                    child_table,
                    &fields.into_iter().collect::<Vec<_>>(),
                    policy,
                )?;
                self.authorize_expression(store, child_table, value, policy)?;
            }
            Expression::Product(a, b) => {
                self.authorize_expression(store, table, a, policy)?;
                self.authorize_expression(store, table, b, policy)?;
            }
            Expression::Total(values) => {
                for value in values {
                    self.authorize_expression(store, table, value, policy)?;
                }
            }
            Expression::Named(name) => {
                let key = (table.clone(), name.clone());
                self.authorize_expression(
                    store,
                    table,
                    self.definitions
                        .get(&key)
                        .expect("checked formula reference"),
                    policy,
                )?;
            }
        }
        Ok(())
    }

    fn evaluate_expression(
        &self,
        store: &RelationStore,
        row: &CheckedRecord,
        expression: &Expression,
        depth: usize,
        budget: &mut Budget,
        policy: &dyn OutputPolicy,
    ) -> Result<Numeric, RelationError> {
        budget.node(depth)?;
        match expression {
            Expression::Field(field) => {
                store.authorize_field(row, field, policy)?;
                numeric_value(
                    row.values().get(field),
                    numeric_kind(&store.spec, row.table_id(), field)?,
                )
            }
            Expression::Lookup {
                ref_field,
                target_field,
            } => {
                let target = reference_target(&store.spec, row.table_id(), ref_field)?;
                if matches!(row.values().get(ref_field), Some(Value::Ref(_))) {
                    budget.record()?;
                }
                let kind = numeric_kind(&store.spec, target, target_field)?;
                let value =
                    store.lookup(row.table_id(), row.id(), ref_field, target_field, policy)?;
                numeric_value(Some(&value), kind)
            }
            Expression::Sum {
                child_table,
                ref_field,
                value,
            } => {
                store.reference_target(child_table, ref_field, row.table_id())?;
                // A host must grant a complete projection before its empty/nonempty state
                // can affect output, including nested numeric and lookup dependencies.
                let mut fields = BTreeSet::from([ref_field.clone()]);
                collect_fields(value, &self.definitions, child_table, &mut fields);
                let fields: Vec<_> = fields.into_iter().collect();
                store.authorize_scan(child_table, &fields, policy)?;
                let mut compile_budget = Budget::new(self.limits);
                let kind = self.check_expression(
                    &store.spec,
                    child_table,
                    value,
                    1,
                    &mut compile_budget,
                    &mut BTreeSet::new(),
                )?;
                let mut sum = 0i64;
                for child in store
                    .records
                    .records()
                    .iter()
                    .filter(|r| r.table_id() == child_table)
                {
                    // Authorize before reading the relation, including unrelated rows.
                    store.authorize_field(child, ref_field, policy)?;
                    budget.record()?;
                    if matches!(child.values().get(ref_field), Some(Value::Ref(r)) if &r.table_id == row.table_id() && &r.record_id == row.id())
                    {
                        let result = self.evaluate_expression(
                            store,
                            child,
                            value,
                            depth + 1,
                            budget,
                            policy,
                        )?;
                        sum = sum
                            .checked_add(result.value().unwrap_or(0))
                            .ok_or(RelationError::Overflow)?;
                    }
                }
                Ok(Numeric::from(kind, sum))
            }
            Expression::Product(left, right) => {
                let left = self.evaluate_expression(store, row, left, depth + 1, budget, policy)?;
                let right =
                    self.evaluate_expression(store, row, right, depth + 1, budget, policy)?;
                let kind = product_kind(left.kind(), right.kind())?;
                match (left.value(), right.value()) {
                    (Some(a), Some(b)) => Ok(Numeric::from(
                        kind,
                        a.checked_mul(b).ok_or(RelationError::Overflow)?,
                    )),
                    _ => Ok(Numeric::Null(kind)),
                }
            }
            Expression::Total(values) => {
                let mut result = None;
                let mut sum = 0i64;
                for value in values {
                    let value =
                        self.evaluate_expression(store, row, value, depth + 1, budget, policy)?;
                    if result.is_some_and(|k| k != value.kind()) {
                        return Err(RelationError::NumericTypeMismatch);
                    }
                    result = Some(value.kind());
                    sum = sum
                        .checked_add(value.value().unwrap_or(0))
                        .ok_or(RelationError::Overflow)?;
                }
                Ok(Numeric::from(result.ok_or(RelationError::EmptyTotal)?, sum))
            }
            Expression::Named(name) => {
                let key = (row.table_id().clone(), name.clone());
                let expression =
                    self.definitions
                        .get(&key)
                        .ok_or(RelationError::MissingFormula {
                            table: key.0,
                            name: key.1,
                        })?;
                self.evaluate_expression(store, row, expression, depth + 1, budget, policy)
            }
        }
    }
}

fn collect_fields(
    expression: &Expression,
    definitions: &BTreeMap<(TableId, String), Expression>,
    table: &TableId,
    fields: &mut BTreeSet<FieldId>,
) {
    match expression {
        Expression::Field(field) => {
            fields.insert(field.clone());
        }
        Expression::Lookup { ref_field, .. } => {
            fields.insert(ref_field.clone());
        }
        Expression::Product(a, b) => {
            collect_fields(a, definitions, table, fields);
            collect_fields(b, definitions, table, fields);
        }
        Expression::Total(values) => {
            for value in values {
                collect_fields(value, definitions, table, fields);
            }
        }
        Expression::Named(name) => {
            if let Some(value) = definitions.get(&(table.clone(), name.clone())) {
                collect_fields(value, definitions, table, fields);
            }
        }
        Expression::Sum { .. } => {}
    }
}

fn reference_target<'a>(
    spec: &'a CheckedAppSpec,
    table: &TableId,
    field: &FieldId,
) -> Result<&'a TableId, RelationError> {
    let definition = spec
        .field(table, field)
        .ok_or_else(|| RelationError::UnknownField {
            table: table.clone(),
            field: field.clone(),
        })?;
    match &definition.field_type {
        FieldType::Ref { target_table } => Ok(target_table),
        _ => Err(RelationError::ReferenceFieldRequired {
            table: table.clone(),
            field: field.clone(),
        }),
    }
}
fn numeric_kind(
    spec: &CheckedAppSpec,
    table: &TableId,
    field: &FieldId,
) -> Result<NumericKind, RelationError> {
    let definition = spec
        .field(table, field)
        .ok_or_else(|| RelationError::UnknownField {
            table: table.clone(),
            field: field.clone(),
        })?;
    match definition.field_type {
        FieldType::Integer => Ok(NumericKind::Integer),
        FieldType::Money => Ok(NumericKind::Money),
        _ => Err(RelationError::NumericFieldRequired {
            table: table.clone(),
            field: field.clone(),
        }),
    }
}
fn numeric_value(value: Option<&Value>, kind: NumericKind) -> Result<Numeric, RelationError> {
    match (value, kind) {
        (None | Some(Value::Null | Value::Blank), kind) => Ok(Numeric::Null(kind)),
        (Some(Value::Integer(v)), NumericKind::Integer) => Ok(Numeric::Integer(*v)),
        (Some(Value::Money(v)), NumericKind::Money) => Ok(Numeric::Money(*v)),
        _ => Err(RelationError::NumericTypeMismatch),
    }
}
fn product_kind(left: NumericKind, right: NumericKind) -> Result<NumericKind, RelationError> {
    match (left, right) {
        (NumericKind::Money, NumericKind::Money) => Err(RelationError::MoneyProductUnsupported),
        (NumericKind::Integer, NumericKind::Integer) => Ok(NumericKind::Integer),
        _ => Ok(NumericKind::Money),
    }
}
struct Budget {
    limits: FormulaLimits,
    nodes: usize,
    records: usize,
}
impl Budget {
    fn new(limits: FormulaLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            records: 0,
        }
    }
    fn node(&mut self, depth: usize) -> Result<(), RelationError> {
        if depth > self.limits.max_depth {
            return Err(RelationError::DepthExceeded);
        }
        self.nodes += 1;
        if self.nodes > self.limits.max_nodes {
            return Err(RelationError::NodeBudgetExceeded);
        }
        Ok(())
    }
    fn record(&mut self) -> Result<(), RelationError> {
        self.records += 1;
        if self.records > self.limits.max_records {
            return Err(RelationError::RecordBudgetExceeded);
        }
        Ok(())
    }
}
