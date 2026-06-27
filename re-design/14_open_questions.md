# Open Questions

This document tracks unresolved design questions for the EvoBase redesign. It
is intentionally explicit so future implementation does not smuggle major
decisions into code without architectural review.

Related documents:

- [00_vision.md](00_vision.md)
- [01_architecture_overview.md](01_architecture_overview.md)
- [05_domain_ir.md](05_domain_ir.md)
- [09_generator_architecture.md](09_generator_architecture.md)
- [13_migration_strategy.md](13_migration_strategy.md)

## DSL And Authoring

1. What is the minimal Rust attribute vocabulary for the first domain package?
2. Should the Rust DSL be purely attribute-based, macro-invocation-based, or a
   small combination?
3. How much ordinary Rust should be allowed inside a domain package?
4. How should domain package names and versions map to Cargo packages?
5. How should renames be declared so stable IR IDs survive source renames?
6. How will low-code edits round-trip with Rust DSL files without creating
   unreadable source?

## Domain IR

1. What serialization format should be canonical for IR snapshots?
2. Which IR schema versioning strategy should be used?
3. Should IR node IDs be generated from paths, explicit annotations, or a
   package registry?
4. How much UI metadata belongs in the core IR versus a UI generator extension?
5. Should generator-specific metadata be stored in the IR or adjacent target
   configuration?
6. What is the minimum IR needed for the first useful prototype?

## Verification

1. Which invariants require Lean4 proofs in strict mode?
2. What is the first verified refinement library?
3. How are Lean4 theorem identifiers linked to Rust DSL declarations?
4. What is the trust model for generated constraints derived from proofs?
5. How should proof failures appear in developer and low-code UIs?
6. Can some generator correctness properties be specified in Lean4 without
   making the generator itself formally verified?

## Refinement Types

1. What is the exact email validation policy?
2. Should `NonEmptyString` trim whitespace before validation?
3. How should `PasswordHash` represent algorithm and version metadata?
4. How should `Money` handle currencies, scale, and rounding?
5. Which refinements can become PostgreSQL domains versus per-column checks?
6. How should refinements depending on external systems be represented?

## Workflows

1. Should workflow state be stored as one state column, separate state tables,
   or event-derived state?
2. What is the default strategy for transition history?
3. How should compensating transitions be represented?
4. How should long-running workflows and timers be modeled?
5. How should concurrent transitions be resolved?
6. What is the default idempotency model for commands?

## Auth

1. What capability grant storage model should be generated first?
2. How should capability revocation interact with JWT TTL?
3. Which policies must be representable in PostgreSQL RLS for a domain to be
   accepted?
4. How should service accounts and machine actors be modeled?
5. How should tenant isolation be represented in the core policy graph?
6. What is the policy explanation format for UI and audit logs?

## Messaging

1. Should the first reliable event mechanism be PostgreSQL outbox polling?
2. What event envelope fields are mandatory?
3. What is the default event versioning policy?
4. How should event replay cursors be represented for SSE?
5. How should projection failures be retried or quarantined?
6. Which async API documentation format should be generated for events?

## Generators

1. Which generator is implemented first: docs, SQL, or OpenAPI?
2. What is the generator plugin boundary?
3. How are generated artifacts stored and reviewed?
4. What source-map format links generated artifacts to IR nodes?
5. How should unsupported IR features block or degrade generation?
6. What is the naming convention for generated SQL objects and endpoints?

## Migration

1. How much of the current table gateway remains long term?
2. Can existing schemas be imported into draft domain packages automatically?
3. How should hand-written migrations coexist with generated migrations?
4. What is the rollback story for generated migrations?
5. How should existing RLS policies be mapped into domain policies?
6. How should current generic messages be classified into domain events versus
   operational notifications?

## Low-Code Platform

1. What is the first low-code authoring surface?
2. Should analysts edit domain packages directly or edit draft IR documents?
3. What review workflow is required before publishing?
4. How are destructive changes explained to non-engineers?
5. How are generated UI layouts customized without forking generated code?
6. What permission is required to change policies or workflows?

## Category Theory And FP Foundations

1. Which laws should be documented as platform laws?
2. Which laws should have property tests?
3. Which laws justify Lean4 proofs?
4. How visible should category theory vocabulary be in user-facing docs?
5. How can the architecture keep the benefits without making contributors feel
   they need advanced theory to participate?

## Operational Questions

1. How are domain package versions deployed across environments?
2. How are generated artifacts promoted from development to production?
3. How should artifact fingerprints be stored?
4. How should generated runtime observability link back to domain nodes?
5. How are emergency production fixes represented back in the domain model?
6. What is the support policy for generated artifacts after generator upgrades?

## First Recommended Decisions

The first implementation planning session should decide:

1. The minimal Domain IR schema.
2. The first sample domain.
3. The first refinement catalog.
4. The first generator target.
5. The strictness level for Lean4 proof obligations.
6. The compatibility story for existing `/rest/{table}` APIs.

These decisions unlock the smallest coherent vertical slice without collapsing
the redesign back into a database-first framework.
