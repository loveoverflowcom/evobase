# CMP ownership, state and lifetime

Use for component APIs, state/effects, feature decomposition or native adapter boundaries.
Inspect the actual app/build/source-set owners; this is a proposed target contract, not a map
of CMP files already implemented. The foundation owns durable rules.

## Dependency and authority map

```text
Android/iOS host → app composition/session → feature route/container → Screen → sections/fields
UI intent → presentation state holder → concrete client port → adapter → authoritative domain
Domain result → safe typed presentation projection → observed state → mounted content
```

Do not assume FFI, a generated Rust binding, HTTP client or secure-storage implementation exists.
Choose the actual client boundary in the foundation/implementation task and document it. If a
native binding is used, keep generated bindings read-only and test the matched native library.
Changing runtime language does not authorize moving validation/policy/workflow truth into Kotlin.

Use feature-first organization when a real feature boundary exists. Runtime lists/details/forms/
approvals can share component contracts while retaining app/navigation authority. Small cohesive
features can stay in few files. Avoid empty layers, module-per-widget, global `utils` buckets,
flag-heavy BaseScreen or a god state holder. Similar appearance alone does not justify sharing.

For a migration, record current/proposed UI, presentation, adapter, platform and domain owners;
dependency/lifetime arrows; old/new file-symbol-package-source-set mapping and all actual callers.
Check package alignment, commonTest/platform tests, `expect`/`actual`, Kotlin/JVM facade names and
Swift-visible entrypoints if present. Folder moves and shorter files do not prove decomposition.

## State and API rules with reasons

| Scope | Rule and failure avoided | Oracle/exception |
|---|---|---|
| Reusable composable API | Prefer `modifier: Modifier = Modifier` as first optional parameter, apply the caller's layout modifier once at the intended root. Separate input focus control when needed. | Trace real callers and assert bounds/focus/semantics; do not mechanically move a modifier that currently targets an input. Private helpers can follow parent-specific contracts. |
| Mutable presentation facts | Keep one observable owner at the appropriate lifetime; facts/events at reusable boundaries, pure projections for cheap derivations. | Driven state/action assertions; `val`, read-only `List` or `data class` does not imply deeply immutable elements. |
| Meaningful async state | Preserve initial/refresh/append, data-with-error, pending action, current record/account and unknown outcome as needed. | Controlled late/reordered completion tests; generic Loading/Success/Error is fine only when it preserves the observable contract. |
| Decomposition | Split by named concern while preserving stable keys, slot/remember identity, scroll, focus, caret/composition and callback ordering. | Same-production-seam regression; a file per Text/Spacer or a new controller per boolean adds no useful boundary. |
| Effects | Name owner, keys, restart/latest callback behavior, cleanup and obsolete-result guards; uncontrolled I/O stays out of render. | Remount/navigation/session/cancellation tests. Local scroll/menu/focus/IME state can stay in UI. |
| Concurrent updates | State-transform lambdas are side-effect-free; execute I/O/events under the owner with explicit ordering. | A CAS/update lambda can retry. Serialized assignment is valid when the ownership/concurrency contract permits it; no blanket `.value` ban. |
| Performance claims | Measure affected behavior before/after; compiler stability annotations are real contracts. | Do not add `@Immutable`, remember or copying everywhere to claim speed or hide mutable aliases. Correctness-driven identity does not require a claimed speedup. |

The rules above are manual review criteria until an actual test/lint owner implements enforcement.
For an exception, state the contract, reason and evidence at the affected boundary; temporary debt
needs a concrete follow-up condition. Do not turn conventions into arbitrary file-length gates.

## Async, input and sensitive state

Guard duplicate action admission and late results independently. Key record/account/session
requests explicitly. Cancellation does not prove server rollback; an unknown approval/save outcome
requires reconciliation at the authoritative boundary rather than blind retry. Keep loaded data
when the refresh contract calls for it. Assert current observable result, not only callback count.

Preserve stable list/record identity across reorder and navigation. A component extraction must
not accidentally remount a field, drop composition/undo or create a second subscription. Native
handles/controllers/subscriptions have exactly one lifetime owner with dispose/failure/cancel paths.

Passwords and tokens stay temporary or in the explicitly selected secure-session owner, never
saveable UI state, routes, tags, logs, previews or evidence. Password-field clearing is a UI lifetime
contract, not memory zeroization. Use synthetic values in tests; do not inherit a prior project's
credential policy or QA authorization. Generated forms can be restored according to their product
contract, but sensitive fields require explicit classification rather than blanket persistence.

## Cross-platform reality

Inspect pinned dependency coordinates/APIs and each target before adding a dependency. Android
support does not imply commonMain/iOS support. Keep platform-specific input/storage/transport at
its source-set seam. No framework upgrade, formatter installation or default architecture migration
is implied by a focused UI task. Discover existing formatting/static gates; otherwise report a
manual-review gap and propose tooling separately.
