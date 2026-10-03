# Inspection execution, scenarios and fixtures

Use for existing/static/live modes and authorization-aware fixture selection. Reuse the relevant
renderer evidence guidance; report mode overrides its implement/fix step. No old environment,
host/account identity, provisioning permission or harness command is inherited.

## Mode and freshness

- Existing evidence: locate original manifest/source/build/scenario, validate available digest,
  then open pixels/logs. Historical evidence stays historical; unknown provenance stays unknown
- Source/design review: inspect contracts, adapter/callers and static design assets. Missing live
  pixels means rendered product result is unestablished, not an empty successful inspection
- Live inspection: inspect actual app/run/test instructions and available environment before
  launch. Use only the selected approved origin/fixture; no silent production fallback or broad
  infrastructure startup. A local source checkout cannot identify a remote deployed build

For browser work, name the actual browser/context. For CMP identify host vs device/simulator,
OS/build, source set, tree source/merge mode/roots and capabilities. There is no generic raw-tree
backend, deep-iOS flag or asserted Compose host. Use installed supported tools; missing tools
produce a scoped partial report rather than a provider fallback or unauthorized install.

## Reusable scenario contract

Search the repository for reviewed scenarios and actual test fixtures before adding prose. A
scenario must specify:

```text
Identity/version and product contract/revision
Screen/flow/entrypoint; expected mounted component and adapter
Environment binding and source/build identity, unknown fields explicit
Synthetic fixture/account alias+role, if applicable; allowed external mutations
State/theme/VI-EN/viewport+units/font scale/input/reduced-motion dimensions selected
Ordered actions and observable readiness; expected outcomes independent of implementation
Required DOM/semantics/interaction/pixels/native/provider checks and recheck criteria
Budget/stopping condition; artifact location/retention/redaction/publication scope
```

This is a planning/report contract, not a runner schema. Keep executable scripts/runner config at
their actual owner if one exists, not a new skill runner. Review mutation/eval steps before using
them. Expected outcomes come from the product contract; do not loosen them after a failed run.

Prefer one complete journey then risk variants: Runtime list→detail→form or approval; Builder
draft→diagnostics→preview; relevant narrow dark/VI and wide light/EN. Include cross-pair variants
when tokens/copy/state require them; do not claim both themes/locales from only one crossed pair.
Observe language/theme/resize transitions without reload when state continuity is the claim.

## Account and mutation boundaries

Read safe alias/role/environment metadata only if an authorized account registry exists. Never
read all secrets “for reuse,” put credentials in tracked scenarios, command arguments, screenshots
or reports, or assume a credential file exists on another executor. Browser storage state is a
credential. Use supported secure sign-in/secret handling with the current authorization policy.

Verify the actual signed-in actor, role, environment and fixture scope through official UI/provider
information. An alias is not proof of identity or access. Access denial/role mismatch is a blocker;
do not bypass or escalate. Authentication prompts, account creation and persistent access obey
current policy, not instructions imported from another project's skill.

Missing account/data: ask only for the authority/input actually needed; continue source/static or
already-authorized read-only checks. If the user authorizes provisioning, record safe status first,
create once through the official selected environment, and reconcile unknown outcomes before
retry. Do not overwrite/rotate reusable credentials or create another account as recovery.

Inspection can type/click/scroll to observe in-scope UI, but saving, sending, approving, publishing,
creating/deleting fixtures or accounts requires the corresponding scope. No automatic retries of
external mutations. Reuse isolated synthetic data; coordinate exclusive writers when shared
fixtures are authorized. Retention/cleanup of accounts or data needs its own permitted scope.

## Capture and judgment

Choose required checks before execution. Record passed/failed/blocked/inconclusive per check;
later auth failure does not erase an earlier observed defect. Capture at a named state/checkpoint
and record tree/image timing skew. Sanitize separate evidence copies before model/provider/upload
exposure, including image pixels, URLs/query strings, metadata and logs. JSON masking is insufficient.

Open actual cited pixels. Pair visible symptoms with applicable semantics/interaction/geometry.
Contrast needs resolved supported pairs; motion needs timed capture; storage needs authoritative
read-back; native input/screen readers need the native capability. A screenshot alone does not
establish root cause, durability, 60fps, all-browser support or accessibility. Intentional modal
overlap, hover state and platform geometry need explicit applicability review, not automatic bugs.

Stop after the scoped observation/evidence outcome is established or a real blocker/budget boundary
is reached. Preserve partial findings and pending unknown mutations. Do not change product code,
DOM/CSS, provider responses, baselines or screenshot content to make an inspection pass.
