# Shared UI ownership and evidence

Read directly from web or CMP; this document does not reload either entrypoint. The foundation
owns the repository-wide claim/evidence vocabulary. UI reports use the same terms and add the
renderer, surface and scenario that bound each claim.

## Canonical design handoff

The canonical companion-design identity is `design/m3-expressive/source/tokens.json`, with:

- `design/m3-expressive/specs/component-contract.md`
- `design/m3-expressive/specs/accessibility-and-states.md`
- `design/m3-expressive/specs/screen-map.md`
- `design/m3-expressive/specs/source-to-design.md`
- Frames in `design/m3-expressive/frames/` and static previews in
  `design/m3-expressive/previews/`

Until integration, resolve these from the exact companion branch **and commit** recorded in
`docs/work-plan/provenance.md`. A branch name is a discovery pointer, not an immutable pin. If the
commit or required bytes cannot be resolved, report a design-input gap; do not fabricate values
or claim fidelity. The foundation import must preserve this identity/path rather than create a
parallel token source. Paths above are handoff contracts, not claims that this checkout has them.

The authored JSON contains paired `color.light`/`color.dark` roles, named type styles, shape,
spacing, layout, state, motion, elevation and provenance. Values are curated semantic design
inputs, not claimed HCT output. Do not independently regenerate a palette in either renderer.
Framework color-scheme APIs cover some roles; extended success/warning or layout/state roles
need an explicit adapter. Generated output is derived data, never a second source of truth.

## Trace actual consumers

For every changed shared input, record:

1. Source role/contract and pinned design bytes
2. Generator or handwritten adapter, its actual input support and output role
3. Component owner and how the role/state is consumed
4. App entrypoint/call site that mounts that component
5. Scenario/evidence at the affected consumer, plus a gap for an unimplemented consumer

This trace is source → adapter → **mounted consumer**, not source → dependency declaration.
An outer Material theme does not restyle custom primitives that ignore it. Changing web CSS
does not prove the CMP mapping changed. Generated files need mapping/drift tests with independent
expected values when implementation is in scope; do not compute the oracle from the same adapter.

## Product and presentation ownership

- Domain declarations/IR own refinements, authorization, workflow transitions, compatibility
  and publish policy. UI metadata is their projection, not a second DSL
- Builder edits draft domain packages, presents diagnostics and previews reviewable changes.
  A preview or “verified” badge does not prove runtime policy, migration or publication
- Runtime lists/details/forms/actions consume validated metadata and server outcomes. Hidden
  or disabled controls communicate capabilities; the server remains the enforcement boundary
- Transport DTOs, provider errors and tokens stay at adapters. Screens consume safe typed
  presentation results and localized messages
- Local focus, menu, scroll and temporary field state stay at their useful lifetime. Do not send
  every keystroke across FFI or introduce a generic state framework for a boolean

## Scope and execution reality

For planning/skill/design prose, validate documents and links. Do not launch apps, install a
toolchain or implement a harness merely to validate guidance. Label future checks **proposed**.
For implementation, first inspect manifests, dispatchers, README/run docs and actual tests.
There is no asserted `xtask`, probe, Gradle suite or static gate in this skill pack.

Reuse a configured harness when present; otherwise identify a small implementation task and its
acceptance contract. A discovered command is not an executed result. A required check selecting
zero tests/files, missing a tool, propagating a failure or reading stale artifacts remains
failed/blocked. Keep pre-existing debt separate; do not loosen a baseline to hide a regression.

## Evidence selected by claim

| Claim boundary | Suitable evidence and its limit |
|---|---|
| Proposed | Plan/design/specification only; no implemented behavior implied |
| Source/static | Inspected source, document validation, compile or lint as actually executed; no rendered behavior |
| Core | Deterministic rule/state assertions at the owner; not DOM, native or provider acceptance |
| DOM | Mounted web structure/name/state/text assertions; no screen-reader guarantee |
| Host | Production CMP component semantics/actions/render on the identified host; no OS/device acceptance |
| Integration | Name the real service/process boundary actually crossed; in-process adapter tests and fake transport do not establish service/provider behavior |
| Native | Exact physical device or simulator, OS/build and driven boundary; simulated and physical results stay distinct |
| Provider | Journey across the named real provider in an authorized environment; fake data cannot support it |

These are claim-boundary categories, not replacement evidence terms. Report the foundation's
precise executed level (for example `example-tested`, `DOM-tested`, `semantics-tested`,
`integration-tested` or `native-runtime-tested`) and name its boundary/double.

Pixels and interactions add separate facts: **captured** is not **inspected**; a callback count is
not the destination; a static SVG/raster preview is not a runtime screenshot. Record screenshot
origin (`design-preview`, `browser-runtime`, `CMP-host`, `native-runtime`) so these are not conflated.
Motion/performance needs timed/measured evidence, accessibility needs the relevant semantics/input
and screen-reader boundary, and persistence needs read-back at the promised storage boundary.

## Scenario and artifact policy

Choose a risk-based matrix covering relevant light/dark, VI/EN, narrow/wide or phone/tablet,
font scale, keyboard/touch, reduced motion and affected states. Shared roles require both themes;
copy/labels require both locales. Explain untested dimensions rather than claiming a blind full
cross-product. For fixes, keep source/fixture/viewport/state comparable in the same-scenario recheck.

Use synthetic QA data and existing approved evidence storage. If none exists, propose an ignored
run directory and retention policy before generating captures; do not assume a path is ignored.
Keep reports, sanitized screenshots, logs and digests attributable to one run. Check ignore rules
before writing sensitive artifacts; never commit credentials/session files, raw user data or
ad-hoc run captures. A deliberately reviewed golden needs an owner/comparator/update procedure.
Sharing or durable upload requires the actual approved destination and recipients.

## Completion report

Include only applicable fields, but keep unknowns explicit:

```text
Scope/mode: planning, review, implementation or inspection; screens and actual consumers
Source/build/design identity: exact revisions plus relevant local changes, or unknown
Contract and trace: canonical role → adapter → production-mounted component
Claim/evidence: proposed/source/static/core/integration/DOM/host/native/provider, named boundary
Scenario: fixture, state, theme, VI/EN, input, viewport and units, font/density/scale
Checks: exact discovered command, selected/executed/skipped counts, result and logs
Images: origin, bytes/path/digest, opened by whom, concrete observations and recheck
Localization/accessibility: performed checks and precise missing runtime capabilities
Artifacts/retention/sharing: approved location, redaction scope and publication state
Not covered: blocked/inconclusive checks and residual risk
```

Documentation quality, a task walkthrough and production UI verification are different results.
Do not report all targets as covered because they share source code.
