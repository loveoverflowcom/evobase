# 2026-10-03 refactor batch

Scope: five sequential commits on `develop`, authorized by the repository owner. The previous
head is preserved at `archive/develop-2026-10-03@4cb5873200f4d735b17b75fdd3cb88c2efdbb322`.
This is an experimental checked-core and browser local-draft slice; hosted/native/provider gates
remain separate. GitHub issues own completion status.

## Changes and checks

| Commit boundary | Owner / invariant | Evidence |
|---|---|---|
| `6e14972` foundation (#3) | Definition, runtime facts and host bindings have separate owners; original code/history/notices remain. | Five skills and document links checked; 9 handoff-checker tests; 142 design objects and 12 ZIP sizes/SHA256 match immutable design pin. |
| `6fa7bb0` AppSpec (#4) | Only bounded checked constructors validate identity, types and whole candidate graphs. Checked values cannot be deserialized as authority. | 7 native and 7 WASI vector groups passed; shared conformance reports byte-identical. Independent v1 golden plus exact malformed/duplicate/future/hostile/scope/value errors. |
| `3809311` local Builder (#3/#4) | Invalid imports never partly apply; saved input has an actual local receipt; failed restore and stale writers preserve source/input. | 16 mounted Chromium scenarios, including exact i64 bytes, synthetic composition events, history/focus, optional fields, atomic import, repeated saves, cross-tab Web Locks and corrupt restore. Six locale/theme/width captures had zero axe violations and no page overflow; all six opened for inspection. |
| `0173daa` relations and pure projections (#5) | Canonical Ref owns reverse edges; insertion derives captures; final-state changes check restrict and immutability. | Its exact staged snapshot passed 19 native core tests and a WASM Builder check. Three focused mounted scenarios passed and their three captures were opened. Pure projection ASTs are checked against the schema; portable formula authoring/export remains deferred. |
| Policy and commands (#6) | Current host facts plus canonical policy construct private checked outputs; exact retry cannot bypass revoked grants. | Simulated verifier/registry only. Real identity/API, transaction/CAS, release registry and durable receipt storage are still required. |

## Reproduce

Tools selected for this run: Rust 1.99.0, Node 24.19.0, Leptos 0.8.11 with paired macro 0.8.10,
wasm-bindgen CLI 0.2.114, Playwright 1.63.0 and axe-core/playwright 4.13.0. Dependencies are locked.
The browser build and runner commands are documented in [the actual consumer](../../apps/evobase-builder/README.md).

```sh
cargo test --workspace --locked
CARGO_TARGET_WASM32_WASIP1_RUNNER="node $PWD/scripts/run-wasi.mjs" \
  cargo test -p evobase-appspec --locked --target wasm32-wasip1
cargo clippy -p evobase-appspec -p evobase-builder --all-targets --locked -- -D warnings
cargo clippy -p evobase-builder --target wasm32-unknown-unknown --all-targets --locked -- -D warnings
cargo fmt -p evobase-appspec -p evobase-builder --check
python3 apps/evobase-builder/generate_tokens.py --check
bash scripts/build-builder.sh
```

The WASI test runner must use an absolute script path because Cargo tests run from the package
directory. Native/WASI vectors are semantic evidence for those targets; actual browser interaction
is separate. Browser composition events do not prove operating-system IME or native CMP.

## Final integrated results

The frozen fifth-commit candidate over `0173daa` passed the following checks before commit:

- Native workspace: 50 tests passed, including 13 unchanged legacy tests and 37 checked-core tests;
  three compile-fail doctests also passed. No failed or ignored tests.
- WASI: the same 37 checked-core tests and three compile-fail doctests passed under Node's WASI runner.
- Strict Clippy: both new packages, all native targets; Builder, all WASM browser targets. Package
  formatting, deterministic token generation, browser build and JavaScript syntax checks passed.
- Mounted Chromium: all 22 scenarios passed with no filter or skipped cases. They cover actual
  grid capture/import guards, local storage races and failed restore, canonical policy persistence,
  pending/interrupted source editing, independent grants and normalized simulated receipt replay
  with current revocation. Six VI/EN × light/dark desktop/compact axe scans had zero violations and
  no page overflow. All 11 final PNG captures were opened for visual inspection.
- Documentation: 9 checker unit tests; metadata, 140 relative links across 58 Markdown files and
  five required skills checked without errors. Imported design hashes remain unchanged.

Browser evidence identifies the dirty pre-commit candidate, source file hashes and actual served
resource hashes; its source hashes matched the frozen delivered app files. Local report SHA256:
`22659adaeb7575ab449b74ca7aadc683591db3afe083162fa13d0f18ae238b2d`.
Served WASM SHA256: `55545ac8056f6a61b56562a9766fa395720e83e310375446dfd6f8dd494d7cc2`.
Builder source SHA256: `7a1139e4ff5edbda9d878e70cba8aa7d9195802d9750511eae650e7e7fe179ce`.
The report's automatic `inspected: false` values are capture metadata, not a manual review result;
visual review is recorded here separately. Relation comparison columns use horizontal scrolling
in the narrow inspector, with a localized hint and keyboard-focusable scroll region.

The added GitHub Actions workflow repeats scoped core/browser checks and uploads browser artifacts.
These local results do not assert that a later GitHub Actions run has passed; its status must be
checked against the published commit separately.

## Review and remaining gates

Independent review traced wire/constructor, graph, output-policy, command and actual browser
storage paths. Regression cases protect canonical decode/compile bounds, normalized Blank
expansion, captured-field forgery, hidden-child count/existence, schema/facts drift, canonical
rule overrides, owner/state and order/line aliases, revoked receipt replay, lost local writes,
failed restoration, oversized programmatic batches before cloning and stale build evidence.

Unimplemented: real host session/registry integration, SQL transactions and concurrency, persistent
releases/migrations/receipts, native CMP, real OS IME/screen reader, live provider dispatch, broader
cardinalities/deletes, zoom/text-scale verification and portable formula declarations. Relationship access never grants authority;
restricted complete-child rollups and Ref-valued policy projections fail closed in this profile.
No production data was migrated and no provider action was dispatched.

Imported design blobs retain three pre-existing whitespace findings in two files so they remain
byte-identical to their reviewed pin. Whitespace checks for authored refactor changes exclude that
unchanged design import. The original planning scope checker is not a product or batch scope test;
its metadata/link checks were run with an explicit document scope.
The whole-workspace Rust formatter also reported pre-existing formatting differences in two untouched
legacy gateway files; formatting checks passed for the two authored Rust packages instead.
