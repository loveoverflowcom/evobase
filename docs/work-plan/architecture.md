# Reboot contract and architectural decision gates

Status: **experimental checked Rust core and Leptos local Builder implemented; verified host/storage/native gates remain planned**.
Execution scope and final artifact identities: [batch evidence](batch-evidence.md).
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

| Boundary | Actual owner or planned seam / consumer | Cannot own |
|---|---|---|
| Raw/checked AppSpec + IDs/codecs | `crates/evobase-appspec`, consumed by vectors and Leptos | DB/network/real grants |
| Pure relation/formula/policy/command decisions | `evobase-appspec::{relations,policy}`; local preview and checked host intents | provider I/O, Kotlin copies |
| Verified identity/registry/current grants | `SessionVerifier` trusted seam; FixtureHost/simulated UI only so far, real API adapter remains PR050 work | client headers/portable artifact authority |
| Fixed-store transactional executor | PostgreSQL host adapter, PR060 | editable derived truth, live network waits |
| Durable worker + provider ports | host worker, PR100/110 | arbitrary native code, exactly-once claims |
| Builder/Runtime presentation | `apps/evobase-builder` local Leptos consumer; hosted Runtime/CMP still planned | trusted tenant routing or business authority |
| Canonical semantic design tokens/contracts | companion M3 source | independently regenerated renderer palettes |

The new independent kernel has direct serde/serde_json/thiserror dependencies. It was freshly
authored; no private VOT product code was copied. Legacy `evobase-core` still owns the older
host/storage traits and is not the new pure kernel. Definition-owned capture/policy/SubmitOrder
rules prevent a separately edited semantic source; real host facts remain external.

Formula support is currently a checked pure query API, not portable authoring/export. Every
relation projection requires exact-definition/fact output authorization. The owner/role adapter
rejects readable Ref fields and restricted complete relation scans rather than leaking a partial
join/aggregate. `LocalPreviewPolicy` is for labeled synthetic local facts only. Broad query/policy
coverage and a verified host adapter are prerequisites for hosted storage.

## Isolation and transactions

Baseline candidate: database-per-tenant + provisioned fixed engine layout. User logical tables/
fields/policies are metadata and typed records. In that planned profile, runtime must have no DDL; provisioner/migrator uses
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
| Host session/native identity + grants/policy subset | 050 partly implemented | owner/role/SubmitOrder fixture negatives exist; verified session/API and unsupported output joins remain open |
| DB fixed codec, lock/roles/pool/support envelope | 060 | two DBs, no-DDL catalogs, transactions/races/restart/bench |
| Release activation and incompatible migration | 070/080 | immutable old pins, impact/backfill/cutover/recovery |
| M3 token mapping and minimum renderer/platform targets | 030/090 | mounted source→adapter trace; DOM/native semantics + inspected pixels |
| Functions state/fencing/provider unknown outcome | 100/110 | actual restart/lost ACK/duplicate/late lease/current-consent tests |
| Portable/local/offline/restore profile | 140 | isolated round-trip/restart/reconciliation; no revived grants/effects |
| VOT/legacy retirement | 150 | source inventory + independent EvoBase gates + recovery/data approval |

Do not call any box implemented merely because its design screen, manifest or fixture exists.
The historical VOT handoff cites different native/hosted format envelopes; those sources were not
freshly inspected here. EvoBase version 1 and its local profile are explicit. Executed test/vector
evidence is in the batch record; DB/benchmark/provider/native acceptance remains future work.
