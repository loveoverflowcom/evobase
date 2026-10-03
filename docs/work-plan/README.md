# EvoBase reboot: skills, designs and sequential PR prompts

**Completed scope: ten additional implementation commits on `develop`, published to that branch.
One tenant is implemented first; multi-tenant work is deferred.** Use the engineering/UI/review skills,
the [single-tenant libSQL ADR](decisions/020-single-tenant-libsql.md) and
[new batch evidence](next-batch-evidence.md). The implemented sequence is:

1. [160 Profile and trust boundary](160-single-tenant-profile.md)
2. [170 Declarative constraints](170-declarative-constraints.md)
3. [180 libSQL fixed store](180-libsql-fixed-store.md)
4. [190 Generic transitions and events](190-generic-transitions.md)
5. [200 Atomic command commit](200-atomic-command-commit.md)
6. [210 Versioned wire contract](210-versioned-appspec-protocol.md)
7. [220 Single-tenant HTTP host](220-single-tenant-http-host.md)
8. [230 Builder constraint authoring](230-builder-constraints.md)
9. [240 Generated HTTP Runtime](240-generated-runtime.md)
10. [250 Integration and evidence](250-batch-integration.md)

Numeric prefixes represent the recommended sequence, may be renumbered, and are not permanent IDs.
Basic storage was delivered before generic commands because its snapshot contract was independent.
GitHub remains the issue tracker; these files define this batch's review boundaries. The next
recommended task is [260 command authoring](260-command-authoring.md). Release evolution,
providers, multi-tenant support and native CMP retain their separate gates.

The earlier five sequential refactor commits are complete. Their historical planning
handoff is now an input to that implementation batch; its prior one-PR selection/stop instructions
are historical, not a new approval requirement. The selected stack remains Rust core/backend,
Leptos web and Compose Multiplatform mobile.

## Start here

Read [foundation decisions](decisions/010-foundation.md), [architecture](architecture.md),
[source provenance/inventory](provenance.md), the owning issue and mapped skills. Pin the actual
base, inspect manifests/tools and execute this bounded batch. Preserve unfinished hosted/native/
provider gates. A future prompt outside the selected batch is not standing permission to build
the entire platform.

The earlier completed batch followed these five checked-contract dependencies:

| Commit scope | Owning issue / prompt | Experimental delivery / remaining gate |
|---|---|---|
| 1. Reconcile vision, import reviewed guidance/design, select foundation profile | #3 / [010](010-contracts-and-bootstrap.md) | E0 delivered in `6e14972`; browser/native/host shell acceptance remains scoped separately. |
| 2. Add bounded raw → checked AppSpec, stable IDs and exact values | #4 / [020](020-appspec-kernel.md) | Kernel delivered in `6fa7bb0`, with native/WASI vectors; later rule additions retain the same bounded contract. |
| 3. Mount a Leptos typed local-draft Builder consuming Rust contracts and canonical design | #3–#4 / [030](030-typed-grid-and-import.md) | Mounted local-draft grid/import and optional field control; final artifact checks are in [batch evidence](batch-evidence.md), no authenticated tenant host. |
| 4. Add N:1/restrict, captures and bounded formula core plus browser inspector | #5 / [040](040-relations-and-formulas.md) | N:1/restrict/capture and bounded query APIs implemented; portable formula authoring/export and advanced relations deferred. |
| 5. Add a bounded policy/command core and simulated browser inspector; verify integrated batch/CI | #6 / [050](050-authority-policies-commands.md) | Definition-owned owner/role and SubmitOrder core implemented with simulated host facts; verified session/API and transactional receipts deferred. |

These historical rows describe delivered boundaries, not a second issue-status database. GitHub owns final issue
status. Close an issue only when all its scoped acceptance is met; otherwise mark the delivered
subset and list remaining gates. In particular, browser tests cannot complete CMP, provider or
fixed-store acceptance.

