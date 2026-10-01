# Typed relations, captured values and constraints

Read for Ref, inverse relation, lookup, rollup, delete, uniqueness or import behavior. Begin with
one Customer/Order/OrderLine/Product slice; do not claim all cardinalities from a picker mock.

## Semantic owner

Ref stores target table/record identity in the same tenant/app instance; labels are display only.
N:1 is canonical on the child; reverse 1:N is derived. A 1:1 Ref adds scoped uniqueness with stated
null semantics. N:M uses a link table or equivalent managed edge contract; relations with business
attributes use real records. Comma-separated IDs are not a relation representation.

Lookup/rollup are derived and not generic inputs. CapturedUnitPrice preserves historical value;
changing Product price does not retroactively change OrderLine history. Derivation never grants
visibility. Picker/search/lookup/aggregate/export must enforce output policy, including count and
existence leakage. Unsupported policy projection rejects instead of widening access.

## Choose the smallest support envelope

Existing VOT baseline supports N:1 and restrict-on-delete; additional cardinalities/delete modes
need explicit implementation and evidence. Reject unsupported tags at decode/compile/runtime.
State typed money precision/currency/overflow rules; do not introduce floats for business totals.
Rollup null/empty/filtered semantics and reference cycles/budgets must be deliberate.

## Evidence matrix

- Rename tables/fields/targets, duplicate display labels, sort/reorder/localize: identity unchanged.
- Missing/wrong-table/cross-app/cross-tenant Ref and ambiguous import: exact diagnostics, no guessing.
- Final-state batch may insert related records together; validate the final graph atomically.
- Reverse edge/lookup/rollup match canonical records after update/delete/rollback/restart.
- Captured price stable after source changes; arithmetic bounds compared to independent oracle.
- Unique/null/N:M edge/delete contracts tested if advertised; concurrency tests at storage boundary.
- Delete versus Ref-create races preserve invariant; UI pre-check is insufficient.
- Restricted target field/count/search cannot leak through label, aggregate or exported projection.

Ask whether the relation is derived or stored before adding a second list of IDs. Trace owner,
update protocol and recovery; a cache/index is not editable canonical truth.
