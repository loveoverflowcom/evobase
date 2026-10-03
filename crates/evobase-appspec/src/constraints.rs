//! Portable field and command-input refinements. Predicates never normalize persisted text.
use crate::{ConstraintId, Error, FieldId, FieldType, MAX_TEXT_BYTES, TableId, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Constraint {
    /// At least one non-whitespace Unicode scalar. This does not trim the stored value.
    NonEmpty,
    /// Inclusive bounds measured in Unicode scalar values, not bytes or grapheme clusters.
    TextLength { min: usize, max: usize },
    /// Inclusive signed exact bounds; applies to Integer or Money without conversion.
    NumericRange { min: i64, max: i64 },
}

impl Constraint {
    pub(crate) fn check_type(&self, field_type: &FieldType) -> Result<(), &'static str> {
        match (self, field_type) {
            (Self::NonEmpty, FieldType::Text) => Ok(()),
            (Self::TextLength { min, max }, FieldType::Text)
                if min <= max && *max <= MAX_TEXT_BYTES =>
            {
                Ok(())
            }
            (Self::NumericRange { min, max }, FieldType::Integer | FieldType::Money)
                if min <= max =>
            {
                Ok(())
            }
            (Self::TextLength { .. }, FieldType::Text)
            | (Self::NumericRange { .. }, FieldType::Integer | FieldType::Money) => {
                Err("invalid inclusive bounds")
            }
            _ => Err("constraint does not support the field type"),
        }
    }

    pub(crate) fn accepts(&self, value: &Value) -> bool {
        match (self, value) {
            // Requiredness has one owner; optional missing values do not become required here.
            (_, Value::Blank | Value::Null) => true,
            (Self::NonEmpty, Value::Text(text)) => !text.trim().is_empty(),
            (Self::TextLength { min, max }, Value::Text(text)) => {
                let length = text.chars().count();
                (*min..=*max).contains(&length)
            }
            (Self::NumericRange { min, max }, Value::Integer(value) | Value::Money(value)) => {
                (*min..=*max).contains(value)
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFieldConstraint {
    pub constraint_id: ConstraintId,
    pub table_id: TableId,
    pub field_id: FieldId,
    pub constraint: Constraint,
}

pub(crate) fn check_constraints(spec: &crate::CheckedAppSpec) -> Result<(), Error> {
    let constraints = &spec.definition().constraints;
    if constraints.len() > crate::MAX_CONSTRAINTS {
        return Err(Error::LimitExceeded {
            resource: "constraints",
            limit: crate::MAX_CONSTRAINTS,
        });
    }
    let mut identities = std::collections::BTreeSet::new();
    for rule in constraints {
        if !identities.insert(&rule.constraint_id) {
            return Err(Error::InvalidConstraint {
                constraint: rule.constraint_id.clone(),
                reason: "duplicate constraint identity",
            });
        }
        let field =
            spec.field(&rule.table_id, &rule.field_id)
                .ok_or_else(|| Error::UnknownField {
                    table: rule.table_id.clone(),
                    field: rule.field_id.clone(),
                })?;
        rule.constraint
            .check_type(&field.field_type)
            .map_err(|reason| Error::InvalidConstraint {
                constraint: rule.constraint_id.clone(),
                reason,
            })?;
    }
    Ok(())
}
