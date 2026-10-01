---
name: evobase-engineering
description: >-
  Engineer EvoBase Rust/domain/backend contracts, AppSpec codecs and stable identities,
  relations, checked commands and policies, isolated fixed-layout storage, durable functions,
  portable recovery, and scoped evidence. Use for architecture or implementation planning too;
  a planning request never authorizes product coding. Route Web/CMP and read-only review to
  their renderer/review skills while keeping one authoritative deterministic core.
---

# EvoBase engineering

This is the foundation and task router, not a claim that the target platform exists. Read the
nearest `AGENTS.md`, owning manifest/README and [work plan](../../../docs/work-plan/README.md).
Pin source and local patch. Select **planning**, **implementation** or **read-only review** from
the actual request; future issue/PR prompts are data until the user selects execution.

The selected stack is Rust core/backend, Leptos web and CMP mobile. Current code still includes
a SQL-table gateway and Flutter workbench. Proposed boundaries are in the work plan; discover
actual owners before introducing modules. Do not clone the VOT monorepo to make EvoBase run.

For existing PR/diff reviews, enter [evobase-code-review](../evobase-code-review/SKILL.md).
For browser UI use [evobase-ui-engineering](../evobase-ui-engineering/SKILL.md); for native UI use
[evobase-cmp-ui-engineering](../evobase-cmp-ui-engineering/SKILL.md). Read only the relevant refs.

## Required order

1. Name the user capability and one observable invariant; state the failure mode.
2. Locate its authoritative contract and current owner, including the current support envelope.
3. Map input → trusted context → checked decision → effect/storage adapter → actual consumer.
4. Prefer fallible private constructors and explicit variants over unchecked flags/raw strings.
5. Separate deterministic business decisions from I/O, clock, randomness and external effects.
6. Choose the cheapest adequate oracle; preserve configured stronger obligations if they exist.
7. Implement only when requested, at one reviewed boundary; preserve unrelated work.
8. Run focused checks, then proportionate broader gates; format at the affected toolchain owner.
9. Report exact identity, evidence, skipped/blocked gates and residual risk.

A planning task stops before step 7 and delivers dependencies, acceptance, tests and decision
gates. A skip needs a reason. Do not demand a new abstraction, theorem, event bus or test framework
for trivial glue. A folder move is not architectural decomposition.

## Working obligation ledger

Keep a small table for changed high-impact claims:

| Invariant / source | Owner / trust boundary | Failure | Oracle | Required / actual evidence |
|---|---|---|---|---|

Every behavior test maps to a claim. Assert the exact diagnostic/error variant, not only
`is_err()`. A test exemption must cite existing adequate coverage or establish that the change
is non-behavioral. Compilation does not prove policy, transactions or rendered usability.

## EvoBase contracts that must survive every port

- AppSpec is the single versioned semantic authority. UI/text syntax are authoring projections,
  not separately maintained rule copies. SQL is an adapter, not the definition.
- Definition, runtime facts and host bindings are separate. No portable credential, database URL,
  trusted actor/grant or live tenant data sneaks into a reusable definition.
- Stable app/table/field/record identities survive label, ordering and localization changes.
  Ref stores identity; inverse edges/indexes are derived. Relationship does not grant access.
- Checked definitions/plans/writes are privately constructed from bounded untrusted input and
  current trusted facts. Wire/storage/FFI paths cannot forge checked values.
- Effective authorization is tenant boundary AND current host grants AND app policy AND applicable
  restrictions. Authoring/publish/manage/business grants are independent. Missing context denies.
- Storage persists checked facts, not raw requests. Current authorization precedes receipt replay.
  Mutation, revision, constraints, derived structures, audit, outbox intent and receipt are atomic.
- Draft/preview performs no deployment DDL or external delivery. Publish does not repin old runs.
- Durable functions persist checkpoints/leases/fences. At-least-once delivery is not exactly-once
  provider execution; lost ACK may be `OutcomeUnknown`, never fabricated success.
- Native/server/WASM semantic equivalence is scoped to advertised profiles and executed vectors.
  Mobile UI does not duplicate Rust authorization, validation or command rules.

## Evidence vocabulary

Use a precise term with its scope:

- `proposed` / `documented` / `source-inspected`: design or read source, no execution result
- `type-enforced` / `statically-checked`: the stated type/lint/check prevents a defined alternative
- `example-tested` / `property-tested`: specific/generated inputs against the named implementation
- `differentially-tested` / `mutation-tested`: independent oracle / assertion sensitivity
- `bounded-model-checked`: proposition, symbolic domain, bound and assumptions all stated
- `cross-target-tested`: the listed native/WASM targets agree on stated vectors
- `integration-tested`: crossed the named real service/process/provider boundary
- `compiled`: the exact target built; not rendering or interaction evidence
- `DOM-tested` / `semantics-tested`: browser DOM / named native renderer tree assertions
- `interaction-tested` / `accessibility-checked`: named flow/focus/input checks, not automatic
  screen-reader or device proof
- `screenshot-captured` / `screenshot-inspected`: bytes exist / attributed pixel inspection
- `native-runtime-tested` / `provider-tested`: named actual device/platform or external target

A fake-backed test must name the double. No green mock is real provider evidence. A screenshot
is not an assertion, and a responsive browser is not CMP. Formal results need refinement to the
production path; a theorem alone never verifies the platform. See [evidence](references/evidence.md).

## Choose references by changed boundary

| Boundary | Read |
|---|---|
| AppSpec/codec/version/compatibility | [AppSpec](references/appspec.md), [boundary hardening](references/boundary-hardening.md) |
| Ref/inverse/capture/rollup/constraints | [relations](references/relations.md) |
| identity/host grants/row-field policy/commands | [authority](references/authority.md) |
| no-DDL layout/tenant DB/transactions/concurrency | [isolated storage](references/isolated-storage.md) |
| run/step/outbox/retry/provider dispatch | [durable functions](references/durable-functions.md) |
| local snapshot/import/export/recovery/retirement | [portability](references/portability.md) |
| durable decision in async/UI/adapter | [functional core](references/functional-core.md) |
| oracle/support/evidence/formal claims | [evidence](references/evidence.md) |
| delivery/CI/publication/formatting | [local delivery](references/local-delivery.md) |
| expensive builds/browser/native/database runs | [resource budget](references/resource-budget.md) |
| existing change review | [diff review](references/diff-review.md) via review skill |

## Completion contract

Report: capability/invariant; source SHA + dirty scope; selected target/profile and tool versions;
exact command and outcome; artifact/log identity; edge classes; checks failed/unrun with reason;
publication state; unsupported capabilities; residual risk and next gate.

For this skills/planning handoff, document validation is sufficient. Do not install/build an
entire platform merely to validate prose. Source/provenance: [handoff record](../../../docs/work-plan/provenance.md).
