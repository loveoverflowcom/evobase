use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ApiDocs {
    pub tables: Vec<TableDoc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableDoc {
    pub name: String,
    pub schema: String,
    pub table: String,
    pub endpoint: String,
    pub columns: Vec<ColumnDoc>,
    pub methods: TableMethods,
    pub rls: RlsDoc,
    pub query: QueryDoc,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnDoc {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub nullable: bool,
    pub has_default: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TableMethods {
    #[serde(rename = "GET")]
    pub get: bool,
    #[serde(rename = "POST")]
    pub post: bool,
    #[serde(rename = "PATCH")]
    pub patch: bool,
    #[serde(rename = "DELETE")]
    pub delete: bool,
}

impl TableMethods {
    pub fn any(&self) -> bool {
        self.get || self.post || self.patch || self.delete
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RlsDoc {
    pub enabled: bool,
    pub policies: Vec<RlsPolicyDoc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RlsPolicyDoc {
    pub name: String,
    pub command: String,
    #[serde(rename = "using", skip_serializing_if = "Option::is_none")]
    pub r#using: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_check: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryDoc {
    pub selectable_columns: Vec<String>,
    pub filter_operators: Vec<&'static str>,
    pub order_supported: bool,
    pub limit_supported: bool,
    pub offset_supported: bool,
}

impl QueryDoc {
    pub fn from_columns(columns: &[ColumnDoc]) -> Self {
        Self {
            selectable_columns: columns.iter().map(|column| column.name.clone()).collect(),
            filter_operators: vec!["eq", "neq", "gt", "gte", "lt", "lte", "like", "ilike"],
            order_supported: true,
            limit_supported: true,
            offset_supported: true,
        }
    }
}
