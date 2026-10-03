//! Bounded, deterministic AppSpec definitions and final-state fact validation.
//! Definitions and scoped facts contain no host grants. Checked values cannot be deserialized.
mod codec;
mod error;
pub mod fixtures;
mod id;
mod model;
pub mod relations;
mod value;

pub use error::Error;
pub use id::{AppId, FieldId, RecordId, Scope, TableId};
pub use model::{CaptureRule, CheckedAppSpec, FieldType, RawAppSpec, RawField, RawTable};
pub use value::{CheckedRecord, CheckedRecords, RawRecord, RecordRef, Value};

pub const FORMAT_VERSION: u32 = 1;
pub const MAX_BYTES: usize = 1_048_576;
pub const MAX_DEPTH: usize = 16;
pub const MAX_NODES: usize = 20_000;
pub const MAX_TABLES: usize = 64;
pub const MAX_FIELDS_PER_TABLE: usize = 256;
pub const MAX_FIELDS: usize = 2_048;
pub const MAX_RECORDS: usize = 4_096;
pub const MAX_NAME_BYTES: usize = 256;
pub const MAX_TEXT_BYTES: usize = 16_384;
