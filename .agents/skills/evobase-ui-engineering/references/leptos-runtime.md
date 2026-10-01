# Leptos runtime boundaries

Use for reactive state, requests, browser APIs, navigation, SSR/hydration or measured performance.
Check the actual pinned Leptos version and build mode before selecting API names; this reference
does not assert a Leptos crate or executable app currently exists.

## Authority and state

```text
Domain package/core: refinements, capabilities, workflow/command rules, typed failures
Client/adapter: transport, DTO mapping, session access and request execution
Presentation owner: meaningful async/state projection and action coordination
Leptos components: render facts, dispatch intents, own appropriate local input/focus state
CSS: consume derived semantic tokens for layout and presentation
```

Generated fields may give immediate feedback from generated validation metadata; they do not
invent a second domain rule. Server/core validation is still authoritative. Components do not
interpret raw provider errors, assemble deployment origins or keep credentials in display state.
Introduce injectable ports only at a concrete service/browser boundary; no trait per helper.

A menu boolean does not need a reducer/MVI system. A publish, verification, approval or long-lived
session journey may need explicit transitions and effects. If so, extract deterministic decisions
at the right owner and keep effects at a named lifetime. Do not duplicate cheap derived facts in
independently writable signals that can disagree.

## Async and interruption

- Separate reads/resources from mutations/actions. Do not mutate during a read or render
- Show idle, loading, data, empty and error distinctly. Refresh/paging can coexist with existing
  data; model those facts if the user can observe them concurrently
- Key requests by relevant package/record/account/session identity and generation. A late result
  from a previous input/account cannot replace current state, even if cancellation was requested
- Guard duplicate submissions independently of stale-result handling. Define reject/queue/replace
  semantics and preserve the matching pending/enabled state
- On navigation, Cancel or unmount, dispose subscriptions and prevent obsolete presentation
  writes. Cancellation does not prove the server rolled back a mutation; reconcile unknown outcomes
- Keep retry policy bounded and typed at its owner. Retry only eligible operations/errors; do not
  blindly repeat approval, save, publish or signup after a timeout
- Avoid an effect that reads a request's output signal and writes it after starting a request,
  retriggering itself. Prefer the pinned version's resource/action primitives where suitable

Regression oracles use fixed IDs/clocks/fixtures and controlled completion ordering. Exercise
A→B→late-A, double click, route leave/reenter, session change, failure/retry and unknown mutation
outcome when relevant. Assert observable current state and dispatched domain effect, not only
private helper order. Core tests cannot prove actual reactive wiring or focus/navigation.

## Browser interop and lifecycle

Use declarative Leptos structure/attributes first and typed node references for component-owned
elements. Handle “not mounted” normally. Keep imperative browser operations in named fallible
helpers with typed errors and accessible/localized fallback. Do not mutate renderer-owned DOM to
make a capture look correct or build HTML from user/provider content.

Listeners, observers, intervals, animation callbacks, object URLs and media streams have named
owners and cleanup. Keep closures alive only for their registered lifetime. Verify repeated
mount/unmount, resize/scroll and disposal when changed; `forget()` is not a general cleanup plan.

Discover which targets exist. Browser-only `window`, storage, clipboard and media access must
not execute during SSR or in an unsupported host. Keep server and hydration initial output
consistent; browser preferences can require client-side reconciliation without losing input.
Feature detection is not permission to accept new browser access prompts.

Router/history owns navigation. Check real Back/Forward, Close/Cancel and destination state;
callback execution alone is not navigation evidence. Clipboard, file input and permission
failures remain typed states, not panics. A native container/WebView needs its own host adapter
evidence if that boundary is introduced; no WebView or desktop wrapper is assumed here.

## Styling and token adapters

Canonical JSON → discovered generator/adapter → CSS custom properties → component stylesheet
→ mounted Leptos consumer. CSS properties are runtime theme roles; CSS/SCSS organization follows
the chosen app pipeline. Keep component markup readable and token-aware. Do not introduce a
utility framework, duplicate palette or second generator just to style one screen.

Semantic classes and component-owned styles are useful; folder layout is not a requirement for
every small component. Preserve actual stylesheet order and cascade when extracting styles.
Desktop viewport width does not imply a panel's available width: use intrinsic wrapping or
container adaptation and assert narrow-panel bounds as well as page overflow.

## Performance without guesses

For a performance request, baseline the actual affected path before an architectural optimization.
Measure source/build/browser/device/configuration together: compressed WASM/JS download size,
startup/first meaningful render, update/long-task cost or retained memory as relevant. Compare
identical fixtures and methods; a compressed-size claim needs the served compression format.

Consider render volume/pagination, reactive invalidation, large clones, heavy pure work in render,
unbounded caches and missing teardown before micro-optimizing. Preserve stable identity, focus,
accessibility and localization. Pagination/virtualization must follow product semantics rather
than silently omit required content. Inspect exact dependency features only when a concrete
measurement warrants it; do not broaden into a workspace-wide toolchain/dependency purge.

Correctness refactors may be justified without a benchmark. Report performance as unmeasured
rather than claiming a speedup from memoization, a smaller file or a guessed crate weight.
