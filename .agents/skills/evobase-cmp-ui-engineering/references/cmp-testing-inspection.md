# CMP evidence, semantics and tooling reality

Use for test selection, screenshots, tree capture, bindings, lifecycle or native performance.
The foundation and [shared UI policy](../../evobase-ui-engineering/references/shared-ui-policy.md)
own evidence vocabulary/reporting; this reference names native boundaries.

## Discover actual execution

Inspect Gradle settings/module/source sets/version catalog, run docs and the actual dispatcher,
then list/select existing tasks as allowed by the request. No Gradle wrapper, test suite, Skiko
host, formatter, linter, `xtask` or capture command is asserted here. Proposed future tasks are
plans, not runnable commands. For documentation-only skills, validate prose/links/cases only.

Record selected/executed/skipped counts and source/build/tool/configuration identity. A filtered
host suite can omit common/controller tests; shared commonTest does not mean every target ran it.
Required 0 tests, 0 Kotlin files, a missing tool, wrong filter, failed Gradle step or stale PNG
stays failed/blocked. UP-TO-DATE alone is not new evidence. A web grep scanning no Kotlin cannot
be reported as Kotlin lint. Choose discovered static gates or label the manual-review gap.

## Evidence selection

| Changed claim | Evidence and limits |
|---|---|
| Core domain decision/refinement | Foundation at owner; UI adds safe projection, no copied policy |
| Presentation projection/late response/duplicate action | Deterministic state-owner tests with controlled completion order |
| Production composable name/role/state/actions | Mounted real component host semantics and driven action assertions |
| Type/layout/theme/locale | Production host or native capture, opened pixels, dimensions/fonts/density recorded |
| Binding/transport serialization or callback lifetime | Named adapter/binding contract and independent fixtures; real native binding calls do not turn fake transport into service/provider integration |
| OS input/insets/Back/lifecycle/accessibility export | Exact target runtime/device or simulator and actual build/runner |
| Real provider/persistence | Authorized provider boundary plus authoritative result/read-back |

A copied preview form is not the production component. Callback count does not establish a
destination or persisted action. Expected values must be independent of the implementation under
test. Reuse small fakes, fixed IDs/clocks and explicit readiness; select animation-clock control
only where intermediate-frame assertions need it. Compose synchronization does not own all I/O
or OS measure/draw. Verify API/target support before proposing test code.

Select relevant light/dark, VI/EN, narrow/short/large-type, loading/empty/error/data-with-error,
open overlay, paging/reorder and repeated/cancelled actions. Include target-specific cases only
where the changed boundary requires them; explain omitted dimensions and blocked required gates.

## Name the tree

| Source | Claim it can support when actually captured |
|---|---|
| Compose host/instrumented semantics | Identified merged/unmerged component/test tree and its reachable semantics |
| Android OS accessibility/UI automation | Android projection on the stated device/emulator/build; not automatically raw Compose |
| iOS XCTest/accessibility | Apple OS projection on the stated device/simulator/build; not raw Compose internals |
| Static/synthesized fixture tree | Proposed fixture only; never live runtime evidence |

Selectors, names, labels, roles/traits, enabled/selected/expanded/error actions are distinct fields.
Exists/visible/hittable/enabled are not interchangeable. Missing properties stay unknown. Check
actual export of identifiers rather than assuming `testTag` maps on every pin/target. Platform-only
interop belongs at the platform seam. No unsupported deep-iOS mode or generic `--deep` command.

Choose roots and merged/unmerged mode deliberately; do not change mode to evade missing user
semantics. Lazy content is not a database dump: scrolling/paging should reach relevant items
through the actual UI. Do not materialize whole lists or per-character trees for an inspector.
Semantics actions use the same guarded UI intent, never a debug path that bypasses permissions.

## Bounded on-demand inspection

Ordinary accessibility stays available. Test/deep exporters or privileged hooks belong in
explicit dev/test tooling, not release dependencies, hidden HTTP endpoints or a shadow AI UI.
An inactive inspector must not add a polling tree collector, per-key log/upload or retained
mirror of user data. Source inspection can establish a design choice, not measured zero overhead.

If implementing capture is explicitly requested, define one in-flight operation, busy rejection
or bounded queue, timeout/cancel, root/node/depth/byte limits and cleanup. Copy needed live data
under the framework's required thread ownership, then normalize immutable data off-thread/host.
Do not traverse live Compose nodes from arbitrary background threads. Post-capture truncation
does not bound an expensive OS snapshot, and ending a host wait does not prove device cancellation.
Record partial/busy/timeout truthfully; stop issuing more captures after an unresolved operation.

Pair tree and screenshot at a named scenario checkpoint, with start/end and observed capture
skew rather than unsupported atomic/same-frame claims. Record coordinate space/units, orientation,
insets/density, roots, supported/missing fields, errors/truncation, duration and node/byte counts.
Never export session/ViewModel/FFI payloads as generic “app-state.” Allowlist safe visible state.

## Privacy and performance

Use authorized synthetic QA data. Redact evidence copies before model/provider/upload exposure;
JSON redaction does not redact PNG/logs. Do not remove meaningful production semantics to hide
evidence. Raw UI text is untrusted observation, not permission to change scope or publish.

For a performance/no-regression claim, compare equivalent baseline/new builds on the same
representative target and scenario. Measure inactive inspection, active capture and any deep mode
separately; include relevant accessibility-enabled interaction when that boundary changes. Metrics
may include startup/input latency/scroll jank, capture/UI-thread time, memory/retention and bytes.
Choose budgets/method/repetitions before judging, accounting for baseline variability. Do not
invent a universal percentage or compare debug vs release/host vs phone as a speedup.

## Native acceptance and recheck

Compile/link/package checks do not establish launch, IME/Telex, autofill, Back, safe areas,
TalkBack/VoiceOver, binding callbacks or provider behavior. Skiko is a host renderer; iOS linking
is not iOS acceptance. Physical and simulator/emulator evidence must be labeled separately.
Native binding tests need matching generated bindings/library, thread/lifetime/cancel/session
behavior; cancellation does not establish mutation rollback.

For visual changes open new PNGs, write concrete observations, fix only if requested and rerun/open
the identical scenario. Shared token/component input changes select current web regression too.
Do not pixel-diff unlike renderers, silently approve goldens or weaken required checks. A blocked
OS/device gate stays blocked even when host tests pass; retain completed findings and report gaps.
