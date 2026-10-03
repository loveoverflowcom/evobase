# Single-tenant libSQL store

`Store::open_local` opens a libSQL file without a hosted account. `open_remote` uses a host-configured
Turso/libSQL primary URL and token. Request scope never chooses a destination. The file records its
configured tenant and rejects a different tenant on reopen. Multi-tenant registry, provisioning,
credential separation and isolation measurements are future work.

The fixed engine schema stores immutable canonical AppSpec releases, an app's pinned definition,
bounded checked facts, data revision, actor-scoped receipts, audit and committed event intents.
Logical tables and fields do not create SQL business tables. A spec identity is the SHA-256 hash
of its exact canonical codec bytes; bootstrap cannot repin or overwrite an existing app.

Facts use the core's whole-app snapshot envelope: at most 4,096 records and 1 MiB canonical JSON.
Definitions and persisted command payloads are independently bounded to 1 MiB. Read and write
semantics revalidate canonical facts against the pinned checked definition. Record identity,
reference and declared field constraints use complete final-state validation; there is no secondary
physical relation index or unique-claim table in this bounded profile. Large-app paging, query
indexing, schema evolution and performance measurements need separate acceptance.

Commands run in an immediate transaction with a two-second embedded lock budget. Current host
authority and registry binding precede facts and receipt access; current row policy precedes
receipt output. Receipt identity binds actor/app/key, immutable definition, original expected
revision and normalized command inputs. Exact original retries can replay after the data revision
has advanced; changing the expected revision under the same key conflicts. New commands perform
revision CAS and atomically persist checked facts, revision, audit, event intents and receipt.
Host authority is re-read immediately before commit and observed membership/identity changes
roll back. This is not a distributed transaction with an external membership provider.

`snapshot` and `catalog` return internal host facts/diagnostics. They are not authorized client
projections. The Gateway must apply current read policy before returning runtime data. Committed
events are passive durable intents; this crate does not deliver external effects or run jobs.
SQLite/Turso runtime privilege separation and remote TLS/provider behavior require their own
environment evidence; embedded conformance does not establish a Turso cloud pass.

| Invariant / owner | Failure | Regression oracle |
| --- | --- | --- |
| Checked canonical snapshots / AppSpec codec | forged, invalid or cross-scope facts persist | real libSQL bootstrap/reopen negatives |
| Fixed layout / storage adapter | a logical schema creates business SQL tables | compare actual `sqlite_schema` for two unrelated apps |
| CAS and immutable pin / store | concurrent writers lose updates or repin a running app | competing independent file connections |
| Current authority / host + checked core | revoked access replays a previous success | receipt/commit revocation and membership-race negatives |
| Atomic state and intents / store transaction | partial failure leaves facts, receipt or event | fail each commit stage and assert all engine counts |
| Durable original receipt / store | restart applies the same business command twice | a separate test process opens the file and retries |

Run focused conformance with `CARGO_BUILD_JOBS=2 cargo test -p evobase-store`.
