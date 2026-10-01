# M3 Expressive companion design contract

Status: **published design specification; no product implementation evidence**.
Design branch: `design/m3-expressive-20261001`.
Immutable pin: [`6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`](https://github.com/loveoverflowcom/evobase/tree/6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e/design/m3-expressive).
All 142 selected public files were checked against current local Git hashes with zero mismatch.
Import this reviewed source identity into the rewrite before coding; do not fork its palette.

Canonical public handoff paths:

- `design/m3-expressive/source/tokens.json`
- `design/m3-expressive/specs/component-contract.md`
- `design/m3-expressive/specs/accessibility-and-states.md`
- `design/m3-expressive/specs/screen-map.md`
- `design/m3-expressive/specs/source-to-design.md`

The [published index](https://github.com/loveoverflowcom/evobase/blob/6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e/design/m3-expressive/README.md)
and [pack map](https://github.com/loveoverflowcom/evobase/blob/6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e/design/m3-expressive/packages/pack-index.json)
own 27 screens in 12 packs, 60 screen SVG frames and 44 representative PNGs. Two additional SVGs
are brand assets. Public per-pack ZIPs contain lightweight source; expanded review ZIPs and contact
sheets are separate private Library deliverables. Pack02 and pack06 each map to two implementation
PRs. The [verified issue map](issue-map.md) covers EvoBase #3–#14; bodies were read back with the
same immutable design/prompt pins. No product acceptance is inferred from these issue publications.

## Minimum design-to-implementation contract

Curated semantic M3 roles retain blue-family heritage; do not claim exact HCT-generated output.
One token source maps to generated/adapted Web CSS and CMP theme with explicit real consumers.
Expressiveness also requires hierarchy/type/shape/grouping/motion/state/interaction; it is not
a global rounded-card or primary-color restyle. Dense typed tables retain readable numeric/data
typography, keyboard/IME behavior and overflow handling.

Desktop Builder and Runtime are distinct. CMP mobile is Runtime-first: permitted forms/actions,
tasks/approvals, runs and messaging; complex builder actions give a clear desktop handoff. Routes
and URL IDs are navigation selectors, never tenant/actor authority.

Review representative wide/narrow, light/dark, Vietnamese/English, long strings, large font and
keyboard/touch states. Empty/loading/error/offline/saving/conflict/revoked/unsupported/unknown-provider
outcome cannot be left as pretty happy-path screenshots. Permission sources, immutable release
pins, preview zero-delivery and simulated/fake states remain visible and accurately worded.

## Honest evidence

Static SVG/PNG frames and semantic HTML/CSS fixture prototype are authored design specifications.
They do not compile Leptos, execute core policy, authenticate tenants, persist function runs or
render CMP. After implementation, obtain named production DOM/semantics/interaction and inspected
pixels, with selected native runtime input/lifecycle gates separately. Never cite design PNGs as
the implementation's visual test result. Keep draft state/fixture identifiers distinct from real
customer data, secrets, grants and provider accounts.
