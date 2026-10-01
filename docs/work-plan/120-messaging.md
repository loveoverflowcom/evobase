# 120 — Expose a managed business messaging inbox

Status: proposed future implementation. Design pack: 10 inbox.
Skills: engineering → storage/authority/functions; Web/CMP for inbox/conversation states.

## Scope / dependencies

110 verified bindings/receipts and 050 current output policy. Managed Conversations/Messages/
Participants/Deliveries collections with one authorized inbound/outbound thread, not full chat
federation/E2EE or VOTex dependency. System receipt/provider fields are not generic editable cells.

## Acceptance / required tests

- Bound channel/tenant identity and authenticated participants; display name isn't account identity.
- Current policy on history/search/attachments/subscription/unread/preview and revoked participant.
- Delivery status comes from dispatcher/provider facts; generic edit cannot forge sent/received.
- Inbound dedup/order/correlation, paged bounded history, duplicate send original-key recovery and
  unknown/lost-ACK states; no speculative re-send on timeout.
- Safe plain/untrusted message/document content; logs/blobs/cache scope and renderer injection tests.
- Web/CMP list/thread/composer/error/offline/conflict/large text/focus/back behavior and inspected pixels.

## Future execution prompt

> Select PR120 after connector receipts/bindings are reviewed. Implement one managed business inbox
> vertical slice on typed collections with protected transport metadata. Use current tenant/channel/
> participant authority and scoped authorized projections for history/search/subscriptions/blobs;
> a Ref or display name never grants conversation access. Outbound submit creates checked intent,
> stable idempotency and real receipt state, never a user-editable delivered flag. Test duplicate
> inbound, bounded paging, revoked participant/output leakage, wrong channel/tenant, forged receipt
> edits, untrusted renderer content and accepted-but-lost-ACK recovery. Bind Web/CMP inbox/thread/
> composer to actual states with clear pending/unknown/retry/error indicators and no blind new-key
> send after timeout. Drive focus/keyboard/touch/Back/offline/repeated flows and inspect selected
> M3 frames. Report actual DB/provider/DOM/native boundaries and any missing target coverage.
> Real recipient delivery needs explicit approval; no federation/E2EE/full chat promise, merge or
> deployment. Stop at the managed inbox contract and handoff to scoped bot flow.
