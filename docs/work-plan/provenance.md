# Source provenance and safe public adaptation

Historical private-source inspection date: 2026-10-01; current public EvoBase inventory date:
2026-10-03. The current foundation commit adapts/imports guidance/design and reconciles documents;
it ports no product code or real data. Later commits in the authorized batch have separate evidence.
The repository owner explicitly
authorized adapting private VOT code/skills into public EvoBase. That permission does not transfer
VOT infrastructure credentials/accounts or authorize retirement/deployment/data migration.

## Pinned baseline

- EvoBase legacy `develop@4cb5873200f4d735b17b75fdd3cb88c2efdbb322`, public; preserved at
  `archive/develop-2026-10-03`. `develop` is now the selected refactor integration branch.
- Guidance/plan input `chore/evobase-reboot-plan@699fa0f4886c1b47c9ee21f6774bbdb9800a6758`;
  canonical imports are `AGENTS.md`, `.agents/skills/evobase-*` and `docs/work-plan/`.
- EvoBase `re-design@5f6a98f684ab34ecc727c9354a51e4806454cc8f`, preserved independently.
- VOT `develop@de39212007404196774fabc5ff98caf5624fea61`, historical private-source reference
  recorded by the planning handoff. That repository is unavailable in this execution workspace;
  no fresh source/test/data/dependency/license inspection or product-code transfer is claimed.
  Vision #1 also cites the earlier private `767f9dfb3835ccbb7bb1568c09b0ddb21d2419cf` snapshot.
  Preserve both identities and do not silently treat them as the same baseline.
