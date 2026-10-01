# Resource and execution budget

Before expensive build/test/browser/native/database runs, inspect disk, free RAM/swap, tool
versions, affected owner and expected artifacts. Choose focused packages/targets/features and
bounded concurrency. Do not run the whole private monorepo to test an extracted kernel.

Record anticipated peak space/memory, timeout, retained evidence and stop conditions. On constrained
machines one heavy build at a time; avoid duplicate browser/native/database workers. A narrow
test failing due to memory is not a correctness pass. Save exact failure and resume with a safe
bounded setting if authorized; never weaken assertions, skip required gates silently or use
unapproved remote compute.

For read-only/design/prose tasks do not manufacture a full build to measure it. Python link/schema
checks and renderer inspection at selected frames are proportional. Tool install only when needed
for the active deliverable and approved source. Do not delete user data, source branches, tests,
fixtures or irreplaceable evidence for space. Cache cleanup needs confirmed ownership/recovery and
authorization; report what is retained or unavailable.

For live inspection set a useful exploration budget (time/actions/images) and retain completed
findings on stop. No blind new-key mutation retry after an unknown outcome: reconcile receipt/
provider state, then retry the exact original intent/key only where the documented idempotency
contract permits. A wait is not permission to use production or another identity.
