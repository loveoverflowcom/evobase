use eonbase_core::{
    AppError, AppResult, Filter, FilterOperator, OrderBy, QualifiedTable, SelectList, TableDelete,
    TableInsert, TableSelect, TableUpdate, validate_identifier,
};
use serde_json::Value;
use url::form_urlencoded;

pub fn parse_select_request(table: &str, raw_query: Option<&str>) -> AppResult<TableSelect> {
    let table = QualifiedTable::parse(table)?;
    let mut select = SelectList::All;
    let mut filters = Vec::new();
    let mut order_by = Vec::new();
    let mut limit = None;
    let mut offset = None;

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "select" => {
                    select = parse_select_list(value.as_ref())?;
                }
                "order" => {
                    order_by = parse_order_by(value.as_ref())?;
                }
                "limit" => {
                    limit = Some(parse_non_negative(value.as_ref(), "limit")?);
                }
                "offset" => {
                    offset = Some(parse_non_negative(value.as_ref(), "offset")?);
                }
                column => {
                    validate_identifier(column)?;
                    filters.push(parse_filter(column, value.as_ref())?);
                }
            }
        }
    }

    Ok(TableSelect {
        table,
        select,
        filters,
        order_by,
        limit,
        offset,
    })
}

pub fn parse_insert_request(table: &str, body: Value) -> AppResult<TableInsert> {
    let table = QualifiedTable::parse(table)?;
    let rows = match body {
        Value::Object(object) => vec![object],
        Value::Array(values) => values
            .into_iter()
            .map(|value| match value {
                Value::Object(object) => Ok(object),
                _ => Err(AppError::BadRequest(
                    "insert payload arrays must contain objects".to_string(),
                )),
            })
            .collect::<AppResult<Vec<_>>>()?,
        _ => {
            return Err(AppError::BadRequest(
                "insert payload must be an object or array of objects".to_string(),
            ));
        }
    };

    Ok(TableInsert { table, rows })
}

pub fn parse_update_request(
    table: &str,
    raw_query: Option<&str>,
    body: Value,
) -> AppResult<TableUpdate> {
    let table = QualifiedTable::parse(table)?;
    let filters = parse_filters(raw_query)?;

    if filters.is_empty() {
        return Err(AppError::BadRequest(
            "update requests require at least one filter".to_string(),
        ));
    }

    let Value::Object(patch) = body else {
        return Err(AppError::BadRequest(
            "update payload must be a JSON object".to_string(),
        ));
    };

    Ok(TableUpdate {
        table,
        patch,
        filters,
    })
}

pub fn parse_delete_request(table: &str, raw_query: Option<&str>) -> AppResult<TableDelete> {
    let table = QualifiedTable::parse(table)?;
    let filters = parse_filters(raw_query)?;

    if filters.is_empty() {
        return Err(AppError::BadRequest(
            "delete requests require at least one filter".to_string(),
        ));
    }

    Ok(TableDelete { table, filters })
}

fn parse_filters(raw_query: Option<&str>) -> AppResult<Vec<Filter>> {
    let mut filters = Vec::new();

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "select" | "order" | "limit" | "offset" => {}
                column => {
                    validate_identifier(column)?;
                    filters.push(parse_filter(column, value.as_ref())?);
                }
            }
        }
    }

    Ok(filters)
}

fn parse_select_list(value: &str) -> AppResult<SelectList> {
    if value == "*" {
        return Ok(SelectList::All);
    }

    let columns: Vec<String> = value
        .split(',')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .map(|column| {
            validate_identifier(column)?;
            Ok(column.to_string())
        })
        .collect::<AppResult<Vec<_>>>()?;

    if columns.is_empty() {
        return Err(AppError::BadRequest(
            "select must contain at least one column".to_string(),
        ));
    }

    Ok(SelectList::Columns(columns))
}

fn parse_order_by(value: &str) -> AppResult<Vec<OrderBy>> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (column, direction) = part.split_once('.').unwrap_or((part, "asc"));
            validate_identifier(column)?;

            let descending = match direction {
                "asc" => false,
                "desc" => true,
                other => {
                    return Err(AppError::BadRequest(format!(
                        "unsupported order direction `{other}`"
                    )));
                }
            };

            Ok(OrderBy {
                column: column.to_string(),
                descending,
            })
        })
        .collect()
}

fn parse_filter(column: &str, raw_value: &str) -> AppResult<Filter> {
    let (operator, value) = match raw_value.split_once('.') {
        Some((operator, value)) => (FilterOperator::parse(operator)?, value.to_string()),
        None => (FilterOperator::Eq, raw_value.to_string()),
    };

    if value.is_empty() {
        return Err(AppError::BadRequest(format!(
            "filter for column `{column}` is missing a value"
        )));
    }

    Ok(Filter {
        column: column.to_string(),
        operator,
        value,
    })
}

fn parse_non_negative(value: &str, field: &str) -> AppResult<i64> {
    let parsed = value
        .parse::<i64>()
        .map_err(|error| AppError::BadRequest(format!("invalid {field}: {error}")))?;

    if parsed < 0 {
        return Err(AppError::BadRequest(format!(
            "{field} must be non-negative"
        )));
    }

    Ok(parsed)
}
