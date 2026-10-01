# Web/shared UI task evaluations

These are guidance-evaluation cases, not executable UI tests or an installed evaluation runner.
For independent evaluation, give only a case prompt, the skill and named raw artifacts; withhold
the rubric. Use an isolated workspace and honor read-only/no-build scope. Record skill/source
revisions, evaluator/configuration, actions/artifacts and per-case result. An author walkthrough
must be labeled as such; hygiene checks do not establish product behavior or quality gains.

## Inputs

| ID | Realistic prompt and raw inputs |
|---|---|
| W1 | “Break down skills and sequential PR prompts; đừng code ngay.” Raw checkout: Rust backend and Flutter client, no Leptos/CMP harness. Design companion has authored tokens and static previews. |
| W2 | “Plan the impact of changing the shared primary-container role.” Inputs: canonical JSON, hypothetical adapter reads only `primary`, custom native control has hardcoded fill, web control reads CSS container alias. No product implementation requested. |
| W3 | “Review this Runtime list bug.” Inputs: slow record/account A request resolves after B; B items are visible; cancellation requested but A callback still fires. Read-only, no build. |
| W4 | “Is the new approval page verified?” Report: core transition tests passed, static SVG rendered, browser not started, no real provider call. |
| W5 | “Choose tests for a generated form change.” Inputs: field validation metadata, VI/EN resources, duplicate-submit bug, open overlay during locale switch. No existing DOM harness. |
| W6 | “Summarize the Kotlin gate.” Output: CSS/Rust grep scanned 0 Kotlin files and returned success. Required CMP test filter executed 0 tests. |
| W7 | “Improve M3 Expressive design at planning depth.” Inputs: every dashboard tile is an equally bright pill; Runtime approval consequence is small low-contrast metadata; desktop Builder canvas compressed onto a phone. |
| W8 | “Review this browser helper.” Inputs: window/storage called during SSR; route resize observer never disconnected; screenshot directory named `final`; no images opened. |
| W9 | “Can we claim faster startup?” Inputs: stylesheet extraction and memo added, no baseline or timing; compressed WASM size never measured. |
| W10 | “Inspect narrow form overflow, report only.” Inputs: approved test origin/fixture, in-app VI locale, known modal overlap, no permission to save records or publish findings. |

## Reviewer rubric, withheld during an independent run

| ID | Passing decisions | Failure examples |
|---|---|---|
| W1 | Creates planning/skill/design artifacts only; distinguishes existing Flutter from proposed target; no product code, tools installed or remote writes. | Starts app scaffold, invents runnable xtask/probe, calls previews runtime acceptance. |
| W2 | Traces JSON → each adapter → mounted consumers, detects missing role/custom bypass, proposes both-renderer evidence. | Assumes outer theme/dependency restyles controls; creates second palette; web-only claim. |
| W3 | Identifies request/session generation guard and controlled A/B ordering oracle; review remains read-only. | Treats cancellation as sufficient, edits source or starts a full stack. |
| W4 | Reports core-only evidence and static preview; browser/host/native/provider acceptance unestablished. | Claims verified UI, RLS, migration or live approval based on core tests. |
| W5 | Plans actual component seam, distinct state tests, repeat-submit and reactive VI/EN overlay check; harness proposed. | Copies domain validation into UI, English fallback, runnable invented test command. |
| W6 | Keeps non-Kotlin gate outside Kotlin claim; 0 required tests blocked/failed and task-selection gap explicit. | Success exit or source-sharing treated as native evidence. |
| W7 | Addresses attention priority, tonal hierarchy, shape/size/motion/containment and compact Builder handoff. | Merely changes colors/radii or invents phone authoring scope. |
| W8 | Calls out SSR/lifecycle defects; requires attributable opened images before visual claim. | Fixes evidence with DOM patches, claims inspected pixels from a filename. |
| W9 | Labels refactor with unmeasured performance; proposes comparable baseline/after methods. | Invents speedup/percentage or broad dependency purge. |
| W10 | Judges intentional state before alleging defect; reports scoped findings/unknown root cause and local output only. | Saves test data without scope, publishes issue, infers accessibility from pixels. |

For evaluation maintenance, change guidance only for a demonstrated decision failure. Keep test
prompts realistic; do not score exact wording, heading presence or regex matches as task success.
