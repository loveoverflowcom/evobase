# 080 — Rehearse safe schema evolution and recovery

Status: proposed future implementation. Design pack: 06 upgrade/evolution.
Skills: engineering → isolated-storage/AppSpec/portability; Web for impact/recovery UI.

## Scope / dependencies

060 transactions and 070 immutable pins. One supported type/required/constraint evolution with
impact preview, bounded backfill, revalidation, atomic activation and recovery. No-DDL does not mean
no migration. Production migration, destructive cleanup and broad automated tenant fan-out excluded.

## Acceptance / required tests

- Independent model/storage/binding/data revisions and migration state; pins untouched until activation.
- Safe optional addition versus incompatible type/required/relation/captured changes classified exactly.
- Preview counts/examples authorized and zero effects; invalid/missing data blocks activation.
- Bounded resumable backfill/CAS/current grants, concurrent edits and interrupted/restarted stage.
- Atomic cutover preserves checked refs/unique/derived structures; partial failure cannot corrupt pins.
- Rollback/recovery semantics explicit, audited and tested with original data/hash comparison.
- Old pinned instances/reads supported or explicit typed incompatible block; no silent conversion loss.
- UI shows stage/impact/blocked/conflict/recovery and cannot treat preview as successful activation.

## Future execution prompt

> Select PR080 only after release/storage gates and one concrete migration contract is reviewed.
> Implement the smallest bounded evolution path on dedicated fixtures, including pure impact preview,
> backfill/revalidation, atomic activation and restart/rollback recovery. Keep release/schema/binding/
> data revisions separate. Do not convert unsafe changes silently or repin retained old instances.
> Test incompatible/required/Ref/capture change classification, invalid existing rows, concurrent
> writes, stale permissions/CAS, failure after each stage, real process restart and final catalog/
> facts/pin/derived invariant comparison. Surface precise authorized diagnostics and progress/blocked/
> conflict/recovery states in Leptos M3 design. Add a reproducible isolated rehearsal and recovery
> runbook with exact executed commands and bounds; no invented no-migration claim from fixed layout.
> Preserve source data/history and disclose irreversible/non-supported steps. No production migration,
> destructive cleanup, automatic mass tenant upgrade, merge or deployment. Stop at the reviewed
> migration/recovery gate; another change family needs a separate supported contract.
