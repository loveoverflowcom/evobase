---
name: evobase-cmp-ui-engineering
description: >-
  Plan, implement when requested, and assess EvoBase Compose Multiplatform mobile Runtime UI:
  lists, record details, generated forms and workflow/approval actions. Cover Kotlin state/lifetime,
  native semantic-token adapters, VI/EN resources, host tests and scoped Android/iOS evidence.
---

# EvoBase CMP mobile UI engineering

Follow [the foundation](../evobase-engineering/SKILL.md) for domain authority/contracts and read
[shared UI policy](../evobase-ui-engineering/references/shared-ui-policy.md) directly. Do not
reload the web router. For audit/report-only work use
[the inspector](../evobase-ui-inspector/SKILL.md). For PR/diff review begin with
[code review](../evobase-code-review/SKILL.md) and preserve its read-only execution scope.

## Resolve target and phase

Pin source/working-tree identity, app entrypoint, actual callers, source sets and targets. At
creation the checkout has no CMP app/harness; future paths, tasks and native adapters remain
**proposed** until source confirms them. Do not infer mobile implementation from design frames.
A `.kt`/`.kts` extension alone does not establish a UI task or a supported target.

The target is mobile **Runtime-first**: list/detail/generated form/workflow actions. Builder
authoring stays on web at suitable width; mobile can show a read-only summary/handoff when the
product contract supports it. Do not expand into a phone graph/schema designer or migrate the
existing Flutter client without authorization. For planning/design-only requests, produce the
contracts, ownership and evidence plan; no product code, tool installation or app launch.

## Native work loop

1. Resolve the affected contract, failure mode and actual state/lifetime owner using
   [architecture and state](references/cmp-architecture-state.md). For decomposition, map current
   → proposed ownership/dependency/lifetime and old → new symbols/source sets before moving files
2. For tokens, component shape/type or localization use
   [native design adapters](references/cmp-design-system.md). Trace canonical JSON → adapter →
   production-mounted control; an outer Material theme alone does not prove custom-control uptake
3. If implementation is requested, change the existing component seam. Reusable content receives
   facts/events; route/container binds effects/lifetime. Keep domain policy at Rust/IR authority
4. Discover actual Kotlin/Gradle/test tools and choose
   [testing and inspection](references/cmp-testing-inspection.md) for changed claims. Assert
   production component semantics/actions; capture/open actual pixels for visual claims
5. For native input, safe areas, Back, lifecycle or bindings, select the affected OS runtime and
   adapter evidence. Host tests and Apple compile/link do not discharge these gates
6. For a visual fix, open attributable before/after, record concrete findings and rerun/open the
   same scenario. Shared inputs also need current web-consumer evidence. Report blocked gates

Keep framework/version/API and commonMain/platform support explicit before proposing dependencies.
No ViewModel/MVI/DI framework, Material dependency, formatter or new module is required merely by
this skill. No runner is installed or assumed. Use the shared report with native fields: px/dp/
points, density/fontScale/fonts, device/host and OS, renderer/source set, tree source/merge mode/
roots, image/tree checkpoint skew and selected/executed/skipped test counts.

## Maintaining guidance

Use [CMP evaluation cases](evaluations/cmp-skill-cases.md) for realistic task walkthroughs.
Frontmatter/link hygiene, author evaluation and production native acceptance remain separate.
Adaptation provenance is in
[the repository record](../../../docs/work-plan/provenance.md). Old hosts/accounts, monorepo
paths, token generators, commands and inherited QA permissions are not part of this skill.
