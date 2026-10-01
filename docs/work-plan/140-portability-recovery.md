# 140 — Establish an explicit portable/local recovery profile

Status: proposed future implementation. Design pack: 12 portability/offline.
Skills: engineering → AppSpec/portability/storage/functions; CMP if local native profile selected.

## Scope / dependencies

020 canonical codec, 080 evolution/recovery, 100 durable fencing. Review package/extension/local
profile ADR first. One definition export/import plus optional safe local snapshot/recovery profile;
offline operation/merge/connector behavior supported only when separately evidenced. No renamed JSON
as migration, cross-host grant transfer, silent effect replay or production restore.

## Acceptance / required tests

- Separate definition/state/bindings and all revisions; version/provenance/integrity/bounds explicit.
- No secrets/DB URLs/real authority in template; local snapshot cannot create server approval/payment.
- Native/WASM/server golden vectors within selected profile, unknown capability typed reject.
- Isolated export/import/restart and failed-stage resume/recovery with Ref/constraints/pin/revision compare.
- Live grants/consent re-resolved; revoked membership not revived; fences/reconciliation stop duplicate
  provider effects after restore.
- UI reports supported/offline/read-only/blocked-conflict and data destination/impact accurately.

## Future execution prompt

> Select PR140 after reviewed codec/recovery/fencing contracts and choose the smallest explicit
> local/portable profile. Implement a versioned definition artifact and, only if selected, bounded
> local snapshot import/export/restart path with separated host bindings and live authority.
> Preserve provenance/notices/identity mapping and reject unsupported versions/capabilities. Test
> hostile package bounds, golden round-trip/native-WASM-server vectors, isolated migration/import,
> failure/restart/recovery and no revoked-grant resurrection or delivered-effect replay. Define
> fencing/provider reconciliation before restore can dispatch anything. Bind portable/offline/
> read-only/unsupported/impact states in Web/CMP to actual supported behavior, not a universal
> offline promise. Real data upload/restore needs destination/data authorization; work with dedicated
> safe fixtures until approved. Report exact compatibility/recovery profile, commands and unrun
> targets/provider boundaries. No production restore, destructive cleanup, broad offline sync,
> merge or deployment. Stop at reproducible portable/local gate for retirement assessment.
