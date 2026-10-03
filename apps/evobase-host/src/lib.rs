//! Generic single-tenant HTTP adapter. Request selectors never select database credentials.
pub mod access;

use access::PilotVerifier;
use axum::{
    Router,
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, Path, RawQuery, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use evobase_appspec::{
    Scope, TableId,
    policy::{
        AuthorityError, Grant, HostAuthority, OwnerRolePolicy, ProjectionPurpose, RequestChannel,
        project,
    },
};
use evobase_protocol::appspec::{
    API_VERSION, ApiErrorDto, CommandMetadataDto, CommandRequestDto, ErrorCodeDto,
    FieldMetadataDto, ListResponseDto, MAX_LIST_LIMIT, MetadataResponseDto, ReceiptResponseDto,
    RecordDto, RevisionDto, TableMetadataDto, ValueDto,
};
use evobase_store::{Snapshot, Store, StoreError};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_REQUEST_BYTES: usize = 262_144;
pub const MAX_RESPONSE_BYTES: usize = 1_048_576;
const MAX_HEADERS: usize = 32;
const MAX_HEADER_BYTES: usize = 8_192;
const MAX_URI_BYTES: usize = 512;

#[derive(Clone)]
pub struct HostState {
    store: Arc<Store>,
    scope: Scope,
    verifier: PilotVerifier,
    release_id: String,
}
impl HostState {
    /// Pin the already checked persistent release at startup. HTTP cannot repin or bootstrap it.
    pub async fn new(
        store: Arc<Store>,
        scope: Scope,
        verifier: PilotVerifier,
    ) -> Result<Self, StoreError> {
        let snapshot = store.snapshot(&scope).await?;
        Ok(Self {
            store,
            scope,
            verifier,
            release_id: snapshot.spec_identity().to_owned(),
        })
    }
    fn authority(&self) -> HostAuthority<PilotVerifier> {
        HostAuthority::new(self.verifier.clone())
    }
    fn authorize(
        &self,
        app: &str,
        headers: &HeaderMap,
        grant: Grant,
        mutation: bool,
    ) -> Result<(String, RequestChannel, u64), ApiFailure> {
        let token = credential(headers)?;
        let origin = headers
            .get(header::ORIGIN)
            .map(|h| h.to_str().map(str::to_owned))
            .transpose()
            .map_err(|_| ApiFailure::denied())?;
        if origin
            .as_ref()
            .is_some_and(|origin| !self.verifier.allows_origin(origin))
        {
            return Err(ApiFailure::denied());
        }
        let channel = if mutation && origin.is_some() {
            RequestChannel::BrowserMutation { origin }
        } else {
            RequestChannel::NativeBearer
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ApiFailure::internal())?
            .as_secs();
        self.authority()
            .resolve(&token, &self.scope, &channel, now, grant)
            .map_err(|_| ApiFailure::denied())?;
        if app != self.scope.app_id().as_str() {
            return Err(ApiFailure::denied());
        }
        Ok((token, channel, now))
    }
    async fn snapshot(&self) -> Result<Snapshot, ApiFailure> {
        let snapshot = self
            .store
            .snapshot(&self.scope)
            .await
            .map_err(ApiFailure::storage)?;
        if snapshot.spec_identity() != self.release_id {
            return Err(ApiFailure::conflict());
        }
        Ok(snapshot)
    }
}

pub fn router(state: HostState) -> Router {
    Router::new()
        .route("/v1/apps/{app}/metadata", get(design_metadata))
        .route("/v1/apps/{app}/runtime-metadata", get(runtime_metadata))
        .route("/v1/apps/{app}/tables/{table}", get(list))
        .route("/v1/apps/{app}/commands", post(command))
        .route("/v1/apps/{app}/receipts/{key}", get(receipt))
        .fallback(|| async { ApiFailure::unsupported() })
        .method_not_allowed_fallback(|| async { ApiFailure::unsupported() })
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            transport_budget,
        ))
        .with_state(state)
}

