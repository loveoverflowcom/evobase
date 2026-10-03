use super::*;
use evobase_appspec::{
    FieldId, Value,
    commands::CommandError,
    policy::{AuthorityError, PolicyError},
};
#[path = "../tests/support/mod.rs"]
mod support;
use support::{FixtureHost, fixture};

async fn counts(store: &Store) -> (i64, i64, i64) {
    let connection = store.connection().unwrap();
    let mut counts = Vec::new();
    for table in ["command_receipts", "command_audit", "committed_events"] {
        let mut rows = connection
            .query(&format!("SELECT count(*) FROM {table}"), ())
            .await
            .unwrap();
        counts.push(rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap());
    }
    (counts[0], counts[1], counts[2])
}

#[tokio::test]
async fn each_failed_commit_stage_rolls_back_facts_revision_audit_events_and_receipt() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open_local(directory.path().join("atomic.db"), "tenant_local")
        .await
        .unwrap();
    let (spec, scope, facts, request) = fixture();
    store.bootstrap(&spec, &facts).await.unwrap();
    let verifier = FixtureHost::new(&scope, store.binding_id());
    let host = HostAuthority::new(verifier);
    for stage in [
        CommitStage::Facts,
        CommitStage::Audit,
        CommitStage::Events,
        CommitStage::Receipt,
    ] {
        assert!(matches!(
            store
                .execute_at(
                    &host,
                    "fixture-verified",
                    &scope,
                    &RequestChannel::NativeBearer,
                    10,
                    1,
                    &request,
                    Some(stage)
                )
                .await,
            Err(StoreError::InjectedFailure)
        ));
        let snapshot = store.snapshot(&scope).await.unwrap();
        assert_eq!(snapshot.facts(), &facts);
        assert_eq!(snapshot.revision(), 1);
        assert_eq!(counts(&store).await, (0, 0, 0));
    }
    let committed = store
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
    assert_eq!(committed.revision(), 2);
    assert!(!committed.replayed());
    assert_eq!(counts(&store).await, (1, 1, 1));
    let replay = store
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
    assert!(replay.replayed());
    assert_eq!(replay.transition(), committed.transition());
    assert_eq!(counts(&store).await, (1, 1, 1));
}

#[tokio::test]
async fn real_event_insert_error_rolls_back_the_business_transition() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open_local(directory.path().join("event-failure.db"), "tenant_local")
        .await
        .unwrap();
    let (spec, scope, facts, request) = fixture();
    store.bootstrap(&spec, &facts).await.unwrap();
    let connection = store.connection().unwrap();
    connection.execute_batch("CREATE TRIGGER test_event_failure BEFORE INSERT ON committed_events BEGIN SELECT RAISE(ABORT, 'injected event failure'); END;").await.unwrap();
    let host = HostAuthority::new(FixtureHost::new(&scope, store.binding_id()));
    assert!(matches!(
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
            .await,
        Err(StoreError::Database(_))
    ));
    assert_eq!(store.snapshot(&scope).await.unwrap().facts(), &facts);
    assert_eq!(store.snapshot(&scope).await.unwrap().revision(), 1);
    assert_eq!(counts(&store).await, (0, 0, 0));
}

