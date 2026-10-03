# 010 — AppSpec foundation and host seams

Status: **selected experimental contract for the current refactor batch; documented E0 decisions**.
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
The cheapest adequate oracle is bounded positive/negative codec and scope vectors through the
public checked constructors; independent relation/arithmetic vectors follow in the next commits.

## Version 1 definition

Select standard UTF-8 JSON with `version: 1`; no separate profile selector and no arbitrary
extension bag. Use `.evobase.json` for the definition interchange filename, not a binary ZIP or
an XLSX authority. The public Rust schema and reviewed golden examples own the precise shape:

- Top-level keys: `version`, `app_id`, `name`, `tables`; optional `capture_rules` defaults to empty.
- A table has stable `id`, presentation `name` and `fields`. A field has stable `id`, presentation
  `name`, `required` (defaults to false) and `field_type`.
- `field_type` is tagged by `type`: `text`, `integer`, `bool`, `money`, or `ref`; `ref` additionally
  names `target_table`. Unknown versions, fields, tags or unsupported types must reject.
- Optional capture rules name `line_table`, `product_ref_field`, `product_price_field` and
  `captured_price_field` by stable IDs. Compilation checks Ref target and Money types. Empty
  capture rules are omitted from canonical output. They define capture semantics, not live facts.
- Canonical encoding orders tables and fields lexically by stable ID. Canonical round-trip is a
  compatibility check, not authorization. Labels/localization/order never supply identity.

Definitions contain no records, tenant selector, actor/grant, password, database URL, connector
account, secret handle or runtime receipt. Runtime records have their own bounded checked input;
host bindings belong to future host storage. The final selected commit may add a bounded pure
policy/command experiment using supplied facts; it does not make those facts verified host grants.
Do not accept a serialized checked plan as proof of
validation. Checked definitions and checked record sets have private construction and no
`Deserialize` implementation; a raw decoded definition is still untrusted.

`version` versions only this definition encoding and semantics. Future immutable release identity,
physical storage layout version, host binding revision and data revision are distinct concepts.
This batch does not implement their lifecycle. An incompatible version requires explicit migration
and compatibility evidence; relabeling a legacy artifact is not a migration.

## Stable IDs and values

Use separate app/table/field/record ID types. Their spellings start with `app_`, `tbl_`, `fld_` or
`rec_`, followed by 1–76 ASCII letters/digits/underscore/hyphen, with an overall maximum of
80 bytes. IDs are validated at every public construction/decoding path. Uniqueness and references
are checked at their appropriate app/table scope; display labels need not be unique.

Initial values are Text, Integer, Bool, Money, Ref, Blank and Null. Blank is an unfilled input;
Null is an explicit absent value. The wire variants remain distinct; required fields reject both,
optional fields admit either. They are not silently converted to zero, false or an empty string.
A Text field may store an empty string independently of Blank.

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
Bounds apply before expensive traversal/compilation; hostile input must return typed diagnostics,
not panic or silently truncate. Duplicate keys/IDs and incompatible references need explicit
negative vectors. Formula evaluation gets a separate bounded, nonrecursive budget in its owning
commit; this ADR does not promise a general expression language.

## Real consumers and target boundaries

The first immediate consumer is a native Rust checked-kernel test suite, followed by a Leptos
browser local-draft Builder. Keep the kernel independent of SQL/network/clock/auth grants; bind
I/O at the host. Test native and WASM semantics separately before making cross-target claims.
The existing Rust workspace declares edition 2024 and minimum Rust 1.89 at the pinned baseline;
actual compiler/Leptos versions and executed targets belong to implementation evidence.

The browser profile edits a local draft and local example records, shows typed diagnostics and
preview, and can exchange a definition. It is neither a published release nor an authenticated
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

Commit 5 selects an owner-or-current-role policy with an explicit readable-field allowlist and
read/write/submit role lists, plus a typed SubmitOrder guard using state/notes, line Ref, quantity
and captured price. This experiment uses companion checked Rust configuration; it adds no raw
version 1 definition keys. It is not yet portable policy authoring/publishing. The browser presents
its supplied actor/roles/grants as simulated facts, never a verified session. Row/field expressions,
SQL/RLS translation, subscriptions and a trusted server adapter remain unsupported. Any future
portable policy/configuration must enter the single versioned AppSpec contract with compatibility
and authoring evidence, rather than become a separately edited rule source.

## Design and deferred gates

Keep `design/m3-expressive/source/tokens.json` as the single authored token source. Preserve its
font notices and component/state/accessibility contracts. Real adapters must trace source → output
→ mounted consumer. Static SVG/PNG fixtures do not prove that trace or pixels in Leptos/CMP.

| Gate | Evidence still needed beyond this document |
|---|---|
| Kernel | Golden canonical/historical/hostile vectors, private-construction audit, native/WASM execution. |
| Grid/import | Typed errors, ambiguity resolution, keyboard/IME/paste, stale/saving/conflict states; no hidden precision loss. |
| Relations/formulas | Independent wrong-scope/missing-target/restrict-delete/capture/overflow/cycle vectors. |
| Foundation shell #3 | Mounted token/nav trace, light/dark, focus, VI/EN, reduced motion, zoom/text scaling, unsent draft preservation; separate native gates. |
| Hosted security/storage | Verified current identity and policy negatives, two real DBs, separate privileges, no-DDL catalog and atomic transaction/concurrency/restart evidence. |
| Broader vision | Immutable release/evolution, native Runtime, durable functions, real providers and portability/recovery before Votable retirement. |

This foundation resolves the minimal implementation contract and preserves historical source and
run guides. Broader issue checkboxes remain open until their actual acceptance evidence exists.
