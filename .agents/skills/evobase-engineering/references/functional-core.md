# Functional core and checked effects

Use when a durable decision is mixed into HTTP/async/UI/storage/provider code. Trace actual caller
and lifetime first. Pure domain rule accepts typed facts/context and returns a checked decision or
typed diagnostic/effect intents; I/O shell loads trusted facts and executes intents atomically.

## Extraction recipe

1. Name invariant and existing behavior/oracle; map current source inputs and side effects.
2. Separate untrusted request, current trusted actor/grants/release/data and host capabilities.
3. Introduce private checked output only if it closes an actual invalid-state path.
4. Move deterministic decision without changing meaning; inject clock/IDs/external facts as data.
5. Retain thin adapters for query/storage/HTTP/FFI; test wire and transaction paths separately.
6. Compare old/new vectors and side-effect traces; delete obsolete duplicated decision once proven.

Good: Rust `decide_submit(checked_definition, current_facts, trusted_context)` returns normalized
checked writes + event intent; host commits them. Bad: UI decides owner/price/grants while server
persists request JSON or SQL recomputes a different rule. Kotlin may own menu/focus/loading state;
moving a local boolean across FFI is needless ceremony.

Ordinary deterministic unit tests require no async/provider mocks for core decisions. Fake adapter
tests cover mapping/failure orchestration, not real isolation or provider behavior. Query/formula
purity prohibits hidden clock/network effects. Do not force a new abstraction onto trivial glue.

Dependencies point from adapters toward domain contract. New shared package must have an immediate
consumer and narrow responsibility, not become a god context owning every renderer/service.
