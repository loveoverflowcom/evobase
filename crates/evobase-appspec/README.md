# evobase-appspec

Deterministic v1 JSON definition kernel. Raw wire DTOs compile into privately constructed
`CheckedAppSpec`; scoped runtime facts validate into privately constructed `CheckedRecords`.
Neither checked type implements Deserialize. `Scope` identifies data; it grants no authority.

Limits apply to both compile and decode: 1 MiB encoded bytes, depth 16, 20,000 JSON token starts,
64 tables, 256 fields per table, 2,048 fields overall, 4,096 records, names 256 UTF-8 bytes,
text values 16,384 UTF-8 bytes. Bounds intersect; maximum counts cannot always be combined.
Unknown keys/tags and duplicate keys reject. Table/field/capture-rule and record ordering is
canonical by stable identity. Field IDs are unique throughout a definition.

Required rejects Blank, Null and absent fields. Optional preserves Blank versus Null; absent
normalizes to Blank. Money is signed integer minor units; no decimals, exponent notation or
floating-point conversion. Text authoring uses empty input for Blank and `null` for Null;
`text:` escapes text literals (`text:null`, `text:`, `text:text:literal`). Reference authoring
accepts stable record IDs or an unambiguous exact display name in the expected table/scope.
Refs grant no access. Capture rules describe source/destination Money fields; capture derivation,
immutability and command authorization are separate consumers of this checked schema.

Only `decode` and `decode_records` are bounded wire entrypoints. Deserializing raw DTOs directly
is not an alternative checked entrypoint. `validate_records` checks a complete candidate final
state, including missing/wrong-table/cross-scope Ref identities; it is not a partial form update.
No DB, clock, random identity generator, network, host session, credentials or live data exists here.

The commerce v1 golden is independently authored and checked in. Shared conformance vectors run
with `cargo test -p evobase-appspec --test vectors` and `cargo run -p evobase-appspec --example conformance`.
The same example can compile for `wasm32-wasip1`; compare stdout from its WASI runtime with native
stdout. This proves the listed semantic vectors, not browser UI, host authentication or storage.

## Relations and pure query projections

`relations::RelationStore` supports canonical N:1 references, derived reverse edges and
restrict-on-delete. `apply_batch` checks the complete final graph and commits all changes together;
deleting both related records in one batch is allowed. Insert derives captured Money from the
final Product facts. Supplied captures reject, and existing capture values and their source
references cannot change. Updating a Product price changes a live lookup while retaining older
line prices. Dependent captures, 1:1, N:M and other delete modes are outside this profile.

`from_checked_snapshot` requires an explicit `LocalPreviewPolicy` marker and restores historical
facts only at a trusted synthetic local persistence/fixture adapter.
It revalidates their schema, types, scope and graph; it cannot prove a historical price's provenance.
An untrusted request body must never enter through this restore path. New authoring input uses
Insert; edited existing facts use Replace. Hosted persistence/restore remains deferred. The browser
Builder uses these operations for candidate grid changes, imports and local save validation.

`CheckedFormulas` compiles bounded pure query projections against checked field identities. The
subset is numeric Field, one-hop Lookup, derived reverse Sum, Product, Total and acyclic Named
reuse. Integer × Money produces Money; Money × Money rejects. Blank/Null propagate through Product;
Sum and Total ignore null values, and an empty Sum is typed zero. Arithmetic uses checked signed
64-bit minor units, with no floating-point or currency conversion. Default budgets are 256 evaluated
nodes, depth 24 and 1,024 visited records, including root, lookup targets and scanned child candidates.
Compilation rejects cycles and mismatched types; evaluation has exact overflow/depth/node/record
errors and creates no effects. App/table/field structure or capture-rule changes reject old checked
projections; labels and order can change without rebinding IDs. These are query APIs: editable portable
formula declarations, formula authoring and formula export remain deferred, as do recursive
expressions, scripts and SQL.

Every relation output requires an explicit `OutputPolicy` attesting the exact current definition and
facts. Complete reverse/aggregate scans require a separate data-independent full-table projection
grant; hosts with row restrictions must reject that unsupported projection before inspecting rows.
`LocalPreviewPolicy` is solely for labeled synthetic local previews and supplies no hosted authority.
Run `cargo test -p evobase-appspec --test relations` for capture/history, final-state rollback,
independent adjacency/arithmetic oracles, bounded expressions and snapshot/output-policy vectors.