- [EvoBase #1](https://github.com/loveoverflowcom/evobase/issues/1) and its transfer comment;
  [VOT #773](https://github.com/loveoverflowcom/vot-workspace/issues/773) owns later source retirement.
- Published M3 design pin `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`: 142 files beneath
  `design/m3-expressive/`, independently matched to reviewed local blob hashes. Full 27-screen/
  12-pack map, canonical tokens/contracts, 60 screen SVGs, representative PNGs and lightweight
  source ZIPs; expanded review ZIPs are separate Library deliverables. Bundled Open Sans notice/
  license metadata states Apache 2.0 / Version 1.10; preserved asset notices own that attribution.
- Design issues EvoBase #3–#14 were individually read back with exact design/prompt commit pins;
  see the [static issue map](issue-map.md). No implementation or source retirement is completed
  by publication of those design specifications.

## Public EvoBase source, tests and data inventory

The following inventory is **source-inspected at**
`4cb5873200f4d735b17b75fdd3cb88c2efdbb322` with exact repository-relative paths.
It records available artifacts, not executed baseline tests or a production data handoff.

| Artifact / path at that SHA | Observed ownership and transfer boundary |
|---|---|
| `Cargo.toml`, `Cargo.lock`; `apps/evobase-server/{Cargo.toml,src/main.rs}` | Seven-member Rust workspace, edition 2024/minimum Rust 1.89; server bootstrap/env/host wiring. No Leptos/CMP member in this baseline. |
| `crates/evobase-core/src/{auth,config,database,storage,rest,docs,messaging}.rs` | Shared host/storage/auth traits and user/config types. This is not already a pure AppSpec kernel. |
| `crates/evobase-auth/src/service.rs`; `crates/evobase-gateway/src/{state,middleware,routes,rest,topic_messaging}.rs`; `crates/evobase-gateway/src/handlers/` | Legacy JWT service and HTTP API. User-only/global routing does not prove current tenant membership or new app policy. |
| `crates/evobase-db/src/postgres.rs` | Concrete table access, introspection and database provisioner. Rewritten admin URL keeps credentials; no separated least-privilege fixed-store claim. |
| `crates/evobase-messaging/src/hub.rs` | In-memory hub/relay/TTL and topic state; restart loses RAM. Not durable outbox/functions evidence. |
| `crates/evobase-protocol/src/{auth,databases,docs,envelope,rest}.rs` | Existing wire DTOs/envelope, separate from the new definition contract. |
| `apps/evobase_workbench/{pubspec.yaml,pubspec.lock,lib/}` | Flutter workbench: client/auth/theme, API explorer, messaging, VI/EN locale files; macOS/Linux/Windows host wrappers. Retain historical functionality/docs; no automatic CMP rewrite. |
| `clients/flutter/evobase_flutter/{pubspec.yaml,pubspec.lock,lib/,example/main.dart}` | Standalone legacy Flutter client and example. No new host authority inherited. |
| `db/migrations/0001_init.sql`, `db/chat_app/0001_init.sql`, `db/todo_app/0002_init.sql` | Legacy schema/bootstrap SQL, not a fixed-layout logical-table codec; not production backups or live-record export. |
| `scripts/api-smoke.sh`, `scripts/evobase*.postman*.json` | Manual backend smoke/API examples, not native/WASM/browser/DB isolation acceptance. No live requests executed for this docs inventory. |
| `docs/`, `re-design/` | Existing run/architecture lessons plus documentation-only Rust-DSL/generated-runtime proposal. The [reconciliation](../../re-design/README.md) preserves earlier text; [legacy guide](../LEGACY_BACKEND.md) preserves original run examples with portable links. |

Source contains **13 Rust test functions**: one config test in
`crates/evobase-core/src/config.rs`, four bootstrap/registry tests in
`crates/evobase-core/src/database.rs` (three use Tokio and fake storage/provisioner), five
introspection/URL-builder tests in `crates/evobase-db/src/postgres.rs`, and three in-memory
messaging/TTL tests in `crates/evobase-messaging/src/hub.rs`. The Swift
`apps/evobase_workbench/macos/RunnerTests/RunnerTests.swift` has an empty `testExample`, not
assertions. No Dart test source or `.github/` CI workflow was found in the baseline. Future batch
checks must not count this inventory as those tests having run.

No live tenant/customer data, production database dump or private VOT data is available for
handoff here. SQL/example/design fixtures remain examples; imported design records are fictional.
`.env` is ignored by baseline `.gitignore`; no runtime environment or credential values are copied
into the public documents. Later data migration needs its own source identity, authorization,
identity mapping, recovery checks and export/import evidence.

## Dependencies and notices at the pinned source

`Cargo.toml` owns these direct workspace dependency declarations: argon2, async-stream,
async-trait, axum, dotenvy, futures-core, http, jsonwebtoken, rand_core, serde, serde_json, sqlx,
thiserror, tokio, tower, tower-http, tracing, tracing-subscriber, url and uuid. Their complete
resolved versions/checksums/transitive graph are pinned by the baseline `Cargo.lock`; SQLx uses
Tokio/rustls/PostgreSQL/UUID/JSON features. This inventory does not claim a transitive license audit.

`apps/evobase_workbench/pubspec.yaml` declares Flutter/flutter_localizations, http, fpdart,
json_annotation, flutter_bloc, equatable, shared_preferences, hive_ce, uuid, intl, logger and
cupertino_icons; dev dependencies are flutter_test, build_runner, json_serializable,
hive_ce_generator and flutter_lints. `clients/flutter/evobase_flutter/pubspec.yaml` declares
Flutter, http, fpdart, shared_preferences and logger plus flutter_test/flutter_lints for development.
Each package's `pubspec.lock` pins resolution; the workbench's `macos/Podfile.lock` pins its
CocoaPods graph. Both Dart manifests declare SDK `^3.11.3`. These are retained baseline
requirements, not the selected future Leptos/CMP dependency graph.

All seven Rust package manifests inherit the root workspace MIT declaration. The baseline has no
root LICENSE file; `clients/flutter/evobase_flutter/LICENSE` is a concrete MIT notice with
copyright 2024 Evobase and remains intact. No independent Dart/workbench or Rust source notice
was added or stripped. Any later reuse/distribution must retain applicable package/file notices;
third-party dependency licenses remain their owners' obligations, not inferred MIT from our root.

The imported M3 design assets are pinned separately at
`6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`. Preserve
`design/m3-expressive/assets/OpenSans-NOTICE.txt` and
`design/m3-expressive/assets/LICENSE-Apache-2.0.txt` with the bundled fonts and source ZIPs.
The design source is not evidence of production renderer support. Private VOT package/file/license
obligations remain an explicit inventory gap before any product-code transfer or retirement.

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

EvoBase paths above were rechecked against the public baseline. VOT paths below the historical
source pin are carried forward from the earlier handoff, not freshly inspected in this workspace.
These are **source-inspected historical claims**, not re-executed build/test/benchmark/provider results. No complete
tenant fixed-store/no-DDL or durable provider-worker evidence is inherited. Test names in the
prompts describe the next oracle to establish, not a pre-existing EvoBase green suite.

## Licenses and obligations

EvoBase Cargo workspace metadata declares MIT; the public baseline has no root LICENSE file.
The earlier handoff reported no root notice in its private VOT snapshot; this execution cannot
independently verify that observation. Do not invent a license file or strip file/package notices. Before copying code,
inventory specific package/file/dependency licenses and preserve attribution; owner permission
covers the approved reuse, not unrelated third-party code or secrets. No blocker is inferred from
the absence of a root notice alone.

Inherited requirements #765/#772/#525/#570/#752/#698/#583 remain open obligations until acceptance
and handoff, not automatically completed by rename/repository transfer. Legacy branches and data
remain untouched; archive refs preserve exact heads rather than deleting history or archiving repo.
