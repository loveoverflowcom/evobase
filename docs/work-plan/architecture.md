# Reboot contract and architectural decision gates

Status: **selected target architecture and experimental foundation contract; host/runtime gates remain planned**.
Authority: [EvoBase #1](https://github.com/loveoverflowcom/evobase/issues/1), latest selected stack,
and the reviewed design handoff. Earlier `re-design/` describes a Rust-DSL/generated-runtime vision; its chapters now point to the
[reconciliation](../../re-design/README.md). [ADR010](decisions/010-foundation.md) selects version 1
JSON, exact whole Money, stable identities, N:1/restrict and browser local-draft/host boundaries.
The current user request authorizes the five-commit batch on `develop`; it does not establish the
future hosted/native/provider evidence described below.

## User-facing ownership

Desktop Leptos Builder: Data → Structure/relations → Rules/policies → Functions → Connections →
Release. Draft authoring is separate from trusted runtime operation. Runtime desktop/mobile shows
only permitted views/forms/actions/tasks/runs/inbox. CMP starts Runtime-first; complex authoring
offers explicit desktop handoff, not a cramped hidden desktop editor.

The product is typed no-code SaaS, not an Excel/SQL clone. A user who knows no Rust/SQL must author
supported relations and business rules. Expert text/Rust frontends are optional projections.

## One semantic authority

Builder typed intent → bounded raw AppSpec → checked definition → immutable release.
Trusted host facts/current grants + checked release + current records → deterministic Rust query/
command decision → checked projections/writes/effect intents → tenant host adapters.

Definition, tenant data and host bindings are separate. Stable table/field/record IDs own meaning;
labels/translations/order do not. Refs store identity; inverse edges/indexes are derived. Policy and
command/formula share type vocabulary and binding graph; they have different effect semantics.

| Boundary | Proposed owner / immediate consumer | Cannot own |
|---|---|---|
| Raw/checked AppSpec + IDs/codecs | narrow Rust model/kernel crate, PR020 | DB/network/real grants |
| Pure relation/formula/policy/command decisions | narrow Rust engine, PR040/050 | provider I/O, Kotlin copies |
| Verified identity/registry/current grants | server host adapter, PR050 | client headers/portable artifact authority |
| Fixed-store transactional executor | PostgreSQL host adapter, PR060 | editable derived truth, live network waits |
| Durable worker + provider ports | host worker, PR100/110 | arbitrary native code, exactly-once claims |
| Builder/Runtime presentation | Leptos/CMP mounted consumers, PR030/090 | trusted tenant routing or business authority |
| Canonical semantic design tokens/contracts | companion M3 source | independently regenerated renderer palettes |

Proposed directory choices are resolved by the first real consumer, not scaffolded now. Prefer
small extraction at existing seams over a generic plugin OS/event bus or one giant context crate.
Current `evobase-core` mixes host/storage traits with shared types; do not assert it is already the
target pure kernel. VOT model/engine depend on narrow identity/directory types; adapt just the
needed invariants, not the whole authentication monorepo.

## Isolation and transactions

Baseline candidate: database-per-tenant + provisioned fixed engine layout. User logical tables/
fields/policies are metadata and typed records. Runtime has no DDL; provisioner/migrator uses
separate privileges/credentials. Host registry resolves DB from authenticated current membership;
URL/body selectors never choose credentials. Current auth precedes receipt replay and dispatch.

Command writes/derived edges/indexes/unique claims/revision/audit/outbox/receipt commit or roll back
together. Begin with a bounded app-instance lock if justified, measure contention; later optimization
cannot weaken invariants. Fixed layout avoids routine business DDL, not schema evolution. Database
separation on one cluster is not physical/resource isolation or automatic per-tenant PITR.

Legacy hazards: EvoBase tenant provisioner rewrites the admin URL using the same credentials for
runtime; VOT materializes concrete business SQL tables with DDL rights. Neither is the proposed
least-privilege fixed-store implementation. Keep those sources as references/history, not production
shortcuts. Shared RLS/schema-per-tenant and compiled SQL adapter require separate ADR/conformance.

## Functions and provider reality

Typed finite plan; immutable run pin; durable run/step/checkpoint/deadline/lease/fence state;
bounded retries and current consent. Business mutation commits before provider I/O. At-least-once
internal delivery + idempotency does not guarantee exactly-once email/Zalo/payment. Lost ACK becomes
OutcomeUnknown/reconciliation. Preview has zero external delivery; simulation badges are truthful.

Email/webhook first. Zalo capability/permissions spike later. Messaging system receipts/provider IDs
are managed and cannot be forged by generic cell edits. Bot is a scoped service actor using allowed
projections/checked commands and approval; retrieved content never expands tool rights.

## Blocking decisions and evidence exits

| Decision | Resolve by | Required evidence / stop condition |
|---|---|---|
| AppSpec name/version/encoding/extension and bounded profiles | [selected in 010](decisions/010-foundation.md), evidence in 020 | version 1 JSON; golden compatibility + hostile decode + no authority/bindings in package |
| Money/null/Blank/Ref/capture and supported cardinalities | [selected in 010](decisions/010-foundation.md), evidence in 020/040 | exact whole i64 Money; distinct Blank/Null; N:1/restrict; independent arithmetic/graph vectors; reject unsupported modes |
| Host session/native identity + grants/policy subset | 050 | forged/revoked/wrong-scope/old-new/output-leak negatives |
| DB fixed codec, lock/roles/pool/support envelope | 060 | two DBs, no-DDL catalogs, transactions/races/restart/bench |
| Release activation and incompatible migration | 070/080 | immutable old pins, impact/backfill/cutover/recovery |
| M3 token mapping and minimum renderer/platform targets | 030/090 | mounted source→adapter trace; DOM/native semantics + inspected pixels |
| Functions state/fencing/provider unknown outcome | 100/110 | actual restart/lost ACK/duplicate/late lease/current-consent tests |
| Portable/local/offline/restore profile | 140 | isolated round-trip/restart/reconciliation; no revived grants/effects |
| VOT/legacy retirement | 150 | source inventory + independent EvoBase gates + recovery/data approval |

Do not call any box implemented merely because its design screen, manifest or fixture exists.
Current VOT kernel formats 4/5 and hosted Web/API 1–3 have different envelopes; new EvoBase profiles
must be explicit. Tests/bench/provider/target evidence are future gates, not results of this document.
