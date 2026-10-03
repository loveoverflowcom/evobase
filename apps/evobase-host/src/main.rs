use evobase_appspec::{AppId, CheckedAppSpec, Scope};
use evobase_host::{
    HostState,
    access::{AccessSource, PilotVerifier},
    router,
};
use evobase_store::Store;
use std::{collections::BTreeSet, env, io::Read, path::PathBuf, sync::Arc};

fn required(name: &str) -> Result<String, &'static str> {
    env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or("Missing required host configuration")
}
fn bounded_file(path: &str) -> Result<Vec<u8>, &'static str> {
    let file = std::fs::File::open(path).map_err(|_| "Cannot open local fixture")?;
    let mut bytes = Vec::new();
    file.take(evobase_appspec::MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read local fixture")?;
    if bytes.len() > evobase_appspec::MAX_BYTES {
        return Err("Fixture exceeds byte budget");
    }
    Ok(bytes)
}

#[tokio::main]
async fn main() {
    if let Err(message) = run().await {
        // Provider URLs, access files and request data are deliberately absent from logs.
        eprintln!("evobase-host: {message}");
        std::process::exit(1);
    }
}
async fn run() -> Result<(), &'static str> {
    let tenant = required("EVOBASE_TENANT_ID")?;
    let app =
        AppId::new(required("EVOBASE_APP_ID")?).map_err(|_| "Invalid configured app identity")?;
    let scope = Scope::new(&tenant, app).map_err(|_| "Invalid configured tenant identity")?;
    let destination = required("EVOBASE_DB_URL")?;
    let store = if destination.starts_with("libsql://") || destination.starts_with("https://") {
        let token = required("EVOBASE_DB_AUTH_TOKEN")?;
        Store::open_remote(&destination, &token, &tenant).await
    } else if let Some(path) = destination.strip_prefix("file:") {
        if path.is_empty() {
            return Err("Invalid configured local database path");
        }
        Store::open_local(path, &tenant).await
    } else {
        return Err("Database URL must be file:, libsql:// or https://");
    }
    .map_err(|_| "Cannot initialize configured store")?;
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args
        .first()
        .is_some_and(|argument| argument == "fixture-bootstrap")
    {
        if args.len() != 3 {
            return Err("Usage: evobase-host fixture-bootstrap SPEC_JSON RECORDS_JSON");
        }
        let spec = CheckedAppSpec::decode(&bounded_file(&args[1])?)
            .map_err(|_| "Invalid fixture AppSpec")?;
        if spec.app_id() != scope.app_id() {
            return Err("Fixture app is outside configured scope");
        }
        let facts = spec
            .decode_records(&scope, &bounded_file(&args[2])?)
            .map_err(|_| "Invalid fixture records")?;
        let snapshot = store
            .bootstrap(&spec, &facts)
            .await
            .map_err(|_| "Fixture bootstrap failed")?;
        println!(
            "Bootstrapped {} revision {} release {}",
            scope.app_id(),
            snapshot.revision(),
            snapshot.spec_identity()
        );
        return Ok(());
    }
    if !args.is_empty() && args != ["serve"] {
        return Err("Usage: evobase-host [serve | fixture-bootstrap SPEC_JSON RECORDS_JSON]");
    }
    let source = AccessSource::File(PathBuf::from(required("EVOBASE_ACCESS_FILE")?));
    let origins: BTreeSet<_> = env::var("EVOBASE_ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let verifier = PilotVerifier::new(
        source,
        scope.clone(),
        store.binding_id().to_owned(),
        origins,
    )
    .map_err(|_| "Invalid server access configuration")?;
    let state = HostState::new(Arc::new(store), scope, verifier)
        .await
        .map_err(|_| "Cannot load configured release")?;
    let address = env::var("EVOBASE_LISTEN").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|_| "Cannot bind HTTP listener")?;
    println!("EvoBase single-tenant pilot host listening");
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .map_err(|_| "HTTP server failed")
}
