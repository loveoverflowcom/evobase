# Assurance review passes

Choose passes from changed boundaries; do not preload unrelated formal tools.

## Types and construction

Trace private checked constructors and every serde/Default/FromRow/FFI/migration/fixture bypass.
Check resource-kind/scope and unsupported-profile errors. An unchecked plan or raw persistence
route is architectural risk even if happy-path tests pass.

## Functional/trust/storage boundaries

Trace one Rust semantic owner through Web/CMP/API/SQL adapters. Current identity/tenant grants
precede reads, mutation and receipt replay. Old/new policy, hidden inputs and immutable pins are
checked. Examine lock/CAS/transaction scope and failure points; external I/O is after commit.
No-DDL isolation needs actual runtime privileges/catalog/concurrency and two-tenant negatives.

## Test/oracle assurance

Verify production code is called, exact errors asserted, fixtures fresh and expected outputs
independent. Look for zero/ignored/filtered tests, generator vacuity, shrink holes and broad `is_err()`.
Mutation must show defects killed; differential must be independent; fuzz only proves robustness.
Formal claims name bounds/assumptions/axioms and production refinement, not just a green theorem.

## UI and execution evidence

Trace canonical tokens/components to mounted Leptos/CMP consumers. Check loading/error/empty/
conflict/repeated/stale/close/back/focus/IME states. HTML design fixture, browser DOM, host semantics,
inspected pixels and real native lifecycle are separate. Verify selected locale/theme/viewport/
fontScale/device inputs and provider fake/real labels.

## Planning/skill changes

Audit scope and prose commands: proposed future code does not authorize implementation. No private
infra/account permissions, nonexistent runners, dead local links or claimed enforcement from docs.
PR prompts have one reviewable outcome, dependency gates, acceptance, negative cases, exact future
evidence and non-goals. Preserve original owner grants/assurance without inventing perpetual rules.
