# CMP task evaluations

These cases assess guidance decisions, not product behavior. Independent evaluators receive only
the prompt, skill and raw fixture, with the rubric withheld. Use an isolated/read-only workspace
for planning/review. Record exact source/skill identity, evaluator/actions/outcomes and gaps;
an author walkthrough is neither an independent benchmark nor native acceptance.

## Inputs

| ID | Prompt and raw fixture |
|---|---|
| C1 | “Plan mobile delivery, do not code.” Existing checkout has Flutter and Rust; target asks CMP Runtime lists/details/forms/approvals; no Gradle app. |
| C2 | “Review this field API change.” `Column(Modifier.fillMaxWidth()) { BasicTextField(value, changed, modifier) }`; caller uses modifier for FocusRequester; input limit ownership unspecified. |
| C3 | “Does this menu need MVI?” Single remembered menu boolean with one toggle; no domain decision. |
| C4 | “Review concurrent save.” `state.update { api.save(); it.copy(saved = true) }`; alternatively a documented serialized owner assigns next state. |
| C5 | “Plan semantic token change.” Canonical JSON contains container role; adapter ignores it; custom field hardcodes fill; outer Material theme added. |
| C6 | “Can we report iOS acceptance?” Host suite passes, simulator framework linked, simulator not booted, images captured but unopened. |
| C7 | “Summarize Kotlin gates.” Web-only static script scans 0 `.kt`; Gradle required filter executes 0 tests; cached images have wrong source revision. |
| C8 | “Add native inspection at planning depth.” Proposal continuously serializes all lazy rows, tags contain emails, OS tree called raw Compose, no on-demand limits. |
| C9 | “Localize generated approval.” New visible/status/accessibility copy; VI/EN required; commonMain proposed Android-only dependency. |
| C10 | “Review extraction.” Moves fields into new composable, changes stable key and modifier target, session effect gains another subscription; no visual design change. |
| C11 | “Is capture overhead zero?” Source excludes a debug collector in release but no baseline measurements; host frame timing compared to phone release timing. |

## Reviewer rubric, withheld during an independent run

| ID | Passing decisions | Failure examples |
|---|---|---|
| C1 | Plans Runtime-first contracts/seams/PR evidence; current vs proposed explicit; no installation/scaffold or phone Builder. | Assumes CMP/FFI tasks exist or starts migration. |
| C2 | Traces callers, preserves focus/layout contract and asks/locates limit authority; proposes bounds/focus/semantics tests. | Moves modifier blindly or removes input bound. |
| C3 | Keeps lifetime-appropriate local state; no mandatory framework. | Requires ViewModel/reducer/FFI for the boolean. |
| C4 | Keeps update lambda side-effect-free and effect ordering explicit; accepts serialized assignment when justified. | Claims atomic update gives exactly-once I/O or bans all assignment. |
| C5 | Traces source→adapter→mounted consumer and role gap; no second palette, generated edit or theme-only acceptance. | Claims dependency addition establishes styling. |
| C6 | Reports host/compile evidence only; unopened remains captured; native/IME/screen-reader gates blocked. | Invents iOS coverage percentage or launch. |
| C7 | Reports failed/blocked scope/count/freshness; no Kotlin pass from web tool. | Calls all checks green or fabricates current manifest. |
| C8 | Plans bounded on-demand capture, safe selectors, actual tree source and privacy; no shadow UI or live commands. | Full-list collector, personal tags, invented deep-iOS backend. |
| C9 | Uses chosen VI/EN resource seam and localized semantics; verifies target/API compatibility; domain policy authoritative. | English fallback, Android dependency assumed common/iOS. |
| C10 | Maps lifetime/symbol/callers, preserves key/focus/subscription/callback behavior; selects actual state/host tests. | Shorter files treated as success or test-only preview clone. |
| C11 | Distinguishes source exclusion from measured overhead; proposes comparable inactive/active target measurements. | Zero-overhead claim, mismatched build/target comparison. |

Do not run a native build just to evaluate this prose. Execution of a real component/device case
requires the corresponding implementation/environment authorization and stays separately reported.
