# 130 — Add a scoped bot with checked tools and human handoff

Design issue: [EvoBase #13](https://github.com/loveoverflowcom/evobase/issues/13). Assets pinned at `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`.
Status: proposed future implementation. Design pack: 11 bot.
Skills: engineering → authority/functions/storage; Web/CMP for scope/approval/handoff.

## Scope / dependencies

050 checked policy/commands, 110 connector consent, 120 conversation handoff. One service actor may
query allowed Products and propose/create OrderDraft through approved checked command. It cannot
MarkPaid, modify inventory/policy/credentials or infer customer identity from display name.
Knowledge/model provider bindings are optional host contracts, not dependency on all VOT/Savy.

## Acceptance / required tests

- Explicit allowed projections/commands, tenant/app/release/binding/policy scope and budgets.
- Tool requests bounded/typed and independently authorized now; model output never a checked plan.
- Retrieved document/message prompt injection cannot widen tools, read hidden data or change grants.
- Cross-tenant search/vector/prompt/history/cache and revoked policy/consent negatives.
- Customer linking verified; risky action approval explicit; unknown provider outcome safely surfaced.
- Human handoff/denied/unsupported/simulated model states and audit provenance accurate.

## Future execution prompt

> Select PR130 after authority/inbox gates and define one bot actor's precise projection/command
> scope. Implement bounded typed tool requests routed through the same Rust policy/command core;
> model/retrieved content is untrusted and cannot produce authority or deserialize checked writes.
> Use a deterministic fake model first and label simulation. Test prompt injection, malformed/extra
> tools, hidden fields/aggregate leaks, cross-tenant index/history/cache, revoked grant/consent,
> unverified customer identity, budget exhaustion and approval/human-handoff paths. Preserve pins,
> correlation/audit and durable unknown-outcome handling. Bind Web/CMP scope/approval/denied/handoff
> screens to real decisions and inspect representative states. Real model/provider connections,
> data sharing and external messages require applicable account/data/recipient approval; pause only
> dependent live steps when absent. Report fake versus real-target evidence and exact negatives;
> no autonomous payment/inventory/policy/credential action or broad AI guarantee. No merge/deploy.
> Stop at the constrained query/order-draft bot contract, leaving new capabilities as separate work.
