//! These vectors cross an actual loopback TCP HTTP boundary and a real local libSQL file.
use evobase_appspec::{CheckedAppSpec, FieldId, RawAppSpec, RecordId, Scope, fixtures};
use evobase_host::{
    HostState,
    access::{AccessSource, PilotAccess, PilotGrant, PilotVerifier},
    router,
};
use evobase_protocol::appspec::{
    API_VERSION, ApiErrorDto, CommandRequestDto, ErrorCodeDto, ListResponseDto,
    MetadataResponseDto, ReceiptResponseDto, RevisionDto, ValueDto,
};
use evobase_store::Store;
use reqwest::{Client, Response, StatusCode};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const ROTATED: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const ORIGIN: &str = "http://127.0.0.1:4173";

struct FixtureHost {
    _directory: tempfile::TempDir,
    access_path: PathBuf,
    access: PilotAccess,
    store: Arc<Store>,
    scope: Scope,
    base: String,
    client: Client,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for FixtureHost {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl FixtureHost {
    async fn new(
        definition: RawAppSpec,
        records: fn(&Scope) -> Vec<evobase_appspec::RawRecord>,
    ) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let scope = Scope::new("pilot", definition.app_id.clone()).unwrap();
        let spec = CheckedAppSpec::compile(definition).unwrap();
        let facts = spec.validate_records(&scope, &records(&scope)).unwrap();
        let store = Arc::new(
            Store::open_local(directory.path().join("host.db"), "pilot")
                .await
                .unwrap(),
        );
        store.bootstrap(&spec, &facts).await.unwrap();
        let access = PilotAccess {
            token_sha256: format!("{:x}", Sha256::digest(TOKEN.as_bytes())),
            actor_id: "actor_alice".into(),
            expires_at: now() + 3600,
            revoked: false,
            active: true,
            membership_revision: 1,
            roles: vec![],
            grants: vec![PilotGrant::Read, PilotGrant::Write, PilotGrant::Design],
        };
        let access_path = directory.path().join("access.json");
        replace_access(&access_path, &access);
        let verifier = PilotVerifier::new(
            AccessSource::File(access_path.clone()),
            scope.clone(),
            store.binding_id().into(),
            BTreeSet::from([ORIGIN.into()]),
        )
        .unwrap();
        let state = HostState::new(store.clone(), scope.clone(), verifier)
            .await
            .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, router(state)).await.unwrap();
        });
        Self {
            _directory: directory,
            access_path,
            access,
            store,
            scope,
            base,
            client: Client::new(),
            task,
        }
    }
    fn url(&self, suffix: &str) -> String {
        format!("{}/v1/apps/{}/{}", self.base, self.scope.app_id(), suffix)
    }
    fn update_access(&mut self, update: impl FnOnce(&mut PilotAccess)) {
        update(&mut self.access);
        replace_access(&self.access_path, &self.access);
    }
    async fn get(&self, suffix: &str) -> Response {
        self.client
            .get(self.url(suffix))
            .bearer_auth(TOKEN)
            .send()
            .await
            .unwrap()
    }
    async fn request(
        &self,
        command: &str,
        record: &str,
        note_field: &str,
        key: &str,
        revision: u64,
        text: &str,
    ) -> CommandRequestDto {
        CommandRequestDto {
            api_version: API_VERSION,
            command_id: command.into(),
            record_id: RecordId::new(record).unwrap(),
            params: BTreeMap::from([(
                FieldId::new(note_field).unwrap(),
                ValueDto::Text(text.into()),
            )]),
            request_key: key.into(),
            expected_revision: RevisionDto::new(revision),
            release_id: self
                .store
                .snapshot(&self.scope)
                .await
                .unwrap()
                .spec_identity()
                .into(),
        }
    }
    async fn post(&self, request: &CommandRequestDto) -> Response {
        self.client
            .post(self.url("commands"))
            .bearer_auth(TOKEN)
            .header("content-type", "application/json")
            .body(request.encode().unwrap())
            .send()
            .await
            .unwrap()
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
fn replace_access(path: &std::path::Path, access: &PilotAccess) {
    let temporary = path.with_extension("new");
    std::fs::write(&temporary, serde_json::to_vec(access).unwrap()).unwrap();
    std::fs::rename(temporary, path).unwrap();
}
async fn assert_error(response: Response, status: StatusCode, code: ErrorCodeDto) -> ApiErrorDto {
    assert_eq!(response.status(), status);
    assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
    let bytes = response.bytes().await.unwrap();
    let error = ApiErrorDto::decode(&bytes).unwrap();
    assert_eq!(error.code, code);
    assert!(!String::from_utf8_lossy(&bytes).contains(TOKEN));
    error
}

#[tokio::test]
async fn actual_http_denies_missing_forged_expired_revoked_and_wrong_app() {
    let mut host = FixtureHost::new(
        fixtures::support_workflow_spec(),
        fixtures::support_workflow_records,
    )
    .await;
    assert_error(
        host.client
            .get(host.url("runtime-metadata"))
            .header("cookie", format!("token={TOKEN}"))
            .header("x-actor-id", "actor_alice")
            .send()
            .await
            .unwrap(),
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    assert_error(
        host.client
            .get(host.url("runtime-metadata"))
            .bearer_auth(ROTATED)
            .send()
            .await
            .unwrap(),
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    assert_error(
        host.client
            .get(format!("{}/v1/apps/app_other/runtime-metadata", host.base))
            .bearer_auth(TOKEN)
            .send()
            .await
            .unwrap(),
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    host.update_access(|access| access.expires_at = now() - 1);
    assert_error(
        host.get("runtime-metadata").await,
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    host.update_access(|access| {
        access.expires_at = now() + 3600;
        access.revoked = true;
    });
    assert_error(
        host.get("runtime-metadata").await,
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    host.update_access(|access| {
        access.revoked = false;
        access.active = false;
    });
    assert_error(
        host.get("runtime-metadata").await,
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    assert_eq!(
        host.store.snapshot(&host.scope).await.unwrap().revision(),
        1
    );
}

#[tokio::test]
async fn actual_http_separates_capabilities_and_projects_owner_fields_and_actions() {
    let mut host = FixtureHost::new(
        fixtures::support_workflow_spec(),
        fixtures::support_workflow_records,
    )
    .await;
    host.update_access(|access| access.grants = vec![PilotGrant::Read]);
    assert_error(
        host.get("metadata").await,
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    let metadata =
        MetadataResponseDto::decode(&host.get("runtime-metadata").await.bytes().await.unwrap())
            .unwrap();
    assert!(metadata.canonical_appspec.is_none());
    assert!(metadata.commands.is_empty());
    let records = ListResponseDto::decode(
        &host
            .get("tables/tbl_requests?limit=1")
            .await
            .bytes()
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(records.records.len(), 1);
    assert_eq!(records.records[0].record_id.as_str(), "rec_request_1");
    assert!(
        !records.records[0]
            .values
            .contains_key(&FieldId::new("fld_request_owner").unwrap())
    );
    host.update_access(|access| access.grants = vec![PilotGrant::Design]);
    assert_error(
        host.get("runtime-metadata").await,
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    assert_error(
        host.get("tables/tbl_requests").await,
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    let request = host
        .request(
            "cmd_resolve_request",
            "rec_request_1",
            "fld_request_note",
            "design_not_write",
            1,
            "Resolved",
        )
        .await;
    assert_error(
        host.post(&request).await,
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    host.update_access(|access| {
        access.grants = vec![PilotGrant::Read, PilotGrant::Write];
        access.actor_id = "actor_outsider".into();
    });
    let records =
        ListResponseDto::decode(&host.get("tables/tbl_requests").await.bytes().await.unwrap())
            .unwrap();
    assert!(records.records.is_empty());
    let metadata =
        MetadataResponseDto::decode(&host.get("runtime-metadata").await.bytes().await.unwrap())
            .unwrap();
    assert!(metadata.tables.is_empty());
    assert!(metadata.commands.is_empty());
    assert_error(
        host.post(&request).await,
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
}

#[tokio::test]
async fn unrelated_apps_share_http_commands_recovery_cas_and_conflicting_intent() {
    for (definition, records, command, record, field, table) in [
        (
            fixtures::support_workflow_spec(),
            fixtures::support_workflow_records as fn(&Scope) -> Vec<_>,
            "cmd_resolve_request",
            "rec_request_1",
            "fld_request_note",
            "tbl_requests",
        ),
        (
            fixtures::library_workflow_spec(),
            fixtures::library_workflow_records as fn(&Scope) -> Vec<_>,
            "cmd_return_loan",
            "rec_loan_1",
            "fld_loan_note",
            "tbl_loans",
        ),
    ] {
        let mut host = FixtureHost::new(definition, records).await;
        let metadata =
            MetadataResponseDto::decode(&host.get("runtime-metadata").await.bytes().await.unwrap())
                .unwrap();
        assert_eq!(metadata.commands[0].command_id, command);
        assert!(
            metadata.commands[0]
                .eligible_record_ids
                .contains(&RecordId::new(record).unwrap())
        );
        let stale = host
            .request(command, record, field, "stale", 99, "Done")
            .await;
        assert_error(
            host.post(&stale).await,
            StatusCode::CONFLICT,
            ErrorCodeDto::Conflict,
        )
        .await;
        assert_eq!(
            host.store.snapshot(&host.scope).await.unwrap().revision(),
            1
        );
        let request = host
            .request(command, record, field, "commit", 1, "Done")
            .await;
        let response = host.post(&request).await;
        assert_eq!(response.status(), StatusCode::OK);
        let receipt = ReceiptResponseDto::decode(&response.bytes().await.unwrap()).unwrap();
        assert_eq!(receipt.revision.get(), 2);
        assert!(!receipt.replayed);
        let retry = host.post(&request).await;
        assert_eq!(retry.status(), StatusCode::OK);
        let replay = ReceiptResponseDto::decode(&retry.bytes().await.unwrap()).unwrap();
        assert!(replay.replayed);
        assert_eq!(replay.revision, receipt.revision);
        let recovery =
            ReceiptResponseDto::decode(&host.get("receipts/commit").await.bytes().await.unwrap())
                .unwrap();
        assert!(recovery.replayed);
        assert_eq!(recovery.record_id, receipt.record_id);
        let mut conflict = request.clone();
        conflict.params.insert(
            FieldId::new(field).unwrap(),
            ValueDto::Text("Changed".into()),
        );
        assert_error(
            host.post(&conflict).await,
            StatusCode::CONFLICT,
            ErrorCodeDto::Conflict,
        )
        .await;
        let list = ListResponseDto::decode(
            &host
                .get(&format!("tables/{table}"))
                .await
                .bytes()
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(list.revision.get(), 2);
        host.update_access(|access| access.revoked = true);
        assert_error(
            host.post(&request).await,
            StatusCode::FORBIDDEN,
            ErrorCodeDto::Denied,
        )
        .await;
        assert_error(
            host.get("receipts/commit").await,
            StatusCode::FORBIDDEN,
            ErrorCodeDto::Denied,
        )
        .await;
    }
}

#[tokio::test]
async fn actual_http_origin_budgets_and_untrusted_authority_fail_closed() {
    let host = FixtureHost::new(
        fixtures::support_workflow_spec(),
        fixtures::support_workflow_records,
    )
    .await;
    let request = host
        .request(
            "cmd_resolve_request",
            "rec_request_1",
            "fld_request_note",
            "bounded",
            1,
            "Done",
        )
        .await;
    assert_error(
        host.client
            .post(host.url("commands"))
            .bearer_auth(TOKEN)
            .header("origin", "https://attacker.invalid")
            .header("content-type", "application/json")
            .body(request.encode().unwrap())
            .send()
            .await
            .unwrap(),
        StatusCode::FORBIDDEN,
        ErrorCodeDto::Denied,
    )
    .await;
    let preflight = host
        .client
        .request(reqwest::Method::OPTIONS, host.url("commands"))
        .header("origin", ORIGIN)
        .send()
        .await
        .unwrap();
    assert_eq!(preflight.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        preflight
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        ORIGIN
    );
    assert!(
        preflight
            .headers()
            .get("access-control-allow-credentials")
            .is_none()
    );
    let mut value = serde_json::to_value(&request).unwrap();
    value["actor_id"] = "actor_admin".into();
    value["grants"] = serde_json::json!(["write"]);
    assert_error(
        host.client
            .post(host.url("commands"))
            .bearer_auth(TOKEN)
            .json(&value)
            .send()
            .await
            .unwrap(),
        StatusCode::BAD_REQUEST,
        ErrorCodeDto::Validation,
    )
    .await;
    assert_error(
        host.get("tables/tbl_requests?tenant_id=other").await,
        StatusCode::BAD_REQUEST,
        ErrorCodeDto::Unsupported,
    )
    .await;
    assert_error(
        host.get("tables/tbl_requests?limit=1&limit=2").await,
        StatusCode::BAD_REQUEST,
        ErrorCodeDto::Unsupported,
    )
    .await;
    assert_error(
        host.client
            .get(host.url("runtime-metadata"))
            .bearer_auth(TOKEN)
            .header("x-padding", "x".repeat(8192))
            .send()
            .await
            .unwrap(),
        StatusCode::BAD_REQUEST,
        ErrorCodeDto::Validation,
    )
    .await;
    assert_error(
        host.client
            .post(host.url("commands"))
            .bearer_auth(TOKEN)
            .header("content-type", "application/json")
            .body("x".repeat(evobase_host::MAX_REQUEST_BYTES + 1))
            .send()
            .await
            .unwrap(),
        StatusCode::BAD_REQUEST,
        ErrorCodeDto::Validation,
    )
    .await;
    assert_eq!(
        host.store.snapshot(&host.scope).await.unwrap().revision(),
        1
    );
}

#[tokio::test]
async fn validation_and_release_errors_leave_persistent_state_unchanged() {
    let host = FixtureHost::new(
        fixtures::support_workflow_spec(),
        fixtures::support_workflow_records,
    )
    .await;
    let mut request = host
        .request(
            "cmd_resolve_request",
            "rec_request_1",
            "fld_request_note",
            "invalid",
            1,
            "Done",
        )
        .await;
    request.params.insert(
        FieldId::new("fld_request_note").unwrap(),
        ValueDto::Integer(123),
    );
    let error = assert_error(
        host.post(&request).await,
        StatusCode::BAD_REQUEST,
        ErrorCodeDto::Validation,
    )
    .await;
    assert_eq!(error.field.as_deref(), Some("fld_request_note"));
    request.params = BTreeMap::from([(
        FieldId::new("fld_request_owner").unwrap(),
        ValueDto::Text("actor_alice".into()),
    )]);
    let error = assert_error(
        host.post(&request).await,
        StatusCode::BAD_REQUEST,
        ErrorCodeDto::Validation,
    )
    .await;
    assert_eq!(error.field.as_deref(), Some("fld_request_owner"));
    request.params = BTreeMap::from([(
        FieldId::new("fld_request_note").unwrap(),
        ValueDto::Text("Done".into()),
    )]);
    request.release_id = "sha256:wrong".into();
    assert_error(
        host.post(&request).await,
        StatusCode::CONFLICT,
        ErrorCodeDto::Conflict,
    )
    .await;
    assert_eq!(
        host.store.snapshot(&host.scope).await.unwrap().revision(),
        1
    );
}

#[tokio::test]
async fn token_rotation_never_combines_old_token_and_new_elevated_membership() {
    let mut host = FixtureHost::new(
        fixtures::support_workflow_spec(),
        fixtures::support_workflow_records,
    )
    .await;
    host.update_access(|access| access.grants = vec![PilotGrant::Read]);
    let request = host
        .request(
            "cmd_resolve_request",
            "rec_request_1",
            "fld_request_note",
            "rotation",
            1,
            "Done",
        )
        .await;
    let old = host.access.clone();
    let mut rotated = old.clone();
    rotated.token_sha256 = format!("{:x}", Sha256::digest(ROTATED.as_bytes()));
    rotated.grants = vec![PilotGrant::Read, PilotGrant::Write];
    rotated.membership_revision += 1;
    let path = host.access_path.clone();
    let writer = std::thread::spawn(move || {
        for i in 0..200 {
            replace_access(&path, if i % 2 == 0 { &rotated } else { &old });
        }
    });
    for _ in 0..20 {
        assert_error(
            host.post(&request).await,
            StatusCode::FORBIDDEN,
            ErrorCodeDto::Denied,
        )
        .await;
    }
    writer.join().unwrap();
    assert_eq!(
        host.store.snapshot(&host.scope).await.unwrap().revision(),
        1
    );
}