Numeric prefixes below are dependency ordering markers, not permanent task IDs. Renumber if
verified dependencies change. This folder owns prompts/gates; GitHub owns discussion/assignment.

## Dependencies reconciled for the current batch

The previous queue called for finishing the remaining #3/#4 browser acceptance: complete structured
authoring/Ref-resolution flows, full accessibility/input evidence and the honest boundary between
local interruption simulation and a real session. Add #5 portable formula declarations, authoring
and export against the same checked semantics. Complete #6 with a verified host/session/API
adapter and supported output-policy coverage, including explicit denial of unsupported restricted
joins/complete scans. The current single-tenant batch supplies a verified host and generic checked
execution for its narrower storage profile; [060 isolated PostgreSQL storage](060-isolated-fixed-store.md)
remains a separate future spike, whose
trusted routing, authorization-before-replay and atomic command executor depend on those contracts.
CMP/native, provider and broader vision gates remain separate.

## Queue and design-pack mapping

All 12 design issues have been published as [EvoBase #3–#14](issue-map.md), pinned to the immutable
design source. Each implementation prompt links its verified owning issue. Start with [#3](https://github.com/loveoverflowcom/evobase/issues/3)
for the selected foundation slice. The user has authorized the batch above; issue publication alone
does not establish implementation or authorize the remainder of the queue.

| Order / outcome | Design pack | Dependency gate | Primary skill |
|---|---|---|---|
| [010 Contracts and bootstrap decisions](010-contracts-and-bootstrap.md) | 01 foundation/shell/identity decision | source/design review | engineering |
| [020 AppSpec checked kernel](020-appspec-kernel.md) | 02 typed data foundation | 010 decisions | engineering |
| [030 Typed grid, field and import](030-typed-grid-and-import.md) | 02 authoring | 020 checked vectors | Web |
| [040 Relations, captures and formulas](040-relations-and-formulas.md) | 03 relations/formulas | 020; 030 UI owner | engineering + Web |
| [050 Host authority, policies and commands](050-authority-policies-commands.md) | 04 policies/actions | 020 + 040 core | engineering + Web |
| [060 Isolated fixed-store spike](060-isolated-fixed-store.md) | 05 hosted isolation | 050 authority/checked commands | engineering |
| [070 Releases, views and pure preview](070-release-and-preview.md) | 06 release/view | 060 storage; 030–050 UI | engineering + Web |
| [080 Schema evolution and recovery](080-schema-evolution.md) | 06 upgrade | 070 pins; 060 transactions | engineering + Web |
| [090 Runtime desktop and CMP](090-runtime-web-cmp.md) | 07 runtime/forms/approval | 050 + 070; 080 if upgraded data | Web + CMP |
| [100 Durable approval function](100-durable-functions.md) | 08 functions/runs | 060 + 070 + 090 approval contract | engineering + Web/CMP |
| [110 Email/webhook connectors](110-connectors.md) | 09 connections | 100 durable dispatch | engineering + Web |
| [120 Managed messaging inbox](120-messaging.md) | 10 inbox | 110 receipts/bindings | engineering + Web/CMP |
| [130 Scoped bot query/command](130-scoped-bot.md) | 11 bot | 050 + 110; 120 handoff | engineering + Web/CMP |
| [140 Portable/local recovery profile](140-portability-recovery.md) | 12 portability/offline | 020 + 080 + 100 fencing | engineering + CMP |
| [150 Legacy retirement handoff](150-legacy-retirement.md) | transfer gate | independent EvoBase acceptance | engineering + review |

Packs 02 and 06 deliberately have two PRs: pure codec/core versus UI, and release pin versus
data migration. A cardinality, provider, target or workflow expansion that exceeds a prompt's
support envelope gets another reviewed item instead of being silently absorbed.

Full vision acceptance also requires the explicit [expansion backlog](backlog/support-expansion.md);
the first bounded slice must not mark those obligations complete.

`develop` is now the user-selected refactor integration branch; its legacy head is preserved at
`archive/develop-2026-10-03@4cb5873200f4d735b17b75fdd3cb88c2efdbb322`. The current batch commits
there sequentially. `rewrite/leptos-cmp` and the earlier foundation branch are preserved references,
not a second active integration target. Future feature branches use the reviewed integration SHA.
No force updates, history deletion, deployment or production migration follow from this batch.
Reviewed skills/design are imported at the foundation gate; companion branches are provenance
inputs, not independently editable canonical definitions.

## Skill breakdown

- `evobase-engineering`: shared trust/type/core/evidence contract; load narrow refs for AppSpec,
  relations, policy, storage, functions or portability
- `evobase-ui-engineering`: Leptos/browser behavior and shared canonical M3/localization contracts
- `evobase-cmp-ui-engineering`: Runtime-first CMP state/lifecycle, native semantics and evidence
- `evobase-ui-inspector`: inspection-only scenario/report workflow; no fixes or inherited QA grants
- `evobase-code-review`: read-only invariant/bypass/evidence review; no remote approval or deployment

Discovery entrypoint: [AGENTS.md](../../AGENTS.md). Skills are repository-local reusable artifacts,
not installed personal skills or a new product test runner.

## Design source and delivery

The designer publishes `design/m3-expressive/source/tokens.json`, component contracts, state/
accessibility matrix, screen map, static frames and a fixture prototype in the companion design
branch `design/m3-expressive-20261001`. The reviewed immutable design pin is
[`6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`](https://github.com/loveoverflowcom/evobase/tree/6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e/design/m3-expressive):
27 screens / 12 packs / 60 screen SVG frames / 44 PNG representatives. All 142 published design
objects match reviewed source hashes. The [verified issue-to-prompt map](issue-map.md) links
all 12 scoped design issues. Import this canonical identity before implementation;
never create a second palette/schema copy. See [design handoff](design-handoff.md).

Design fixtures are intended behavior/specification. They do not prove compiled Leptos, native
CMP, authentication, tenant isolation, durable dispatch or real connector support. Pure preview,
mock delivery, pending/conflict/revoked/unknown states must remain visible in actual implementation.

## Evidence boundaries

[Batch evidence](batch-evidence.md) records exact executed checks and artifact/source identities.
The browser matrix must use the newly rebuilt bundle; an earlier green run is not evidence for
a later source change. The added CI workflow is configuration until its remote run is observed.


The foundation commit establishes document metadata/local-link/scope/privacy/provenance checks
only. Subsequent implementation commits report their own exact native/WASM/browser commands and
results. DB/provider/device claims remain unestablished unless their real boundary was executed.
Future test names in prompts are acceptance obligations, not proof of a green suite. Discover real
runners; do not invent current `xtask`/Gradle commands.

Document checks include `git diff --check`, local-link checks for modified README/re-design files,
and the existing handoff checker
(`python3 -m unittest discover -s docs/work-plan/scripts -p 'test_*.py'` for its own regression suite).

The handoff checker validates skill/work-plan metadata/links and narrow privacy patterns. Its
original planning-only path allowlist intentionally rejects product source, imported design assets
and the reconciled root/re-design documents. Use explicit foundation document paths when checking
that metadata; inspect the full commit scope separately. A dirty-tree check cannot prove a whole
commit is docs-only. The checker is not a product harness or a complete secret scanner. The original
planning handoff records frontmatter validation; this import does not install personal skills.

Preserved refs: `archive/develop-2026-10-01` at `4cb5873200f4d735b17b75fdd3cb88c2efdbb322`;
`archive/re-design-2026-10-01` at `5f6a98f684ab34ecc727c9354a51e4806454cc8f`;
`rewrite/leptos-cmp` initially at the legacy develop head. The additional
`archive/develop-2026-10-03` preserves that same head before this batch. Skills/prompts originate
from `chore/evobase-reboot-plan@699fa0f4886c1b47c9ee21f6774bbdb9800a6758`; their original
planning-only status is historical. This refactor batch changes `develop` within the authorized scope.
