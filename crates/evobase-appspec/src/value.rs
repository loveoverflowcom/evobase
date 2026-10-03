use crate::{
    CheckedAppSpec, Error, FieldId, FieldType, MAX_RECORDS, MAX_TEXT_BYTES, RecordId, Scope,
    TableId,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Value {
    Blank,
    Null,
    Text(String),
    Integer(i64),
    Bool(bool),
    /// Exact signed minor units. JSON floats and exponent notation are rejected.
    Money(i64),
    Ref(RecordRef),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordRef {
    pub scope: Scope,
    pub table_id: TableId,
    pub record_id: RecordId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRecord {
    pub scope: Scope,
    pub id: RecordId,
    pub table_id: TableId,
    #[serde(deserialize_with = "unique_values")]
    pub values: BTreeMap<FieldId, Value>,
}

fn unique_values<'de, D: Deserializer<'de>>(de: D) -> Result<BTreeMap<FieldId, Value>, D::Error> {
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = BTreeMap<FieldId, Value>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a field-value object without duplicate keys")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut values = BTreeMap::new();
            while let Some((field, value)) = map.next_entry::<FieldId, Value>()? {
                if values.insert(field.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate field value {field}"
                    )));
                }
                if values.len() > crate::MAX_FIELDS_PER_TABLE {
                    return Err(serde::de::Error::custom("too many field values"));
                }
            }
            Ok(values)
        }
    }
    de.deserialize_map(Visitor)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedRecord {
    raw: RawRecord,
}
impl CheckedRecord {
    pub fn id(&self) -> &RecordId {
        &self.raw.id
    }
    pub fn table_id(&self) -> &TableId {
        &self.raw.table_id
    }
    pub fn scope(&self) -> &Scope {
        &self.raw.scope
    }
    pub fn values(&self) -> &BTreeMap<FieldId, Value> {
        &self.raw.values
    }
    pub fn as_raw(&self) -> &RawRecord {
        &self.raw
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedRecords {
    scope: Scope,
    records: Vec<CheckedRecord>,
}
impl CheckedRecords {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }
    pub fn records(&self) -> &[CheckedRecord] {
        &self.records
    }
    pub fn find(&self, table: &TableId, record: &RecordId) -> Option<&CheckedRecord> {
        self.records
            .iter()
            .find(|r| r.table_id() == table && r.id() == record)
    }
    pub fn to_raw(&self) -> Vec<RawRecord> {
        self.records.iter().map(|r| r.raw.clone()).collect()
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        crate::codec::bounded_encode(&self.records.iter().map(|r| &r.raw).collect::<Vec<_>>())
    }
}

impl CheckedAppSpec {
    /// Validate the entire candidate final state, so insert order never determines Ref validity.
    /// This checks persisted facts; command authorization and capture derivation are separate owners.
    pub fn validate_records(
        &self,
        scope: &Scope,
        records: &[RawRecord],
    ) -> Result<CheckedRecords, Error> {
        if scope.app_id() != self.app_id() {
            return Err(Error::WrongApp {
                expected: self.app_id().clone(),
                actual: scope.app_id().clone(),
            });
        }
        if records.len() > MAX_RECORDS {
            return Err(Error::LimitExceeded {
                resource: "records",
                limit: MAX_RECORDS,
            });
        }
        crate::codec::bounded_encode(records)?;
        let mut sorted: Vec<_> = records.iter().collect();
        sorted.sort_by(|a, b| (&a.table_id, &a.id).cmp(&(&b.table_id, &b.id)));
        let mut identities = BTreeMap::new();
        for record in &sorted {
            if &record.scope != scope {
                return Err(Error::CrossScope {
                    record: record.id.clone(),
                });
            }
            if self.table(&record.table_id).is_none() {
                return Err(Error::UnknownTable(record.table_id.clone()));
            }
            if identities
                .insert((&record.table_id, &record.id), *record)
                .is_some()
            {
                return Err(Error::DuplicateRecord {
                    table: record.table_id.clone(),
                    record: record.id.clone(),
                });
            }
        }
        let mut checked = Vec::with_capacity(records.len());
        for record in sorted {
            let table = self.table(&record.table_id).expect("table checked above");
            for field_id in record.values.keys() {
                if self.field(&record.table_id, field_id).is_none() {
                    return Err(Error::UnknownField {
                        table: record.table_id.clone(),
                        field: field_id.clone(),
                    });
                }
            }
            let mut values = BTreeMap::new();
            for field in &table.fields {
                let value = record.values.get(&field.id).unwrap_or(&Value::Blank);
                if matches!(value, Value::Blank | Value::Null) {
                    if field.required {
                        return Err(Error::Required {
                            record: record.id.clone(),
                            field: field.id.clone(),
                        });
                    }
                } else {
                    let correct = matches!(
                        (&field.field_type, value),
                        (FieldType::Text, Value::Text(_))
                            | (FieldType::Integer, Value::Integer(_))
                            | (FieldType::Bool, Value::Bool(_))
                            | (FieldType::Money, Value::Money(_))
                            | (FieldType::Ref { .. }, Value::Ref(_))
                    );
                    if !correct {
                        return Err(Error::WrongType {
                            record: record.id.clone(),
                            field: field.id.clone(),
                            expected: field.field_type.name(),
                        });
                    }
                    if let Value::Text(text) = value {
                        if text.len() > MAX_TEXT_BYTES {
                            return Err(Error::LimitExceeded {
                                resource: "text bytes",
                                limit: MAX_TEXT_BYTES,
                            });
                        }
                    }
                    if let Value::Ref(reference) = value {
                        if &reference.scope != scope {
                            return Err(Error::CrossScope {
                                record: reference.record_id.clone(),
                            });
                        }
                        let FieldType::Ref { target_table } = &field.field_type else {
                            unreachable!("type checked above")
                        };
                        if &reference.table_id != target_table {
                            return Err(Error::WrongReferenceTable {
                                record: reference.record_id.clone(),
                                expected: target_table.clone(),
                                actual: reference.table_id.clone(),
                            });
                        }
                        if !identities.contains_key(&(&reference.table_id, &reference.record_id)) {
                            if let Some(other) =
                                records.iter().find(|other| other.id == reference.record_id)
                            {
                                return Err(Error::WrongReferenceTable {
                                    record: reference.record_id.clone(),
                                    expected: target_table.clone(),
                                    actual: other.table_id.clone(),
                                });
                            }
                            return Err(Error::MissingReference {
                                table: target_table.clone(),
                                record: reference.record_id.clone(),
                            });
                        }
                    }
                }
                values.insert(field.id.clone(), value.clone());
            }
            checked.push(CheckedRecord {
                raw: RawRecord {
                    values,
                    ..record.clone()
                },
            });
        }
        // Optional Blank normalization can enlarge sparse input; checked facts must remain encodable.
        crate::codec::bounded_encode(&checked.iter().map(|r| &r.raw).collect::<Vec<_>>())?;
        Ok(CheckedRecords {
            scope: scope.clone(),
            records: checked,
        })
    }

