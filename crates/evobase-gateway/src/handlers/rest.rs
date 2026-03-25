use axum::{
    Extension, Json,
    extract::{Path, RawQuery, State},
};
use evobase_core::{
    AppError, AppResult, AuthContext, Filter, FilterOperator, QualifiedTable, SelectList,
    TableDetailFields, TableSelect, validate_identifier,
};
use evobase_protocol::ApiResponse;
use serde_json::Value;

use crate::{
    ApiResult, AppState,
    rest::{
        parse_delete_request, parse_insert_request, parse_select_request, parse_update_request,
    },
};

pub async fn select_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<ApiResponse<Vec<Value>>>> {
    let request = parse_select_request(&table, raw_query.as_deref())?;
    let rows = state
        .storage
        .select_rows(request, Some(&current_user))
        .await?;

    Ok(Json(ApiResponse::new(rows)))
}

pub async fn insert_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    Json(body): Json<Value>,
) -> ApiResult<Json<ApiResponse<Vec<Value>>>> {
    let request = parse_insert_request(&table, body)?;
    let rows = state
        .storage
        .insert_rows(request, Some(&current_user))
        .await?;

    Ok(Json(ApiResponse::new(rows)))
}

pub async fn update_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    RawQuery(raw_query): RawQuery,
    Json(body): Json<Value>,
) -> ApiResult<Json<ApiResponse<Vec<Value>>>> {
    let request = parse_update_request(&table, raw_query.as_deref(), body)?;
    let rows = state
        .storage
        .update_rows(request, Some(&current_user))
        .await?;

    Ok(Json(ApiResponse::new(rows)))
}

pub async fn delete_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<ApiResponse<Vec<Value>>>> {
    let request = parse_delete_request(&table, raw_query.as_deref())?;
    let rows = state
        .storage
        .delete_rows(request, Some(&current_user))
        .await?;

    Ok(Json(ApiResponse::new(rows)))
}

pub async fn select_row_by_primary_key(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path((table, value)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let table = QualifiedTable::parse(&table)?;
    let detail_fields = resolve_detail_fields(&state, &table).await?;
    let field = resolve_primary_key_field(&table, &detail_fields)?;
    let row = get_by_field(&state, &current_user, table.clone(), &field, value)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("record not found in `{}`", table_name(&table))))?;

    Ok(Json(ApiResponse::new(row)))
}

pub async fn select_row_by_unique_field(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path((table, lookup, value)): Path<(String, String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let table = QualifiedTable::parse(&table)?;
    let field = parse_lookup_field(&lookup)?;
    let detail_fields = resolve_detail_fields(&state, &table).await?;

    if !detail_fields.unique_fields.iter().any(|candidate| candidate == &field) {
        return Err(AppError::BadRequest(format!(
            "field `{field}` is not a unique field on table `{}`",
            table_name(&table)
        ))
        .into());
    }

    let row = get_by_field(&state, &current_user, table.clone(), &field, value)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("record not found in `{}`", table_name(&table))))?;

    Ok(Json(ApiResponse::new(row)))
}

fn parse_lookup_field(lookup: &str) -> AppResult<String> {
    let field = lookup
        .strip_prefix("by-")
        .ok_or_else(|| AppError::NotFound("detail endpoint not found".to_string()))?;
    validate_identifier(field)?;
    Ok(field.to_string())
}

fn resolve_primary_key_field(
    table: &QualifiedTable,
    detail_fields: &TableDetailFields,
) -> AppResult<String> {
    match detail_fields.primary_key.as_slice() {
        [field] => Ok(field.clone()),
        [] => Err(AppError::BadRequest(format!(
            "table `{}` has no primary key detail endpoint",
            table_name(table)
        ))),
        _ => Err(AppError::BadRequest(format!(
            "table `{}` has a composite primary key; use unique by-field detail endpoints",
            table_name(table)
        ))),
    }
}

async fn resolve_detail_fields(
    state: &AppState,
    table: &QualifiedTable,
) -> AppResult<TableDetailFields> {
    let schema_was_specified = table.schema.is_some();
    let mut detail_fields = state.storage.describe_detail_fields(table.clone()).await?;

    match detail_fields.len() {
        0 => Err(AppError::NotFound(format!(
            "table `{}` was not found",
            table_name(table)
        ))),
        1 => Ok(detail_fields.pop().expect("single detail metadata must exist")),
        _ if !schema_was_specified => Err(AppError::BadRequest(
            "table name is ambiguous; use schema.table".to_string(),
        )),
        _ => Err(AppError::Internal(
            "multiple table metadata entries matched a schema-qualified name".to_string(),
        )),
    }
}

async fn get_by_field(
    state: &AppState,
    current_user: &AuthContext,
    table: QualifiedTable,
    field: &str,
    value: String,
) -> AppResult<Option<Value>> {
    let request = TableSelect {
        table,
        select: SelectList::All,
        filters: vec![Filter {
            column: field.to_string(),
            operator: FilterOperator::Eq,
            value,
        }],
        order_by: Vec::new(),
        limit: Some(1),
        offset: None,
    };

    let mut rows = state.storage.select_rows(request, Some(current_user)).await?;
    Ok(rows.pop())
}

fn table_name(table: &QualifiedTable) -> String {
    match &table.schema {
        Some(schema) => format!("{schema}.{}", table.table),
        None => table.table.clone(),
    }
}
