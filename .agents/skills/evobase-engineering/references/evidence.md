# Proportional evidence and honest assurance

Map claims to oracles before writing tests. First ordinary deterministic examples/exhaustive finite
cases; add properties, differential replay, mutation sensitivity, fuzz robustness or bounded formal
methods only where useful/configured. These are complementary scopes, not a universal ranking.

## Techniques

- Core test: exact values/errors, edge partitions and independent expected decisions.
- Property: domain-aware generator/shrinker; invalid and valid cases; deterministic replay seed.
- Differential: genuinely independent oracle, pinned comparator inputs and target profile.
- Mutation: named injected defect and kill result; green tests without kills prove little sensitivity.
- Fuzz: bounded decoder/codec robustness; no crash is not policy correctness or compatibility proof.
- Kani/model check: proposition, assumptions, symbolic domain, bounds, unwinding and reachable paths.
- Lean/theorem: executable semantics/refinement, axiom/assumption scope and production linkage.
- Integration: actual DB/process/provider/identity boundary, not a fake or ignored test annotation.
- UI: production DOM/semantics, real scenario/input, inspected pixels and selected native boundary
  separately. A snapshot alone misses behavior; a host tree alone misses OS lifecycle/IME.

Never lower an obligation, edit expected output or accept a golden merely to turn a gate green.
Check test count, feature/target filters, ignored/skip results and run freshness. Record source SHA,
patch, tool versions, exact command/status, fixture/double/provider identity and artifact/log hash.
An assertion in a planning document is a proposed acceptance criterion, not verifier output.

Generated run logs/captures belong outside tracked source by default. Retain reproducible fixture
inputs and short evidence records; approved design assets are authored deliverables, not live-run
proof. Do not delete evidence/caches without authorized recoverability and resource need.

## Phase exits

AppSpec: codecs and native/WASM vectors. Storage: real roles/catalog/tenant negatives/concurrency/
failure/restart/bench. Functions: checkpoint/fence/retry/unknown-ACK/current-consent. UI: actual
Leptos and CMP consumers with selected semantics/interactions/accessibility/pixels/native runs.
Connectors: separate mock/provider/real-target evidence. Recovery: isolated export/import/restart.
Missing one level prevents that claim; it does not erase completed lower-level work.
