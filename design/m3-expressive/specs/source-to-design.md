# Source grounding and decisions

Verified on 2026-10-01:
- EvoBase vision #1 plus comment: https://github.com/loveoverflowcom/evobase/issues/1
- Existing source baseline: develop@4cb5873200f4d735b17b75fdd3cb88c2efdbb322
- Flutter main.dart uses Material3 plus blue seed, theme mode cubit. It is a workbench for old REST/auth/messaging rather than a fully implemented AppSpec product.
- No AGENTS.md or committed canonical complete token package in the EvoBase baseline recursive tree. Relevant skill adaptation is owned by the companion plan branch.
- VOT source audit reports current relation baseline many-to-one Ref + restrict delete. Do not label general 1:1/N:M/migrations as already available.

## Design preserves vision, not speculative implementation claims

| Vision boundary | Design response |
|---|---|
| Portable definition / runtime state / host bindings | Builder draft, Runtime data, connector account/binding are visibly separate surfaces |
| Stable IDs and typed Ref | relation chips + field inspector + import resolution; readable labels, hidden persistent IDs |
| Relations do not imply permissions | projected authorized picker; policy scenario check; unavailable output instead of data leakage |
| Captured value vs live lookup | OrderLine price stays in the order; current catalog price is a different field |
| No logical edit requires SQL DDL | draft labels do not imply deployment; fixed-store remains measured spike/ADR gate |
| Tenant isolation is separate from AppSpec | platform-operator-only isolation screen; ordinary Runtime names workspace, not database details |
| Durable functions + external uncertainty | finite ordered steps, pinned version, simulated preview, unknown result/reconcile state |
| Optional Email/Zalo/messaging/bot | capability/consent surfaces; Zalo not faked from phone number; bot scope has forbidden commands |
| Supported local profile remains | offline local draft != confirmed server action; portability/recovery gated by ADR/conformance |

## Open choices

- `.evobase` artifact extension/codec, import identity mapping and compatibility: ADR before shipping.
- Fixed-layout codec/index/constraint/query support envelope and DB-per-tenant operations: measure before settling production adapter. No physical isolation/PITR promise.
- 1:1, N:M, delete cascade/set-null, general evolution and offline sync: target choices with separate gates, unsupported controls disabled until conformance.
- Leptos component library/accessibility/virtualization adapter: inspect package evidence before selecting; no JS framework swap inferred from visual source.
- Compose Multiplatform Expressive availability on each target: independently verify; AndroidX release availability does not settle CMP/iOS support.
- Functions provider retry/idempotency/reconciliation and Zalo capabilities: per-provider contract, no exactly-once promise.

## Ownership

This branch publishes original design assets and prototypes only. Companion `chore/evobase-reboot-plan` owns `.agents/skills` and sequential `docs/work-plan` PR prompts. `rewrite/leptos-cmp` is reserved for future product implementation. Legacy archive refs are non-destructive; no default-branch switch, merge, deploy, DB migration or VOT retirement is performed by this design branch.

## Asset provenance

- EvoBase mark and relation-orbit SVG: original authored vector work in this task; no copied product icon pack.
- Open Sans regular/semibold files: Apache License 2.0, Version 1.10; full assets/LICENSE-Apache-2.0.txt plus original Debian attribution metadata in assets/OpenSans-NOTICE.txt.
- Screen samples: fictional organizations/order IDs/content, no customer records, credentials, tokens or private configs.
- Source vision issue JSON contains only the already-public vision and comment; private repository links in that public issue are contextual references, not embedded private code/data.

Native-source deliverables and PNGs do not imply mounted Leptos/CMP, backend, benchmark or provider tests exist. Future PR gates name exactly which evidence is still required.

Foundation pack covers workspace/session-expired/denied states. A new provider sign-in journey is deferred until identity/host-auth ADR; settings is not evidence of a complete auth journey.

Font verification: both TTFs are Version1.10; embedded copyright is Google2010–2011 (regular) /2011(semibold). Original Debian notice declares Apache-2.0. The misleading prior OFL filename/claims were corrected before publication. Full Apache2.0 text matches /usr/share/common-licenses/Apache-2.0; no font binary changed.

PNG exports use a256-color palette for small issue previews. Editable layered SVGs preserve the vector source; palette export is reproducible and visually inspected, not browser/native evidence.
