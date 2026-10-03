# Isolated fixed-layout storage

Use for tenant provisioning, adapters, transactional state, constraints, no-DDL and evolution.
Database-per-tenant plus fixed engine layout is the isolation-first spike candidate from #1;
it is not an established production scale guarantee. No automatic schema/shared-RLS fallback.

## Write the ADR/support envelope first

Define canonical typed record codec; fixed release/records/edges/index/unique-claim/run/outbox/
receipt/audit structures; app/release revision; lock/isolation protocol; bounds; recovery. Business
tables are metadata, not physical DDL after every edit. Derived structures have one canonical
record owner and are rebuilt/updated within the chosen consistency boundary.

Tenant registry resolves storage from verified current context. Provisioner/migrator/runtime roles
and credentials are separate. Runtime is not owner/superuser/BYPASSRLS and has no DDL/unneeded
CONNECT/role membership. Do not reuse the legacy admin URL credentials for runtime databases.
No cross-database extension or search_path trick around the boundary.

## Transaction protocol

Acquire scoped lock/serialization boundary with explicit budget; load current membership,
immutable release pin and current facts; validate checked intent/CAS/idempotency; apply canonical
facts and derived edges/indexes/unique claims; update revision/audit/outbox/receipt; commit once.
After current authorization, an exact committed retry returns the original bounded receipt.
Do not hold transaction/lock while waiting for provider/network/AI/human approval.

## Acceptance evidence at real DB boundary

- Two independent tenant databases with distinct runtime credentials, same initial release.
- Forged tenant selectors/direct credentials/Ref/API/subscription/cache/blob/log/search/bot paths
  cannot reach another tenant. One API example is not complete isolation proof.
- Runtime role fails prohibited DDL/privileges; add supported logical table/optional field/Ref and
  publish without physical business tables/columns. Compare actual catalogs before/after.
- Concurrent unique claims, Ref create/delete, CAS, identical/conflicting retries and release races.
- Inject failure at each commit stage; facts/derived claims/revision/audit/outbox/receipt all rollback.
- Real process restart preserves committed data and receipt; ignored tests count as unrun.
- Measure fixture size, read/write/join/rollup cost, storage bytes, lock wait and pool bounds.

Database separation in one cluster is data isolation, not hard CPU/RAM/I/O isolation. Bounded pools,
idle eviction and fair scheduling need measured envelopes. Do not infer scale from CPU count.
No-DDL is not no-migration: incompatible changes require impact/backfill/revalidate/cutover/recovery.
Production provisioning/migration and destructive cleanup need separate explicit authorization.
