# AppSpec and portable semantic identity

Use for schema, codec, compiler or version/profile changes. The current authority is EvoBase #1;
the final extension/encoding/API are ADR decisions, not settled by this skill.

## Before implementation

1. Inventory source model/engine variants and advertised server/Web/CMP format profiles separately.
   VOT's native format support exceeds its hosted authoring profile; do not port that ambiguity.
2. State bytes/nodes/depth/table/field/expression/record limits, unknown-field/version policy and
   canonical ordering. Bound input before allocating or recursively traversing it.
3. Separate raw definition from checked definition and runtime facts from host bindings. Checked
   constructors are private/fallible; successful deserialization never means authorization.
4. Define package/model/release/storage/binding/data revisions separately. Same integer cannot
   silently mean all of them. Release identity is immutable; label/localization is presentation.
5. Pick one canonical semantic model. Grid/rule builder/expert text resolve labels to stable IDs;
   they cannot require dual hand-edited sources.

## Required checks

- Stable IDs, valid kind/scope, duplicates, unknown/removed ID, rename/reorder preservation.
- Canonical encode/decode/encode fixed point, historical fixtures, incompatible/future format reject.
- Hostile nesting/size/collections, duplicate keys where meaningful, numeric range and unknown tags.
- Unsupported target capability yields exact typed diagnostics, never silent downgrade.
- No secret, real host grants, trusted actor, provider credential or tenant data in template export.
- Native/WASM vectors execute the same checked semantics within an explicit supported profile.

Golden fixtures represent accepted contracts, not regenerated output to bless a regression. Keep
reviewed negative fixtures and migration evidence; round-trip alone does not prove semantics.
Pure formulas have no I/O and explicit evaluation budgets; arbitrary recursion/SQL/scripts remain
unsupported until a separate safe profile exists.

## Report

Raw/checked boundary, changed versions, supported profiles, exact fixtures/vectors and bounds,
construction bypass audit, unexecuted targets and residual compatibility risk.
