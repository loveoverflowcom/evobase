# 250 — Harden and verify the integrated profile

Status: active in the ten-commit single-tenant batch. Owner: #7/#18/#19.

## Why

Exercise real storage/HTTP/browser boundaries, review invariants and publish ten commits.

## Scope and acceptance

Deliver the named boundary under the [profile ADR](decisions/020-single-tenant-libsql.md).
Record executed checks and unsupported acceptance in [batch evidence](next-batch-evidence.md).

## Dependencies

The earlier numbered items supply this batch's contracts; independently test this boundary before integration.

## Risks / unknowns

Remote Turso credentials and CMP are unavailable in this profile. Multi-tenant routing, production
migration and full release evolution remain deferred; local tests do not establish those claims.
