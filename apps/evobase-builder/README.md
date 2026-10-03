# Local Leptos Builder

This browser entrypoint mounts the typed AppSpec Builder with synthetic commerce
records. It edits a local draft and saves a checked receipt to this browser's
`localStorage`; it has no hosted authentication, server records or publishing.
Below 600 CSS px it offers a read-only draft summary and a wide-screen handoff.

The observable invariant is that invalid cell edits or TSV imports never partially
change the checked dataset. `evobase-appspec` owns parsing, identity, requiredness,
types, exact signed 64-bit values, captures and final-state validation. Leptos owns pending
input, focus, table selection, theme, locale and draft lifecycle. Table and record
keys are stable Rust identities; labels are presentation data.

Install the Rust `wasm32-unknown-unknown` target and `wasm-bindgen-cli` with the
same version as `wasm-bindgen` in `Cargo.lock`. Then run from the repository root:

```sh
scripts/build-builder.sh
python3 -m http.server 4173 --directory apps/evobase-builder/dist
```

Open `http://localhost:4173`. The build checks generated tokens before compiling.
Leptos 0.8.11 and its paired 0.8.10 macro are pinned; a newer semver-compatible
macro changed its generated typed-builder API and does not compile with this pair.

The canonical authored design input is
`design/m3-expressive/source/tokens.json` from the design handoff pinned at
`6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`. `generate_tokens.py` maps its exact
light/dark semantic colors, typography, spacing, shape, layout, states and motion
into `tokens.css`. `builder.css` consumes these roles in the mounted `App` rather
than generating another palette. The bundled Open Sans assets retain their
Apache license and notice in the build output. Regenerate after an intentional
canonical token change with `python3 apps/evobase-builder/generate_tokens.py`.

TSV headers use field identities from the visible header disclosure. Imports are
bounded to 100 data rows and 64 KiB, require exact column counts, and identify
invalid row/column or missing required field. Empty input means `Blank`; `null`
means explicit `Null`; text literals `null`, empty text or a leading `text:`
are represented with the `text:` escape. Money inputs are exact integer minor
units. No business number passes through JavaScript numeric JSON conversion.

Grid edits, imports and local saves enter the production `RelationStore` batch
commands. Existing captured prices and their product references are read-only,
and the Rust command rejects changes independently of that presentation state.
New line imports omit or leave the captured-price column blank; the checked Insert
derives it from the final catalog facts. Supplying a captured price rejects the
whole import with a row/column diagnostic. Catalog edits keep existing historical
prices intact. Restoring a prior local snapshot uses an explicit
`LocalPreviewPolicy` marker: this checks the stored graph without attesting to
historical provenance or granting hosted authority.

The relation inspector is an independent synthetic example using the same Rust
store and bounded pure query APIs. It demonstrates live price versus capture,
reverse references, subtotal/total and restrict-delete failures. These query
examples do not implement portable formula authoring or deployment.

The policy editor changes the canonical AppSpec policy and SubmitOrder rule arrays;
its callback marks that draft unsaved, and local Save/Cancel handles the same source
as grid edits. Rule authoring pauses while saving or during the interruption
simulation. A separate command sandbox exercises fixture actors, independent host
grants, owner/current-role policies, normalization, immutable retry intent and
current-authority checks before receipt replay. Sandbox writes and receipts stay
in memory and are explicitly simulated; there is no verified host/API adapter or
atomic persistent command transaction here.

The data-rule inspector authors canonical AppSpec constraints for text non-empty,
Unicode character length and inclusive integer/money ranges. Adding a rule upgrades
the definition to format 2, compiles it and validates all existing records before
changing the draft. Invalid rules retain both inputs and prior facts. Grid, form,
paste/import, save and restore use the same Rust evaluator; optional Blank/Null
values remain optional. Save/Cancel include these declarations with the definition.

Local persistence stores opaque canonical definition and record JSON strings.
Restore passes through the checked Rust codecs, including their byte/depth/node
budgets. Failed restoration preserves the source bytes and disables saving in
that session. Web Locks serialize cross-tab writes and compare the exact prior
snapshot; stale writers retain their input and show a conflict. Unsupported lock
APIs decline saving. Repeated saves of unchanged canonical content reuse the
same revision. Pending writes disable editing and cancellation until the receipt.
The interruption control is explicitly a local session simulation.

The browser runner drives actual mounted controls and stores sanitized synthetic
captures in an explicitly supplied artifact directory:

```sh
PLAYWRIGHT_MODULE_PATH=/path/to/playwright/index.mjs \
AXE_MODULE_PATH=/path/to/@axe-core/playwright/dist/index.mjs \
BUILDER_URL=http://localhost:4173 EVIDENCE_DIR=/path/to/qa-run \
node scripts/check_builder.mjs
```

Its report distinguishes DOM scenarios, axe findings and captured images; pixels
must be opened separately for inspection. Browser composition events do not prove
native IME, screen-reader or CMP behavior. The local fixture does not establish
hosted conflict resolution, auth expiration, deployment or provider behavior.
