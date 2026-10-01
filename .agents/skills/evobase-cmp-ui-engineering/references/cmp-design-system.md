# Native design and resource adapters

Use for colors/type/shape, component states, localization, visual parity and accessibility.
Shared attention intent is [M3 Expressive](../../evobase-ui-engineering/references/material3-expressive.md);
shared strings/input policy is
[localization and accessibility](../../evobase-ui-engineering/references/localization-accessibility.md).

## Canonical input, explicit mapping

Resolve the pinned design handoff through
[shared UI policy](../../evobase-ui-engineering/references/shared-ui-policy.md). The sole canonical
token identity is `design/m3-expressive/source/tokens.json`. Planned build-time adapters derive
web CSS properties and CMP light/dark scheme, type and shape roles from those same bytes. An
implementation must choose actual generator ownership/tasks; none is asserted by this skill.

- Map every relevant `color.light`/`color.dark` role, including extended status roles outside
  framework `ColorScheme`; do not drop unsupported fields silently
- Adapt type size/weight/lineHeight, shape, spacing/layout, state layers, motion and elevation
  explicitly. JSON numbers need defined CSS px/native dp/sp mappings, not unit guessing
- Preserve token role identity and provenance; current curated values are not HCT output
- Do not hand-edit generated output, copy a palette into the app or independently call a seed
  generator. Dynamic color remains deferred until semantic-role/contrast safeguards exist
- Add deterministic mapping/drift tests at the adapter owner when implementation is requested,
  with independent expected role values and missing-role/invalid-input negative controls

Trace token → adapter → theme/extension → actual component → app mounting call site. Custom
Foundation controls can be valid if they consume roles and meet interaction/semantics contracts.
Adding Material3 or an outer expressive theme alone does not prove them restyled. Confirm the
pinned artifact supports a claimed expressive component/motion API on every affected target.

## Adaptation rather than pixel equality

Preserve meaning and relative hierarchy while choosing native geometry, focus/input, touch area,
scrolling, sheets and safe-area behavior. Web CSS px, native dp and image physical px differ.
Record density, fontScale, actual fonts and units. Compare web and CMP for semantic parity and
task clarity; do not pixel-diff unlike renderers as though they share a rasterizer.

Runtime mobile renders task lists, details, forms and workflow/approval actions. Forms retain
readable VI/EN labels, validation and a reachable submit when IME/large text changes the viewport.
Lists use stable keys and restrained selection/refresh feedback. Approval contains the record,
action and consequence, with current capability/pending/error state. No phone Builder canvas
is implied. Compact Builder access uses only the reviewed summary/handoff contract.

For visual parity, open attributable reference and current CMP-before capture before changing
styling. Keep reference | CMP-before | CMP-after, selected state and revision identity. Historical
design frames can guide intent but cannot establish current web/native behavior. When CMP does
not exist, produce adapter/state/evidence plans instead of invented “before” screenshots.

## Resource and semantic contracts

Use chosen typed/native resources for VI and EN, including labels, error/state descriptions,
action labels and parameterized counts. Generated field metadata references keys rather than
duplicated localized sentences. Reuse safe typed failure presentation instead of raw provider text.
Do not transplant a web macro or introduce a second i18n framework just for a label.

Stable selectors are locale-independent and scoped to record/control identity. Do not use a row
index, personal text, email or secret; hashing personal data is not automatically anonymization.
`testTag` selects, accessible name explains; assert both when relevant. Preserve independent
nested actions when merging/clearing semantics and include relevant dialog/sheet roots.

Native labels/roles/states/actions, focus order, disabled/pending/error feedback and touch targets
need host assertions; font wrapping/contrast need opened pixels. Real TalkBack/VoiceOver, Telex/IME,
safe area, autofill, Back/gestures and lifecycle require selected native runtime evidence. Follow
[testing and inspection](cmp-testing-inspection.md) without converting host results into OS claims.
