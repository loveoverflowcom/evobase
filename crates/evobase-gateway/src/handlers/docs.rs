use axum::{Json, extract::{Path, State}};
use evobase_core::{AppError, QualifiedTable};
use evobase_protocol::{ApiDocsDto, ApiResponse, TableDocDto};

use crate::{ApiResult, AppState};

pub async fn list_docs(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<ApiDocsDto>>> {
    let tables = state.storage.describe_tables(None).await?;
    let dto = ApiDocsDto::from(evobase_core::ApiDocs { tables });
    Ok(Json(ApiResponse::new(dto)))
}

pub async fn get_table_docs(
    State(state): State<AppState>,
    Path(table): Path<String>,
) -> ApiResult<Json<ApiResponse<TableDocDto>>> {
    let table = QualifiedTable::parse(&table)?;
    let schema_was_specified = table.schema.is_some();
    let mut tables = state.storage.describe_tables(Some(table)).await?;

    let table_doc = match tables.len() {
        0 => Err(AppError::NotFound("table docs not found".to_string())),
        1 => Ok(tables.pop().expect("single table doc must exist")),
        _ if !schema_was_specified => Err(AppError::BadRequest(
            "table name is ambiguous; use schema.table".to_string(),
        )),
        _ => Err(AppError::Internal(
            "multiple table docs matched a schema-qualified name".to_string(),
        )),
    }?;

    let dto = TableDocDto::from(table_doc);
    Ok(Json(ApiResponse::new(dto)))
}
