# Report contract and authorized sharing

Use for completed and partial inspections. There is no implemented report assembler asserted
in this skill pack. Produce legible Markdown/structured evidence with available local tools;
future automation requires an explicitly scoped implementation task and tests. Metadata validation
does not authenticate vision, build provenance, redaction or product behavior.

## Report identity and check outcomes

Record mode/scope and production screen/actual consumers; source + relevant local changes;
build/deployed identity; pinned design; renderer/target/tool versions; scenario/fixture/dependency
boundary; approved environment/account role; theme/VI-EN/input/font scale/viewport with units;
reviewer/run times; checks and evidence; artifacts/redaction/retention/sharing status.

Use `unknown` where not observed. Overall **complete** means required scoped checks finished,
not “no defects.” Blocked/inconclusive checks make required coverage partial. A zero-finding
result is meaningful only for completed required checks. Keep defects found before later blockers.

Separate proposed/source/static/core/integration/DOM/host/native/provider claims using
[shared UI policy](../../evobase-ui-engineering/references/shared-ui-policy.md). Name exact real
boundaries; fake-backed interaction cannot become provider acceptance. Add captured vs inspected
pixels independently. A design preview is not a browser/host/native screenshot.

## Attributable image record

Each cited image needs origin, path/bytes/digest where available, capture source/build/scenario
checkpoint, theme/locale/state, viewport/units/scale, captured time, inspected time/tool/reviewer,
concrete visible observations and redaction scope. Hash inspected bytes after permitted redaction;
do not hash another file with the same name. Record image/tree checkpoint skew and missing fields.
If no image-capable tool was used, retain captured-only status and disclose the gap.

Capture context plus detail when both are needed. One frame can suffice if it unambiguously shows
the control and context; explain it. Unopened images, authored booleans and successful XML/report
generation do not prove visual acceptance. Static comparisons require matching inputs; historical
bytes do not prove current HEAD. Preserve before/fix-or-unchanged/recheck honestly.

## Finding record

```text
Title/component and stable symptom identity
Kind: bug, contract deviation, usability hypothesis or polish
Severity + concrete impact; confidence + basis (kept separate)
Authority: product contract/revision, internal guideline/heading or verified external clause
Applicable scope and intentional-state/exception considered
Actual vs expected; preconditions/steps/frequency
Evidence: image(s), exact DOM/semantics/actions/checks and named source when inspected
Root cause: unknown unless source evidence establishes it
Suggested owning seam/fix, clearly proposed
Same-scenario acceptance/recheck and behavior to preserve
Related finding/issue only if actually checked
```

Avoid invented metrics or arbitrary severity from an aesthetic preference. A missing primary
action can be a concrete usability concern, but a scan heuristic is not measured user research.
Only establish a standards violation after checking the authoritative standard version/clause
and applicability; measured token pairs alone cannot establish screen-level conformance.

## Local output and publication

Default is local report/draft, never publish. Use the repository's actual evidence directory and
ignore/retention policy; if absent propose one, do not claim storage already exists. Keep secrets,
session files and raw user content outside tracked reports/scenarios. Local file paths do not
become attachments readable by another user or executor.

If sharing is explicitly authorized for the named report/files, destination and audience:

1. Review sanitized report, image pixels, DOM/tree/logs, metadata and URLs for private data.
   Record who reviewed which copies. Do not expose raw credentials or private user content
2. For requested issue publication, search matching open/closed findings by stable component,
   rule and symptom. New source/run/viewport alone does not create a new bug. Closed matches
   need triage; do not reopen, close or change unrelated metadata without the requested authority
3. Use only an available authorized upload/attachment transport. Do not invent endpoints, create
   buckets, run CI for storage or commit captures merely to obtain URLs. Verify recipient access
   and retention. Missing transport leaves attachment sharing blocked and the report local
4. Record destination/intent/content-evidence digest and pending status before a write. Read back
   the returned item and verify body/links/access before claiming published. On timeout/unknown
   outcome reconcile by identity before retry; do not blindly duplicate an issue/upload/comment

Publishing a plan PR does not authorize posting unrelated UI findings. Report drafts and any
partial sharing separately from successful publication. Fixes and golden approval remain outside
inspection scope unless explicitly requested. Never conceal failure by changing expected results,
thresholds or scenarios.