async fn transport_budget(
    State(state): State<HostState>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    let count = headers.len();
    let bytes = headers
        .iter()
        .map(|(name, value)| name.as_str().len().saturating_add(value.as_bytes().len()))
        .sum::<usize>();
    if count > MAX_HEADERS
        || bytes > MAX_HEADER_BYTES
        || request.uri().to_string().len() > MAX_URI_BYTES
    {
        return ApiFailure::validation().into_response();
    }
    if headers.get_all(header::ORIGIN).iter().count() > 1
        || headers.get_all(header::CONTENT_TYPE).iter().count() > 1
    {
        return ApiFailure::validation().into_response();
    }
    let origin = headers.get(header::ORIGIN).cloned();
    if origin.as_ref().is_some_and(|value| {
        value
            .to_str()
            .ok()
            .is_none_or(|value| !state.verifier.allows_origin(value))
    }) {
        return ApiFailure::denied().into_response();
    }
    if request.method() == Method::OPTIONS {
        // CORS advertises the bearer-only contract; it does not authenticate an operation.
        if origin.is_none() {
            return ApiFailure::denied().into_response();
        }
        let mut response = StatusCode::NO_CONTENT.into_response();
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("authorization, content-type"),
        );
        add_response_headers(&mut response, origin);
        return response;
    }
    let mut response = next.run(request).await;
    add_response_headers(&mut response, origin);
    response
}
fn add_response_headers(response: &mut Response, origin: Option<HeaderValue>) {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    if let Some(origin) = origin {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Origin"));
    }
}
fn credential(headers: &HeaderMap) -> Result<String, ApiFailure> {
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let value = values.next().ok_or_else(ApiFailure::denied)?;
    if values.next().is_some() {
        return Err(ApiFailure::denied());
    }
    let value = value.to_str().map_err(|_| ApiFailure::denied())?;
    let token = value
        .strip_prefix("Bearer ")
        .ok_or_else(ApiFailure::denied)?;
    if token.len() != 64 {
        return Err(ApiFailure::denied());
    }
    Ok(token.to_owned())
}
fn no_query(query: &Option<String>) -> Result<(), ApiFailure> {
    if query.is_some() {
        return Err(ApiFailure::unsupported());
    }
    Ok(())
}
fn field_metadata(field: &evobase_appspec::RawField) -> FieldMetadataDto {
    FieldMetadataDto {
        field_id: field.id.clone(),
        name: field.name.clone(),
        field_type: field.field_type.clone(),
        required: field.required,
    }
}

