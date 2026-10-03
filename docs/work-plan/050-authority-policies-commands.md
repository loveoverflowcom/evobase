# 050 — Establish trusted host authority and checked policy/commands

Design issue: [EvoBase #6](https://github.com/loveoverflowcom/evobase/issues/6). Assets pinned at `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`.
Status: implemented experimental definition-owned owner/role policies and SubmitOrder checked
intents with FixtureHost/simulated browser facts. [Batch evidence](batch-evidence.md) records exact
checks. Verified session/API, durable receipts and transactional persistence remain open.
Design pack: 04 policies/actions.
Skills: engineering → authority, boundary-hardening, core; Web for policy/command projection.

## Scope / dependencies

020/040 core and reviewed identity/policy ADR. Replace VOT-specific Gateway deployment with a narrow
verified EvoBase session/actor adapter; no client actor/tenant headers as authority. Support a small
typed owner/role policy and SubmitOrder command. No claim of arbitrary SQL/RLS policy or complete auth
product from a demo token. Provider/OAuth/persistent access changes require applicable approval.

## Delivered boundary

`RawAppSpec.policies` and `.submit_rules` own portable abstract rules, never host grants.
`SessionVerifier` and opaque checked context/projection/write types expose the trusted host seam;
all current exercised identities are fixture/simulation supplied. Ref fields are not readable
policy outputs, and complete restricted relation scans deny. SubmitOrder returns checked write/
audit/receipt intent; the host must atomically persist expected revision and all related facts.
The verified transport/provider/API adapter is the next gate before PR060 storage.

## Acceptance / required tests

- Verified session → canonical actor → current app/tenant grants → registry routing; forged/expired/
  revoked/missing/wrong-app/tenant/origin context denies with bounded safe diagnostics.
- Separate Design/Publish/Manage/business grants; AppSpec cannot self-grant platform/admin facts.
- Old/new row state + field mutability + business guards; owner/tenant manipulation blocked.
- Query/picker/lookup/aggregate/export output policy; hidden field/count/existence leak negatives.
- Opaque checked decisions/writes, normalized values/trusted captures; wire/DB/FFI bypass audit.
- Current authorization before receipt replay; exact key+intent replay and changed-intent conflict.
- Structured policy sentence/diagnostic/source rule in Builder and honest unsupported subset.

## Original prompt and remaining acceptance

The prompt below preserves the original target and gates. The current user-authorized batch
delivered the experimental subset named above; it does not satisfy every original criterion.
Follow up on the remaining gate rather than treating the complete prompt as implemented.

> Select PR050 after core gates and resolve host session/native identity and policy subset first.
> Trace existing verified VOT context/current-membership-before-replay invariants without importing
> private deployment/account settings. Build a narrow trusted EvoBase adapter and deterministic
> typed policy/SubmitOrder command, with private checked outputs consumed by host adapters.
> URL/body IDs remain selectors, never database/credential authority. Test forged/revoked/missing/
> wrong-scope/origin context, independent capability grants, old/new state/mutability, business guards,
> receipt replay revocation and output leaks across Ref/picker/lookup/aggregate/export. Do not silently
> drop policy when translation is unsupported. Bind structured Leptos policy/command editing and
> diagnostics to the same Rust definition; UI visibility is not enforcement. Add API path negative
> tests with explicit fixture identity and safe origin handling. Run exact scoped core/API/UI checks,
> formatting and inspected permission/error states, disclosing any fake identity boundary. No
> production provisioning, persistent credential expansion, merge or deploy. Stop at authority and
> checked-command gate before PR060 real persistence.
