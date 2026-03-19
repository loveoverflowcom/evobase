use serde_json::{Map, Value};

use crate::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct QualifiedTable {
    pub schema: Option<String>,
    pub table: String,
}

#[derive(Debug, Clone)]
pub enum SelectList {
    All,
    Columns(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum FilterOperator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    ILike,
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub column: String,
    pub operator: FilterOperator,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct OrderBy {
    pub column: String,
    pub descending: bool,
}

#[derive(Debug, Clone)]
pub struct TableSelect {
    pub table: QualifiedTable,
    pub select: SelectList,
    pub filters: Vec<Filter>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct TableInsert {
    pub table: QualifiedTable,
    pub rows: Vec<Map<String, Value>>,
}

#[derive(Debug, Clone)]
pub struct TableUpdate {
    pub table: QualifiedTable,
    pub patch: Map<String, Value>,
    pub filters: Vec<Filter>,
}

#[derive(Debug, Clone)]
pub struct TableDelete {
    pub table: QualifiedTable,
    pub filters: Vec<Filter>,
}

impl QualifiedTable {
    pub fn parse(input: &str) -> AppResult<Self> {
        let parts: Vec<_> = input.split('.').collect();

        match parts.as_slice() {
            [table] => {
                validate_identifier(table)?;
                Ok(Self {
                    schema: None,
                    table: (*table).to_string(),
                })
            }
            [schema, table] => {
                validate_identifier(schema)?;
                validate_identifier(table)?;
                Ok(Self {
                    schema: Some((*schema).to_string()),
                    table: (*table).to_string(),
                })
            }
            _ => Err(AppError::BadRequest(format!(
                "invalid table path `{input}`; expected `table` or `schema.table`"
            ))),
        }
    }

    pub fn quoted(&self) -> String {
        match &self.schema {
            Some(schema) => format!(
                "{}.{}",
                quoted_identifier(schema),
                quoted_identifier(&self.table)
            ),
            None => quoted_identifier(&self.table),
        }
    }
}

impl FilterOperator {
    pub fn parse(value: &str) -> AppResult<Self> {
        match value {
            "eq" => Ok(Self::Eq),
            "neq" => Ok(Self::Neq),
            "gt" => Ok(Self::Gt),
            "gte" => Ok(Self::Gte),
            "lt" => Ok(Self::Lt),
            "lte" => Ok(Self::Lte),
            "like" => Ok(Self::Like),
            "ilike" => Ok(Self::ILike),
            other => Err(AppError::BadRequest(format!(
                "unsupported filter operator `{other}`"
            ))),
        }
    }

    pub fn sql_operator(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Neq => "!=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::Like => "LIKE",
            Self::ILike => "ILIKE",
        }
    }
}

pub fn validate_identifier(identifier: &str) -> AppResult<()> {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return Err(AppError::BadRequest(
            "identifier cannot be empty".to_string(),
        ));
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(AppError::BadRequest(format!(
            "identifier `{identifier}` must start with a letter or underscore"
        )));
    }

    if chars.any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_')) {
        return Err(AppError::BadRequest(format!(
            "identifier `{identifier}` contains unsupported characters"
        )));
    }

    Ok(())
}

pub fn quoted_identifier(identifier: &str) -> String {
    format!("\"{identifier}\"")
}
