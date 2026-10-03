# Single-tenant libSQL batch evidence

Base: `1ae1457a12a9894bfd35a550028e198ffaaacf38`, on `develop`, 2026-10-03.
Scope: ten additional commits under [ADR 020](decisions/020-single-tenant-libsql.md).
GitHub issues #1 and #3–#21 were read; the selected consumers are #7, #9 and #16–#19,
with bootstrap release identity from #8. This records bounded delivery, not closure of those epics.

## Delivered sequence

| Order | Commit | Review boundary |
| --- | --- | --- |
| 1 | `93f10f6` | Skills, issue reconciliation, single-tenant profile and ten-step queue |
| 2 | `9f2306d` | Compatible v1/v2 definitions and checked declarative field constraints |
| 3 | `c62b52c` | Fixed-layout local/remote libSQL adapter and immutable bootstrap snapshots |
| 4 | `abdb6c3` | Generic finite transitions, typed inputs and checked event projections |
| 5 | `0cb8f41` | Atomic facts/revision/audit/events/receipt commit, CAS and replay |
| 6 | `513ebf1` | Bounded, versioned, exact-value scope-free wire DTOs |
| 7 | `840c7f2` | Current operator bearer authority and actual generic HTTP host |
| 8 | `8b7f470` | Mounted Builder constraint authoring with atomic existing-data checks |
| 9 | `e7b69e2` | Generated HTTP Runtime, exact retry and protected context handling |
| 10 | Commit containing this final evidence | CI integration, fixture runner, review corrections and queue maintenance |

Basic snapshot storage preceded generic commands because its checked codec boundary was independent.
The dependency queue was renumbered to reflect the actual sequence. No business SQL DDL, production
migration, legacy retirement, issue closure or external message was part of this batch.

## Evidence boundaries

The actual local commands below use Rust 1.99.0, locked Cargo dependencies, two build jobs, Node,
wasm-bindgen 0.2.114, Playwright 1.63.0, Chromium and axe 4.13.0. Build artifacts, tokens and local
databases live outside tracked source. The reproducible commands are also in the checked-in CI.

| Executed check | Boundary and result |
| --- | --- |
| `cargo test --workspace --locked` | Passed. AppSpec: 50 runtime vectors plus 5 compile-fail doctests. Store: 13 actual local-libSQL tests, including a spawned process reopening and replaying. Host: 6 actual TCP tests. Protocol: 11 wire vectors. Existing legacy tests also passed; they do not establish a real PostgreSQL provider test. |
| `cargo test -p evobase-appspec --locked --target wasm32-wasip1` with `scripts/run-wasi.mjs` | Passed the same 55 AppSpec checks, including exact constraints, generic commands and authority/policy negatives. |
| `cargo clippy -p evobase-appspec -p evobase-builder --all-targets --locked -- -D warnings` | Passed the native client/core gate. |
| `cargo clippy -p evobase-builder --target wasm32-unknown-unknown --all-targets --locked -- -D warnings` | Passed the actual WASM client gate. |
| `cargo clippy -p evobase-store -p evobase-protocol -p evobase-host --all-targets --no-deps --locked -- -D warnings` | Passed changed backend packages. `--no-deps` keeps unrelated legacy-core lint debt outside this new gate. |
| Portable protocol `cargo check --no-default-features --locked --target wasm32-unknown-unknown` | Passed. Legacy protocol modules are feature-gated; the browser consumes AppSpec DTOs without native dependencies. |
| Scoped five-package `cargo fmt --check`; canonical token generator `--check` | Passed. Shared design source remains authoritative. |
| `bash scripts/build-builder.sh` | Built the mounted Leptos/WASM artifact and copied canonical tokens/assets. |
| `node scripts/check_builder.mjs` | 24/24 actual mounted-browser scenarios passed: constraints, atomic existing-row rejection, grid/form/import, save/cancel, exact i64, relations/policy and VI/EN × light/dark desktop/compact. No detected axe violations or page overflow in the checked matrix. |
| `bash scripts/check-runtime.sh` | **4/4 scenarios passed after the final UI fixes.** Fresh actual local libSQL/HTTP hosts for two unrelated AppSpecs. Checks generated forms, untouched optional inputs, typed rejection, CAS conflict, current denial, stale response cancellation, lost ACK and identical retry/recovery. Route interception delays/drops responses after actual server execution; it does not replace business decisions. |

The Runtime runner supplies only synthetic authority. Its second support record belongs to Bob;
Alice's auditor/editor roles allow it initially. Removing the read role exercises projection loss
and automatic selected-record rebinding. Library adds a second record and optional note input to
exercise preservation. The canonical fixtures are unchanged. The wrapper deletes its own temporary
credentials/databases and stops its hosts after each run.

## Review corrections and failed intermediate checks

Read-only engineering review traced checked construction, wire/storage/authority boundaries and both
mounted consumers. Corrections include coherent token/grant snapshots, fresh expiry time, complete
context comparison before transaction commit/output, finite-state validation on import/restore,
Read/Write intersection for visible actions, typed transport budgets, duplicate-header rejection,
exact retry retention, untouched optional omission, and cancellation/generation fencing.

Final UI review required clearing protected metadata/records/actions/receipts after Denied while
retaining the bound draft and unresolved exact request privately. Reauthorization restores a draft
only for the same surviving record. Any automatic replacement-row selection clears prior buffers.
This is client context handling; the host still authorizes every read/write/replay.

