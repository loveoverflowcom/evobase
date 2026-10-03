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
