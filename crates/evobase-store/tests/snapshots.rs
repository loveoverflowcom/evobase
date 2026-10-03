use evobase_appspec::{CheckedAppSpec, Scope, fixtures};
use evobase_store::{Store, StoreError};

#[tokio::test]
async fn two_unrelated_specs_use_the_same_fixed_catalog_and_reopen_checked_facts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tenant.db");
    let store = Store::open_local(&path, "tenant_local").await.unwrap();
    let catalog = store.catalog().await.unwrap();
    let commerce = CheckedAppSpec::compile(fixtures::example_spec()).unwrap();
    let commerce_scope = Scope::new("tenant_local", commerce.app_id().clone()).unwrap();
    let commerce_facts = commerce
        .validate_records(&commerce_scope, &fixtures::example_records(&commerce_scope))
        .unwrap();
    let first = store.bootstrap(&commerce, &commerce_facts).await.unwrap();
    let other = CheckedAppSpec::decode(br#"{"version":1,"app_id":"app_library","name":"Library","tables":[{"id":"tbl_books","name":"Books","fields":[{"id":"fld_title","name":"Title","required":true,"field_type":{"type":"text"}}]}]}"#).unwrap();
    let other_scope = Scope::new("tenant_local", other.app_id().clone()).unwrap();
    let other_facts = other.decode_records(&other_scope, br#"[{"scope":{"tenant_id":"tenant_local","app_id":"app_library"},"id":"rec_book","table_id":"tbl_books","values":{"fld_title":{"type":"text","value":"Storage contracts"}}}]"#).unwrap();
    store.bootstrap(&other, &other_facts).await.unwrap();
    assert_eq!(store.catalog().await.unwrap(), catalog);
    assert_eq!(
        store
            .bootstrap(&commerce, &commerce_facts)
            .await
            .unwrap()
            .revision(),
        1
    );
    drop(store);
    let reopened = Store::open_local(&path, "tenant_local").await.unwrap();
    let snapshot = reopened.snapshot(&commerce_scope).await.unwrap();
    assert_eq!(snapshot.spec(), &commerce);
    assert_eq!(snapshot.facts(), &commerce_facts);
    assert_eq!(snapshot.spec_identity(), first.spec_identity());
    assert_eq!(snapshot.revision(), 1);
    assert_eq!(
        reopened.snapshot(&other_scope).await.unwrap().facts(),
        &other_facts
    );
}

#[tokio::test]
async fn tenant_binding_and_bootstrap_never_overwrite_existing_facts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tenant.db");
    let store = Store::open_local(&path, "tenant_local").await.unwrap();
    let spec = CheckedAppSpec::compile(fixtures::example_spec()).unwrap();
    let scope = Scope::new("tenant_local", spec.app_id().clone()).unwrap();
    let facts = spec
        .validate_records(&scope, &fixtures::example_records(&scope))
        .unwrap();
    store.bootstrap(&spec, &facts).await.unwrap();
    let empty = spec.validate_records(&scope, &[]).unwrap();
    assert!(matches!(
        store.bootstrap(&spec, &empty).await,
        Err(StoreError::AppAlreadyExists)
    ));
    let foreign_scope = Scope::new("tenant_other", spec.app_id().clone()).unwrap();
    assert!(matches!(
        store.snapshot(&foreign_scope).await,
        Err(StoreError::ScopeMismatch)
    ));
    assert!(matches!(
        Store::open_local(&path, "tenant_other").await,
        Err(StoreError::ScopeMismatch)
    ));
    assert_eq!(store.snapshot(&scope).await.unwrap().facts(), &facts);
}

#[tokio::test]
async fn remote_credentials_require_secure_urls_before_connecting() {
    for url in [
        "http://example.invalid",
        "file:///tmp/tenant.db",
        "https://user:password@example.invalid",
        "libsql://example.invalid?token=secret",
        "https://example.invalid#secret",
    ] {
        assert!(matches!(
            Store::open_remote(url, "fixture-secret", "tenant_local").await,
            Err(StoreError::InvalidRemoteConfiguration)
        ));
    }
    assert!(matches!(
        Store::open_remote("https://example.invalid", "", "tenant_local").await,
        Err(StoreError::InvalidRemoteConfiguration)
    ));
}

#[tokio::test]
async fn unsupported_or_foreign_partial_store_is_denied_before_schema_initialization() {
    let directory = tempfile::tempdir().unwrap();
    for (version, tenant, expected_foreign) in
        [(2, "tenant_local", false), (1, "tenant_other", true)]
    {
        let path = directory
            .path()
            .join(format!("partial-{version}-{tenant}.db"));
        let database = libsql::Builder::new_local(&path).build().await.unwrap();
        let connection = database.connect().unwrap();
        connection.execute_batch("CREATE TABLE engine_metadata(singleton INTEGER, schema_version INTEGER, tenant_id TEXT);").await.unwrap();
        connection
            .execute(
                "INSERT INTO engine_metadata VALUES (1, ?1, ?2)",
                libsql::params![version, tenant],
            )
            .await
            .unwrap();
        let result = Store::open_local(&path, "tenant_local").await;
        if expected_foreign {
            assert!(matches!(result, Err(StoreError::ScopeMismatch)));
        } else {
            assert!(matches!(result, Err(StoreError::UnsupportedSchema)));
        }
        let mut rows = connection
            .query(
                "SELECT count(*) FROM sqlite_schema WHERE type = 'table'",
                (),
            )
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            1
        );
    }
}