Earlier runs failed while core/host source was in flight (a moved variable and incorrect error-variant
name); both were corrected before the final checks. An initial backend clippy command including
legacy dependencies reported pre-existing `async_trait` double-must-use lints in `evobase-core`;
the scoped no-deps gate passed. The first browser runs hit collapsed-details click visibility in
the harness: Builder 23/24 and Runtime 2/4. The harness now opens the real summary before interacting;
controls and assertions were not bypassed. A late host formatting mismatch was corrected and the
final scoped format gate was rerun. These intermediate failures are not described as passes.

## Final source, build and artifact identity

The last source review used Runtime SHA-256 `2fc17442e9a01d8b6bcf7d692b5d1bc5e70425bbac8d7301504895a71db17d73`.
Final browser runs were taken at `8b7f470` plus the exact pending source now committed in `e7b69e2`;
reports record the dirty status, source-file hashes and browser artifact identity rather than
pretending the earlier HEAD alone identifies the build. Builder records served response hashes;
Runtime records built JS/WASM hashes. The final integration commit adds CI/docs and includes already-tested fixture-helper formatting
and transaction negative assertions. No UI semantic change followed those final browser runs.

Artifacts remain in this execution workspace; CI recreates and uploads separate Builder/Runtime
bundles. Local evidence inventory:

| Evidence | Local artifact | SHA-256 |
| --- | --- | --- |
| workspace tests | `/workspace/scratch/workspace-test.log` | `1961a8cc8b08e6574234942669bd3d0e8a94a1a304b9a167baf2c4bee8f517ce` |
| WASI core | `/workspace/scratch/core-wasi.log` | `ce9d2824d2751f35b94e08b99aa91945e9677b2a7b5ca59f4406e416109d119b` |
| native strict clippy | `/workspace/scratch/client-clippy-native-final.log` | `eaf7389a2cc84320b9e192cab5a939bbfa26248531592f5324113d90d72db52a` |
| WASM strict clippy | `/workspace/scratch/client-clippy-wasm-final.log` | `0de99f512f7a1afaa290185aea36050647a372b9a778d51cd5189769c4c39236` |
| backend scoped clippy | `/workspace/scratch/backend-clippy-scoped.log` | `6948a037d1122b48e257aa9a7f3aadc5804c5c7b365ccecbfb7d49f125d5b824` |
| Builder browser | `/workspace/scratch/builder-verified/builder-browser-report.json` | `c6738cf7e98c108bb728bd1ba3eb7e8d360cb9463b65e5b7f3b214074844e9ec` |
| Runtime browser | `/workspace/scratch/runtime-verified/runtime-browser-report.json` | `73a07636ea6bf000b73cdfa61b09b4a2deb6a3a82047e01ded5b3e21af36895b` |
| Build artifact | `apps/evobase-builder/dist/evobase_builder.js` | `b287c7f9bd8d595bb15c1061493e986fe4377a64c7ddfda43fe2e71d97f56550` |
| Build artifact | `apps/evobase-builder/dist/evobase_builder_bg.wasm` | `2c73d5b5eea456cb0d2719ee5f81ae34ec04454dec99f3cebf20c867949ee499` |
| Build artifact | `target/debug/evobase-host` | `96caa1a45fced49ebce5549a30541db5d447784dc1cb321302e74e75a6bcd8e8` |

The following final-run captures were actually opened and inspected for layout, visible state,
readable action labels, focus and compact reflow. Other captures in the reports are generated
artifacts, not an inspection claim. DOM/axe assertions and pixels remain separate evidence.

| Opened capture | SHA-256 |
| --- | --- |
| `runtime-verified/runtime-support-en-light-1200.png` | `2885aa7bd91e4bbdd59f1c573530613d5b1efcf43c1fff3871466deb18c86511` |
| `runtime-verified/runtime-support-vi-dark-390.png` | `9efe18f7c122b2e9ab20c9ba02bcdc9c5b7c2b585a3e58641bb9af270fd85834` |
| `runtime-verified/runtime-support-recovered-en-light.png` | `13ed6e8a5016340ebbf95d772974a817eac6a9ebe933211eb3ae0400b6c0df6b` |
| `runtime-verified/runtime-library-form-en-light.png` | `ff5dc16e22f0710c763854c2c9adb56ef6aa273213c8c156fb01b09906a68335` |
| `builder-verified/constraints-vi-light.png` | `8cc99b4f1ba7f69a7f6725c3a48e7da3c62889c0f6f1976fe1fd693ed6fd52df` |
| `builder-verified/builder-en-dark-desktop.png` | `567351b5f35739ce1bfdfd0fa34895aa5738805b0b89aeb8c3720d6ff01a1489` |

Publication uses a normal fast-forward push of exactly ten descendants of the recorded base to
`origin/develop`. The final commit identity is resolved by `git log` (an evidence file cannot
contain its own commit hash). Remote branch equality and the triggered CI run are checked and
reported separately after pushing; the local checks above do not imply a remote CI pass.

## Remaining gates and next work

Turso/libSQL remote-primary support is implemented, but no remote account/credential was configured:
actual Turso TLS/transaction/disconnect/operational tests are **unrun**. Local tests are local evidence.
One configured tenant is implemented; multi-tenant provisioning/routing/credential isolation and
cross-tenant service tests are deferred as requested. Host authority is rechecked immediately before
commit, but is not in a distributed transaction with an external identity provider.

The supported store uses bounded whole-app snapshots (1 MiB / 4,096 records), first-page Runtime
lists with explicit truncation, immutable bootstrap pins and passive committed event intents. Full
publish/repin/evolution/backfill, secondary indexes/uniqueness expansion, durable dispatch/providers,
CMP/native, screen-reader/native IME and receipt recovery across page reload remain unrun or outside
scope. Window closure loses an unresolved Runtime request. See the [expansion backlog](backlog/support-expansion.md).
The next recommended bounded consumer is [260 command authoring](260-command-authoring.md).
