# Migration Strategy

This document describes how EvoBase can move from its current PostgREST-inspired
backend shape to the domain-driven verification-first platform described in
this handbook.

Related documents:

- [00_vision.md](00_vision.md)
- [01_architecture_overview.md](01_architecture_overview.md)
- [05_domain_ir.md](05_domain_ir.md)
- [09_generator_architecture.md](09_generator_architecture.md)

## Current State

Current EvoBase includes:

- Rust crates for core, db, auth, gateway, messaging, and protocol.
- PostgreSQL adapter.
- PostgREST-like REST table access.
- JWT authentication.
- PostgreSQL RLS integration.
- SSE messaging.
- generated table docs from database introspection.
- a workbench client.

This is a useful foundation. The migration should not discard it abruptly. It
should introduce the domain-first pipeline beside the existing database-first
runtime and gradually shift source-of-truth responsibility.

## Migration Principles

### Documentation First

This handbook is the first artifact. The project should agree on vocabulary
before building runtime code.

### Compatibility Where Useful

Existing REST table access can continue for legacy or bootstrap use. New
domain-led APIs should be generated from commands, queries, workflows, and read
models.

### Domain IR Before Generators

Do not build many independent generators directly from Rust macro syntax. Build
the Domain IR first.

### Verification Incrementally

Start with lightweight validation and a small Lean4 refinement library. Expand
formal proofs where risk justifies it.

### Generated Artifacts Must Be Reviewable

Migrations, policies, endpoints, and event contracts should be diffable before
deployment.

## Proposed Phases

### Phase 0: Vocabulary And Architecture

Deliverables:

- this `re-design/` handbook.
- architectural decision records for Rust DSL and Domain IR.
- glossary of domain modeling terms.
- examples of target domain packages.

Exit criteria:

- team agrees that the domain model is the source of truth.
- team agrees that PostgreSQL is a generated target, not the domain model.

### Phase 1: Domain IR Skeleton

Deliverables:

- conceptual IR schema.
- node IDs and metadata model.
- type graph.
- entity/value object representation.
- command/query representation.
- validation diagnostics model.

No runtime code is implied by this document, but implementation should start
with IR before target generators.

Exit criteria:

- sample domain package can lower into serialized IR.
- diagnostics can reference source declarations.

### Phase 2: Rust DSL Frontend

Deliverables:

- Rust attribute vocabulary.
- `syn` extraction plan.
- procedural macro diagnostics plan.
- lowering rules from Rust AST to IR.
- examples for entities, value objects, commands, and events.

Exit criteria:

- authors can express a small domain slice.
- no generator reads Rust syntax directly.

### Phase 3: Refinement Library

Deliverables:

- refinement catalog.
- runtime validator strategy.
- SQL constraint mapping strategy.
- OpenAPI and UI mapping strategy.
- Lean4 specs for a small set of core refinements.

Initial refinements:

- `NonEmptyString`.
- `PositiveInt`.
- `Email`.
- identity types.
- `Money`.

Exit criteria:

- refinements flow through IR to at least docs and planned constraints.

### Phase 4: First Generators

Recommended order:

1. Documentation generator.
2. OpenAPI generator.
3. SQL schema generator.
4. REST command/query generator.
5. Auth/RLS generator.
6. Messaging contract generator.

Docs and OpenAPI should come early because they reveal whether the domain model
is understandable.

Exit criteria:

- a sample domain package produces reviewable generated artifacts.
- every generated artifact references IR node provenance.

### Phase 5: Workflow System

Deliverables:

- workflow graph in IR.
- typestate modeling convention.
- transition validation.
- workflow docs generator.
- command endpoint generation for transitions.
- event generation from transitions.
- SQL state constraints.

Exit criteria:

- order lifecycle example can be generated end to end.
- illegal transitions are rejected in validation and runtime plans.

### Phase 6: Auth Integration

Deliverables:

- actor/role/capability/policy/resource model.
- generated endpoint guards.
- generated RLS where representable.
- policy test matrix.
- UI action visibility metadata.

Exit criteria:

- every generated public command and query requires explicit policy.
- generated RLS maps back to domain policy nodes.

### Phase 7: Messaging Integration

Deliverables:

- event graph in IR.
- event contract generator.
- outbox strategy.
- SSE stream contract generator.
- projection metadata.
- event versioning rules.

Exit criteria:

- domain events, projections, and SSE contracts derive from the same event
  graph.

### Phase 8: Low-Code Platform Surface

Deliverables:

- admin UI metadata generator.
- low-code form metadata.
- workflow diagram metadata.
- policy builder metadata.
- generated artifact preview.
- package version workflow.

Exit criteria:

- low-code edits can produce validated domain package changes.
- engineers can review generated diffs before publishing.

## Coexistence With Existing Runtime

During migration, current endpoints can coexist:

```text
/rest/{table}              legacy database-first table gateway
/domain/{resource}/...     generated domain-first API
/events                    current SSE transport, fed by domain events over time
/docs                      gradually shifts from DB introspection to domain docs
```

The exact paths are implementation decisions. The architectural point is that
legacy table exposure should not block domain-first generation.

## Migrating Existing Schemas

Existing PostgreSQL schemas can be imported as draft domain models:

1. introspect tables.
2. infer entities and identities.
3. infer relationships.
4. map constraints to candidate refinements.
5. detect possible aggregates.
6. mark unknown business rules as open questions.
7. generate a draft domain package for review.

This import is a bootstrap aid, not a perfect reverse engineering process.
Human review is required.

## Migrating Auth

Existing JWT and RLS policies can be mapped into the new model:

- JWT subject becomes actor ID.
- JWT role claims become roles.
- RLS predicates become candidate policies.
- ownership checks become resource policies.
- admin token use becomes a temporary admin actor/capability model.

The migration should identify where existing policies are table-centric and
need domain concepts.

## Migrating Messaging

Existing generic messages can be classified:

- true domain events.
- operational notifications.
- system telemetry.
- client-only notifications.

Only true domain events should enter the Event Graph. Other messages may remain
transport-level features or become projections of domain events.

## Migration Risk

### Risk: Recreating PostgREST With More Steps

Mitigation:

- prioritize workflows, policies, events, and refinements before generic CRUD.
- generate domain commands, not just table endpoints.

### Risk: Lean4 Adoption Friction

Mitigation:

- start with a verified refinement library.
- require proofs only for critical domains.
- provide clear diagnostics and examples.

### Risk: Generator Drift

Mitigation:

- all generators consume Domain IR.
- source maps and provenance required.
- golden tests and capability declarations.

### Risk: Low-Code Weakens Guarantees

Mitigation:

- low-code edits produce domain package changes.
- same validation and verification pipeline.
- review workflow for migrations and policy changes.

## Success Measures

- A sample commerce domain can generate SQL, REST, OpenAPI, auth, messaging,
  docs, and workflow diagrams.
- Public endpoints are domain commands and queries, not table operations.
- Refined types generate validation and SQL constraints.
- Workflows reject illegal transitions before deployment.
- Policies are explicit and deny-by-default.
- Events drive projections and SSE streams.
- Generated artifacts are deterministic and explainable.

## Design Rule

Migrate source of truth first. Runtime replacement is secondary.
