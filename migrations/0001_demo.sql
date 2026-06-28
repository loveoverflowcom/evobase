CREATE TABLE IF NOT EXISTS demo_users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT demo_users_email_non_empty CHECK (length(trim(email)) > 0),
    CONSTRAINT demo_users_email_shape CHECK (position('@' in email) > 1)
);

-- statement-breakpoint

CREATE TABLE IF NOT EXISTS demo_orders (
    id UUID PRIMARY KEY,
    buyer_id UUID NOT NULL REFERENCES demo_users(id),
    total_cents BIGINT NOT NULL,
    currency TEXT NOT NULL,
    status TEXT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT demo_orders_total_positive CHECK (total_cents > 0),
    CONSTRAINT demo_orders_currency_known CHECK (currency IN ('USD', 'VND')),
    CONSTRAINT demo_orders_status_known CHECK (status IN ('draft', 'paid', 'shipped', 'cancelled'))
);

-- statement-breakpoint

CREATE INDEX IF NOT EXISTS demo_orders_buyer_id_idx ON demo_orders(buyer_id);

-- statement-breakpoint

CREATE TABLE IF NOT EXISTS demo_domain_events (
    id UUID PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- statement-breakpoint

CREATE INDEX IF NOT EXISTS demo_domain_events_aggregate_id_idx ON demo_domain_events(aggregate_id);