async fn design_metadata(
    State(state): State<HostState>,
    Path(app): Path<String>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> Result<Response, ApiFailure> {
    state.authorize(&app, &headers, Grant::Design, false)?;
    no_query(&query)?;
    let snapshot = state.snapshot().await?;
    let (token, channel, now) = state.authorize(&app, &headers, Grant::Design, false)?;
    metadata_response(&state, &snapshot, true, &token, &channel, now)
}
async fn runtime_metadata(
    State(state): State<HostState>,
    Path(app): Path<String>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> Result<Response, ApiFailure> {
    state.authorize(&app, &headers, Grant::Read, false)?;
    no_query(&query)?;
    let snapshot = state.snapshot().await?;
    let (token, channel, now) = state.authorize(&app, &headers, Grant::Read, false)?;
    metadata_response(&state, &snapshot, false, &token, &channel, now)
}
fn metadata_response(
    state: &HostState,
    snapshot: &Snapshot,
    design: bool,
    token: &str,
    channel: &RequestChannel,
    now: u64,
) -> Result<Response, ApiFailure> {
    let spec = snapshot.spec();
    let authority = HostAuthority::new(
        state
            .verifier
            .snapshot_for_operation()
            .map_err(|_| ApiFailure::denied())?,
    );
    let grant = if design { Grant::Design } else { Grant::Read };
    let original_context = authority
        .resolve(token, &state.scope, channel, now, grant)
        .map_err(|_| ApiFailure::denied())?;
    let mut tables = Vec::new();
    for table in &spec.definition().tables {
        let policy = spec
            .definition()
            .policies
            .iter()
            .find(|p| p.table_id == table.id);
        if !design {
            let Some(raw_policy) = policy else {
                continue;
            };
            let checked_policy = OwnerRolePolicy::check(spec, raw_policy.clone())
                .map_err(|_| ApiFailure::internal())?;
            let visible = project(
                &authority,
                token,
                &state.scope,
                channel,
                now,
                snapshot.facts(),
                &checked_policy,
                ProjectionPurpose::Query,
            )
            .map_err(|_| ApiFailure::denied())?;
            if visible.rows().is_empty() {
                continue;
            }
        }
        tables.push(TableMetadataDto {
            table_id: table.id.clone(),
            name: table.name.clone(),
            fields: table
                .fields
                .iter()
                .filter(|field| design || policy.is_some_and(|p| p.read_fields.contains(&field.id)))
                .map(field_metadata)
                .collect(),
        });
    }
    let declarations = if design {
        spec.definition()
            .commands
            .iter()
            .cloned()
            .map(|command| (command, vec![]))
            .collect()
    } else {
        match evobase_appspec::commands::available_command_records(
            &authority,
            token,
            &state.scope,
            channel,
            now,
            spec,
            snapshot.facts(),
        ) {
            Ok(commands) => commands,
            Err(evobase_appspec::commands::CommandError::Authority(
                AuthorityError::MissingGrant,
            )) => vec![],
            Err(error) => return Err(ApiFailure::command_error(error)),
        }
    };
    let mut commands: Vec<_> = declarations
        .iter()
        .filter_map(|(command, eligible_record_ids)| {
            let machine = spec
                .definition()
                .state_machines
                .iter()
                .find(|m| m.machine_id == command.state_machine_id)?;
            let table = tables.iter().find(|t| t.table_id == machine.table_id)?;
            let input_fields: Option<Vec<_>> = command
                .inputs
                .iter()
                .map(|input| {
                    table
                        .fields
                        .iter()
                        .find(|f| f.field_id == input.field_id)
                        .cloned()
                        .map(|mut field| {
                            field.required = input.required;
                            field
                        })
                })
                .collect();
            Some(CommandMetadataDto {
                command_id: command.command_id.to_string(),
                name: command.command_id.to_string(),
                table_id: machine.table_id.clone(),
                input_fields: input_fields?,
                eligible_record_ids: eligible_record_ids.clone(),
            })
        })
        .collect();
    let current_context = state
        .authority()
        .resolve(token, &state.scope, channel, now, grant)
        .map_err(|_| ApiFailure::denied())?;
    if current_context != original_context {
        return Err(ApiFailure::denied());
    }
    if !design && !commands.is_empty() {
        match state
            .authority()
            .resolve(token, &state.scope, channel, now, Grant::Write)
        {
            Ok(context) if context == original_context => {}
            Err(AuthorityError::MissingGrant) => commands.clear(),
            _ => return Err(ApiFailure::denied()),
        }
    }
    let response = MetadataResponseDto {
        api_version: API_VERSION,
        app_id: state.scope.app_id().clone(),
        release_id: snapshot.spec_identity().to_owned(),
        revision: RevisionDto::new(snapshot.revision()),
        canonical_appspec: if design {
            Some(
                String::from_utf8(spec.encode().map_err(|_| ApiFailure::internal())?)
                    .map_err(|_| ApiFailure::internal())?,
            )
        } else {
            None
        },
        tables,
        commands,
        supported: vec![
            "owner_role_scalar_read".into(),
            "first_page_limit_1_256".into(),
            "declared_commands".into(),
            "receipt_recovery".into(),
        ],
    };
    json_bytes(response.encode().map_err(|_| ApiFailure::internal())?)
}
async fn list(
    State(state): State<HostState>,
    Path((app, table)): Path<(String, String)>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> Result<Response, ApiFailure> {
    let (token, channel, now) = state.authorize(&app, &headers, Grant::Read, false)?;
    let limit = match query {
        None => 100,
        Some(query) => query
            .strip_prefix("limit=")
            .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|n| (1..=MAX_LIST_LIMIT).contains(n))
            .ok_or_else(ApiFailure::unsupported)?,
    };
    let table = TableId::new(&table).map_err(|_| ApiFailure::validation())?;
    let snapshot = state.snapshot().await?;
    let raw_policy = snapshot
        .spec()
        .definition()
        .policies
        .iter()
        .find(|p| p.table_id == table)
        .ok_or_else(ApiFailure::denied)?;
    let policy = OwnerRolePolicy::check(snapshot.spec(), raw_policy.clone())
        .map_err(|_| ApiFailure::internal())?;
    let authority = HostAuthority::new(
        state
            .verifier
            .snapshot_for_operation()
            .map_err(|_| ApiFailure::denied())?,
    );
    let original_context = authority
        .resolve(&token, &state.scope, &channel, now, Grant::Read)
        .map_err(|_| ApiFailure::denied())?;
    let projection = project(
        &authority,
        &token,
        &state.scope,
        &channel,
        now,
        snapshot.facts(),
        &policy,
        ProjectionPurpose::Query,
    )
    .map_err(|_| ApiFailure::denied())?;
    let records = projection
        .rows()
        .iter()
        .take(limit)
        .map(|row| {
            Ok(RecordDto {
                record_id: row.id().clone(),
                values: row
                    .values()
                    .iter()
                    .map(|(id, value)| {
                        ValueDto::from_domain(value, &state.scope).map(|value| (id.clone(), value))
                    })
                    .collect::<Result<_, _>>()
                    .map_err(|_| ApiFailure::internal())?,
            })
        })
        .collect::<Result<_, ApiFailure>>()?;
    let response = ListResponseDto {
        api_version: API_VERSION,
        app_id: state.scope.app_id().clone(),
        table_id: table,
        release_id: snapshot.spec_identity().to_owned(),
        revision: RevisionDto::new(snapshot.revision()),
        has_more: projection.rows().len() > limit,
        records,
    };
    let current_context = state
        .authority()
        .resolve(&token, &state.scope, &channel, now, Grant::Read)
        .map_err(|_| ApiFailure::denied())?;
    if current_context != original_context {
        return Err(ApiFailure::denied());
    }
    json_bytes(response.encode().map_err(|_| ApiFailure::internal())?)
}

async fn command(
    State(state): State<HostState>,
    Path(app): Path<String>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
    body: Result<Bytes, axum::extract::rejection::BytesRejection>,
) -> Result<Response, ApiFailure> {
    // A command's declared capability is resolved again by the checked core in its transaction.
    let (token, channel, now) = state.authorize(&app, &headers, Grant::Write, true)?;
    no_query(&query)?;
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_none_or(|v| v != "application/json")
    {
        return Err(ApiFailure::validation());
    }
    let body = body.map_err(|_| ApiFailure::validation())?;
    let request = CommandRequestDto::decode(&body).map_err(|_| ApiFailure::validation())?;
    if request.release_id != state.release_id {
        return Err(ApiFailure::conflict());
    }
    let raw = evobase_appspec::commands::RawCommandRequest {
        command_id: evobase_appspec::CommandId::new(&request.command_id)
            .map_err(|_| ApiFailure::validation())?,
        record_id: request.record_id,
        idempotency_key: request.request_key,
        inputs: request
            .params
            .into_iter()
            .map(|(id, value)| (id, value.to_domain(&state.scope)))
            .collect(),
    };
    let committed = state
        .store
        .execute(
            &state.authority(),
            &token,
            &state.scope,
            &channel,
            now,
            request.expected_revision.get(),
            &raw,
        )
        .await
        .map_err(ApiFailure::storage)?;
    receipt_response(&state, &committed)
}
async fn receipt(
    State(state): State<HostState>,
    Path((app, key)): Path<(String, String)>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> Result<Response, ApiFailure> {
    let (token, channel, now) = state.authorize(&app, &headers, Grant::Write, false)?;
    no_query(&query)?;
    if key.is_empty()
        || key.len() > 96
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(ApiFailure::validation());
    }
    let committed = state
        .store
        .receipt(
            &state.authority(),
            &token,
            &state.scope,
            &channel,
            now,
            &key,
        )
        .await
        .map_err(ApiFailure::storage)?
        .ok_or_else(ApiFailure::not_found)?;
    receipt_response(&state, &committed)
}
fn receipt_response(
    state: &HostState,
    committed: &evobase_store::CommitReceipt,
) -> Result<Response, ApiFailure> {
    if committed.spec_identity() != state.release_id {
        return Err(ApiFailure::conflict());
    }
    let transition = committed.transition();
    let response = ReceiptResponseDto {
        api_version: API_VERSION,
        app_id: state.scope.app_id().clone(),
        release_id: committed.spec_identity().to_owned(),
        request_key: committed.request_key().to_owned(),
        command_id: transition.command_id().to_string(),
        record_id: transition.record_id().clone(),
        revision: RevisionDto::new(committed.revision()),
        replayed: committed.replayed(),
    };
    json_bytes(response.encode().map_err(|_| ApiFailure::internal())?)
}
fn json_bytes(bytes: Vec<u8>) -> Result<Response, ApiFailure> {
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(ApiFailure::unsupported());
    }
    let mut response = Body::from(bytes).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    Ok(response)
}

pub struct ApiFailure {
    status: StatusCode,
    code: ErrorCodeDto,
    message: &'static str,
    field: Option<String>,
}
impl ApiFailure {
    fn denied() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: ErrorCodeDto::Denied,
            message: "Access denied",
            field: None,
        }
    }
    fn validation() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: ErrorCodeDto::Validation,
            message: "Invalid request",
            field: None,
        }
    }
    fn unsupported() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: ErrorCodeDto::Unsupported,
            message: "Unsupported request",
            field: None,
        }
    }
    fn conflict() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: ErrorCodeDto::Conflict,
            message: "Revision or intent conflict",
            field: None,
        }
    }
    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: ErrorCodeDto::NotFound,
            message: "Receipt unavailable",
            field: None,
        }
    }
    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: ErrorCodeDto::Internal,
            message: "Operation unavailable",
            field: None,
        }
    }
    fn storage(error: StoreError) -> Self {
        match error {
            StoreError::ScopeMismatch
            | StoreError::Authority(_)
            | StoreError::BindingMismatch
            | StoreError::AuthorityChanged => Self::denied(),
            StoreError::Conflict { .. } | StoreError::IntentConflict => Self::conflict(),
            StoreError::Command(error) => Self::command_error(error),
            StoreError::AppNotFound => Self::not_found(),
            _ => Self::internal(),
        }
    }
    fn validation_field(field: Option<evobase_appspec::FieldId>) -> Self {
        let mut error = Self::validation();
        error.field = field.map(|id| id.to_string());
        error
    }
    fn command_error(error: evobase_appspec::commands::CommandError) -> Self {
        use evobase_appspec::commands::CommandError;
        match error {
            CommandError::Authority(_) | CommandError::Policy(_) => Self::denied(),
            CommandError::UnknownCommand(_) => Self::unsupported(),
            CommandError::InvalidInput { field, .. } => Self::validation_field(field),
            CommandError::ImmutableField(field) | CommandError::GuardFailed { field } => {
                Self::validation_field(Some(field))
            }
            CommandError::Kernel(
                evobase_appspec::Error::WrongType { field, .. }
                | evobase_appspec::Error::Required { field, .. }
                | evobase_appspec::Error::ConstraintViolation { field, .. }
                | evobase_appspec::Error::InvalidStateValue { field, .. },
            ) => Self::validation_field(Some(field)),
            CommandError::InvalidTransition { .. }
            | CommandError::RevisionRequired
            | CommandError::Kernel(_) => Self::validation(),
        }
    }
}
impl IntoResponse for ApiFailure {
    fn into_response(self) -> Response {
        let dto = ApiErrorDto {
            api_version: API_VERSION,
            code: self.code,
            message: self.message.to_owned(),
            field: self.field,
        };
        let bytes = dto.encode().unwrap_or_else(|_| b"{\"api_version\":1,\"code\":\"internal\",\"message\":\"Operation unavailable\",\"field\":null}".to_vec());
        let mut response = (
            self.status,
            [(header::CONTENT_TYPE, "application/json")],
            bytes,
        )
            .into_response();
        add_response_headers(&mut response, None);
        response
    }
}
