# EvoBase reboot: skills, designs and sequential PR prompts

**Current scope: design and handoff only. Do not execute these implementation prompts yet.**
The user selected Rust backend/core, Leptos web and Compose Multiplatform mobile, then explicitly
asked for skills and PR prompts first, with M3 Expressive design as a priority. Product source,
build manifests and runtime behavior are unchanged by this planning PR.

## Start here

1. Review [architecture/decisions](architecture.md), [source provenance](provenance.md) and the
   companion M3 Expressive design handoff. Resolve the decisions that block the next item.
2. Pick one bounded PR below. The prompt is a proposed future instruction, not standing permission
   to implement the whole queue. Design issue publication is not feature completion.
3. Load its mapped skills; inspect actual source/manifests/tools. Implement only after the user
   selects that PR. Stop at its gate; report exact evidence and remaining unsupported capabilities.

Numeric prefixes are recommended ordering markers, not permanent task IDs. Renumber if verified
dependencies change. This folder owns implementation prompts/gates; GitHub issues own discussion,
assignment and status. Do not maintain a competing status database here.

## Queue and design-pack mapping

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

Future implementation branches start from the exact reviewed predecessor/integration SHA supplied
for that task and normally open a draft PR toward `rewrite/leptos-cmp`. Do not automatically merge
predecessors or switch the default branch. Import reviewed skills/design commits at the foundation
gate; companion branches are inputs, not two editable canonical definitions.

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
branch `design/m3-expressive-20261001`. Pin its immutable commit/issue links before implementation;
never create a second palette/schema copy. See [design handoff](design-handoff.md).

Design fixtures are intended behavior/specification. They do not prove compiled Leptos, native
CMP, authentication, tenant isolation, durable dispatch or real connector support. Pure preview,
mock delivery, pending/conflict/revoked/unknown states must remain visible in actual implementation.

## Evidence for this planning change

Only metadata/local-link/scope/privacy/provenance/document checks and design inspection are in
scope. Product compile/test/DB/provider/device claims are not established. Future test names in
prompts are acceptance obligations, not executed tests. Exact runnable commands are discovered or
implemented by the selected future PR, not invented as current `xtask`/Gradle commands.

Current docs-only checks from the repository root:

```sh
python3 docs/work-plan/scripts/check_handoff.py --base 4cb5873200f4d735b17b75fdd3cb88c2efdbb322
python3 -m unittest discover -s docs/work-plan/scripts -p 'test_*.py'
git diff --check
```

The checker validates this handoff's local metadata/links and narrowly scoped privacy patterns;
`--base` includes committed changes since the pinned baseline as well as the dirty working tree;
without it the scope check covers only changes since HEAD plus untracked files. It reports the
exact comparison and cannot claim a docs-only commit from the default dirty-tree check alone.
it is not a product build/test harness or a complete secret scanner. Skill frontmatter was also
validated with the installed skill-creator validator; no personal skill installation occurred.

Preserved refs: `archive/develop-2026-10-01` at `4cb5873200f4d735b17b75fdd3cb88c2efdbb322`;
`archive/re-design-2026-10-01` at `5f6a98f684ab34ecc727c9354a51e4806454cc8f`;
`rewrite/leptos-cmp` initially at the develop head. Original/default branches remain unchanged.
Skills/prompts are reviewed on `chore/evobase-reboot-plan`; no product code has been rewritten.
