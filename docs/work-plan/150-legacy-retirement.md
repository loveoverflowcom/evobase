# 150 — Prepare and execute a separately authorized legacy handoff

Status: proposed later gate, **not authorized retirement execution by this planning PR**.
Design mapping: source handoff, [VOT #773](https://github.com/loveoverflowcom/vot-workspace/issues/773).
Skills: engineering → portability/evidence; read-only review for scope assessment.

## Dependencies / scope

Independent EvoBase build/run and required supported E1–E5/recovery acceptance, reviewed advanced
support backlog and source/data/license inventory. Preserve exact legacy branches/history and
remaining shared packages. VOT retirement is a different repository/data boundary; confirm specific
app/service/routes/build/docs removal and any real data migration before execution.

## Acceptance / evidence

- Every inherited obligation disposition explicit, evidence/source owner linked; rename isn't completion.
- Selected ported code/test contracts/data mapping and notices preserved; no whole-monorepo dependency.
- Regression/export/import/restart/recovery on dedicated fixtures, provider effects reconciled safely.
- Reverse dependency/caller/build/package graph shows selected obsolete surfaces only; shared packages
  remain if used. No unapproved secret/private config/public data spill.
- Rollback plan/source refs and user approval at private VOT/data boundary; no branch deletion/default
  switch/repo archive/force/merge/deploy/drop DB implied.

## Future execution prompt

> Before selecting retirement execution, read VOT #773, inherited obligation inventory and all
> required EvoBase evidence. Produce a read-only handoff readiness assessment with exact source
> SHAs, selected code/tests/data/notices, reverse dependency map, missing gates and recovery plan.
> Do not treat the product rename, new branch or design completion as permission to delete VOT or
> production data. Ask for the precise remaining repository/data actions when authorization is
> absent; complete safe inventory independently. Only after explicit scope approval, remove the
> narrowly obsolete app/service/routes/build/docs surfaces on a new reviewed branch, preserve
> still-used packages/history and run independent EvoBase plus remaining-source regression and
> isolated recovery checks. Do not close inherited issues without acceptance evidence. Report
> exact changed paths, remote SHA, passed/failed/unrun gates, rollback refs and residual data/provider
> risk. Never force-update/delete legacy refs, switch default, archive repository, drop DB, merge
> or deploy from this prompt. Stop when the approved handoff/removal gate is established; further
> cleanup needs another reviewed item.
