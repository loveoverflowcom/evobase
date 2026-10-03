# EvoBase

EvoBase is becoming a no-code SaaS platform built around typed business tables. Users author data,
relations and rules through a structured Builder; a versioned **AppSpec** defines their meaning;
a deterministic Rust core checks them; a tenant host supplies storage, verified authority and effects.
The product authority is [vision #1](https://github.com/loveoverflowcom/evobase/issues/1).

The selected stack is **Rust core/backend, Leptos web and Compose Multiplatform mobile**.
Leptos owns Builder and browser Runtime. CMP starts with mobile Runtime: permitted lists, record
forms and actions. SQL and expert Rust syntax are adapters/frontends, not prerequisites for users.

## Current and remaining scope

| State | Scope and evidence |
|---|---|
| Existing baseline | Rust/PostgreSQL table gateway, JWT authentication, introspection and RAM SSE/relay; Flutter client/workbench. Source exists at `4cb5873200f4d735b17b75fdd3cb88c2efdbb322`; this is not an AppSpec host or a durable worker. |
| Implemented experimental slice | `evobase-appspec` owns bounded version 1 definitions/facts, exact values, N:1/restrict/captures, bounded pure query formulas and definition-owned owner/role/SubmitOrder rules. The Leptos Builder mounts typed local drafts/import plus relation/policy previews. Browser facts are local/simulated; formulas are query APIs, with portable formula authoring/export still deferred. See [batch evidence](docs/work-plan/batch-evidence.md) for executed scope. |
| Planned host/runtime | Real verified current session/grants and API adapter, broader policy authoring/output joins, transactional checked-command persistence, database-per-tenant fixed store, immutable releases/evolution, native CMP Runtime and durable automation. These require their own acceptance evidence. |
| Deferred expansions | 1:1/N:M, cascade deletion, broader expression/policy languages, live Email/Zalo/webhooks, messaging/bot, offline authority, production migration and Votable retirement. See the [expansion backlog](docs/work-plan/backlog/support-expansion.md). |

A design frame or compiled package does not establish authentication, tenant isolation, native mobile
behavior or provider delivery. Draft preview has zero external dispatch; reusable definitions contain
neither live records nor secrets, grants or host bindings.

## Read and contribute

- [Foundation decisions](docs/work-plan/decisions/010-foundation.md): AppSpec version, IDs, exact values and host seams.
- [Work plan and issue map](docs/work-plan/README.md): the authorized batch, dependencies and remaining gates.
- [Architecture](docs/work-plan/architecture.md) and [source inventory/provenance](docs/work-plan/provenance.md).
- [M3 Expressive design source](design/m3-expressive/README.md): pinned tokens/contracts; static fixtures are design evidence.
- [Engineering guidance](AGENTS.md): repository owners and scoped verification.
- [Historical redesign reconciliation](re-design/README.md): retained ideas and superseded Rust-DSL/generated-SQL assumptions.

`develop` is the refactor integration branch. The previous develop head is preserved at
[`archive/develop-2026-10-03`](https://github.com/loveoverflowcom/evobase/tree/archive/develop-2026-10-03)
with SHA `4cb5873200f4d735b17b75fdd3cb88c2efdbb322`. History and legacy run documentation remain available.

## Run the local Builder

Follow the [Leptos Builder run guide](apps/evobase-builder/README.md) to build the browser/WASM
app and serve it locally. Drafts save to localStorage; this does not publish to the legacy backend
or authenticate a tenant. The guide documents supported inputs and the browser evidence runner.

## Run the existing backend

The original backend architecture, environment setup, SQL migration and API examples are preserved
in the [legacy backend run guide](docs/LEGACY_BACKEND.md). The [docs map](docs/README.md) points to
its detailed architecture and Flutter lessons. Those commands run the existing backend, not the
planned tenant host. No production migration or legacy removal is part of this refactor batch.
