# Host authority, policy and checked commands

Use for authentication, membership, selector routing, policies, commands, projections, receipts
or bot/tool authority. Existing EvoBase user-only JWT context and global storage do not establish
the new tenant/app boundary. VOT's Gateway-dependent adapter is reference material, not an EvoBase
authentication deployment to copy verbatim.

## Trace the complete authority path

Verified host session → canonical actor → current membership/grants → trusted tenant registry →
selected app/release/bindings → checked policy decision → authorized output/checked write.

URL/body/header IDs are selectors checked against trusted context. Never derive credential/DB URL
from them, accept client actor/owner claims, or trust an unsigned cookie/header. Cookie mutations
need exact allowed-origin/CSRF protection; native bearer/session needs a separately verified adapter.
Do not invent identity providers or new persistent access from an implementation request.

Effective permission = host tenant boundary AND current host grants AND app policy AND applicable
restrictions. Separate Design, Publish, Manage and business capabilities. Tenant admin cannot
exceed platform ceiling; AppSpec cannot self-assert admin/verified identity or grant connector consent.

## Policy subset

Define typed read/write/field/output semantics, null/deny precedence and policy revision. Check old
and proposed new write state plus field mutability; changing owner/tenant to open access is invalid.
Business guards are separate from permission. UI hidden actions are presentation only.

Ref is no authorization shortcut: query, picker, join, lookup, aggregate, export, subscription,
search and bot projection all apply current output policy. Unsupported query/RLS translation
rejects the target or runs trusted equivalent scoped runtime enforcement; never drops a predicate.
Any SQL policy subset needs independent differential vectors including null, joins, deny and aggregates.

## Checked command evidence

- Private CheckedDefinition/DecisionPlan/CheckedWrite constructors; wire/DB/FFI cannot forge them.
- Missing/forged/expired/revoked identity, wrong tenant/app/action and stale grants deny.
- Current authorization checked before exact-retry receipt replay and dispatch; a stale success
  receipt must not bypass revoked authority or disclose unauthorized data.
- Checked writes use normalized validated values/captured trusted facts, never raw input persistence.
- Full versus partial form/update semantics deliberate; missing hidden inputs must not erase data.
- Same idempotency key+same bounded intent replays receipt; same key+different intent conflicts.
- Release pins/CAS/old-new policy check and business guards tested through actual API path.
- Logs/errors/caches do not disclose actor/tenant data across scope.

Report what is enforceable today; a whole-instance membership pilot is not row/field policy support.
