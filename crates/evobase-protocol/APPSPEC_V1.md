# Generic AppSpec HTTP protocol v1

`evobase_protocol::appspec` owns raw JSON transport. The deterministic Rust AppSpec core owns
typed values, constraints, policies and checked execution; the host owns verified credentials,
the configured tenant/app/release, current grants and persistent receipts. A deserialized DTO
is never a checked command or trusted context. The wire contract has `api_version: 1`, independent
of the portable AppSpec format version.

The single-tenant host binds the tenant from host configuration. App and table path IDs select
resources inside that scope and confer no access. Command bodies contain no actor, tenant,
grants, trusted context, checked plan, credential or database selector. A Ref contains only
`table_id` and `record_id`; the adapter supplies its configured scope before the core validates
the raw value. Cross-scope stored Refs are rejected before a scope-free output is constructed.

| Route | Wire result / supported input |
|---|---|
| `GET /v1/apps/{app}/metadata` | `MetadataResponseDto`, requires Design for canonical definition |
| `GET /v1/apps/{app}/runtime-metadata` | `MetadataResponseDto`, permitted fields/actions, `canonical_appspec: null` |
| `GET /v1/apps/{app}/tables/{table}?limit=100` | `ListResponseDto`, authorized records, default limit 100, range 1–256 |
| `POST /v1/apps/{app}/commands` | `CommandRequestDto` → `ReceiptResponseDto` after persistent commit |
| `GET /v1/apps/{app}/receipts/{request_key}` | current authorization → committed `ReceiptResponseDto` |

The list response includes `has_more` when authorized rows were omitted by the limit. It exposes
no unauthorized total. V1 provides no filter, order, offset, continuation, join, arbitrary select
or SQL input. The HTTP adapter rejects unsupported and duplicate query parameters rather than
ignoring them. Request bodies are JSON; responses use their DTO JSON shape directly, rather than
the separate legacy `ApiResponse`/`ErrorEnvelope` wrappers.

`MetadataResponseDto` identifies the app, immutable release and storage data revision. Design
metadata carries the canonical checked AppSpec JSON as a UTF-8 **string**, so a browser does not
decode exact embedded numbers through JavaScript before the Rust AppSpec codec. Runtime metadata
omits that string and projects permitted tables/fields and command input fields. Labels and
ordering are presentation; stable `app_`, `tbl_`, `fld_`, `rec_` and `cmd_` IDs retain identity.
Portable IDs are bounded to 80 ASCII letters, digits, underscores and hyphens with the indicated
nonempty kind prefix. Metadata does not itself grant execution rights, and the server rechecks
current authority for every query, command and receipt recovery.

Runtime command metadata includes `eligible_record_ids` for the current intersection of readable
records and command Write/state authority. Clients offer an action only for those records;
execution rechecks current authority and state. Design metadata uses an empty eligibility list.

Command example:

```json
{"api_version":1,"command_id":"cmd_update","record_id":"rec_a","params":{"fld_count":{"type":"integer","value":9223372036854775807}},"request_key":"request_a","expected_revision":"0","release_id":"sha256:abc"}
```

`params` maps stable field IDs to raw tagged values; the adapter maps it to core `inputs` and
maps `request_key` to core `idempotency_key`. Missing params remain absent: they do not overwrite
a value. Explicit `{"type":"blank"}` and `{"type":"null"}` are distinct. Text uses ordinary
JSON escaping. `integer` and `money` hold exact signed i64 numbers; money means integer minor
units. Fractional numbers, exponent notation, overflow and unknown value tags are rejected by
the typed decoder. Rust clients consume response **bytes/text** using the DTO's `decode` method;
JavaScript `JSON.parse` is unsuitable for these exact-number DTOs. Native client decoder evidence
is required separately before claiming CMP compatibility.

