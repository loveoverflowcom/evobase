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
| Implemented experimental slice | `evobase-appspec` owns compatible v1/v2 definitions, exact values, N:1/restrict/captures, bounded formulas, declarative field constraints and finite commands/events. The Leptos Builder authors local drafts and constraints. A separate generated Runtime consumes the verified single-tenant HTTP host, immutable bootstrap release and libSQL transactions with atomic audit/events/receipts and current-authority replay checks. See [new batch evidence](docs/work-plan/next-batch-evidence.md) and [earlier evidence](docs/work-plan/batch-evidence.md). |
| Remaining host/runtime | Full command authoring, publish/evolution/backfill, broader policy/query support, multi-tenant provisioning/routing/isolation, native CMP Runtime and durable automation. Remote Turso support is implemented but needs actual service conformance evidence. |
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

## Run the single-tenant libSQL host and Runtime

The [host quickstart](apps/evobase-host/README.md) documents local files or a remote Turso/libSQL
primary, immutable fixture bootstrap and operator-issued bearer access. Configure one tenant and
one application per host. The [generated Runtime guide](apps/evobase-builder/README.md#generated-http-runtime)
opens the same browser build at `?runtime=1` and reads current permitted data/actions from that host.
The support and library examples use the same generic core, storage and UI. Multi-tenant routing,
remote-service acceptance and production identity remain future gates.

## Run the existing backend

The original backend architecture, environment setup, SQL migration and API examples are preserved
in the [legacy backend run guide](docs/LEGACY_BACKEND.md). The [docs map](docs/README.md) points to
its detailed architecture and Flutter lessons. Those commands run the existing backend. No
production migration or legacy removal is part of this refactor batch.