#[tokio::test]
async fn denied_invalid_conflicting_and_raced_authority_never_leave_partial_state() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open_local(directory.path().join("authority.db"), "tenant_local")
        .await
        .unwrap();
    let (spec, scope, facts, request) = fixture();
    store.bootstrap(&spec, &facts).await.unwrap();
    let verifier = FixtureHost::new(&scope, store.binding_id());
    let host = HostAuthority::new(verifier.clone());
    let mut invalid = request.clone();
    invalid.inputs.insert(
        FieldId::new("fld_request_owner").unwrap(),
        Value::Text("actor_alice".to_owned()),
    );
    assert!(
        matches!(store.execute(&host, "fixture-verified", &scope, &RequestChannel::NativeBearer, 10, 1, &invalid).await, Err(StoreError::Command(CommandError::ImmutableField(field))) if field.as_str() == "fld_request_owner")
    );
    assert_eq!(counts(&store).await, (0, 0, 0));
    let mut wrong_type = request.clone();
    wrong_type
        .inputs
        .insert(FieldId::new("fld_request_note").unwrap(), Value::Integer(7));
    assert!(
        matches!(store.execute(&host, "fixture-verified", &scope, &RequestChannel::NativeBearer, 10, 1, &wrong_type).await, Err(StoreError::Command(CommandError::InvalidInput { field: Some(field), reason: "wrong type or text limit" })) if field.as_str() == "fld_request_note")
    );
    let mut constraint_violation = request.clone();
    constraint_violation.inputs.insert(
        FieldId::new("fld_request_note").unwrap(),
        Value::Text(String::new()),
    );
    assert!(
        matches!(store.execute(&host, "fixture-verified", &scope, &RequestChannel::NativeBearer, 10, 1, &constraint_violation).await, Err(StoreError::Command(CommandError::InvalidInput { field: Some(field), reason: "input constraint failed" })) if field.as_str() == "fld_request_note")
    );
    let mut raced = FixtureHost::new(&scope, store.binding_id());
    raced.race_on_read = 3;
    let raced_host = HostAuthority::new(raced);
    assert!(matches!(
        store
            .execute(
                &raced_host,
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &request
            )
            .await,
        Err(StoreError::AuthorityChanged)
    ));
    assert_eq!(counts(&store).await, (0, 0, 0));
    assert_eq!(store.snapshot(&scope).await.unwrap().facts(), &facts);
    let mut role_race = FixtureHost::new(&scope, store.binding_id());
    role_race.remove_roles_on_read = 3;
    assert!(matches!(
        store
            .execute(
                &HostAuthority::new(role_race),
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &request
            )
            .await,
        Err(StoreError::AuthorityChanged)
    ));
    let mut expires = FixtureHost::new(&scope, store.binding_id());
    expires.expires_on_clock_read = 3;
    assert!(matches!(
        store
            .execute(
                &HostAuthority::new(expires),
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &request
            )
            .await,
        Err(StoreError::Authority(AuthorityError::Expired))
    ));
    let mut wrong_binding = FixtureHost::new(&scope, store.binding_id());
    wrong_binding.binding_id = "store:different".to_owned();
    assert!(matches!(
        store
            .execute(
                &HostAuthority::new(wrong_binding),
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &request
            )
            .await,
        Err(StoreError::BindingMismatch)
    ));
    assert_eq!(counts(&store).await, (0, 0, 0));
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
    let mut changed = request.clone();
    changed.inputs.insert(
        FieldId::new("fld_request_note").unwrap(),
        Value::Text("Different intent".to_owned()),
    );
    assert!(matches!(
        store
            .execute(
                &host,
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                1,
                &changed
            )
            .await,
        Err(StoreError::IntentConflict)
    ));
    assert!(matches!(
        store
            .execute(
                &host,
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                2,
                &request
            )
            .await,
        Err(StoreError::IntentConflict)
    ));
    verifier.revoke();
    assert!(matches!(
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
            .await,
        Err(StoreError::Authority(AuthorityError::Revoked))
    ));
    assert!(matches!(
        store
            .receipt(
                &host,
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                &request.idempotency_key
            )
            .await,
        Err(StoreError::Authority(AuthorityError::Revoked))
    ));
    assert_eq!(counts(&store).await, (1, 1, 1));
}

#[tokio::test]
async fn receipt_recovery_checks_current_record_policy() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open_local(directory.path().join("receipt-policy.db"), "tenant_local")
        .await
        .unwrap();
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
    // A trusted host fact change simulates loss of current row access after receipt creation.
    let mut rows = store.snapshot(&scope).await.unwrap().facts().to_raw();
    rows[0].values.insert(
        FieldId::new("fld_request_owner").unwrap(),
        Value::Text("actor_bob".to_owned()),
    );
    let final_facts = spec.validate_records(&scope, &rows).unwrap();
    store
        .connection()
        .unwrap()
        .execute(
            "UPDATE applications SET facts_json = ?1 WHERE app_id = ?2",
            params![final_facts.encode().unwrap(), scope.app_id().as_str()],
        )
        .await
        .unwrap();
    assert!(matches!(
        store
            .receipt(
                &host,
                "fixture-verified",
                &scope,
                &RequestChannel::NativeBearer,
                10,
                &request.idempotency_key
            )
            .await,
        Err(StoreError::Command(CommandError::Policy(
            PolicyError::Denied
        )))
    ));
}
