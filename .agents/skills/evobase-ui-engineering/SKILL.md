---
name: evobase-ui-engineering
description: >-
  Plan, implement when requested, and assess EvoBase Leptos web UI, reactive browser behavior,
  semantic styling, VI/EN localization and visual evidence. Own shared UI policy and route CMP
  mobile UI to evobase-cmp-ui-engineering. Shared tokens require tracing both real renderers.
---

# EvoBase web UI engineering

Use [the foundation](../evobase-engineering/SKILL.md) for domain ownership, contracts and evidence.
This skill supplies presentation criteria, not permission to implement, install tools or publish.
For a planning/design-only request, produce the consumer map, contracts and evidence plan, then
stop before product code. For report-only inspection use
[the inspector](../evobase-ui-inspector/SKILL.md). For a PR/diff review start with
[code review](../evobase-code-review/SKILL.md); renderer guidance does not change its read-only scope.

## Resolve the surface

Pin source SHA and relevant working-tree changes. Identify the actual application entrypoint,
screen, state owner and mounted component. At this skill's creation, the checkout has a Rust
backend and Flutter client; Leptos, CMP, token adapters and UI harnesses are **proposed**. Recheck
HEAD rather than treating this dated observation as permanent. A design frame is not an app.

- Leptos, browser interop or CSS: read [shared UI policy](references/shared-ui-policy.md) and
  [Leptos runtime](references/leptos-runtime.md)
- CMP/Kotlin screen: use [CMP engineering](../evobase-cmp-ui-engineering/SKILL.md) directly;
  a Gradle file alone does not establish UI scope
- Shared design token/component contract: map source → each adapter → each production-mounted
  consumer and select evidence for both affected renderers
- Rust domain/client policy behind the UI: foundation plus the affected renderer boundary

The target is Leptos web Builder/Runtime and CMP mobile **Runtime-first**. Mobile handles lists,
record details, generated forms and workflow/approval actions. Do not invent a phone Builder
canvas or silently migrate Flutter because it exists in the old checkout.

## Work at the correct owner

1. Resolve the user-visible contract and failure mode. Separate design/proposed behavior from
   behavior supported by source and executed evidence
2. Read the canonical design handoff named in [shared UI policy](references/shared-ui-policy.md).
   For visual changes read [M3 Expressive](references/material3-expressive.md); for text,
   semantics or focus read [localization and accessibility](references/localization-accessibility.md)
3. Preserve domain authority in the Rust/domain package owner. A generated form projects the
   same contract; client checks do not become a second refinement, policy or workflow truth
4. If implementation is requested, change the existing component/adapter seam. Keep simple
   menu/focus state local; model consequential async transitions only where useful. Reuse the
   locale resources, token pipeline and harness actually present
5. Select and run only discovered checks in the authorized scope using
   [web evidence](references/web-evidence.md). For visuals, open attributable before/after
   captures, record findings and recheck the same scenario after fixes
6. Report exact claim, evidence boundary, commands/results and remaining gaps using the shared
   report. A compile, static frame or unopened PNG cannot establish production UI acceptance

## References by need

- [Shared UI policy](references/shared-ui-policy.md): canonical inputs, consumer tracing,
  phase boundaries and completion fields, shared directly with CMP
- [M3 Expressive](references/material3-expressive.md): attention hierarchy, five levers,
  screen-appropriate intensity and adaptive Builder/Runtime composition
- [Localization and accessibility](references/localization-accessibility.md): VI/EN copy,
  typed errors, focus, semantic states and reduced motion
- [Leptos runtime](references/leptos-runtime.md): async ownership, stale responses, browser
  cleanup, SSR/hydration boundaries and measured performance
- [Web evidence](references/web-evidence.md): discovery, deterministic component scenarios,
  inspected pixels and honest coverage
- [Task evaluations](evaluations/ui-skill-cases.md): realistic planning/review cases; document
  validation and an author walkthrough are not product tests

Adaptation provenance is maintained in
[the repository provenance record](../../../docs/work-plan/provenance.md). No old deployment
hosts, private account permissions or assumed command runners are inherited.
