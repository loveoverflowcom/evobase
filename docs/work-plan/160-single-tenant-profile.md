# 160 — Select the single-tenant storage and authority profile

Status: active in the ten-commit single-tenant batch. Owner: #7/#6.

## Why

Resolve format, adapter, trust and evidence boundaries before behavior changes.

## Scope and acceptance

Deliver the named boundary under the [profile ADR](decisions/020-single-tenant-libsql.md).
Record executed checks and unsupported acceptance in [batch evidence](next-batch-evidence.md).

## Dependencies

Source/issue/skill inspection at the pinned baseline.

## Risks / unknowns

Remote Turso credentials and CMP are unavailable in this profile. Multi-tenant routing, production
migration and full release evolution remain deferred; local tests do not establish those claims.
