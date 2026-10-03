//! Versioned, bounded raw transport for the generic AppSpec host.
//! Decoding a DTO establishes syntax only: the host supplies scope and current authority,
//! and the semantic core checks every command before an adapter may persist it.

use evobase_appspec::{AppId, FieldId, FieldType, RecordId, RecordRef, Scope, TableId, Value};
use serde::de::{DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};

pub const API_VERSION: u32 = 1;
pub const MAX_WIRE_BYTES: usize = 1_048_576;
pub const MAX_WIRE_DEPTH: usize = 16;
pub const MAX_WIRE_NODES: usize = 20_000;
pub const MAX_REQUEST_KEY_BYTES: usize = 96;
pub const MAX_RELEASE_ID_BYTES: usize = 128;
pub const MAX_LIST_LIMIT: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WireError {
    UnsupportedVersion {
        found: u32,
    },
    LimitExceeded {
        resource: &'static str,
        limit: usize,
    },
    DuplicateKey {
        key: String,
    },
    InvalidJson {
        message: String,
    },
    Validation {
        field: &'static str,
        message: String,
    },
}
impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion { found } => write!(f, "unsupported API version {found}"),
            Self::LimitExceeded { resource, limit } => write!(f, "{resource} exceeds {limit}"),
            Self::DuplicateKey { key } => write!(f, "duplicate JSON key {key}"),
            Self::InvalidJson { message } => message.fmt(f),
            Self::Validation { field, message } => write!(f, "{field}: {message}"),
        }
    }
}
impl std::error::Error for WireError {}

/// Storage data revision, encoded as a canonical decimal string to avoid JavaScript rounding.
/// This is distinct from AppSpec's format version and the immutable release identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RevisionDto(u64);
impl RevisionDto {
    pub const fn new(revision: u64) -> Self {
        Self(revision)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}
impl Serialize for RevisionDto {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> Deserialize<'de> for RevisionDto {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let text = String::deserialize(de)?;
        if text.is_empty()
            || text.len() > 20
            || !text.bytes().all(|byte| byte.is_ascii_digit())
            || (text.len() > 1 && text.starts_with('0'))
        {
            return Err(serde::de::Error::custom(
                "revision must be a canonical u64 decimal string",
            ));
        }
        text.parse::<u64>()
            .map(Self)
            .map_err(|_| serde::de::Error::custom("revision exceeds u64"))
    }
}

/// A Ref names records inside the app selected by the route. It supplies no tenant selector.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefDto {
    pub table_id: TableId,
    pub record_id: RecordId,
}

