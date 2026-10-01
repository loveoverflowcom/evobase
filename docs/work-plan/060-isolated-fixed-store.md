# 060 — Prove isolated fixed-layout PostgreSQL storage

Design issue: [EvoBase #7](https://github.com/loveoverflowcom/evobase/issues/7). Assets pinned at `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`.
Status: proposed future spike, not production scalability claim. Design pack: 05 hosted isolation.
Skills: engineering → isolated-storage, authority, evidence/resource budget.

## Scope / dependencies

050 trusted host/checked commands and reviewed storage ADR. Two dedicated test tenant databases,
fixed engine tables, separate provisioner/migrator/runtime roles, bounded pools/app lock. Preserve
typed canonical records plus derived structures; do not copy concrete business-table DDL adapter.
No production DB migration, shared-RLS fallback or full scale guarantee.

## Acceptance / required real-boundary evidence

- Runtime no owner/superuser/BYPASSRLS/DDL/unneeded CONNECT; actual privilege/negative assertions.
- Authenticated registry routing, independent same-release tenant data/release changes, forged
  selectors/Ref/cache/subscription/blob/log/search paths cannot cross scope where implemented.
- Supported logical table/optional field/Ref and minimal storage activation/conformance seam succeeds
  without business-table/column DDL; full release lifecycle/UI/pinned-run acceptance remains PR070;
  compare real DB catalogs before/after, not mocked permission metadata.
- One transaction: checked facts, edges/indexes/unique claims, revision/audit/outbox/receipt/pin.
- Inject each stage failure, simultaneous unique/Ref-delete/CAS/idempotent/conflicting/release writes.
- Real process restart/data+receipt recovery; ignored or in-process tests cannot satisfy restart gate.
- Measured fixture sizes/read/write/join/rollup/lock-wait/storage/pool envelope with exact environment.

## Future execution prompt

> After selecting PR060, pin authority/core and inspect legacy provisioner credential hazards.
> Resolve fixed codec/layout/locking/role/pool ADR, then implement the smallest real PostgreSQL
> two-tenant spike consuming opaque checked commands. Separate runtime credentials from admin/
> provisioner/migrator; verified registry selects DB, never raw client IDs. Commit canonical facts,
> derived projections/claims/revision/audit/outbox/receipt/release pin atomically and authorize before
> receipt replay. Add real role/catalog/no-DDL/tenant-negative/concurrency/failure-injection/restart
> tests, with documented dedicated fixture provisioning and cleanup boundaries. Keep no-DDL release
> activation a minimal storage conformance seam; PR070 owns full lifecycle/UI/pinned-run behavior.
> Bound resources and
> measure the stated workloads; report data versus resource isolation and support envelope honestly.
> Discover or add a real supported test runner and retain exact logs/commands/test counts. No mocked
> DB result as integration proof, production data, automatic alternative isolation profile, merge
> or deployment. If a real DB/tool permission blocks acceptance, stop dependent claims and report
> the exact blocker. Exit only with reproducible E2 evidence or explicit unpassed gate for review.
