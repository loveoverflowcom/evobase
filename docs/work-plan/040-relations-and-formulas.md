# 040 — Deliver stable relations, captured prices and pure rollups

Design issue: [EvoBase #5](https://github.com/loveoverflowcom/evobase/issues/5). Assets pinned at `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`.
Status: implemented experimental N:1/restrict, captures and bounded pure query APIs plus local
browser inspector. [Batch evidence](batch-evidence.md) records executed vectors. Portable formula
authoring/export, advanced cardinalities and complete restricted policy joins remain open.
Design pack: 03 relations/formulas.
Skills: engineering → relations/AppSpec; Web for relation inspector/formula projection.

## Scope / dependencies

020 checked kernel; 030 mounted Builder. Start N:1 + derived reverse + restrict-on-delete, typed
lookup/rollup and captured whole-money OrderLine value. Relation builder chooses target/display/
required/delete contract. Unsupported 1:1/N:M/delete/recursive formula options reject visibly;
full vision expansions remain in backlog, not falsely completed here.

## Delivered boundary

The formula subset is currently `CheckedFormulas` query APIs: Field, Lookup, reverse Sum, Product,
Total and acyclic Named reuse. It does not add portable formula declarations to version 1 or
provide formula editing/export. An explicit current output policy must attest exact definition/
facts; complete scans need a data-independent full-table grant. The owner/role host adapter
rejects unsupported restricted relation scans. Local preview is clearly simulated.

## Acceptance / required tests

- Rename/sort/reorder/duplicate labels preserve Ref, inverse relation and expression binding.
- Wrong-table/missing/cross-app/cross-tenant Ref, ambiguous import and unsupported modes fail exactly.
- Final-state batch supports intentional graph edits while restrict/delete constraints remain checked.
- Lookup versus captured price visibly distinguished; product-price change preserves past line values.
- Pure typed non-recursive expression subset has null/empty/overflow/cycle/budget diagnostics and
  independent graph/arithmetic oracle; query never creates effects.
- Canonical records own Ref; inverse/index projections do not become editable second truth.
- Builder uses same checked semantics and explains diagnostic/source IDs; current-output-policy seam
  is explicit, no relation-based grant before PR050.

## Original prompt and remaining acceptance

The prompt below preserves the original target and gates. The current user-authorized batch
delivered the experimental subset named above; it does not satisfy every original criterion.
Follow up on the remaining gate rather than treating the complete prompt as implemented.

> After selecting this item, pin PR020/030 and design pack03. Inspect reusable VOT Ref/rollup/capture
> contracts and implement one narrow authoritative Rust relation/formula slice plus its Leptos
> inspector/projection. N:1/restrict is the initial support envelope; reject and document other
> cardinalities/delete modes until their separate concurrency gate. Test identity preservation,
> final-state graph validation, malformed/ambiguous/cross-scope references, captured-price history,
> exact expression diagnostics and independent overflow/null/empty/budget vectors. Keep all formulas
> pure and output-policy enforcement at a required seam; Ref existence never gives target access.
> Bind UI typed intents to that core, use canonical M3 components, and test relation-chip/focus/
> error/cancel/repeated/stale flows with inspected pixels. Do not introduce SQL DDL, a general
> language, live provider or Kotlin rule copy. Run real discovered scoped/target checks and report
> supported/deferred semantics, source identity and evidence gaps. Stop at checked relation/capture
> gate; PR050 establishes host/policy authority before hosted operation.
