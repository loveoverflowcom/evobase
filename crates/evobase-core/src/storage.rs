use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    AuthContext, QualifiedTable, TableDelete, TableDoc, TableInsert, TableSelect, TableUpdate,
    error::AppResult,
};

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

#[async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn create_user(&self, username: &str, password_hash: &str) -> AppResult<UserRecord>;
    async fn find_user_by_username(&self, username: &str) -> AppResult<Option<UserRecord>>;
    async fn find_user_by_id(&self, user_id: Uuid) -> AppResult<Option<UserRecord>>;
    async fn describe_tables(&self, table: Option<QualifiedTable>) -> AppResult<Vec<TableDoc>>;
    async fn select_rows(
        &self,
        request: TableSelect,
        auth: Option<&AuthContext>,
    ) -> AppResult<Vec<serde_json::Value>>;
    async fn insert_rows(
        &self,
        request: TableInsert,
        auth: Option<&AuthContext>,
    ) -> AppResult<Vec<serde_json::Value>>;
    async fn update_rows(
        &self,
        request: TableUpdate,
        auth: Option<&AuthContext>,
    ) -> AppResult<Vec<serde_json::Value>>;
    async fn delete_rows(
        &self,
        request: TableDelete,
        auth: Option<&AuthContext>,
    ) -> AppResult<Vec<serde_json::Value>>;
}
