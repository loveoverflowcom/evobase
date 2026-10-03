# EvoBase engineering handoff

## Start with the active task

Read the nearest owner documentation and pin the source revision before acting. A request for
designs, skills, review or PR prompts authorizes that deliverable, not implementation of the
future prompts. The current user request authorizes ten additional implementation commits
on `develop` and a push, with one tenant first and multi-tenant work deferred. The earlier
five-commit batch is complete. Execute this bounded batch without repeating
the old one-PR selection gate; report incomplete acceptance explicitly. Broader queue items remain
future work.

The product authority is [EvoBase #1](https://github.com/loveoverflowcom/evobase/issues/1).
The selected future stack is Rust backend/core, Leptos web and Compose Multiplatform mobile.
Legacy `README.md`, `re-design/` and Flutter/backend source describe an earlier baseline;
[the work plan](docs/work-plan/README.md) explains the reconciliation and evidence gaps.

## Route to one owner

- Domain, AppSpec, policy, identity, persistence, workflows, portability and architecture:
  [.agents/skills/evobase-engineering/SKILL.md](.agents/skills/evobase-engineering/SKILL.md)
- Leptos/browser UI and shared design inputs:
  [.agents/skills/evobase-ui-engineering/SKILL.md](.agents/skills/evobase-ui-engineering/SKILL.md)
- Compose Multiplatform native UI:
  [.agents/skills/evobase-cmp-ui-engineering/SKILL.md](.agents/skills/evobase-cmp-ui-engineering/SKILL.md)
- Existing PR/diff review, read-only:
  [.agents/skills/evobase-code-review/SKILL.md](.agents/skills/evobase-code-review/SKILL.md)
- UI inspection/report only:
  [.agents/skills/evobase-ui-inspector/SKILL.md](.agents/skills/evobase-ui-inspector/SKILL.md)

## Required guardrails

Name the observable invariant, its authoritative owner, failure mode and cheapest adequate
regression oracle before behavior changes. Keep business decisions in the deterministic Rust
core; browser/CMP presentation state may remain local. Never trust URL/body actor or tenant
selectors as authority, deserialize checked plans, or grant access through a Ref.

Design source lives in the companion design handoff at
`design/m3-expressive/source/tokens.json`; pin its commit before importing it. Shared tokens and
component contracts have one owner. A responsive browser capture does not establish CMP native
behavior. Screenshots must actually be opened before claiming inspected pixels.

Report exact evidence level, source/build identity, executed/failed/unrun checks and residual
risk. Discover actual manifests, runners and CI; future command names in prompts are proposals,
not installed tools. Docs-only work needs link/frontmatter/scope checks, not a platform build.

Preserve old branches/history non-destructively. No forced updates, branch deletion, default
branch changes, repository archival, merge, deployment, production migration or external-provider
test follows implicitly from this handoff. Separate code publication, CI-trigger and external
action authorization from read-only review. Do not inherit infrastructure or QA permissions from
the source repository.
