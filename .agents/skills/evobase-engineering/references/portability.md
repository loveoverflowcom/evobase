# Portability, recovery and retirement

Read for package/snapshot export/import, offline/native profiles, restore or VOT/legacy retirement.
EvoBase owns the new product; ownership transfer does not complete the inherited obligations.

## Portable boundaries

Separate reusable definition, optional local data snapshot and host bindings. Pin package/model/
release/storage/binding/data versions and provenance. No real credentials, DB URL, grants or live
provider account in portable definition. A local snapshot cannot grant server membership/payment/
approval authority. Define supported local/server/WASM profiles and reject unsupported capabilities.

Decoder/record counts/bytes/depth/budgets and hostile/malformed corpus matter. A renamed `.votable`
JSON is not a versioned `.evobase` migration; extension/encoding remain ADR decisions. Restore
current auth/consent from live host facts, fence work and reconcile outbox/provider history.
Restore must not resurrect revoked grants or replay delivered effects.

## Recovery rehearsal

1. Inventory definition/data/binding ownership, identity mapping, licenses/notices and source SHAs.
2. Export a dedicated safe fixture; verify integrity/codec/profile and stage read-only impact preview.
3. Import into an isolated authorized target; revalidate Ref/constraints/revisions/release pins.
4. Reconcile provider outcomes/effects without live dispatch; compare independent query/command vectors.
5. Exercise failure at each stage, restart, rollback/resume and documented operator recovery.
6. Preserve source history and explain what cannot be restored/PITR'd from the chosen backup model.

## Retirement gate

Independent EvoBase builds/tests and required E1–E5 support envelope pass before removing legacy
app/service/routes/build references. Preserve still-used shared packages. VOT #773 is a separate
handoff/removal issue; public code work does not authorize private VOT retirement or production
data deletion. Obtain explicit execution/data approval at that boundary.

Record implemented/planned/deferred inherited obligations (#765/#772/#525/#570/#752/#698/#583).
Do not close source issues merely because the product owner or repository name changed. Missing
data/provider/device evidence blocks the corresponding claim, not all harmless documentation work.