`RevisionDto` serializes the storage u64 data revision as a canonical decimal string: no sign,
leading zeroes (except `"0"`), decimal point or exponent. AppSpec format/API versions are small
version numbers, and the immutable release is a separately bounded opaque identity (currently
the host's canonical-definition hash). Request keys contain 1–96 ASCII letters, digits,
underscores or hyphens. Release IDs contain 1–128 ASCII letters, digits, underscores, hyphens or
colons. Normalized command intent, release and expected revision participate in the host's
immutable replay identity; the persistent adapter owns comparison and current-authority checks.
Retry the same key and exact intent after an uncertain response; recover the receipt before
reporting committed success. A new key may represent a different business operation.

`ReceiptResponseDto` identifies app, command, record, release, request key, committed data
revision and whether the command response was a replay. It is a commit receipt, not proof of
external-provider delivery. `ApiErrorDto` contains `api_version`, a typed `code`, bounded
`message` and optional `field` path. Codes are `denied`, `unsupported`, `conflict`, `validation`,
`not_found` and `internal`; consumers should use codes rather than parsing human messages.
The HTTP owner maps these to appropriate status codes and sanitizes storage/internal failures.

Both `decode` and `encode` enforce 1 MiB of bytes, container depth 16 and 20,000 lexical token
starts. Decode checks limits before DTO allocation, then rejects duplicate keys in **every**
JSON object, then performs typed decode with unknown fields denied. It does not round-trip JSON
through a floating-point dynamic value. Limits also include 256 values per record/request,
16 KiB per text value, 256 bytes per label, 64 tables, 2,048 metadata fields, 64 commands and 256
listed records. Error messages are bounded to 1,024 bytes and field paths to 256 bytes; hostile
parse diagnostics and duplicate-key names are truncated on UTF-8 boundaries. The wire byte
limit also applies to the escaped canonical AppSpec string in a metadata response.

Public serde derives support raw DTO interoperability. Callers must use `decode`/`encode` at
untrusted boundaries to obtain the stated budgets and complete duplicate-key checks. Neither
route constructs trusted authority or checked plans from JSON. The gateway's request-body limit
must reject oversized bodies before collecting bytes; the protocol's byte check is an additional
boundary guard. No JSON DTO can bypass the core's private checked constructors.

Browser/WASM clients depend on `evobase-protocol` with `default-features = false`. The AppSpec
module then depends only on portable AppSpec/serde crates. The default `legacy` feature retains
all previous root exports and their native core/UUID dependencies for existing server consumers.

| Existing consumer | Compatibility in this slice |
|---|---|
| Rust generic AppSpec host/client | New v1 DTOs and routes; shared bounded exact-number decoder |
| Legacy SQL-table REST, auth, docs and envelopes | Existing modules/exports retained; separate contracts |
| Flutter SQL/PostgREST-style SDK | Retained; no claim that queries target the AppSpec routes |
| Legacy SSE/messaging | Separate token/stream contract; no v1 subscription added |
| CMP/native AppSpec client | No implemented decoder or device evidence in the protocol slice |

The focused `tests/appspec_wire.rs` vectors cover serializer/decoder i64 extremes, u64 revisions,
Blank/Null/absent/text/Ref, unknown versions/tags/keys, forged authority fields, duplicates,
oversize/deep/node/collection input, cross-scope output and bounded diagnostics. Gateway tests
must additionally cross the actual HTTP and persistent-authority boundaries; this crate's
serializer vectors alone do not establish HTTP, browser lifecycle or native runtime evidence.

| Observable invariant | Owner / boundary | Failure | Cheapest adequate oracle |
|---|---|---|---|
| Exact i64 values and revision identity survive transport | Protocol Rust serializer/decoder | Rounded or ambiguous numeric intent | i64/u64 extremes and invalid numeric vectors |
| Hostile input cannot bypass bounded raw decode | Protocol bytes → raw DTO | Oversize/deep/duplicate/unknown input accepted | Exact `WireError` vectors and bounded diagnostics |
| Wire cannot choose tenant or construct checked authority | Protocol Ref conversion + host/core | Forged scope or checked plan | Authority-key negatives and cross-scope Ref projection |
| Runtime metadata offers stable, unambiguous identities | Protocol projection DTO + host policy owner | Renamed identity or duplicate per-record eligibility | Rename/duplicate metadata vectors; HTTP authority checks owned by gateway |

Focused commands are `cargo test -p evobase-protocol --no-default-features --test appspec_wire`
and `cargo check -p evobase-protocol --no-default-features --target wasm32-unknown-unknown`.
The former is Rust example-test evidence; the latter only establishes target compilation.
During this slice the native focused test run passed all 11 vectors and the portable WASM check
passed with Rust/Cargo 1.99.0. Actual HTTP, WASM vector execution and CMP/native decoding are
separate acceptance gates.
