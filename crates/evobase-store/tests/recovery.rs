mod support;
use evobase_appspec::policy::{HostAuthority, RequestChannel};
use evobase_appspec::{
    CheckedAppSpec, CommandId, FieldId, RecordId, Scope, Value, commands::RawCommandRequest,
    fixtures,
};
use evobase_store::{CommitReceipt, Store, StoreError};
use std::sync::Arc;
use support::{FixtureHost, fixture};

#[tokio::test]
async fn unrelated_support_and_library_commands_use_the_same_fixed_engine() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open_local(directory.path().join("unrelated.db"), "tenant_local")
        .await
        .unwrap();
    let catalog = store.catalog().await.unwrap();
    let (spec, scope, facts, request) = fixture();
    store.bootstrap(&spec, &facts).await.unwrap();
    let host = HostAuthority::new(FixtureHost::new(&scope, store.binding_id()));
    assert_eq!(
        store
            .execute(
                &host,
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &request
            )
            .await
            .unwrap()
            .transition()
            .state(),
        "resolved"
    );
    let library = CheckedAppSpec::compile(fixtures::library_workflow_spec()).unwrap();
    let library_scope = Scope::new("tenant_local", library.app_id().clone()).unwrap();
    let library_facts = library
        .validate_records(
            &library_scope,
            &fixtures::library_workflow_records(&library_scope),
        )
        .unwrap();
    store.bootstrap(&library, &library_facts).await.unwrap();
    let library_request = RawCommandRequest {
        command_id: CommandId::new("cmd_return_loan").unwrap(),
        record_id: RecordId::new("rec_loan_1").unwrap(),
        idempotency_key: "return_once".to_owned(),
        inputs: [(
            FieldId::new("fld_loan_note").unwrap(),
            Value::Text("Returned to library".to_owned()),
        )]
        .into_iter()
        .collect(),
    };
    let library_host = HostAuthority::new(FixtureHost::new(&library_scope, store.binding_id()));
    assert_eq!(
        store
            .execute(
                &library_host,
                "fixture-verified",
                &library_scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &library_request
            )
            .await
            .unwrap()
            .transition()
            .state(),
        "returned"
    );
    assert_eq!(store.catalog().await.unwrap(), catalog);
    assert_eq!(store.snapshot(&scope).await.unwrap().revision(), 2);
    assert_eq!(store.snapshot(&library_scope).await.unwrap().revision(), 2);
}

async fn race_commands(
    first: Arc<Store>,
    second: Arc<Store>,
    host: Arc<HostAuthority<FixtureHost>>,
    scope: Scope,
    left_request: RawCommandRequest,
    right_request: RawCommandRequest,
) -> Vec<Result<CommitReceipt, StoreError>> {
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let left_barrier = barrier.clone();
    let left_host = host.clone();
    let left_scope = scope.clone();
    let left = tokio::spawn(async move {
        left_barrier.wait().await;
        first
            .execute(
                &left_host,
                "fixture-verified",
                &left_scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &left_request,
            )
            .await
    });
    let right = tokio::spawn(async move {
        barrier.wait().await;
        second
            .execute(
                &host,
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &right_request,
            )
            .await
    });
    vec![left.await.unwrap(), right.await.unwrap()]
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn competing_connections_commit_once_and_reject_stale_cas() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("competing.db");
    let first = Arc::new(Store::open_local(&path, "tenant_local").await.unwrap());
    let second = Arc::new(Store::open_local(&path, "tenant_local").await.unwrap());
    let (spec, scope, facts, request) = fixture();
    first.bootstrap(&spec, &facts).await.unwrap();
    let host = Arc::new(HostAuthority::new(FixtureHost::new(
        &scope,
        first.binding_id(),
    )));
    let mut other = request.clone();
    other.idempotency_key = "competing_write".to_owned();
    let mut results =
        race_commands(first.clone(), second, host, scope.clone(), request, other).await;
    let (left, right) = (results.remove(0), results.remove(0));
    assert!(matches!(
        (left, right),
        (
            Ok(_),
            Err(StoreError::Conflict {
                expected: 1,
                actual: 2
            })
        ) | (
            Err(StoreError::Conflict {
                expected: 1,
                actual: 2
            }),
            Ok(_)
        )
    ));
    assert_eq!(first.snapshot(&scope).await.unwrap().revision(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn independent_connections_replay_identical_original_request() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("same-key.db");
    let first = Arc::new(Store::open_local(&path, "tenant_local").await.unwrap());
    let second = Arc::new(Store::open_local(&path, "tenant_local").await.unwrap());
    let (spec, scope, facts, request) = fixture();
    first.bootstrap(&spec, &facts).await.unwrap();
    let host = Arc::new(HostAuthority::new(FixtureHost::new(
        &scope,
        first.binding_id(),
    )));
    let mut results = race_commands(
        first.clone(),
        second,
        host,
        scope.clone(),
        request.clone(),
        request,
    )
    .await;
    let (left, right) = (results.remove(0), results.remove(0));
    let left = left.unwrap();
    let right = right.unwrap();
    assert_eq!(left.transition(), right.transition());
    assert_eq!(left.revision(), right.revision());
    assert_ne!(left.replayed(), right.replayed());
    assert_eq!(first.snapshot(&scope).await.unwrap().revision(), 2);
}

#[tokio::test]
async fn separate_process_reopens_committed_facts_and_original_receipt() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("restart.db");
    let store = Store::open_local(&path, "tenant_local").await.unwrap();
    let (spec, scope, facts, request) = fixture();
    store.bootstrap(&spec, &facts).await.unwrap();
    let host = HostAuthority::new(FixtureHost::new(&scope, store.binding_id()));
    store
        .execute(
            &host,
            "fixture-verified",
            &scope,
            &RequestChannel::NativeBearer,
            10,
            1,
            &request,
        )
        .await
        .unwrap();
    drop(store);
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "process_receipt_child", "--nocapture"])
        .env("EVOBASE_TEST_RESTART_DATABASE", &path)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn process_receipt_child() {
    let Some(path) = std::env::var_os("EVOBASE_TEST_RESTART_DATABASE") else {
        return;
    };
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let store = Store::open_local(path, "tenant_local").await.unwrap();
        let (_, scope, _, request) = fixture();
        let host = HostAuthority::new(FixtureHost::new(&scope, store.binding_id()));
        assert_eq!(store.snapshot(&scope).await.unwrap().revision(), 2);
        let recovered = store
            .receipt(
                &host,
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                &request.idempotency_key,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(recovered.transition().state(), "resolved");
        assert!(recovered.replayed());
        let retry = store
            .execute(
                &host,
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &request,
            )
            .await
            .unwrap();
        assert_eq!(retry.transition(), recovered.transition());
        assert!(retry.replayed());
        assert_eq!(store.snapshot(&scope).await.unwrap().revision(), 2);
    });
}
