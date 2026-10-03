# M3 Expressive for EvoBase

Use for hierarchy, composition, adaptive layouts, shape and motion. Canonical numerical values
and component states come from the design handoff in [shared UI policy](shared-ui-policy.md).
This is product guidance, not an automated compliance gate or a new palette.

## Intent before styling

Name the user's goal, primary content, next important action, secondary tools and density.
Expressive design uses **color, shape, size, motion and containment** to direct attention; a
Material dependency, blue palette and rounded cards do not establish that result.

| EvoBase context | Composition and expressive intensity |
|---|---|
| Builder home/package overview | Medium: clear package identity, useful progress/diagnostics, one primary next action |
| Entity/refinement/workflow authoring | Low–medium: focused working canvas/editor, quiet inspectors, legible graph/type distinctions |
| Verification/change preview | Low: evidence and actionable diagnostics lead; clearly distinguish draft, pending, passed, failed and unknown |
| Runtime list/detail | Medium–low: record/task content leads; readable statuses, familiar list/grid and one next action |
| Generated form/workflow approval | Low: predictable fields, consequence and confirmation context, visible validation and pending states |
| Auth/security/destructive actions | Low: trust, familiar input, precise outcomes and restrained motion |

These intensities guide judgment. They are not rules that all screens must have a hero, card or
singular button. Independent business tasks may need several actions; establish relative priority
without hiding required choices. Do not import a paper-reading brand or unrelated product metaphor.

## Five levers with observable acceptance

### Color and tonal hierarchy

Consume semantic paired roles (`primary`/`onPrimary`, containers, surfaces, errors and extended
status roles) from the canonical JSON. Use tone/space for grouping and outline where it improves
structure or a control's affordance; a categorical ban on borders harms data-heavy screens.
Dark mode is a designed composition, not inversion. Inspect resolved foreground/background pairs
including focus/error/selection, overlays and state-layer alpha in both themes.

Good: the primary workflow action has strong contrast while metadata remains quiet.
Counterexample: every toolbar item, status and panel is equally saturated; color alone marks risk.
Oracle: opened matching captures plus resolved contrast and semantic-state assertions, not a hex scan.

### Shape and geometry

Use shape roles to distinguish interactive actions, work regions and entities. Keep recognizability,
hit area and keyboard focus. Expressive transformations may signal press, expansion or a changed
task state when supported by the target; do not add arbitrary asymmetric containers everywhere.
Adapt CSS geometry to native dp intentionally rather than requiring identical pixels.

Good: action/state shapes and sheet containment reinforce navigation.
Counterexample: every data cell becomes a pill, or static cards lift on hover as though clickable.
Oracle: semantic role/action plus opened capture and actual hit-area/focus evidence.

### Size, typography and density

Use named type roles and relative hierarchy; avoid component-local type scales. Runtime touch
targets use the canonical target contract. Desktop grids may be dense, but their controls remain
reachable and legible. Loaded fonts and Vietnamese diacritics need actual inspection; naming a
font in CSS does not prove it loaded. Numbers use the data role where tabular alignment helps.

Good: record title/action is distinguishable from metadata, long VI labels wrap predictably.
Counterexample: headline, body and metadata have nearly equal weight; larger type clips a submit.
Oracle: loaded-font identity, narrow/large-type captures and geometry assertions.

### Motion and feedback

Use motion to communicate navigation, expansion, state-layer feedback or processing. Preserve
immediate feedback when reduced motion is enabled. Do not turn a canonical easing/duration into
a claim of spring physics: verify pinned API support and record the actual renderer mapping.
Avoid decorative endless loops and motion that changes a critical control's target mid-input.

Good: a panel transition explains origin/destination; pending state disables duplicate submission.
Counterexample: static capability cards bounce, or success animation hides a failed server outcome.
Oracle: driven state transition plus timed capture/trace where motion is claimed; static pixels
cannot establish smoothness. Reduced-motion behavior has its own driven check.

### Containment, rhythm and adaptive layout

Group related work with meaningful sections and tonal surfaces. Avoid both nested-card chrome and
unstructured floating controls. Use the canonical spacing rhythm; inspect baseline alignment,
line length, wrapping and information density before adding shadows or containers.

Builder desktop authoring uses working region + appropriately sized navigation/inspector. At
compact widths present a readable summary and higher-width handoff; do not shrink a full graph
editor or make unavailable edits appear supported. Runtime compact/mobile uses list/detail/form
and workflow action patterns, with modal/sheet navigation and safe-area/IME handling at the native
owner. Collapse or move secondary tools without losing record/task context.

Good: mobile detail retains context and a reachable next action when the keyboard is open.
Counterexample: a desktop inspector steals the phone viewport, or an approval action is clipped.
Oracle: resize/orientation/input-state evidence and correct resulting navigation/focus state.

## Review order and exceptions

Open attributable frames before judging. Inspect identity/content/action priority, then the five
levers, then alignment/spacing/wrapping and changed states. Read
[localization/accessibility](localization-accessibility.md) and the renderer evidence reference.
Use concrete symptoms and user impact; a “three-second scan” is a heuristic, not measured research
or an automatic severity rule. Distinguish bug/contract deviation/usability hypothesis/polish.

If a platform adaptation differs, explain its role, target constraint and oracle. Dynamic color
is deferred until semantic-role and contrast safeguards are designed and tested; do not enable it
by default or independently replace the canonical palette. Design previews establish static
composition only. Final acceptance requires the actual production-mounted renderer's evidence.
