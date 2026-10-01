# Review report contract

Lead with actionable findings; if none, say no introduced defects found within the inspected scope.
Each finding has title, P0–P3, confidence, exact path/line range, scenario, consequence, source/oracle
evidence, BASE comparison and smallest correction. Suggestions/pre-existing gaps separate.

Then include:

| Changed invariant | Source / owner | Prevention path | Evidence requirement / actual result | Disposition |
|---|---|---|---|---|

State BASE/HEAD/dirty snapshot, inspected paths/targets, tests actually executed with exact commands,
failed/unrun/blocked checks and reasons, freshness/artifact identity, fake/real boundaries and
residual risk. A docs-only review says no product build/behavior evidence was established.
Do not present a hypothetical acceptance test or author assertion as executed evidence.

For UI add source/build/render identity, semantic/pixel/native scopes and inspected assets. For
formal work add proposition/domain/bounds/assumptions/refinement gaps. Never blanket-approve the
platform from one green package. Report is the deliverable; posting it externally is a separate action.
