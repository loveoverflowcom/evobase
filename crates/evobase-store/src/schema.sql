-- Fixed engine layout. Logical AppSpec tables never become SQL tables.
CREATE TABLE IF NOT EXISTS engine_metadata (
    singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    tenant_id TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS app_releases (
    app_id TEXT NOT NULL,
    spec_identity TEXT NOT NULL,
    spec_json BLOB NOT NULL CHECK(length(spec_json) <= 1048576),
    PRIMARY KEY(app_id, spec_identity)
);
CREATE TABLE IF NOT EXISTS applications (
    app_id TEXT PRIMARY KEY,
    spec_identity TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    facts_json BLOB NOT NULL CHECK(length(facts_json) <= 1048576)
);
CREATE TABLE IF NOT EXISTS command_receipts (
    app_id TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    request_key TEXT NOT NULL,
    intent_json BLOB NOT NULL CHECK(length(intent_json) <= 1048576),
    request_json BLOB NOT NULL CHECK(length(request_json) <= 1048576),
    receipt_json BLOB NOT NULL CHECK(length(receipt_json) <= 1048576),
    PRIMARY KEY(app_id, actor_id, request_key)
);
CREATE TABLE IF NOT EXISTS committed_events (
    app_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    event_index INTEGER NOT NULL CHECK(event_index >= 0),
    event_json BLOB NOT NULL CHECK(length(event_json) <= 1048576),
    PRIMARY KEY(app_id, revision, event_index)
);
CREATE TABLE IF NOT EXISTS command_audit (
    app_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    audit_json BLOB NOT NULL CHECK(length(audit_json) <= 1048576),
    PRIMARY KEY(app_id, revision)
);
