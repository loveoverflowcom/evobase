# 100 — Execute one declarative durable approval function

Status: proposed future implementation. Design pack: 08 functions/runs.
Skills: engineering → durable-functions/storage/authority; Web/CMP for designer/run/task states.

## Scope / dependencies

060 transactions, 070 immutable releases, 090 approval contract. OrderSubmitted → approval task →
simulated notice → wait event/deadline → checked transition. Finite typed branching/mapping with
explicit budgets; no arbitrary uploaded code/general loops/shell/network. Fake provider labeled.

## Acceptance / required real-boundary tests

- Persistent run/step/input/pin/checkpoint/deadline/attempt/lease/fence state and tenant quotas.
- Command+event/outbox atomic; external worker I/O after commit, never wait while DB lock held.
- Restart each boundary, duplicate event/timer, committed-step receipt replay and release-v2/v1-run.
- Competing workers, expired lease and late completion reject by fence; idempotency keys stable.
- Transient retry bounded/backoff, permanent/auth/validation stop, quarantine/manual retry grant/audit.
- Lost provider ACK → OutcomeUnknown/correlation/reconciliation; pause/cancel not recall promise.
- Current grants/connector consent at dispatch, preview zero-delivery, simulation no live outbox effect.
- Function/run UI explains real durable status and pinned version/error/unknown state.

## Future execution prompt

> Select PR100 after durable storage/pins/runtime contracts. Implement one finite typed approval
> function and actual persistent executor/worker, with immutable definition pins, bounded retries,
> deadlines/checkpoints and leases/fences. Keep business command+event/outbox atomic; dispatch only
> after commit with current grants/consent. Build a controlled fake provider explicitly labeled
> simulated and support accepted-but-lost-ACK OutcomeUnknown rather than success fiction. Test real
> process restart at each state, duplicate triggers/timers/committed step, same-key replay, competing
> workers/late lease, permanent/transient classifications, cancel/revoke, unknown-outcome recovery
> and publish-v2 while v1 run remains. Bind declarative Leptos designer and Web/CMP run/task states
> to the actual state machine with inspected diagnostic/error/unknown frames. Discover exact durable
> test commands and retain trace/log identity; mocks do not prove restart. No real recipient/provider
> effect, arbitrary code, exactly-once provider claim, merge or deployment. Stop at one demonstrated
> E3 flow; expanding primitives requires separate acceptance.
