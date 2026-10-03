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
    let commerce_facts = commerce.validate_records(&commerce_scope, &fixtures::example_records(&commerce_scope)).unwrap();
    let first = store.bootstrap(&commerce, &commerce_facts).await.unwrap();
    let other = CheckedAppSpec::decode(br#"{"version":1,"app_id":"app_library","name":"Library","tables":[{"id":"tbl_books","name":"Books","fields":[{"id":"fld_title","name":"Title","required":true,"field_type":{"type":"text"}}]}]}"#).unwrap();
    let other_scope = Scope::new("tenant_local", other.app_id().clone()).unwrap();
    let other_facts = other.decode_records(&other_scope, br#"[{"scope":{"tenant_id":"tenant_local","app_id":"app_library"},"id":"rec_book","table_id":"tbl_books","values":{"fld_title":{"type":"text","value":"Storage contracts"}}}]"#).unwrap();
    store.bootstrap(&other, &other_facts).await.unwrap();
    assert_eq!(store.catalog().await.unwrap(), catalog);
    assert_eq!(store.bootstrap(&commerce, &commerce_facts).await.unwrap().revision(), 1);
    drop(store);
    let reopened = Store::open_local(&path, "tenant_local").await.unwrap();
    let snapshot = reopened.snapshot(&commerce_scope).await.unwrap();
    assert_eq!(snapshot.spec(), &commerce);
    assert_eq!(snapshot.facts(), &commerce_facts);
    assert_eq!(snapshot.spec_identity(), first.spec_identity());
    assert_eq!(snapshot.revision(), 1);
    assert_eq!(reopened.snapshot(&other_scope).await.unwrap().facts(), &other_facts);
}

#[tokio::test]
async fn tenant_binding_and_bootstrap_never_overwrite_existing_facts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tenant.db");
    let store = Store::open_local(&path, "tenant_local").await.unwrap();
    let spec = CheckedAppSpec::compile(fixtures::example_spec()).unwrap();
    let scope = Scope::new("tenant_local", spec.app_id().clone()).unwrap();
    let facts = spec.validate_records(&scope, &fixtures::example_records(&scope)).unwrap();
    store.bootstrap(&spec, &facts).await.unwrap();
    let empty = spec.validate_records(&scope, &[]).unwrap();
    assert!(matches!(store.bootstrap(&spec, &empty).await, Err(StoreError::AppAlreadyExists)));
    let foreign_scope = Scope::new("tenant_other", spec.app_id().clone()).unwrap();
    assert!(matches!(store.snapshot(&foreign_scope).await, Err(StoreError::ScopeMismatch)));
    assert!(matches!(Store::open_local(&path, "tenant_other").await, Err(StoreError::ScopeMismatch)));
    assert_eq!(store.snapshot(&scope).await.unwrap().facts(), &facts);
}
