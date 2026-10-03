# 200 — Commit commands, receipts and events atomically

Status: active in the ten-commit single-tenant batch. Owner: #7/#17.

## Why

Serialize CAS and authorize before idempotent replay; rollback every commit stage.

## Scope and acceptance

Deliver the named boundary under the [profile ADR](decisions/020-single-tenant-libsql.md).
Record executed checks and unsupported acceptance in [batch evidence](next-batch-evidence.md).

## Dependencies

The earlier numbered items supply this batch's contracts; independently test this boundary before integration.

## Risks / unknowns

Remote Turso credentials and CMP are unavailable in this profile. Multi-tenant routing, production
migration and full release evolution remain deferred; local tests do not establish those claims.