/// Exact tagged values. Integer/Money JSON numbers must be parsed by an exact integer decoder;
/// browser consumers use this Rust decoder rather than JSON.parse for these values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ValueDto {
    Blank,
    Null,
    Text(String),
    Integer(i64),
    Bool(bool),
    Money(i64),
    Ref(RefDto),
}
impl ValueDto {
    /// A raw value conversion, not authorization or record validation. Scope comes from the host.
    pub fn to_domain(&self, scope: &Scope) -> Value {
        match self {
            Self::Blank => Value::Blank,
            Self::Null => Value::Null,
            Self::Text(value) => Value::Text(value.clone()),
            Self::Integer(value) => Value::Integer(*value),
            Self::Bool(value) => Value::Bool(*value),
            Self::Money(value) => Value::Money(*value),
            Self::Ref(value) => Value::Ref(RecordRef {
                scope: scope.clone(),
                table_id: value.table_id.clone(),
                record_id: value.record_id.clone(),
            }),
        }
    }
    /// The adapter must check scope before exposing a stored Ref in this scope-free projection.
    pub fn from_domain(value: &Value, scope: &Scope) -> Result<Self, WireError> {
        Ok(match value {
            Value::Blank => Self::Blank,
            Value::Null => Self::Null,
            Value::Text(value) => Self::Text(value.clone()),
            Value::Integer(value) => Self::Integer(*value),
            Value::Bool(value) => Self::Bool(*value),
            Value::Money(value) => Self::Money(*value),
            Value::Ref(value) if &value.scope == scope => Self::Ref(RefDto {
                table_id: value.table_id.clone(),
                record_id: value.record_id.clone(),
            }),
            Value::Ref(_) => return Err(validation("value", "cross-scope Ref")),
        })
    }
    fn validate(&self) -> Result<(), WireError> {
        if let Self::Text(value) = self {
            limit(value.len(), evobase_appspec::MAX_TEXT_BYTES, "text bytes")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandRequestDto {
    pub api_version: u32,
    pub command_id: String,
    pub record_id: RecordId,
    #[serde(deserialize_with = "unique_values")]
    pub params: BTreeMap<FieldId, ValueDto>,
    pub request_key: String,
    pub expected_revision: RevisionDto,
    pub release_id: String,
}

/// The only reviewed list query parameter in v1. Filters, joins, ordering, offsets and scans
/// require a later capability contract and must not be silently ignored by the HTTP adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListQueryDto {
    pub limit: Option<u16>,
}
impl ListQueryDto {
    pub fn limit_or_default(&self) -> Result<usize, WireError> {
        let value = usize::from(self.limit.unwrap_or(100));
        if value == 0 || value > MAX_LIST_LIMIT {
            return Err(validation("limit", "expected 1..256"));
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldMetadataDto {
    pub field_id: FieldId,
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableMetadataDto {
    pub table_id: TableId,
    pub name: String,
    pub fields: Vec<FieldMetadataDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandMetadataDto {
    pub command_id: String,
    pub name: String,
    pub table_id: TableId,
    pub input_fields: Vec<FieldMetadataDto>,
    /// Runtime actions are offered only for currently Read/Write/state-authorized records.
    /// Design metadata uses an empty list, since definition visibility does not grant execution.
    pub eligible_record_ids: Vec<RecordId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataResponseDto {
    pub api_version: u32,
    pub app_id: AppId,
    pub release_id: String,
    pub revision: RevisionDto,
    /// Present only in the Design-authorized projection. Runtime receives permitted metadata.
    pub canonical_appspec: Option<String>,
    pub supported: Vec<String>,
    pub tables: Vec<TableMetadataDto>,
    pub commands: Vec<CommandMetadataDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordDto {
    pub record_id: RecordId,
    #[serde(deserialize_with = "unique_values")]
    pub values: BTreeMap<FieldId, ValueDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListResponseDto {
    pub api_version: u32,
    pub app_id: AppId,
    pub table_id: TableId,
    pub release_id: String,
    pub revision: RevisionDto,
    pub records: Vec<RecordDto>,
    /// Indicates omitted authorized rows. V1 has no continuation cursor or offset support.
    pub has_more: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptResponseDto {
    pub api_version: u32,
    pub app_id: AppId,
    pub command_id: String,
    pub record_id: RecordId,
    pub release_id: String,
    pub request_key: String,
    pub revision: RevisionDto,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCodeDto {
    Denied,
    Unsupported,
    Conflict,
    Validation,
    NotFound,
    Internal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiErrorDto {
    pub api_version: u32,
    pub code: ErrorCodeDto,
    pub message: String,
    pub field: Option<String>,
}
impl ApiErrorDto {
    pub fn new(code: ErrorCodeDto, message: impl Into<String>, field: Option<String>) -> Self {
        Self {
            api_version: API_VERSION,
            code,
            message: message.into(),
            field,
        }
    }
}

trait WireMessage {
    fn api_version(&self) -> u32;
    fn validate(&self) -> Result<(), WireError>;
}
macro_rules! wire_methods {
    ($($ty:ty),+ $(,)?) => {$ (
        impl $ty {
            pub fn decode(bytes: &[u8]) -> Result<Self, WireError> { decode(bytes) }
            pub fn encode(&self) -> Result<Vec<u8>, WireError> { encode(self) }
        }
    )+};
}
wire_methods!(
    CommandRequestDto,
    MetadataResponseDto,
    ListResponseDto,
    ReceiptResponseDto,
    ApiErrorDto
);

impl WireMessage for CommandRequestDto {
    fn api_version(&self) -> u32 {
        self.api_version
    }
    fn validate(&self) -> Result<(), WireError> {
        command_id(&self.command_id)?;
        request_key(&self.request_key)?;
        release_id(&self.release_id)?;
        values(&self.params)
    }
}
impl WireMessage for MetadataResponseDto {
    fn api_version(&self) -> u32 {
        self.api_version
    }
    fn validate(&self) -> Result<(), WireError> {
        release_id(&self.release_id)?;
        limit(self.tables.len(), evobase_appspec::MAX_TABLES, "tables")?;
        limit(self.commands.len(), 64, "commands")?;
        limit(self.supported.len(), 64, "supported capabilities")?;
        for capability in &self.supported {
            limit(capability.len(), 256, "capability bytes")?;
        }
        let mut field_count = 0;
        let mut table_ids = BTreeSet::new();
        let mut field_ids = BTreeSet::new();
        for table in &self.tables {
            if !table_ids.insert(&table.table_id) {
                return Err(validation("tables", "duplicate table identity"));
            }
            name(&table.name)?;
            metadata_fields(&table.fields)?;
            field_count += table.fields.len();
            for field in &table.fields {
                if !field_ids.insert(&field.field_id) {
                    return Err(validation("fields", "duplicate field identity"));
                }
            }
        }
        limit(field_count, evobase_appspec::MAX_FIELDS, "fields")?;
        let mut command_ids = BTreeSet::new();
        for command in &self.commands {
            if !command_ids.insert(&command.command_id) {
                return Err(validation("commands", "duplicate command identity"));
            }
            command_id(&command.command_id)?;
            name(&command.name)?;
            metadata_fields(&command.input_fields)?;
            limit(
                command.eligible_record_ids.len(),
                evobase_appspec::MAX_RECORDS,
                "eligible records",
            )?;
            let mut records = BTreeSet::new();
            for record in &command.eligible_record_ids {
                if !records.insert(record) {
                    return Err(validation(
                        "eligible_record_ids",
                        "duplicate record identity",
                    ));
                }
            }
        }
        Ok(())
    }
}
impl WireMessage for ListResponseDto {
    fn api_version(&self) -> u32 {
        self.api_version
    }
    fn validate(&self) -> Result<(), WireError> {
        release_id(&self.release_id)?;
        limit(self.records.len(), MAX_LIST_LIMIT, "records")?;
        let mut record_ids = BTreeSet::new();
        for record in &self.records {
            if !record_ids.insert(&record.record_id) {
                return Err(validation("records", "duplicate record identity"));
            }
            values(&record.values)?;
        }
        Ok(())
    }
}
impl WireMessage for ReceiptResponseDto {
    fn api_version(&self) -> u32 {
        self.api_version
    }
    fn validate(&self) -> Result<(), WireError> {
        command_id(&self.command_id)?;
        request_key(&self.request_key)?;
        release_id(&self.release_id)
    }
}
impl WireMessage for ApiErrorDto {
    fn api_version(&self) -> u32 {
        self.api_version
    }
    fn validate(&self) -> Result<(), WireError> {
        limit(self.message.len(), 1_024, "error message bytes")?;
        if let Some(field) = &self.field {
            limit(field.len(), 256, "error field bytes")?;
        }
        Ok(())
    }
}

fn validation(field: &'static str, message: &str) -> WireError {
    WireError::Validation {
        field,
        message: message.to_owned(),
    }
}
fn limit(length: usize, max: usize, resource: &'static str) -> Result<(), WireError> {
    if length > max {
        Err(WireError::LimitExceeded {
            resource,
            limit: max,
        })
    } else {
        Ok(())
    }
}
fn command_id(value: &str) -> Result<(), WireError> {
    if value.len() <= 4
        || value.len() > 80
        || !value.starts_with("cmd_")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(validation("command_id", "expected a stable cmd_ identity"));
    }
    Ok(())
}
pub fn validate_request_key(value: &str) -> Result<(), WireError> {
    request_key(value)
}
fn request_key(value: &str) -> Result<(), WireError> {
    if value.is_empty()
        || value.len() > MAX_REQUEST_KEY_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(validation(
            "request_key",
            "expected 1..96 ASCII letters, digits, '_' or '-'",
        ));
    }
    Ok(())
}
fn release_id(value: &str) -> Result<(), WireError> {
    if value.is_empty()
        || value.len() > MAX_RELEASE_ID_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':'))
    {
        return Err(validation("release_id", "invalid release identity"));
    }
    Ok(())
}
fn name(value: &str) -> Result<(), WireError> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(validation("name", "invalid display name"));
    }
    limit(value.len(), evobase_appspec::MAX_NAME_BYTES, "name bytes")
}
fn metadata_fields(fields: &[FieldMetadataDto]) -> Result<(), WireError> {
    limit(
        fields.len(),
        evobase_appspec::MAX_FIELDS_PER_TABLE,
        "fields per table",
    )?;
    let mut identities = BTreeSet::new();
    for field in fields {
        if !identities.insert(&field.field_id) {
            return Err(validation("fields", "duplicate field identity"));
        }
        name(&field.name)?;
    }
    Ok(())
}
fn values(values: &BTreeMap<FieldId, ValueDto>) -> Result<(), WireError> {
    limit(
        values.len(),
        evobase_appspec::MAX_FIELDS_PER_TABLE,
        "field values",
    )?;
    for value in values.values() {
        value.validate()?;
    }
    Ok(())
}
fn unique_values<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<BTreeMap<FieldId, ValueDto>, D::Error> {
    struct Values;
    impl<'de> Visitor<'de> for Values {
        type Value = BTreeMap<FieldId, ValueDto>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("unique field values")
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<FieldId, ValueDto>()? {
                if values.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("duplicate field value"));
                }
                if values.len() > evobase_appspec::MAX_FIELDS_PER_TABLE {
                    return Err(serde::de::Error::custom("too many field values"));
                }
            }
            Ok(values)
        }
    }
    de.deserialize_map(Values)
}

fn decode<T: DeserializeOwned + WireMessage>(bytes: &[u8]) -> Result<T, WireError> {
    preflight(bytes)?;
    // This second, streaming pass rejects duplicates even inside tagged values. Numbers are
    // never rewritten through serde_json::Value, so the typed third pass retains exact i64s.
    let mut duplicate = None;
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let result = UniqueKeys {
        duplicate: &mut duplicate,
    }
    .deserialize(&mut decoder);
    if let Some(key) = duplicate {
        return Err(WireError::DuplicateKey {
            key: diagnostic(key, 256),
        });
    }
    result.map_err(json_error)?;
    decoder.end().map_err(json_error)?;
    let dto: T = serde_json::from_slice(bytes).map_err(json_error)?;
    version(&dto)?;
    dto.validate()?;
    Ok(dto)
}
fn version<T: WireMessage>(dto: &T) -> Result<(), WireError> {
    if dto.api_version() == API_VERSION {
        Ok(())
    } else {
        Err(WireError::UnsupportedVersion {
            found: dto.api_version(),
        })
    }
}
fn json_error(error: serde_json::Error) -> WireError {
    let message = error.to_string();
    if message.starts_with("too many field values") {
        return WireError::LimitExceeded {
            resource: "field values",
            limit: evobase_appspec::MAX_FIELDS_PER_TABLE,
        };
    }
    WireError::InvalidJson {
        message: diagnostic(message, 1_024),
    }
}
fn diagnostic(mut message: String, limit: usize) -> String {
    if message.len() > limit {
        let mut end = limit.saturating_sub(3);
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        message.push_str("...");
    }
    message
}
fn encode<T: Serialize + WireMessage>(dto: &T) -> Result<Vec<u8>, WireError> {
    version(dto)?;
    dto.validate()?;
    struct Buffer(Vec<u8>);
    impl std::io::Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.0.len().saturating_add(bytes.len()) > MAX_WIRE_BYTES {
                return Err(std::io::Error::other("wire byte limit"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = Buffer(Vec::new());
    serde_json::to_writer(&mut buffer, dto).map_err(|error| {
        if error.is_io() {
            WireError::LimitExceeded {
                resource: "bytes",
                limit: MAX_WIRE_BYTES,
            }
        } else {
            json_error(error)
        }
    })?;
    preflight(&buffer.0)?;
    Ok(buffer.0)
}

/// Bound bytes, nesting and token starts before any DTO/string allocation. JSON syntax remains
/// the serde parser's responsibility; braces and escapes inside strings do not affect depth.
fn preflight(bytes: &[u8]) -> Result<(), WireError> {
    limit(bytes.len(), MAX_WIRE_BYTES, "bytes")?;
    let (mut depth, mut nodes) = (0usize, 0usize);
    let (mut string, mut escape, mut atom) = (false, false, false);
    for &byte in bytes {
        if string {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == b'"' {
                string = false;
            }
            continue;
        }
        match byte {
            b'"' => {
                string = true;
                atom = false;
                nodes += 1;
            }
            b'{' | b'[' => {
                depth += 1;
                nodes += 1;
                atom = false;
                limit(depth, MAX_WIRE_DEPTH, "depth")?;
            }
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
                atom = false;
            }
            b':' | b',' | b' ' | b'\n' | b'\r' | b'\t' => atom = false,
            _ if !atom => {
                nodes += 1;
                atom = true;
            }
            _ => {}
        }
        limit(nodes, MAX_WIRE_NODES, "nodes")?;
    }
    Ok(())
}

struct UniqueKeys<'a> {
    duplicate: &'a mut Option<String>,
}
impl<'de> DeserializeSeed<'de> for UniqueKeys<'_> {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, de: D) -> Result<(), D::Error> {
        de.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for UniqueKeys<'_> {
    type Value = ();
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JSON without duplicate object keys")
    }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while seq
            .next_element_seed(UniqueKeys {
                duplicate: self.duplicate,
            })?
            .is_some()
        {}
        Ok(())
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                *self.duplicate = Some(key);
                return Err(serde::de::Error::custom("duplicate JSON key"));
            }
            map.next_value_seed(UniqueKeys {
                duplicate: self.duplicate,
            })?;
        }
        Ok(())
    }
}
