# 240 — Run generated lists and declared actions over HTTP

Status: completed for the bounded single-tenant profile; broader issue gates remain open. Owner: #9/#19.

## Why

Consume permitted metadata and durable receipts; preserve intent across lost acknowledgements.

## Scope and acceptance

Deliver the named boundary under the [profile ADR](decisions/020-single-tenant-libsql.md).
Record executed checks and unsupported acceptance in [batch evidence](next-batch-evidence.md).

## Dependencies

The earlier numbered items supply this batch's contracts; independently test this boundary before integration.

## Risks / unknowns

Remote Turso credentials and CMP are unavailable in this profile. Multi-tenant routing, production
migration and full release evolution remain deferred; local tests do not establish those claims.
