# 020 — Single-tenant libSQL execution profile

Status: selected for the next ten-commit batch; execution evidence is recorded separately.
Date: 2026-10-03. Base: `1ae1457a12a9894bfd35a550028e198ffaaacf38`.
Authority: user request for ten further commits and push to `develop`; issues
[#7](https://github.com/loveoverflowcom/evobase/issues/7),
[#16](https://github.com/loveoverflowcom/evobase/issues/16),
[#17](https://github.com/loveoverflowcom/evobase/issues/17),
[#18](https://github.com/loveoverflowcom/evobase/issues/18) and
[#19](https://github.com/loveoverflowcom/evobase/issues/19).

## Why and decision

A checked local draft needs durable execution through the same portable rules. The current
SubmitOrder fixture and legacy SQL-table server are separate references, not the generic host.
Implement one configured tenant first using a fixed-layout libSQL adapter. Local files need no
hosted account; the remote primary connection supports Turso/libSQL with server-only credentials.
Do not create business SQL tables for logical definitions. Multi-tenant provisioning, database
routing and cross-tenant isolation remain deferred; a configured tenant key is not isolation proof.

## Semantic compatibility

New constraints, state machines and commands use definition format 2. Historical format 1 stays
readable and canonically encodable with its original semantics, only when those new declarations
are absent. Unknown/future versions fail. Older decoders reject format 2 rather than dropping rules.
Stable IDs, exact signed numeric values, Blank/Null/absent and capture semantics remain unchanged.
Text lengths count Unicode scalar values, non-empty checks whitespace without rewriting stored
text, numeric ranges are inclusive. Optional missing values retain the existing requiredness rules.

## Storage and authority

Persist bounded canonical checked definition/fact snapshots, immutable content-addressed release
pins and independent data revisions. A serialized write transaction re-loads checked facts,
resolves current host authorization, compares the immutable request identity and expected revision,
then commits facts, audit, event/outbox intents and receipt once. Receipt replay first verifies
current grants and row policy. Request identity binds actor, scope, release, command, record,
expected revision and normalized typed inputs. Failure rolls back all state.

Use an operator-configured bearer pilot, explicitly separate from a production identity provider:
server-held token verifier, configured canonical actor/current roles/grants, expiry and revocation.
No body/header actor claim selects authority or storage credentials. Design/Read/Write/Submit are
independent; output projections and Runtime metadata enforce current policy. Bearer-only transport
does not introduce ambient cookie authorization. Remote credentials never enter portable AppSpec.

## Supported envelope and remaining work

Whole-app snapshots retain the core's 1 MiB, 4,096-record and decode/evaluation limits. This is a
small-app correctness profile, not a scale benchmark. Bootstrap pins a checked immutable definition;
draft editing does not activate it. Full publish/evolution/backfill and durable worker delivery are
future work. Event persistence establishes committed intent, not external delivery.

Run local real-libSQL constraints/CAS/race/rollback/reopen/process-restart tests and actual HTTP
authorization/protocol checks. Native/WASI vectors and browser interaction/capture evidence are
separate. Without a configured Turso account, remote provider tests are explicitly unrun. CMP,
hosted sync, provider delivery, legacy retirement and production migrations remain outside this batch.

## Obligation ledger

| Invariant / source | Owner / boundary | Failure | Cheapest adequate oracle |
|---|---|---|---|
| Declarative rules enforce every final state (#16/#17) | AppSpec raw → checked | invalid facts or illegal transition | exact core diagnostics, unrelated schemas, historical codecs |
| One durable mutation (#7) | libSQL transaction | partial facts/events/receipt or duplicate retry | real local DB rollback, CAS races, restart |
| Current access before replay/output (#6/#18) | verified host → policy → store | forged/revoked success or hidden data | actual HTTP negatives plus transaction revalidation |
| Wire preserves meaning (#19) | bounded DTO → core | precision loss or authority injection | exact serializer/decoder and hostile vectors |
| UI reports durable truth (#9/#16) | mounted Builder/Runtime | false saved or fresh-key retry after lost ACK | browser flows against actual fixture host |
