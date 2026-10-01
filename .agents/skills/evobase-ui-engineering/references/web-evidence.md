# Web evidence and command discovery

Use when selecting regression or visual evidence for a Leptos surface. Combine
[shared UI policy](shared-ui-policy.md) and only the changed runtime/design references.

## Discover, do not inherit a harness

Read current manifests, run docs, dispatcher/source and nearby tests. Record app entrypoint,
features/build mode, exact test selection and browser tooling actually available. At creation
time there is no asserted Leptos app, UI probe, strict static gate or `xtask` UI command.

For planning, propose the smallest test/render seam and acceptance checks; do not run a guessed
command or implement infrastructure. For implementation, reuse configured commands. If absent,
report the gap or include a harness task only when authorized by the implementation scope.
Source-discovered, command-executed and outcome-asserted are separate states.

Keep exact failures. Wrong paths, missing tools, zero required files/tests, failed steps, stale
builds and images from a different revision cannot be summarized as passed. A text grep is not
AST-aware localization/accessibility proof, and a web-only gate does not inspect Kotlin.

## Choose the cheapest real oracle

| Change | Select at the actual owner |
|---|---|
| Domain/refinement/workflow decision | Foundation/core deterministic tests; UI adds projection evidence only |
| Conditional rendering/action/state | Mount the real Leptos component with injected facts/fakes; assert text/role/state/result |
| DTO/error/metadata mapping | Adapter contract fixtures with independent expected output |
| Route, dialog, focus, language or theme transition | Real DOM and driven browser inputs with resulting state/history/focus |
| Layout/token/type/copy-length | Attributable rendered capture, opened pixels and relevant geometry/contrast |
| Provider/persistence journey | Named real boundary and read-back in the authorized test environment |

Use small stateful fakes rather than expectation-heavy mock internals. Fixed clocks/IDs,
controlled completion ordering and isolated fixtures make race tests meaningful. Wait for
observable readiness with bounds; arbitrary sleeps are not a readiness oracle. Shared test
support is justified by real reuse, not an empty package or a copied “TestWorld” architecture.

For a behavior fix, select a test that would fail under the old behavior when practical. Existing
tests count only if their assertions cover the changed claim. CSS-only polish can use visual
evidence instead of an artificial pure test; explain the selected oracle and gaps.

## Representative scenarios

- Runtime generated form: field defaults, validation, pending/success/error, repeated submit,
  cancel/back, long VI copy and large text
- Runtime list/detail: empty/data/error, retained data during refresh, paging/reorder and
  stale record/account completion; observable detail destination
- Workflow/approval: capability presentation, pending, typed failure, unknown outcome and
  current revision; no badge substitutes for policy enforcement
- Builder: draft editing, diagnostics/preview states, open inspector, resize and higher-width
  handoff; no direct production mutation masquerading as preview
- Shared controls/tokens: actual mounted consumers in light/dark and VI/EN, with affected
  default/focus/selected/disabled/error states

Select dimensions by risk, including narrow/short containers, keyboard input, zoom/large font
and reduced motion where relevant. Do not assume canonical static frame dimensions are the only
supported runtime dimensions. A locale-switch scenario tests an in-app transition, not just two
fresh pages. Treat required missing checks as blocked/inconclusive.

## Visual loop

1. Pin contract, build, fixture and selected state; identify actual browser and dependency boundary
2. Drive the production surface using supported browser/test tools within the approved fixture
3. Capture context and detail, plus applicable DOM/semantics/console facts
4. Open every cited image. Record exact inspected bytes, state, units, theme/locale and concrete
   observations; an artifact filename or authored `inspected: true` field is insufficient
5. If fixes are requested, fix the owning seam and rerun/open the same scenario. Keep comparable
   before/after inputs; if a fixture was wrong, explain and rerun both corrected sides
6. Report findings and exact evidence boundary, including images captured but never inspected

Design SVG/raster previews prove only static composition. Browser captures against fake services
can establish DOM/actions/pixels while provider behavior remains untested. Chromium does not
establish Firefox/Safari support; browser emulation does not establish phone IME/safe areas.
Do not edit DOM, replace responses or alter screenshots and present the result as a live product.

Never approve a golden, loosen thresholds, remove scenarios or switch a strict gate to report-only
to hide failure. An implemented comparator with a reviewed baseline is different from manual
inspection, and neither certifies all UI states. Preserve source/build/scenario identity and the
shared report's coverage/retention/redaction fields. Documentation-only changes need structural
and task-case validation, not application builds.
