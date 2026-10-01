# 030 — Implement typed Leptos grid authoring and safe import

Status: proposed future implementation. Design pack: 02 grid/field/import/ref.
Skills: engineering + Web; UI inspector for scoped review evidence.

## Scope / non-goals

First mounted Leptos Builder consumer of PR020: select table, inspect/add supported optional field,
edit typed draft cells and resolve import diagnostics. Implement real keyboard/IME/paste/save state,
not only static cards. Fixture/local draft preview may be simulated and labeled. No hosted publish,
tenant auth, live provider, advanced formulas, arbitrary SQL or duplicate Kotlin domain validator.

## Dependencies / risks

020 checked API/vectors and canonical design commit. Confirm actual Leptos version/API/toolchain and
token adapter mounted by the app. Draft UI cannot silently mutate deployed schema or runtime records.

## Acceptance / required evidence

- Stable selectors/IDs, Ref chips display labels but emit identities; labels never become keys.
- Bounded paste/import: invalid type/missing/ambiguous target has precise row/column diagnostic and
  explicit resolve; no guessed links or partially committed corrupt batch.
- Keyboard/focus/IME composition, repeated Save, dirty/cancel/back, stale async response and conflict
  states preserve input and history; saves have deterministic idempotency/receipt contract.
- Canonical tokens → Web adapter → mounted controls; expressive hierarchy/grouping/data typography.
- DOM/interaction/accessibility and inspected wide/narrow light/dark VI/EN/error/conflict frames;
  fake-backed local draft evidence is labeled, not hosted integration or CMP proof.

## Future execution prompt

> Select PR030 only after PR020 gate and pin reviewed M3 pack02. Inspect actual source owners and
> build one real Leptos typed grid/field/import slice consuming Rust checked authoring commands.
> Resolve canonical tokens into one Web adapter with actual mounted consumers. Keep draft data and
> pure preview separate from deployed runtime and show simulated/local state honestly. Preserve
> data-dense numeric typography and M3 Expressive hierarchy/grouping/shape/state, keyboard/IME/paste,
> focused error diagnostics and dirty/save/conflict behavior. Cover ambiguous Ref import without
> guessing, repeated submit, Cancel/Back and obsolete async completions with production-path tests.
> Drive selected browser scenarios, capture and actually inspect changed pixels; record DOM/input/
> accessibility and viewport/theme/locale identities separately. Discover real runner commands;
> no invented VOT harness or responsive-browser-as-CMP claim. Run formatting/focused/final aggregate
> checks and report exact passed/failed/unrun stages. No hosted publish, DB DDL, real provider or
> merger/deployment. Stop at a usable checked draft-grid acceptance gate and prepare PR040 handoff.
