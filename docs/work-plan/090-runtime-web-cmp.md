# 090 — Deliver authorized Runtime desktop and CMP forms/actions

Design issue: [EvoBase #9](https://github.com/loveoverflowcom/evobase/issues/9). Assets pinned at `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`.
Status: proposed future implementation. Design pack: 07 runtime/forms/approval.
Skills: engineering + Web + CMP; inspector for evidence review.

## Scope / dependencies

050 authority, 070 pinned releases, 080 only for upgraded data. One permitted order list/detail/form
and approval task presentation on Leptos and CMP. CMP starts Runtime-first; complex authoring has
desktop handoff. Share Rust semantics through reviewed bounded API/FFI contract; Kotlin owns only
presentation/lifecycle. Real durable function wait/dispatch is PR100, not an animation/status badge.

## Acceptance / required evidence

- Canonical tokens/components map to actual mounted CSS/CMP consumers, light/dark/VI/EN and fontScale.
- Server/current actor grants and release pin determine fields/actions; forged selectors/revoked
  grants fail, hidden inputs preserved and exact errors visible.
- Values/events separation, loading/error/offline/conflict/unsupported/repeated/stale/cancel/back
  state, lifecycle/focus/IME/scroll and safe exact-key recovery across interruption.
- Native session adapter and API decoder validate profiles; browser identity isn't implicitly native.
- Browser DOM/keyboard/interaction + CMP production semantics/host tests + opened pixels separately.
- Selected actual Android/iOS/native target runtime gates reported; Linux host proof not iOS/device.

## Future execution prompt

> After selecting PR090, pin core/authority/release and design pack07, inspect available actual
> Web/CMP targets and decide minimum supported native target. Build one runtime order/approval
> vertical slice consuming the same checked Rust semantics, not duplicated Kotlin permission or
> validation rules. Native presentation owns local focus/navigation/loading while session/API/FFI
> adapter preserves current authority, pinned release and typed diagnostic/idempotency contracts.
> Mount canonical M3 tokens and component state contracts in both real consumers. Cover repeated
> submit, delayed obsolete result, revoked grant, hidden input preservation, interrupted save,
> Cancel/Back/Forward, IME and large text/touch semantics. Run named browser DOM/input checks and
> real CMP component/controller/host tests, capture and actually inspect representative frames.
> Execute selected native runtime gates where tools/permissions allow; disclose blocked device/iOS
> coverage, never infer it from responsive browser or Skiko. No live provider, arbitrary mobile
> Builder, general workflow engine, merge or deployment. Stop at usable authorized Runtime gate.
