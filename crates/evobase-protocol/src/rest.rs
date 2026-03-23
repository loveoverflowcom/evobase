use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TableQueryParams {
    pub select: Option<String>,
    pub order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InsertBody {
    Single(serde_json::Map<String, serde_json::Value>),
    Multiple(Vec<serde_json::Map<String, serde_json::Value>>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchBody {
    #[serde(flatten)]
    pub fields: serde_json::Map<String, serde_json::Value>,
}
