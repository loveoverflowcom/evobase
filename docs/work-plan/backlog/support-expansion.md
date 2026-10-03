# Support expansions needed before full vision acceptance

These are deliberately outside the first bounded PRs. They remain obligations, not completed or
abandoned features. Promote each when its immediate consumer and prerequisite evidence are real;
split into a reviewed issue/PR with design state, support ADR and exact regression gate.

## After the single-tenant libSQL batch

The ten-commit batch delivers the bounded profile in [ADR 020](../decisions/020-single-tenant-libsql.md),
not the full #7/#8/#9/#16/#17/#18/#19 epics. The next recommended consumer is
[260 command authoring](../260-command-authoring.md), before a separate publish/evolution slice.
Keep these obligations outside the completed queue:

- Real Turso primary conformance: credentials, remote transactions/CAS/retry/disconnect and operational
  restore evidence. Local libSQL tests establish local behavior only.
- Multi-tenant database provisioning, verified routing, separate credentials and cross-tenant negative
  tests. The user explicitly deferred this; one configured tenant is not isolation evidence.
- Publish/repin/evolution/backfill with historical release recovery and concurrent-authority checks;
  bootstrap immutability is the delivered subset of #8.
- General event authoring/schema reuse and durable outbox worker delivery; stored event intents do
  not establish dispatch or a provider receipt.
- Runtime pagination/search and receipt recovery across page reload. The current client bounds its
  first page, reports truncation and retains unresolved exact requests in window memory only.

## Advanced relation/constraint semantics

1:1 uniqueness/null semantics, N:M link/edge uniqueness, other delete policies and relations with
attributes. Requires PR040/060; test final-state and concurrent unique/create/delete/rollback/restart,
output-policy leaks and identity-preserving UI. Initial N:1/restrict is not full #1 relation acceptance.

## Broader policy/query adapter support

Additional row/field expressions, joins/aggregates, bounded typed search/order and SQL/RLS projections.
Requires PR050/060; independent runtime/native/query differential oracle, null/deny/joins/aggregates,
actual output-leak negative tests and measured envelope. Unsupported projection must fail closed.

## Zalo and additional connectors

Capability/provider permissions, recipient/account identity, verified webhook/consent and idempotency/
unknown-outcome behavior require provider-specific spike after PR110. Email tests do not prove Zalo,
payment or arbitrary messaging parity. Real account/persistent access/recipient tests need approval.

## Additional native/offline and isolation profiles

Android/iOS/native lifecycle/IME/accessibility and local/offline reconciliation beyond selected PR090/
140 target. Shared RLS/schema-per-tenant or compiled relational adapter beyond PR060 requires ADR,
same semantic conformance and isolation/current-auth/concurrency evidence; no automatic fallback.

## Optimization and formal assurance

Index/query planner/pool/fair scheduling improvements follow measured bottleneck and preserved
transactions/policy. Formal Lean/Kani/Quint work targets selected small propositions with production
refinement, not a blanket prerequisite or claim of verified platform. Dedicated physical isolation,
per-tenant restore/PITR and high-scale fan-out need actual operational evidence.
