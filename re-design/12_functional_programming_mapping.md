# Functional Programming Mapping

Functional programming concepts are foundational to the redesigned EvoBase
because they help keep domain declarations pure, transformations composable,
and generated runtime effects controlled.

Related documents:

- [03_refinement_types.md](03_refinement_types.md)
- [04_lean4_verification.md](04_lean4_verification.md)
- [05_domain_ir.md](05_domain_ir.md)
- [11_category_theory_mapping.md](11_category_theory_mapping.md)

## Practical Mapping

| FP Concept | EvoBase Use |
| --- | --- |
| pure functions | validators, projections, policy predicates |
| composition | DSL to IR to artifact pipeline |
| algebraic data types | entities, events, workflow states, decisions |
| functor | generators and mapped schemas |
| monad | validation, generation, policy context, effects |
| Reader | environment-dependent generation and policy evaluation |
| State | migration planning, workflow state, projection rebuild |
| Either | fail-fast errors |
| Validation | error accumulation |
| Free Monad | declarative workflow or command programs |
| Tagless Final | multiple interpretations of domain operations |

## Pure Functions

Pure functions return the same output for the same input and do not perform
side effects.

EvoBase should model these as pure where possible:

- refinement predicates.
- value object constructors.
- policy predicates over explicit context.
- projection transformations.
- migration compatibility checks.
- IR validators.
- generator planning.

Practical benefit:

- easier testing.
- deterministic generation.
- safer parallel execution.
- better compatibility with Lean4 specifications.

## Composition

Composition builds larger behavior from smaller behavior.

Examples:

```text
parse_input -> refine -> authorize -> execute_transition -> emit_event
```

```text
parse_rust -> lower_to_ir -> validate_ir -> generate_artifacts
```

Practical benefit:

- each step has a narrow responsibility.
- failures can be localized.
- new steps can be inserted without rewriting the system.

## Algebraic Data Types

Algebraic data types include products and sums:

- Product: all fields together, like a struct.
- Sum: one alternative, like an enum.

EvoBase uses ADTs for:

- value objects.
- command payloads.
- event payloads.
- workflow states.
- policy decisions.
- validation errors.

ADTs make domain possibilities explicit. If `OrderState` has four variants, the
generator and docs can enumerate all four.

## Functor

A functor maps over structure without changing its shape.

Practical examples:

- map domain types to OpenAPI schemas.
- map domain events to client SDK event types.
- map read models to UI table columns.
- map validation errors to diagnostics.

Generators are functor-like interpretations from Domain IR into artifacts.

## Monad

Monads sequence computations that include context, effects, or failure.

EvoBase uses monadic ideas for:

- validation pipelines.
- generation with diagnostics.
- policy evaluation with actor context.
- workflow execution with transaction context.
- event publication with outbox context.

This keeps sequencing explicit:

```text
Raw command
  -> parse result
  -> refinement result
  -> policy result
  -> transition result
  -> event result
```

## Reader

Reader represents computations that depend on shared environment.

EvoBase examples:

- generator reads target capabilities.
- policy evaluation reads actor context and request environment.
- migration planner reads current deployed package version.
- docs generator reads branding and visibility settings.

Reader-style design avoids global state. The environment is explicit.

## State

State represents computations that transform state.

EvoBase examples:

- workflow transition: current state to next state.
- migration planner: current schema model to next schema model.
- projection: current read model state plus event to new read model state.
- generator naming: current name registry to updated registry.

The important design move is to make state transitions explicit and typed.

## Either

Either represents success or failure:

```text
Either<Error, Value>
```

EvoBase uses Either-like behavior when one error should stop the pipeline:

- Rust syntax parse failure.
- unknown referenced type.
- unsupported generator target.
- policy rule references missing resource.

Fail-fast is best when downstream work would be meaningless.

## Validation

Validation accumulates errors instead of stopping at the first error.

EvoBase should use validation-style behavior for:

- multiple invalid fields in a command.
- multiple missing docs.
- multiple unsupported refinements in a generator target.
- multiple policy coverage gaps.
- multiple UI metadata issues.

Practical benefit:

- domain authors receive a useful list of problems.
- low-code users do not fix one issue only to discover another.

## Free Monad

A free monad separates declaring a program from interpreting it.

Practical EvoBase use:

- declare a workflow as steps and transitions.
- interpret it as REST endpoints.
- interpret it as SQL constraints.
- interpret it as docs.
- interpret it as UI actions.

The domain declaration is free from choosing a single runtime interpretation.

Example:

```text
Workflow declaration:
  require PaidOrder
  check ship_order policy
  set ShippedOrder
  emit OrderShipped

Interpretations:
  REST endpoint
  SQL transaction
  UI action
  documentation
  test scenario
```

## Tagless Final

Tagless Final models a language by its operations and allows multiple
interpretations.

Practical EvoBase use:

- domain operations can be interpreted as validation, documentation, SQL,
  runtime behavior, or tests.
- policy declarations can be interpreted as RLS, runtime checks, UI visibility,
  or audit docs.
- event declarations can be interpreted as SSE, Kafka schema, projection input,
  or client SDK types.

Tagless Final thinking helps prevent the DSL from being tied too tightly to one
backend target.

## Effect Boundaries

EvoBase should keep pure declaration and effectful runtime separate:

| Pure Layer | Effectful Interpretation |
| --- | --- |
| domain model | database transaction |
| policy predicate over context | JWT parsing and DB lookup |
| projection transformation | persisted read model update |
| event declaration | event bus delivery |
| generator plan | file output and deployment |

This makes testing and verification easier.

## Error Modeling

Errors should be ADTs, not strings:

- parse error.
- validation error.
- policy denial.
- workflow source-state mismatch.
- refinement failure.
- generator unsupported feature.
- migration unsafe change.

Each error variant should carry structured context and source references.

## Practical Benefits

Functional programming foundations give EvoBase:

- deterministic generation.
- composable validation.
- explicit effects.
- richer type modeling.
- better diagnostics.
- easier property testing.
- cleaner proof boundaries.
- safer workflow modeling.

## Design Rule

Keep the domain model pure for as long as possible. Push effects to generated
runtime interpreters with explicit boundaries.
