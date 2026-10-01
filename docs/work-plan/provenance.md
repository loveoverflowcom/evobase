# Source provenance and safe public adaptation

Source inspection date: 2026-10-01. This change adapts engineering/design guidance and writes
future PR prompts; it ports no product code or real data. The repository owner explicitly
authorized adapting private VOT code/skills into public EvoBase. That permission does not transfer
VOT infrastructure credentials/accounts or authorize retirement/deployment/data migration.

## Pinned baseline

- EvoBase `develop@4cb5873200f4d735b17b75fdd3cb88c2efdbb322`, public; default develop.
- EvoBase `re-design@5f6a98f684ab34ecc727c9354a51e4806454cc8f`, preserved independently.
- VOT `develop@de39212007404196774fabc5ff98caf5624fea61`, private source reference.
- [EvoBase #1](https://github.com/loveoverflowcom/evobase/issues/1) and its transfer comment;
  [VOT #773](https://github.com/loveoverflowcom/vot-workspace/issues/773) owns later source retirement.

## Guidance adaptation

| Source package under VOT `.agents/skills/` | EvoBase owner | Preserved / deliberately changed |
|---|---|---|
| `vot-engineering` | `evobase-engineering` | invariants/types/pure core/evidence/resource/delivery; narrow EvoBase AppSpec/policy/storage/functions refs; no private workspace/xtask/deployment assumptions |
| `vot-code-review` | `evobase-code-review` | read-only assurance/invariant/bypass/freshness; proportional passes, no phantom obligation registry |
| `vot-ui-engineering` | `evobase-ui-engineering` | Leptos async/DOM/visual/localization/shared-token ownership; canonical new design handoff |
| `vot-cmp-ui-engineering` | `evobase-cmp-ui-engineering` | feature/lifecycle/semantics/native evidence; Runtime-first and independent target discovery |
| `vot-ui-inspector` | `evobase-ui-inspector` | scenario/account-scope/report/pixel judgment; no inherited QA creation grant or nonexistent report CLI |

All source packages are pinned at the VOT revision above. New concise references are adaptations,
not verbatim copies or assertions that VOT runners have been installed. Source attribution and
existing notices must remain when later code is selected for reuse. Public skill documents omit
private operator hosts/IPs, account metadata, personal filesystem paths, internal endpoints/tokens,
branding-specific authentication deployment and unrelated product routing.

Renderer guidance additionally drew from the pinned source packages' reference files:

- `vot-ui-engineering/references/`: shared-ui-policy, material3-expressive, localization, ui-styling,
  web-execution, leptos-async, leptos-dom-browser-interop, leptos-ui-testing, leptos-wasm-performance,
  visual-verification, probe-usage, visual-review-guide
- `vot-cmp-ui-engineering/references/`: cmp-architecture, cmp-conventions, cmp-design-system,
  cmp-testing, semantics-inspection, visual-review, research-adaptations; its cmp-skill-cases evaluation
- `vot-ui-inspector/references/`: reuse, execution, report, catalog, publication

All listed references are Markdown under the source package. Old runner scripts, executable scenarios,
account data and credential files were not retained. The UI skill task walkthroughs evaluated guidance
decisions only, not live rendering/provider behavior. Foundation/review guidance was synthesized from
its source entrypoints, invariant audit and narrow diff-review contract, with EvoBase-specific refs.

## Product evidence informing future prompts

- EvoBase `crates/evobase-core/src/auth.rs`, `crates/evobase-gateway/src/state.rs`:
  current user-only/global storage context is not tenant/app/current-membership enforcement.
- EvoBase `crates/evobase-db/src/postgres.rs`: legacy provisioner/runtime credential seam.
- EvoBase `crates/evobase-messaging/src/hub.rs`: RAM messaging is not durable worker evidence.
- VOT `apps/votable/shared/model/src/{lib,application}.rs`: bounded typed model and complete-input
  form contract; core supports versions beyond current hosted authoring.
- VOT `apps/votable/shared/engine/src/{lib,business}.rs`: private CompiledModule/DecisionPlan/
  CheckedWrite, validated normalized facts and immutable business semantics.
- VOT `services/votable-api/src/{auth,policy,store}.rs`: verified host context/current grants,
  independent capabilities, current membership before receipt replay, scoped lock and atomic
  checked writes/revision/pin/audit/outbox/receipt. Adapter uses concrete SQL tables, not fixed store.
- VOT `apps/votable/docs/adr/`: ADR0009/0010/0011 inform business pins/commands/atomic facts.

These are **source-inspected**, not re-executed build/test/benchmark/provider results. No complete
tenant fixed-store/no-DDL or durable provider-worker evidence is inherited. Test names in the
prompts describe the next oracle to establish, not a pre-existing EvoBase green suite.

## Licenses and obligations

EvoBase Cargo workspace metadata declares MIT; no root LICENSE file was found in inspected
EvoBase/VOT roots. Do not invent a license file or strip file/package notices. Before copying code,
inventory specific package/file/dependency licenses and preserve attribution; owner permission
covers the approved reuse, not unrelated third-party code or secrets. No blocker is inferred from
the absence of a root notice alone.

Inherited requirements #765/#772/#525/#570/#752/#698/#583 remain open obligations until acceptance
and handoff, not automatically completed by rename/repository transfer. Legacy branches and data
remain untouched; archive refs preserve exact heads rather than deleting history or archiving repo.
