use crate::{
    AppId, Error, FORMAT_VERSION, FieldId, MAX_FIELDS, MAX_FIELDS_PER_TABLE, MAX_NAME_BYTES,
    MAX_TABLES, TableId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAppSpec {
    pub version: u32,
    pub app_id: AppId,
    pub name: String,
    pub tables: Vec<RawTable>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capture_rules: Vec<CaptureRule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<crate::policy::RawOwnerRolePolicy>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub submit_rules: Vec<crate::policy::RawSubmitOrderRule>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTable {
    pub id: TableId,
    pub name: String,
    pub fields: Vec<RawField>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawField {
    pub id: FieldId,
    pub name: String,
    #[serde(default)]
    pub required: bool,
    pub field_type: FieldType,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum FieldType {
    Text,
    Integer,
    Bool,
    Money,
    Ref { target_table: TableId },
}
impl FieldType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Integer => "integer",
            Self::Bool => "bool",
            Self::Money => "money",
            Self::Ref { .. } => "ref",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureRule {
    pub line_table: TableId,
    pub product_ref_field: FieldId,
    pub product_price_field: FieldId,
    pub captured_price_field: FieldId,
}

/// A definition checked for profile limits, identity uniqueness and reference/capture graph.
/// Only compile/decode can construct it. It intentionally has no Deserialize implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedAppSpec {
    definition: RawAppSpec,
}

fn check_name(name: &str, path: String) -> Result<(), Error> {
    if name.is_empty() || name.len() > MAX_NAME_BYTES || name.chars().any(char::is_control) {
        return Err(Error::InvalidName { path });
    }
    Ok(())
}

impl CheckedAppSpec {
    pub fn compile(mut definition: RawAppSpec) -> Result<Self, Error> {
        if definition.version != FORMAT_VERSION {
            return Err(Error::UnsupportedVersion {
                found: definition.version,
                supported: FORMAT_VERSION,
            });
        }
        if definition.tables.len() > MAX_TABLES {
            return Err(Error::LimitExceeded {
                resource: "tables",
                limit: MAX_TABLES,
            });
        }
        if definition.capture_rules.len() > MAX_FIELDS {
            return Err(Error::LimitExceeded {
                resource: "capture rules",
                limit: MAX_FIELDS,
            });
        }
        check_name(&definition.name, "app.name".to_owned())?;
        definition.tables.sort_by(|a, b| a.id.cmp(&b.id));
        let mut table_ids = BTreeSet::new();
        let mut field_ids = BTreeSet::new();
        let mut count = 0;
        for table in &mut definition.tables {
            if !table_ids.insert(table.id.clone()) {
                return Err(Error::DuplicateTable(table.id.clone()));
            }
            check_name(&table.name, format!("{}.name", table.id))?;
            if table.fields.len() > MAX_FIELDS_PER_TABLE {
                return Err(Error::LimitExceeded {
                    resource: "fields per table",
                    limit: MAX_FIELDS_PER_TABLE,
                });
            }
            count += table.fields.len();
            if count > MAX_FIELDS {
                return Err(Error::LimitExceeded {
                    resource: "fields",
                    limit: MAX_FIELDS,
                });
            }
            table.fields.sort_by(|a, b| a.id.cmp(&b.id));
            for field in &table.fields {
                // Field identities are globally unique within an AppSpec, even across tables.
                if !field_ids.insert(field.id.clone()) {
                    return Err(Error::DuplicateField {
                        table: table.id.clone(),
                        field: field.id.clone(),
                    });
                }
                check_name(&field.name, format!("{}.{}.name", table.id, field.id))?;
            }
        }
        for table in &definition.tables {
            for field in &table.fields {
                if let FieldType::Ref { target_table } = &field.field_type
                    && !table_ids.contains(target_table)
                {
                    return Err(Error::UnknownTable(target_table.clone()));
                }
            }
        }
        definition.capture_rules.sort_by(|a, b| {
            (&a.line_table, &a.captured_price_field).cmp(&(&b.line_table, &b.captured_price_field))
        });
        definition
            .policies
            .sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
        definition
            .submit_rules
            .sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
        let spec = Self { definition };
        let mut captured = BTreeSet::new();
        let capture_targets: BTreeSet<_> = spec
            .definition
            .capture_rules
            .iter()
            .map(|r| (&r.line_table, &r.captured_price_field))
            .collect();
        for rule in &spec.definition.capture_rules {
            if !captured.insert((&rule.line_table, &rule.captured_price_field)) {
                return Err(Error::InvalidCapture {
                    reason: format!(
                        "duplicate capture {}.{}",
                        rule.line_table, rule.captured_price_field
                    ),
                });
            }
            let reference = spec
                .field(&rule.line_table, &rule.product_ref_field)
                .ok_or_else(|| Error::UnknownField {
                    table: rule.line_table.clone(),
                    field: rule.product_ref_field.clone(),
                })?;
            let FieldType::Ref { target_table } = &reference.field_type else {
                return Err(Error::InvalidCapture {
                    reason: "capture source must be a reference".to_owned(),
                });
            };
            if capture_targets.contains(&(target_table, &rule.product_price_field)) {
                return Err(Error::InvalidCapture {
                    reason: "capture sources cannot be captured fields".to_owned(),
                });
            }
            let source = spec
                .field(target_table, &rule.product_price_field)
                .ok_or_else(|| Error::UnknownField {
                    table: target_table.clone(),
                    field: rule.product_price_field.clone(),
                })?;
            let target = spec
                .field(&rule.line_table, &rule.captured_price_field)
                .ok_or_else(|| Error::UnknownField {
                    table: rule.line_table.clone(),
                    field: rule.captured_price_field.clone(),
                })?;
            if source.field_type != FieldType::Money || target.field_type != FieldType::Money {
                return Err(Error::InvalidCapture {
                    reason: "capture source and destination must be money".to_owned(),
                });
            }
        }
        // Bound the complete definition before cloning it into checked policy rules.
        crate::codec::bounded_encode(&spec.definition)?;
        if spec.definition.policies.len() > 64 || spec.definition.submit_rules.len() > 64 {
            return Err(Error::LimitExceeded {
                resource: "policy/command rules",
                limit: 64,
            });
        }
        let mut rule_ids = BTreeSet::new();
        let mut policy_tables = BTreeSet::new();
        let mut policies = Vec::new();
        for rule in &spec.definition.policies {
            if !rule_ids.insert(rule.rule_id.clone())
                || !policy_tables.insert(rule.table_id.clone())
            {
                return Err(Error::InvalidPolicy {
                    rule_id: rule.rule_id.clone(),
                });
            }
            policies.push(
                crate::policy::OwnerRolePolicy::check(&spec, rule.clone()).map_err(|_| {
                    Error::InvalidPolicy {
                        rule_id: rule.rule_id.clone(),
                    }
                })?,
            );
        }
        for rule in &spec.definition.submit_rules {
            if !rule_ids.insert(rule.rule_id.clone()) {
                return Err(Error::InvalidPolicy {
                    rule_id: rule.rule_id.clone(),
                });
            }
            let policy = policies
                .iter()
                .find(|policy| policy.rule_id() == rule.policy_rule_id)
                .ok_or_else(|| Error::InvalidPolicy {
                    rule_id: rule.rule_id.clone(),
                })?;
            crate::policy::SubmitOrderRule::check(&spec, policy, rule.clone()).map_err(|_| {
                Error::InvalidPolicy {
                    rule_id: rule.rule_id.clone(),
                }
            })?;
        }
        Ok(spec)
    }
    pub fn definition(&self) -> &RawAppSpec {
        &self.definition
    }
    pub fn app_id(&self) -> &AppId {
        &self.definition.app_id
    }
    pub fn tables(&self) -> &[RawTable] {
        &self.definition.tables
    }
    pub fn table(&self, id: &TableId) -> Option<&RawTable> {
        self.definition.tables.iter().find(|table| &table.id == id)
    }
    pub fn field(&self, table: &TableId, id: &FieldId) -> Option<&RawField> {
        self.table(table)?
            .fields
            .iter()
            .find(|field| &field.id == id)
    }
}
