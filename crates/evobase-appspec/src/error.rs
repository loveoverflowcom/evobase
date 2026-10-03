use crate::{AppId, ConstraintId, FieldId, RecordId, TableId};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("invalid {kind} identity: {value}")]
    InvalidId { kind: &'static str, value: String },
    #[error("invalid tenant identity")]
    InvalidTenant,
    #[error("invalid JSON: {message}")]
    InvalidJson { message: String },
    #[error("unsupported AppSpec version {found}; supported version is {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },
    #[error("AppSpec version {version} does not support {declaration}")]
    UnsupportedDeclaration {
        version: u32,
        declaration: &'static str,
    },
    #[error("{resource} exceeds limit {limit}")]
    LimitExceeded {
        resource: &'static str,
        limit: usize,
    },
    #[error("invalid name: {path}")]
    InvalidName { path: String },
    #[error("duplicate table identity: {0}")]
    DuplicateTable(TableId),
    #[error("duplicate field identity {field} in table {table}")]
    DuplicateField { table: TableId, field: FieldId },
    #[error("unknown table: {0}")]
    UnknownTable(TableId),
    #[error("unknown field {field} in table {table}")]
    UnknownField { table: TableId, field: FieldId },
    #[error("invalid capture rule: {reason}")]
    InvalidCapture { reason: String },
    #[error("invalid or unsupported policy/command rule: {rule_id}")]
    InvalidPolicy { rule_id: String },
    #[error("invalid constraint {constraint}: {reason}")]
    InvalidConstraint {
        constraint: ConstraintId,
        reason: &'static str,
    },
    #[error("constraint {constraint} failed for field {field} in record {record}")]
    ConstraintViolation {
        record: RecordId,
        field: FieldId,
        constraint: ConstraintId,
    },
    #[error("expected app {expected}; received app {actual}")]
    WrongApp { expected: AppId, actual: AppId },
    #[error("record {record} crosses the expected scope")]
    CrossScope { record: RecordId },
    #[error("duplicate record {record} in table {table}")]
    DuplicateRecord { table: TableId, record: RecordId },
    #[error("required field {field} is blank in record {record}")]
    Required { record: RecordId, field: FieldId },
    #[error("field {field} in record {record} expects {expected}")]
    WrongType {
        record: RecordId,
        field: FieldId,
        expected: &'static str,
    },
    #[error("reference {record} expects table {expected}; received table {actual}")]
    WrongReferenceTable {
        record: RecordId,
        expected: TableId,
        actual: TableId,
    },
    #[error("missing reference {record} in table {table}")]
    MissingReference { table: TableId, record: RecordId },
    #[error("ambiguous reference label {label} in table {table}")]
    AmbiguousReference { table: TableId, label: String },
    #[error("invalid {expected} input: {input}")]
    InvalidCell {
        expected: &'static str,
        input: String,
    },
    #[error("{kind} overflows the signed 64-bit range")]
    Overflow { kind: &'static str },
}
