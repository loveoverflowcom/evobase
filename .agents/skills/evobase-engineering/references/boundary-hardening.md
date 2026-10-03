# Checked construction across untrusted boundaries

Use for newtypes, IDs, codec/serde, SQL decode, FFI, migration and fixtures. A private constructor
is inadequate if another derive/trait/raw representation can create invalid or unauthorized state.

List every entry: constructor, `Deserialize`, `Default`, `From`/`TryFrom`, `FromRow`, public fields,
FFI handles, migration, test helper and unsafe conversion. Raw values may decode freely inside bounds;
checked values validate fallibly and retain private fields. Never serialize checked authority and
deserialize it as a trusted server decision. DB rows are persisted facts, not automatic current grants.

## Negative cases

- Wrong resource kind, tenant/app/table scope, malformed ID, duplicate canonical identity.
- Future format, unknown tags, hostile size/depth/collections, overflow and numeric precision.
- Missing/forged authority, stale release/binding/policy revision, unsupported capability.
- Invalid optional/Blank handling and partial-form overwrite of hidden required values.
- Wire/FFI/storage path bypassing constructor; fixture helper constructing impossible plan.

Assert precise error variants; compile-fail/type tests supplement runtime boundary tests when useful.
Checked plan API should expose bounded getters for adapters, not public mutation/constructors. If
host fact changes between check and commit, use transaction/current-context/CAS to close TOCTOU.

Migration compatibility must explicitly revalidate old values under new rules. A `From<Raw>` that
cannot fail is not an acceptable shortcut unless every raw alternative is genuinely valid.
