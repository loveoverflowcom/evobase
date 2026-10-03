# Redesign reconciliation

Status: historical proposals reconciled with [EvoBase #1](https://github.com/loveoverflowcom/evobase/issues/1)
on 2026-10-03. The chapter bodies preserve the earlier Rust-DSL/Domain-IR/generated-runtime proposal;
they are reference material, not the active implementation contract. Their original immutable source
is [baseline `4cb5873`](https://github.com/loveoverflowcom/evobase/tree/4cb5873200f4d735b17b75fdd3cb88c2efdbb322/re-design).

The active path is **structured Builder → raw AppSpec → checked definition → Rust decisions → host adapters**.
AppSpec owns semantics; tenant records and host identity/storage/connector bindings are separate.
Users need no Rust or SQL. Leptos is the browser consumer; CMP mobile is Runtime-first.
[Foundation ADR](../docs/work-plan/decisions/010-foundation.md) selects the first experimental profile.

| Chapters | Retained principle | Current reconciliation |
|---|---|---|
| [Vision](00_vision.md), [architecture](01_architecture_overview.md) | Define business meaning once; deterministic core | AppSpec replaces the Rust source package as authority; runtime interprets checked definitions. |
| [Language](02_domain_language.md), [IR](05_domain_ir.md) | Typed vocabulary, diagnostics, stable graph IDs | Structured authoring is primary. Rust/text views are optional projections of the same model; JSON version 1 is the selected interchange contract. |
| [Workflows](06_workflow_system.md) | Explicit commands/transitions and effects | Finite durable plans, checkpoints, leases and unknown outcomes are planned; RAM messaging does not implement them. |
| [Auth](07_auth_model.md) | Default denial, capability/policy distinction | Current verified host facts AND grants AND app policy; no authority in definitions, selectors or Refs. Local drafts make no sign-in claim. |
| [Messaging](08_messaging_model.md) | Managed events and delivery state | Durable outbox/receipts/current consent are future host work; no exactly-once provider promise. |
| [Generators](09_generator_architecture.md) | Deterministic projections and unsupported diagnostics | SQL generation is an optional later adapter. The fixed-store candidate does not create business tables per edit; its DB evidence is still pending. |
| [Low-code](10_low_code_platform.md) | Discoverable structured authoring | Leptos Builder is primary; CMP Runtime gets lists/forms/actions and explicit desktop handoff for authoring. |
| [Migration](13_migration_strategy.md) | Versioned impact/recovery and historical pins | Definition evolution and runtime-data migration remain separate; no-DDL does not eliminate migration. No production data is moved. |
| [Open questions](14_open_questions.md) | Deliberate support envelopes | The [ADR](../docs/work-plan/decisions/010-foundation.md) resolves version 1 encoding/types/IDs and host seams; broader questions remain in the [queue](../docs/work-plan/README.md). |

The refinement, Lean4, category and functional-programming chapters retain optional engineering ideas.
Selective proofs need a stated proposition, bound and refinement to production code; no chapter proves
the whole platform. Arbitrary generators, general plugin execution, full Excel compatibility and
unrestricted languages are deferred. Existing SQL gateway/Flutter behavior is documented separately
in [the legacy guide](../docs/LEGACY_BACKEND.md); it is not silently migrated by this reconciliation.
