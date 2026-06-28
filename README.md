# EvoBase Domain-First Demo

This branch is a clean rebuild demo based on `re-design/`.

The demo shows the new direction:

```text
Domain model -> verified-ish specification -> generated/runtime adapters
```

It includes:

- Rust domain model with refinements and typestate workflow.
- Repository/event interfaces that are independent from PostgreSQL.
- PostgreSQL adapter implementing those interfaces.
- SQL migration for users, orders, and domain events.
- Axum backend exposing domain commands and queries.
- Lean4 proof sketch for order workflow safety in `lean/`.

## Run

1. Start PostgreSQL:

```bash
docker compose up -d postgres
```

2. Copy `.env.example` to `.env`.
3. Start the API:

```bash
cargo run -p evobase-api
```

Migrations are applied at startup.

## Demo Flow

Create a verified user:

```bash
curl -X POST http://127.0.0.1:3000/demo/users \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","verified":true}'
```

Create a draft order:

```bash
curl -X POST http://127.0.0.1:3000/demo/orders \
  -H "Content-Type: application/json" \
  -H "x-actor-id: <user-id>" \
  -H "x-actor-verified: true" \
  -H "x-capabilities: pay_order,cancel_order" \
  -d '{"buyer_id":"<user-id>","total_cents":1250,"currency":"USD"}'
```

Pay the order:

```bash
curl -X POST http://127.0.0.1:3000/demo/orders/<order-id>/pay \
  -H "Content-Type: application/json" \
  -H "x-actor-id: <user-id>" \
  -H "x-actor-verified: true" \
  -H "x-capabilities: pay_order,cancel_order" \
  -d '{"payment_id":"00000000-0000-0000-0000-000000000001"}'
```

Ship the order:

```bash
curl -X POST http://127.0.0.1:3000/demo/orders/<order-id>/ship \
  -H "x-actor-id: <shipper-user-id>" \
  -H "x-actor-verified: true" \
  -H "x-capabilities: ship_order"
```

Inspect the mini Domain IR:

```bash
curl http://127.0.0.1:3000/demo/domain-ir
```

Inspect stored domain events:

```bash
curl http://127.0.0.1:3000/demo/events
```

## Lean4

If Lean4/Lake is installed:

```bash
cd lean
lake env lean EvobaseDemo/Workflow.lean
```
