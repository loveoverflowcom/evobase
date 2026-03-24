use serde::Serialize;

use crate::DatabaseDto;

#[derive(Debug, Clone, Serialize)]
pub struct ApiDocsDto {
    pub database: DatabaseDto,
    pub tables: Vec<TableDocDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableDocDto {
    pub name: String,
    pub schema: String,
    pub table: String,
    pub endpoint: String,
    pub columns: Vec<ColumnDocDto>,
    pub methods: TableMethodsDto,
    pub rls: RlsDocDto,
    pub query: QueryDocDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnDocDto {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub nullable: bool,
    pub has_default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RlsDocDto {
    pub enabled: bool,
    pub policies: Vec<RlsPolicyDocDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RlsPolicyDocDto {
    pub name: String,
    pub command: String,
    #[serde(rename = "using", skip_serializing_if = "Option::is_none")]
    pub r#using: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_check: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableMethodsDto {
    #[serde(rename = "GET")]
    pub get: bool,
    #[serde(rename = "POST")]
    pub post: bool,
    #[serde(rename = "PATCH")]
    pub patch: bool,
    #[serde(rename = "DELETE")]
    pub delete: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryDocDto {
    pub selectable_columns: Vec<String>,
    pub filter_operators: Vec<&'static str>,
    pub order_supported: bool,
    pub limit_supported: bool,
    pub offset_supported: bool,
}

impl ApiDocsDto {
    pub fn new(database: DatabaseDto, tables: Vec<TableDocDto>) -> Self {
        Self { database, tables }
    }
}

impl From<evobase_core::TableDoc> for TableDocDto {
    fn from(doc: evobase_core::TableDoc) -> Self {
        Self {
            name: doc.name,
            schema: doc.schema,
            table: doc.table,
            endpoint: doc.endpoint,
            columns: doc.columns.into_iter().map(Into::into).collect(),
            methods: doc.methods.into(),
            rls: doc.rls.into(),
            query: doc.query.into(),
        }
    }
}

impl From<evobase_core::ColumnDoc> for ColumnDocDto {
    fn from(col: evobase_core::ColumnDoc) -> Self {
        Self {
            name: col.name,
            data_type: col.data_type,
            nullable: col.nullable,
            has_default: col.has_default,
        }
    }
}

impl From<evobase_core::RlsDoc> for RlsDocDto {
    fn from(rls: evobase_core::RlsDoc) -> Self {
        Self {
            enabled: rls.enabled,
            policies: rls.policies.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<evobase_core::RlsPolicyDoc> for RlsPolicyDocDto {
    fn from(policy: evobase_core::RlsPolicyDoc) -> Self {
        Self {
            name: policy.name,
            command: policy.command,
            r#using: policy.r#using,
            with_check: policy.with_check,
        }
    }
}

impl From<evobase_core::TableMethods> for TableMethodsDto {
    fn from(methods: evobase_core::TableMethods) -> Self {
        Self {
            get: methods.get,
            post: methods.post,
            patch: methods.patch,
            delete: methods.delete,
        }
    }
}

impl From<evobase_core::QueryDoc> for QueryDocDto {
    fn from(query: evobase_core::QueryDoc) -> Self {
        Self {
            selectable_columns: query.selectable_columns,
            filter_operators: query.filter_operators,
            order_supported: query.order_supported,
            limit_supported: query.limit_supported,
            offset_supported: query.offset_supported,
        }
    }
}
