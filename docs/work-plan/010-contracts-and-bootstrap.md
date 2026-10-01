# 010 — Resolve reboot contracts and bootstrap decisions

Status: proposed next PR; planning/docs only, no product scaffolding.
Design pack: 01 foundation/shell/identity decision. Skills: engineering, Web/CMP as design criteria, review.

## Why / boundary

Current SQL-led/Flutter implementation and Rust-DSL/generated-runtime documents contradict #1's
AppSpec-first target. Resolve authority, supported profile and source transfer before code starts.
No empty Leptos/CMP monorepo, production auth, DDL, workflow engine or legacy retirement in this PR.

## Dependencies / decisions

Review pinned baseline/source inventory, preserved refs and companion design commit. Decide minimal
AppSpec encoding/version/extension ADR, typed value vocabulary, host identity seam, target/toolchain
support and canonical design import. Directory names remain proposed until immediate consumers exist.

## Acceptance / evidence

- README/re-design reconciliation explicitly separates current/planned/deferred and preserves history.
- Definition/runtime/binding versions, stable identities, policy/command boundaries and zero-dispatch
  preview are one contract; optional expert syntax cannot become primary-authoring requirement.
- Inventory source code/tests/data/dependencies/notices and inherited obligations with source SHA/path.
- Choose minimal independent Rust core, Leptos and CMP boundaries; justified consumer for each.
- Design pack mapping includes non-happy/permission/unsupported/simulation states and review decisions.
- Metadata/link/privacy/scope/provenance checks pass; no product build claim from prose.

## Future execution prompt

> After this item is explicitly selected, read AGENTS.md, the five relevant repository skills,
> docs/work-plan/architecture.md and the immutable M3 companion handoff. Pin the selected reviewed
> base and inspect actual code/manifests before revising docs. Deliver a docs-only E0 PR that
> reconciles legacy vision with EvoBase #1, imports reviewed skill/design references at their single
> canonical paths, records source/data/license handoff and resolves the decisions required by PR020.
> Keep architecture choices narrow, explain remaining unknowns and proposed targets, and preserve
> original branches/history. Do not write product scaffolding, install a full toolchain for prose,
> migrate data, merge or deploy. Run document/asset/skill checks appropriate to changed files;
> report exact commands, source identities, inspected design states and residual gaps. Publish a
> draft PR only within explicit publication scope, verify its exact remote SHA, and stop for contract
> review. Next gate is authorization of PR020, not automatic implementation of the queue.
