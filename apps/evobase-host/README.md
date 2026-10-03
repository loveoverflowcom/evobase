# Single-tenant AppSpec pilot host

This binary is separate from the legacy SQL gateway. One operator-configured tenant, database
binding and app release run through the generic checked Rust core. HTTP selectors cannot select
a database, supply an actor or grant capabilities. The startup release is immutable and content
addressed; this pilot has no HTTP publish, schema editor, identity provider or raw SQL endpoint.

Local libSQL is executable without an account. Remote libSQL/Turso uses the same adapter with
an operator-supplied URL and token. Running local tests does not establish a Turso cloud test.
Multiple tenants, hosted provisioning and separate database runtime/migrator privileges are future
work. Local SQLite/libSQL engine connections can perform DDL; the HTTP API exposes no such route.

## Operator configuration

```sh
export EVOBASE_TENANT_ID=pilot
export EVOBASE_APP_ID=app_support
export EVOBASE_DB_URL=file:/tmp/evobase-pilot.db
export EVOBASE_ACCESS_FILE=/tmp/evobase-access.json
export EVOBASE_LISTEN=127.0.0.1:8080
export EVOBASE_ALLOWED_ORIGINS=http://127.0.0.1:4173
```

For remote storage, set `EVOBASE_DB_URL=libsql://YOUR_DATABASE.turso.io` and
`EVOBASE_DB_AUTH_TOKEN` in the server environment. Never put these in AppSpec, a browser form,
the access file, URLs or committed fixtures. Use TLS for remote HTTP exposure.

The operator issues a static bearer using **32 cryptographically random bytes**, encoded as
64 lowercase hexadecimal characters (`openssl rand -hex 32`). The host rejects other formats,
hashes the incoming bearer with SHA-256 and compares it in constant time to the configured hash.
Keep the bearer private; a hash does not make a human-chosen weak password safe. This pilot does
not implement login/refresh or silently trust a cookie, unsigned actor header or URL actor.

Create a private access JSON file (the example hash below is a placeholder, not a usable token):

```json
{
  "token_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
  "actor_id": "actor_alice",
  "expires_at": 1800000000,
  "revoked": false,
  "active": true,
  "membership_revision": 1,
  "roles": [],
  "grants": ["read", "write"]
}
```

Replace the placeholder with the SHA-256 of the token's exact ASCII bytes (no newline). Set a
short explicit Unix expiry and file permissions such as `chmod 600`. Store the raw token outside
the repository. Design, Publish, Manage, Read, Write and Submit are independent host capabilities;
the current generic transition profile requires Write. Design alone grants neither Read nor Write.
Publish/Manage/Submit configuration does not enable a route in this pilot.

The file is read for every authority resolution as one coherent token/membership snapshot.
To revoke, atomically replace it with `revoked: true` or `active: false` and increment
`membership_revision`. To rotate, replace the hash and expiry together. Token rotation cannot
combine a previously valid token with elevated grants from the new snapshot. Current authority
is checked again by the transaction before command/replay. A committed command remains committed
when access is revoked; recovery then denies rather than revealing the old success receipt.

## Checked fixture bootstrap

The CLI accepts local bounded JSON files, validates the AppSpec and same-scope records, and
boots an immutable release. It cannot overwrite an existing app with different data.

```sh
cargo run -p evobase-host -- fixture-bootstrap /path/to/spec.json /path/to/records.json
cargo run -p evobase-host -- serve
```

The host requires an already bootstrapped app. Core workflow fixtures `app_support` and
`app_library` exercise unrelated definitions through these same routes; host integration tests
serialize those fixtures to local checked files/data, with no domain-specific controller.

## Versioned HTTP subset

All operations use `Authorization: Bearer TOKEN`. Browser requests also require an exact
configured Origin; mutations use the actual HTTP Origin for the core CSRF check. CORS permits
Authorization/Content-Type, GET/POST/OPTIONS and the configured origins, without cookie credentials.

| Route | Required authority | Output |
|---|---|---|
| `GET /v1/apps/{app}/metadata` | Design | Checked canonical definition and logical metadata |
| `GET /v1/apps/{app}/runtime-metadata` | Read | Visible table/read-field metadata; allowed actions |
| `GET /v1/apps/{app}/tables/{table}?limit=100` | Read + row/field policy | Visible scalar values |
| `POST /v1/apps/{app}/commands` | Write + command row/state/input policy | Atomic commit receipt |
| `GET /v1/apps/{app}/receipts/{key}` | Write + current command row policy | Authorized original receipt |

Runtime actions include eligible visible record IDs. Hidden rows do not advertise commands.
The table endpoint supports only the stable first page, `limit` 1–256; `has_more` reports omitted
authorized rows. It has no cursor/offset, arbitrary filters, joins, Ref projection, aggregates,
create/delete or SQL. Unknown query parameters are rejected. Hidden rows do not contribute to the
returned records or `has_more`. Ref/grant/body actor inputs do not provide authority.

The command envelope is owned by `evobase-protocol::appspec`: API version 1, command/record IDs,
scope-free typed params, immutable release identity, exact decimal-string expected revision and
request key. Retry preserves the same key and normalized intent. Stale revisions and conflicting
intent return 409; denied authority returns 403; validation errors return stable diagnostics.
No provider effects are delivered by this process.

Limits: 32 headers/8 KiB total, 512-byte URI, 256 KiB command body, 1 MiB encoded response, at most
256 rows, and the checked AppSpec/core collection/depth bounds. Errors are sanitized; DB URLs,
tokens, SQL and raw invalid values are not returned. Responses disable caching.

## Obligation ledger

| Invariant / owner | Failure | Regression oracle |
|---|---|---|
| Host facts bind token, actor, scope and current capabilities | Forged/expired/revoked tokens or mixed config generations | Real TCP HTTP negatives and access rotation test |
| Checked core owns row/field/command decisions | Hidden metadata/rows, implicit Design-to-Write grant | HTTP projection and independent capability tests |
| Store atomically commits revision/intent/receipt | Repeated mutation or stale successful receipt | HTTP same-key, conflict and revocation vectors plus store conformance |
| Operator config selects one tenant and immutable release | App selectors reach another binding | Wrong app, request authority fields and release negatives |

Focused check: `cargo test -p evobase-host`. See the batch evidence for executed checks and any
remote/provider/native gates that remain unrun.
