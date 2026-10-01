# 070 — Publish immutable releases with pure preview and views

Design issue: [EvoBase #8](https://github.com/loveoverflowcom/evobase/issues/8). Assets pinned at `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`.
Status: proposed future implementation. Design pack: 06 release/view.
Skills: engineering → AppSpec/authority/storage; Web for release/view designer.

## Scope / dependencies

060 transactional host; 030–050 authoring; reviewed release/version contract. Separate draft/model,
checked immutable release and tenant runtime pin. Generated view/form/action exposes permitted
projection. Preview validates against controlled facts with zero deployment/provider effects.
Compatible release activation only; incompatible data evolution stays in PR080.

## Acceptance / required tests

- Design/Publish/run/manage rights independent and current at activation; preview is no grant.
- Draft edits have no runtime DDL/data/release change; immutable release IDs/hashes verified.
- Publish v2 cannot silently repin existing v1 business instances/runs; old semantics retained.
- Preview explanation/diff/examples use authoritative core, mutate neither DB/outbox nor provider.
- Activation rechecks current facts/CAS/permissions; repeated clicks and lost ACK recover original
  intent/key safely, conflicting publish surfaces deliberate conflict.
- Complete form inputs/projection fields obey policy; missing hidden inputs cannot erase facts.
- Views/forms/filters unsupported subset rejects; no unbounded arbitrary queries or duplicate rules.

## Future execution prompt

> After selecting PR070, pin storage/authority and M3 release/view pack. Implement compatible
> immutable release publication, app-instance pins and generated permitted views/forms using one
> checked definition. Draft preview is a pure simulation with explicit release/fact identity and
> zero real delivery; do not let preview modify deployment or grants. Preserve v1 rules/pins after
> v2 publish and retain old releases. Test activation permission/CAS/fact recheck, concurrent/repeated
> publish, interrupted response recovery, old pin semantics and no-preview mutation/outbox. Bind
> Leptos impact/diff/diagnostic/save/publish states to real results and label simulation accurately.
> Verify form inputs are complete according to command contract and that output projection cannot
> expose hidden fields through views/filters/export. Run real core/DB/API/browser checks and inspect
> compatible/blocked/conflict/pinned preview frames. No incompatible backfill, live provider, default
> branch change, merge or deployment. Report exact supported publish envelope and stop at immutable
> release/zero-effect preview gate; PR080 handles data migration separately.