    pub fn decode_records(&self, scope: &Scope, bytes: &[u8]) -> Result<CheckedRecords, Error> {
        crate::codec::preflight(bytes)?;
        let raw: Vec<RawRecord> =
            serde_json::from_slice(bytes).map_err(crate::codec::json_error)?;
        self.validate_records(scope, &raw)
    }

    /// Text authoring projection shared by local builders. This returns a value, never authority.
    /// Empty input is Blank; literal `null` is Null. `text:` escapes reserved text literals.
    /// Money uses integer minor-unit text.
    pub fn parse_cell(
        &self,
        scope: &Scope,
        table: &TableId,
        field: &FieldId,
        text: &str,
        records: &[RawRecord],
    ) -> Result<Value, Error> {
        if scope.app_id() != self.app_id() {
            return Err(Error::WrongApp {
                expected: self.app_id().clone(),
                actual: scope.app_id().clone(),
            });
        }
        let definition = self
            .field(table, field)
            .ok_or_else(|| Error::UnknownField {
                table: table.clone(),
                field: field.clone(),
            })?;
        if definition.field_type == FieldType::Text {
            if let Some(literal) = text.strip_prefix("text:") {
                if literal.len() > MAX_TEXT_BYTES {
                    return Err(Error::LimitExceeded {
                        resource: "text bytes",
                        limit: MAX_TEXT_BYTES,
                    });
                }
                return Ok(Value::Text(literal.to_owned()));
            }
        }
        if text.is_empty() {
            return Ok(Value::Blank);
        }
        if text == "null" {
            return Ok(Value::Null);
        }
        let invalid = || Error::InvalidCell {
            expected: definition.field_type.name(),
            input: text.to_owned(),
        };
        let integer = || -> Result<i64, Error> {
            if !text
                .bytes()
                .enumerate()
                .all(|(i, c)| c.is_ascii_digit() || (i == 0 && c == b'-'))
            {
                return Err(invalid());
            }
            text.parse::<i64>().map_err(|e| match e.kind() {
                std::num::IntErrorKind::PosOverflow | std::num::IntErrorKind::NegOverflow => {
                    Error::Overflow {
                        kind: definition.field_type.name(),
                    }
                }
                _ => invalid(),
            })
        };
        match &definition.field_type {
            FieldType::Text => {
                if text.len() > MAX_TEXT_BYTES {
                    return Err(Error::LimitExceeded {
                        resource: "text bytes",
                        limit: MAX_TEXT_BYTES,
                    });
                }
                Ok(Value::Text(text.to_owned()))
            }
            FieldType::Integer => Ok(Value::Integer(integer()?)),
            FieldType::Money => Ok(Value::Money(integer()?)),
            FieldType::Bool => match text {
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                _ => Err(invalid()),
            },
            FieldType::Ref { target_table } => {
                if records.len() > MAX_RECORDS {
                    return Err(Error::LimitExceeded {
                        resource: "records",
                        limit: MAX_RECORDS,
                    });
                }
                let record_id = if text.starts_with("rec_") {
                    let id = RecordId::new(text)?;
                    if !records
                        .iter()
                        .any(|r| &r.scope == scope && &r.table_id == target_table && r.id == id)
                    {
                        if let Some(other) = records.iter().find(|r| r.id == id) {
                            if &other.scope != scope {
                                return Err(Error::CrossScope { record: id });
                            }
                            return Err(Error::WrongReferenceTable {
                                record: id,
                                expected: target_table.clone(),
                                actual: other.table_id.clone(),
                            });
                        }
                        return Err(Error::MissingReference {
                            table: target_table.clone(),
                            record: id,
                        });
                    }
                    id
                } else {
                    let target = self.table(target_table).expect("compiled graph");
                    let display_field = target
                        .fields
                        .iter()
                        .find(|f| {
                            f.field_type == FieldType::Text && f.id.as_str().ends_with("_name")
                        })
                        .or_else(|| {
                            target
                                .fields
                                .iter()
                                .find(|f| f.field_type == FieldType::Text)
                        });
                    let mut matches = records.iter().filter(|r| {
                        &r.scope == scope
                            && &r.table_id == target_table
                            && display_field.is_some_and(|field| {
                                r.values.get(&field.id) == Some(&Value::Text(text.to_owned()))
                            })
                    });
                    let Some(first) = matches.next() else {
                        return Err(invalid());
                    };
                    if matches.next().is_some() {
                        return Err(Error::AmbiguousReference {
                            table: target_table.clone(),
                            label: text.to_owned(),
                        });
                    }
                    first.id.clone()
                };
                Ok(Value::Ref(RecordRef {
                    scope: scope.clone(),
                    table_id: target_table.clone(),
                    record_id,
                }))
            }
        }
    }
}
