# VI/EN, semantics and accessible states

Use when UI copy, form feedback, status, keyboard/focus, labels, layout or motion changes.
Follow the canonical component/state specifications named in
[shared UI policy](shared-ui-policy.md). Reuse the localization owner actually present; the new
Leptos/CMP resource adapters are proposed until implemented. Select their format deliberately
in the foundation PR rather than inventing competing web/native copy systems.

## Localized contract

- Inventory every changed visible and accessible string: headings, buttons, placeholders,
  help/errors, empty/loading states, menus, toasts, labels, descriptions and action/state names
- Use semantic keys or the chosen typed resource API in **VI and EN**. Do not force a web macro
  into CMP or a native-only framework into Leptos
- Use parameterized resources for dynamic values and framework plural/date/number handling;
  do not concatenate translated fragments or render internal enum/IR identifiers as user copy
- Map typed domain/provider failures to safe presentation messages. Validation diagnostics
  should identify the actionable field/rule; technical/provider details stay out of the UI
- Localization fallback belongs to the resource layer. Do not embed English fallbacks or call
  an untranslated string complete; report the specific missing translation for review
- Generated field labels/help/errors need provenance to the domain metadata and locale key.
  A generated form must not rewrite the underlying refinement or authorization rule

When changing a locale in-app, retain appropriate draft, selection and open-overlay state. Verify
labels, errors and accessible names update without relying on a full reload that bypasses the
reactive transition. Check VI diacritics, combining input, font fallback, long labels and realistic
data. User-entered names/content are data, not text to translate or put in a locator.

## Component states

For affected interactive controls, select default, hover where supported, focused, pressed,
disabled, selected/expanded, error and pending states. Screens may also require idle/loading,
empty, normal data, refresh/append, data-with-error, unauthorized and conflict states. Do not
collapse an error into empty or discard valid records while a refresh is pending unless the
contract says so. A disabled affordance must have a understandable reason where relevant.

Meaning cannot rely on color, opacity, icon shape or motion alone. Keep pending/outcome feedback
visible and semantically available. Verification/proof badges name their exact scope and revision;
a missing check shows unknown/pending rather than green success. Approval screens communicate the
target record/action and consequence before dispatching the same guarded domain intent.

## Web semantics and input

Prefer native HTML controls, landmarks, headings, associated labels and described error text.
ARIA supplements a real contract; it does not repair an arbitrary clickable element automatically.
Give each reachable control a visible focus indication. Test keyboard activation, Tab order,
Escape/Close/Cancel, focus entry/restoration, modal background behavior and resulting route/history.
“Dialog closed” includes no invisible overlay and appropriate focus/history after dismissal.

Use stable semantic selectors with localized accessible-name assertions. Do not use localized
text, index, email, record title, secret or token as the only persistent test identity. A selector
is not an accessible name. Preserve independent nested actions and relevant overlay roots.

## Native semantics and input

Compose semantics supplies localized name/role/state/error/actions. Choose merged/unmerged trees
deliberately. A `testTag` is not an accessible label; do not encode JSON/locators in
`contentDescription` or create a parallel AI-only control tree. Reuse real production components.

Host semantics/callback tests do not establish TalkBack/VoiceOver, IME/Telex composition, autofill,
safe areas, OS Back, gestures or lifecycle. Select device/simulator evidence for changed native
boundaries using [CMP testing/inspection](../../evobase-cmp-ui-engineering/references/cmp-testing-inspection.md).
Never claim raw Compose internals from an OS accessibility projection.

## Visual accessibility and evidence

Inspect resolved contrast pairs, non-color cues, target bounds, readable text and reflow in both
themes and locales. Canonical token calculations support only their measured pairs, not a whole
screen including transparency/images/gradients. If citing an external accessibility standard,
verify its version/clause from the authoritative source and explain applicability; an internal
threshold or heuristic is not automatic standards conformance.

Use the design's touch-target and focus-ring contracts with named units; CSS px, dp, physical px
and iOS points are not interchangeable. Large font/zoom states must retain critical actions and
readable flow. Reduced motion removes disruptive transforms/springs while preserving immediate
state feedback; verify the actual web/native adapter, not just a preference flag.

Keep evidence separate: resource inventory/static check, mounted semantics, driven keyboard/touch,
opened pixels, screen reader and native runtime. Document omissions. No source scan, green token
contrast report or screenshot alone establishes full accessibility acceptance.
