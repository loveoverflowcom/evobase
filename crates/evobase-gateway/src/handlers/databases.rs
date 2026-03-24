use axum::{
    Json,
    extract::{Path, State},
};
use evobase_core::{AppError, QualifiedTable};
use evobase_protocol::{
    ApiResponse, BootstrapDatabaseRequestDto, DatabaseCatalogDto, DatabaseDto, TableDocDto,
};

use crate::{ApiResult, AppState};

pub async fn list_databases(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<DatabaseCatalogDto>>> {
    let databases = state.database_manager.list_databases().await;
    let dto =
        DatabaseCatalogDto::from_core(state.database_manager.default_database_id(), databases);
    Ok(Json(ApiResponse::new(dto)))
}

pub async fn get_database(
    State(state): State<AppState>,
    Path(database_id): Path<String>,
) -> ApiResult<Json<ApiResponse<DatabaseDto>>> {
    let database = state.database_manager.get_database(&database_id).await?;
    let dto = DatabaseDto::from_core(state.database_manager.default_database_id(), database);
    Ok(Json(ApiResponse::new(dto)))
}

pub async fn bootstrap_database(
    State(state): State<AppState>,
    Json(request): Json<BootstrapDatabaseRequestDto>,
) -> ApiResult<Json<ApiResponse<DatabaseDto>>> {
    let database = state
        .database_manager
        .bootstrap_database(request.into())
        .await?;
    let dto = DatabaseDto::from_core(state.database_manager.default_database_id(), database);
    Ok(Json(ApiResponse::new(dto)))
}

pub async fn list_database_docs(
    State(state): State<AppState>,
    Path(database_id): Path<String>,
) -> ApiResult<Json<ApiResponse<evobase_protocol::ApiDocsDto>>> {
    let (database, tables) = state
        .database_manager
        .describe_tables(Some(&database_id), None)
        .await?;

    let dto = evobase_protocol::ApiDocsDto::new(
        DatabaseDto::from_core(state.database_manager.default_database_id(), database),
        tables.into_iter().map(TableDocDto::from).collect(),
    );
    Ok(Json(ApiResponse::new(dto)))
}

pub async fn get_database_table_docs(
    State(state): State<AppState>,
    Path((database_id, table)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<TableDocDto>>> {
    let table = QualifiedTable::parse(&table)?;
    let schema_was_specified = table.schema.is_some();
    let (_, mut tables) = state
        .database_manager
        .describe_tables(Some(&database_id), Some(table))
        .await?;

    let table_doc = resolve_single_table_doc(&mut tables, schema_was_specified)?;
    Ok(Json(ApiResponse::new(TableDocDto::from(table_doc))))
}

fn resolve_single_table_doc(
    tables: &mut Vec<evobase_core::TableDoc>,
    schema_was_specified: bool,
) -> Result<evobase_core::TableDoc, AppError> {
    match tables.len() {
        0 => Err(AppError::NotFound("table docs not found".to_string())),
        1 => Ok(tables.pop().expect("single table doc must exist")),
        _ if !schema_was_specified => Err(AppError::BadRequest(
            "table name is ambiguous; use schema.table".to_string(),
        )),
        _ => Err(AppError::Internal(
            "multiple table docs matched a schema-qualified name".to_string(),
        )),
    }
}
