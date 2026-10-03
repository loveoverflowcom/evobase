# 010 — AppSpec foundation and host seams

Status: **implemented experimental version 1 contract; hosted/native/product expansion gates remain open**.
Executed checks and artifact scope are recorded in [batch evidence](../batch-evidence.md).
Date: 2026-10-03. Authority: [EvoBase #1](https://github.com/loveoverflowcom/evobase/issues/1),
[foundation issue #3](https://github.com/loveoverflowcom/evobase/issues/3) and the user's request
for approximately five sequential commits on `develop`. This ADR is not runtime acceptance,
production authentication evidence or completion of the issue's Web/CMP shell checklist.

Source baseline: `4cb5873200f4d735b17b75fdd3cb88c2efdbb322`.
Guidance import: `chore/evobase-reboot-plan@699fa0f4886c1b47c9ee21f6774bbdb9800a6758`.
Canonical design import: `6ae452f144e49f8aaa5f8d5fb6b3e681c4c4ef6e` at
`design/m3-expressive/`. [Inventory/provenance](../provenance.md) records the available and unavailable sources.

## Decision and observable invariant

A reusable definition describes meaning and never supplies runtime data or trusted host authority.
A rename/reorder cannot change a stored relationship or business rule. The Rust core is the
semantic owner; browser and future native UI project its checked result. Failure modes are
silently divergent rules, relationship drift, secret/data leakage in templates and forged access.
The oracle is bounded positive/negative codec and scope vectors through the public checked
constructors plus independent relation/arithmetic vectors. The implementation lives in
`crates/evobase-appspec`; Leptos consumes it in `apps/evobase-builder`. Evidence applies only to
executed targets/scenarios, not the entire product vision.

## Version 1 definition

Select standard UTF-8 JSON with `version: 1`; no separate profile selector and no arbitrary
extension bag. Use `.evobase.json` for the definition interchange filename, not a binary ZIP or
an XLSX authority. The public Rust schema and reviewed golden examples own the precise shape:

- Top-level keys: `version`, `app_id`, `name`, `tables`; optional `capture_rules`, `policies` and
  `submit_rules` default to empty and are omitted from canonical output when empty.
- A table has stable `id`, presentation `name` and `fields`. A field has stable `id`, presentation
  `name`, `required` (defaults to false) and `field_type`.
- `field_type` is tagged by `type`: `text`, `integer`, `bool`, `money`, or `ref`; `ref` additionally
  names `target_table`. Unknown versions, fields, tags or unsupported types must reject.
- Optional capture rules name `line_table`, `product_ref_field`, `product_price_field` and
  `captured_price_field` by stable IDs. Compilation checks Ref target and Money types. Empty
  capture rules are omitted from canonical output. They define capture semantics, not live facts.
- Optional `policies` hold typed owner/current-role rules: `rule_id`, `revision`, `table_id`,
  `owner_field`, readable-field allowlist and read/write/submit abstract-role lists. Optional
  `submit_rules` hold typed SubmitOrder guards, referencing their policy and order/line field IDs.
  Rule IDs start with `rule_`, are at most 96 ASCII letters/digits/underscores and are globally
  unique across both lists. Each list is limited to 64 rules, with at most one policy per table.
  Role IDs start with `role_` and follow the same 96-byte character limit. These are abstract
  roles; membership, current grants and verified actor identities are not portable fields.
- Canonical encoding orders tables/fields by stable ID, captures by table/destination field and
  policies/submit rules by rule ID. Canonical round-trip is a compatibility check, not authorization.
  Labels/localization/order never supply identity.

Definitions contain no records, tenant selector, actor/grant, password, database URL, connector
account, secret handle or runtime receipt. Runtime records have their own bounded checked input;
host bindings belong to future host storage. The bounded pure policy/command experiment uses
supplied facts; it does not make those facts verified host grants. Do not accept a serialized
checked plan as proof of validation. Checked definitions and checked record sets have private construction and no
`Deserialize` implementation; a raw decoded definition is still untrusted.

`version` versions only this definition encoding and semantics. Future immutable release identity,
physical storage layout version, host binding revision and data revision are distinct concepts.
This batch does not implement their lifecycle. An incompatible version requires explicit migration
and compatibility evidence; relabeling a legacy artifact is not a migration.

## Stable IDs and values

Use separate app/table/field/record ID types. Their spellings start with `app_`, `tbl_`, `fld_` or
`rec_`, followed by 1–76 ASCII letters/digits/underscore/hyphen, with an overall maximum of
80 bytes. IDs are validated at every public construction/decoding path. Uniqueness and references
are checked at their appropriate scope: table/field IDs are unique throughout a definition,
record IDs within `(table_id, record_id)`. Display labels need not be unique.

Initial values are Text, Integer, Bool, Money, Ref, Blank and Null. Blank is an unfilled input;
Null is an explicit absent value. The wire variants remain distinct; required fields reject both,
optional fields admit either, and an absent optional field normalizes to Blank. They are not
converted to zero, false or an empty string. Typed text authoring uses empty input for Blank and
`null` for Null; the `text:` escape permits literal text (`text:null`, `text:`,
`text:text:literal`). A Text value may store an empty string independently of Blank.

Money is a signed `i64` count of whole minor units and uses an exact tagged JSON integer. Integer
and Money are distinct types. Arithmetic uses checked operations; fractions, exponent/float
inputs, overflow and implicit cross-type arithmetic reject. Currency conversion, mixed-currency
arithmetic, percentages and arbitrary decimal scale are outside this profile. Browser consumers
must preserve JSON bytes through Rust decoding/encoding and accept exact numeric text; passing
Money through JavaScript `Number` can lose precision and is not the contract.

A Ref contains tenant/app/table/record scope, and its table must match the declared target.
Runtime scope is host/local context, not part of the reusable definition and not authority.
The first relation profile supports **N:1** only: child Ref is canonical, reverse relations are
computed. Delete behavior is **restrict** by version 1 convention; there is no configurable
cascade flag. General 1:1 uniqueness, N:M, cross-tenant sharing and cascade deletion are deferred
until their own semantics/conformance gates. Refs never grant permission to see their target.

## Decode and evaluation budgets

The selected kernel budget is 1 MiB JSON, depth 16, 20,000 JSON nodes, 64 tables, 256 fields per
table, 2,048 total fields, 4,096 runtime records, 256-byte names and 16,384-byte text values.
The byte/depth/node budget applies to decode and also bounded serialization in direct compile
and record-validation entrypoints. Bounds intersect; their maximum counts cannot necessarily be
combined. Unknown/duplicate keys, unsupported tags, duplicate IDs and incompatible references
reject with diagnostics. Raw DTO deserialization alone never constructs checked values.

The formula query API compiles Field, one-hop Lookup, reverse Sum, Product, Total and acyclic Named
reuse. Defaults are 256 evaluated nodes, depth 24 and 1,024 visited records (including scanned
child candidates). Arithmetic is checked; Integer × Money yields Money, Money × Money rejects.
These are checked pure query APIs, not editable portable formula declarations: formula authoring
and export remain deferred. Every relation output needs an explicit policy attesting the exact
checked definition/facts. Complete scans require a data-independent full-table grant; the
owner/role adapter denies them rather than partially leaking a restricted join/aggregate.

## Real consumers and target boundaries

The immediate consumers are native/WASI checked-kernel vectors and a Leptos browser local-draft
Builder. Keep the kernel independent of SQL/network/clock/auth grants; bind I/O at the host.
The batch compares shared native/WASI conformance output; browser/WASM rendering has its own
artifact identity and fresh scenario checks in the [evidence record](../batch-evidence.md).
The existing Rust workspace declares edition 2024 and minimum Rust 1.89 at the pinned baseline;
actual compiler/Leptos versions and executed targets belong to implementation evidence.

The browser profile edits a local draft and synthetic example records, shows typed diagnostics,
bounded TSV imports and relation/policy preview. Its localStorage snapshot carries opaque canonical
definition/record JSON strings and is checked again on restore. It is neither a published release nor an authenticated
multi-tenant host. Builder/Runtime navigation and compact Builder summary remain separate from
native CMP behavior. CMP Runtime is the selected mobile direction; no CMP package, toolchain,
Android/iOS execution or native acceptance is established by this ADR.

Host authority will follow: verified host session → canonical actor → current membership/grants →
trusted tenant registry → checked app/release → policy/command. Tenant/app/actor IDs in URLs or
bodies are selectors checked against that context; they never choose credentials or grants.
Authentication provider and browser/native session adapters remain a later ADR. Do not reuse the
legacy username/password JWT service as evidence that this seam is implemented.

Local draft access needs no claim of authenticated authority. A future denied/expired host session
must preserve unsent draft state and disclose its local status; draft restoration must never restore
revoked grants or dispatch effects. Preview is pure and has zero network delivery, deployment DDL
or live connector action. A saved definition is not consent to deploy or send anything.

## Bounded policy/command experiment

The owner-or-current-role policy and typed SubmitOrder rule are now definition-owned in `policies`
and `submit_rules`; their checked constructors reject out-of-definition rule/configuration swaps.
The command checks state/notes, line Ref, quantity and captured price and returns an opaque checked
write/audit/receipt intent. It does not commit a database transaction. Current host/session facts
are resolved through the `SessionVerifier` seam for every query/command and before receipt replay.
Tests use a FixtureHost, and the browser presents supplied actor/roles/grants as simulated facts.
Neither path implements a verified authentication provider or transport/API adapter.

The read allowlist rejects Ref output fields; arbitrary policy joins, restricted complete relation
scans, SQL/RLS translation, subscriptions and durable receipt storage are unsupported. The pure
policy layer provides scoped row/field projections, picker/lookup/export-purpose projections and
visible aggregates within this subset. Portable rule encoding is available, but a general
structured rule authoring/publishing journey remains a future gate. The host must atomically commit
expected revision, facts, derived structures, audit, outbox and receipt before claiming persistence.

## Design and deferred gates

Keep `design/m3-expressive/source/tokens.json` as the single authored token source. Preserve its
font notices and component/state/accessibility contracts. Real adapters must trace source → output
→ mounted consumer. Static SVG/PNG fixtures do not prove that trace or pixels in Leptos/CMP.

| Gate | Delivered subset and remaining evidence |
|---|---|
| Kernel | Bounded checked definitions/facts and native/WASI vectors implemented; cross-target claims are limited to the recorded vectors. |
| Grid/import | Mounted Leptos local draft, exact typed edits/TSV and interruption/conflict controls implemented; final browser artifact/scenario evidence owns coverage, with broad accessibility/native/hosted gaps still open. |
| Relations/formulas | N:1/restrict, captures and bounded pure query projections implemented; 1:1/N:M, portable formula authoring/export and general restricted policy joins remain deferred. |
| Foundation shell #3 | Canonical Web token adapter and compact read-only draft are mounted; real host session/Runtime routing and separate native CMP acceptance remain unimplemented. |
| Policy/commands #6 | Definition-owned owner/role rules and SubmitOrder checked intents implemented with fixture host facts; verified session/API and transactional receipts remain unimplemented. |
| Hosted security/storage | Two real DBs, separate privileges, no-DDL catalogs and atomic transaction/concurrency/restart evidence required after verified host adapter. |
| Broader vision | Immutable release/evolution, native Runtime, durable functions, real providers and portability/recovery before Votable retirement. |

This foundation resolves the minimal implementation contract and preserves historical source and
run guides. Broader issue checkboxes remain open until their actual acceptance evidence exists.
