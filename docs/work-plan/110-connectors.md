# 110 — Configure verified Email/webhook connector contracts

Design issue: [EvoBase #11](https://github.com/loveoverflowcom/evobase/issues/11). Assets pinned at `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e`.
Status: proposed future implementation. Design pack: 09 connections.
Skills: engineering → durable-functions/authority/boundaries; Web for mapping/test/consent.

## Scope / dependencies

100 durable dispatcher and unknown-outcome contract. One Email action + verified webhook inbound
adapter with typed capability schema/mapping/receipts/consent/quotas. Provider secrets stay host-side
handles, never AppSpec. Zalo and arbitrary provider parity deferred to explicit capability spike.

## Acceptance / required tests

- Allowlisted connector capabilities/input/output/provider identity and safe typed mapping validation.
- Account/consent/secret handle lifecycle scoped tenant/channel; current revoke checked at dispatch.
- Verified webhook signature/replay/dedup/correlation and wrong-tenant/channel rejects; payload cannot
  self-assert actor/tenant authority.
- Stable correlation/idempotency, accepted-but-timeout unknown outcome/reconciliation, bounded retry.
- Mock versus provider sandbox versus authorized real target evidence recorded separately.
- Test/preview sends zero real deliveries unless explicit account/recipient/content approval; UI
  consent/failure/unsupported/rate-limited/unknown states truthful and credentials never exposed.

## Future execution prompt

> Select PR110 after durable dispatch gate and resolve actual provider/capabilities/account access.
> Implement a narrow Email action and verified inbound webhook contract with host secret handles,
> typed mapping, tenant/channel binding, consent, quotas and normalized receipt states. Do not place
> credentials or real grants in definition/fixtures/designs or accept webhook actor/tenant claims.
> Test signature/replay/dedup/wrong scope, revoked consent, rate limits, stable correlation, lost ACK/
> unknown reconciliation and permanent/transient errors with controlled fakes first. Build Leptos
> account/capability/map/test/consent states from actual supported contracts, not imagined provider
> parity. Real provider setup/persistent access and external test delivery need applicable explicit
> approval identifying account/data/recipient/environment; pause dependent steps if absent and
> complete harmless fixture tests. Report mock/provider/real-target evidence separately and exact
> executed commands. No Zalo parity claim, live dispatch from preview, production deploy or merge.
> Stop at the reviewed Email/webhook envelope; later adapters get their own capability gate.
