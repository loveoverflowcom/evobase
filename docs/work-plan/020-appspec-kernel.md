# 020 — Build a bounded checked AppSpec kernel

Design issue: [EvoBase #4](https://github.com/loveoverflowcom/evobase/issues/4). Assets pinned at `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`.
Status: proposed future implementation. Design pack: 02 data foundation.
Skills: engineering → AppSpec, boundary-hardening, functional-core, evidence.

## Why / scope

Establish portable semantics before storage or GUI. Extract/adapt narrowly from pinned VOT model/
engine with preserved notices and independent dependencies; current VOT native/host format profiles
are not automatically EvoBase formats. One Customer/Order/OrderLine/Product definition plus typed
input facts; no DB/auth session, general language, arbitrary scripts or visual product claims.

## Dependencies / risks

010 reviewed encoding/version/identity/money/null/profile decisions. Fresh source inventory confirms
what is reusable. Replacing narrow identity dependencies must preserve resource-kind/scope invariants.

## Acceptance / required tests

- Raw bounded DTO → checked definition with private construction; unknown/incompatible tags reject.
- Stable app/table/field/record IDs survive rename/reorder/localization; duplicates/kind/scope invalid.
- Explicit required/optional/Blank, exact numeric precision/ranges and deterministic diagnostics.
- Canonical golden encode/decode/encode, legacy/future incompatibility, hostile bytes/nesting/counts.
- No bindings/secrets/real host grants/tenant data in portable definition; checked plans not wire-trusted.
- Native and WASM same vectors within advertised profile; tests fail for realistic bypass/regression.

## Future execution prompt

> After selection and PR010 review, pin the authorized base and import the reviewed contracts/skills.
> Inspect VOT source at the provenance SHA, then build the smallest independent EvoBase raw/checked
> AppSpec kernel for the customer-order-line-product fixture. Use fallible private IDs/constructors,
> bounded decode/compile and one canonical serialized definition; do not copy the whole monorepo.
> Add positive/negative golden fixtures and deterministic tests for exact diagnostics, IDs, types,
> overflow/null/Blank and future/hostile formats. Keep clock/IDs/current identity external trusted
> facts and query evaluation pure. Discover pinned Rust/WASM tools/runners and document exact commands;
> installing approved tools is not a reason to broaden this PR. Run focused tests, native/WASM vectors,
> formatting and proportional aggregate checks on final code. Report unsupported profiles and unrun
> targets explicitly. Open a draft PR toward the selected rewrite integration base only if authorized;
> no merge/deploy/DDL/live data. Stop when core/golden/cross-target gate is established or a required
> decision/tool permission blocks it. PR030 may consume only these checked APIs.
