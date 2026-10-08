CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TABLE IF NOT EXISTS commerce_subscriptions (
  id uuid PRIMARY KEY, subject_id text NOT NULL, subject_segment integer NOT NULL CHECK (subject_segment IN (1, 2)),
  plan_code text NOT NULL, status text NOT NULL CHECK (status IN ('active', 'cancelled')),
  created_at timestamptz NOT NULL, updated_at timestamptz NOT NULL, UNIQUE(subject_id, subject_segment)
);
CREATE INDEX IF NOT EXISTS commerce_subscription_status_idx ON commerce_subscriptions(status, updated_at);
CREATE TABLE IF NOT EXISTS commerce_outbox (
  id uuid PRIMARY KEY, event_type text NOT NULL, aggregate_id text NOT NULL, payload jsonb NOT NULL,
  created_at timestamptz NOT NULL, published_at timestamptz
);
CREATE INDEX IF NOT EXISTS commerce_outbox_pending_idx ON commerce_outbox(created_at) WHERE published_at IS NULL;
