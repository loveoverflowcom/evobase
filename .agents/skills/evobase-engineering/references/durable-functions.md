# Declarative durable functions and external effects

Use for run/step state, triggers, timers, approvals, retries, outbox and connector dispatch. Current
RAM SSE/relay and a committed outbox intent do not establish a durable execution worker.

## Narrow first flow

OrderSubmitted → checked approval task → enqueue simulated notice → wait approval/deadline →
checked transition. Definitions are finite typed plans with explicit budgets/timeouts/retry/error
branches; arbitrary uploaded JS/Rust/shell/SQL/network execution is outside this profile.

Plan evaluation is deterministic and I/O-free. Trusted clock, IDs and provider facts enter as
typed context. Command+event/outbox intent commits atomically; workers act after commit. Persist
run/step IDs, pinned definition, input snapshot/reference rules, status, attempts, deadlines,
checkpoints, leases/fences and tenant quotas. Publish v2 never silently changes a v1 run.

## Reliability cases

- Restart after each state boundary; duplicate trigger/timer/event; committed step retry returns
  receipt rather than repeating business mutation.
- Two workers contend; lease expires; late worker completion fails fencing and cannot overwrite.
- Validation/auth/permanent failure does not retry forever. Transient retry bounded with backoff,
  quarantine/dead-letter and permissioned/audited manual retry.
- Provider accepted request but ACK lost: `OutcomeUnknown` with correlation/idempotency and
  reconciliation; never assert exactly-once email/Zalo/payment or fabricated success.
- Current grants/connector consent recheck at dispatch; pause/cancel/revoke cannot recall an
  already external effect. Describe cancellation and unknown-outcome UX honestly.
- Preview/simulation produces zero provider delivery and no live outbox effects. Fake provider
  receipts are visibly simulated in UI, tests and reports.

External effects require an authorized real account/recipient/environment before provider tests.
Do not use a product feature request to send test messages to real people. Inbound webhook must be
verified, deduplicated and bound to tenant/channel; untrusted payload never establishes actor rights.
Bot messages/retrieved documents remain untrusted; tools only request typed permitted actions.

## Handoff evidence

State-transition table; durable DB/process boundary; pin and fencing identity; retry envelope;
fake versus real provider evidence; exact failure injection/restart commands; remaining unsupported
primitives and residual delivery risks.
