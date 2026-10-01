# EvoBase · M3 Expressive screen system

27 complete screen references, desktop + compact, plus light/dark component representatives. Original static SVG frames and PNG raster previews; semantic HTML/CSS/MJS fixture prototype; 12 lightweight source feature packs (expanded raster review packs delivered separately); shared tokens, component, accessibility and implementation contracts.

**Status:** design and planning only. No production Leptos/CMP/Rust runtime implementation; no browser/native/provider pass is claimed. Sample records are fictional. The refined mobile Runtime favors concrete contained task/order rows; Builder authoring is desktop-focused with compact summaries.

## Review

- `frames/`: all27desktop/compact layered SVG anatomy; issue-scoped PNG representatives in `previews/`. Expanded PNG review packs/overviews are delivered separately through Library.
- `source/index.html`: local fixture prototype (serve this folder through an ordinary approved static HTTP server; no external network calls)
- `source/tokens.json`: canonical authored token contract; `tokens.css` generated
- `frames/`: layered editable original SVG, not screenshots
- `specs/screen-map.md`: screen-by-screen goals, flows, states and target gates
- `specs/component-contract.md`: M3 family mapping, web/CMP adapters and behavior
- `specs/accessibility-and-states.md`: accessibility + interrupted/repeated-flow acceptance
- `specs/source-to-design.md`: grounded baseline, open decisions, provenance
- `packages/pack-index.json`: issue-scoped ZIP inventory/checksums

`python3 build.py` regenerates token CSS, fixture data and contact sheets. `python3 render_static.py` renders layered frames to PNG using Inkscape and bundled fonts; browser automation is not used. This design source has no runtime dependencies other than Python/Pillow and an existing trusted Inkscape install for static export.

## Recommended issue order

01 Foundation and shells → 02 Typed grid/import → 03 Relations/formulas → 04 Authority/policies/commands → 05 Hosted isolated-store spike → 06 Preview/release/evolution → 07 Runtime web/CMP → 08 Durable functions → 09 Connectors → 10 Messaging → 11 Scoped bot → 12 Portability/recovery.

The companion plan intentionally splits AppSpec kernel vs grid and release vs schema evolution into separate PRs. Each ZIP is independently reviewable, while imports must preserve the canonical token identity rather than fork it. General 1:1/N:M, arbitrary migrations, provider delivery and offline authority remain gated future capabilities.
