# 260 — Author finite commands in the Builder

Status: next recommended task; not part of the completed ten-commit batch. Owner: #17/#9.

## Why

The checked command contract and generated Runtime now have real consumers. Builder users still
need a structured way to author those declarations rather than supplying JSON fixtures.

## Scope and acceptance

Author one-table state machines, typed command inputs, finite equality transitions and event field
projections using stable IDs. Compile and validate existing facts atomically before draft mutation.
Save, restore and cancel must retain canonical definitions and precise diagnostics. Reuse the
checked Rust contract and current design tokens. Exercise two unrelated schemas through mounted
controls; preview remains pure and cannot publish or dispatch effects.

## Dependencies and boundary

Delivered constraints, generic commands, protocol and generated Runtime from tasks 170–240.
Keep publish/evolution, reusable event-schema expansion, cross-record workflows and multi-tenant
provisioning in separate reviewed slices. This queue item does not authorize those expansions.

## Verification

Core conformance for generated declarations, actual browser author/save/cancel flows, source/build
identity, VI/EN and light/dark captures opened for inspection. Reject invalid existing final states
without losing inputs or changing facts.
