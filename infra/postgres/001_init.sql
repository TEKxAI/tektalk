CREATE TABLE IF NOT EXISTS users (
  id uuid PRIMARY KEY, phone_e164 text NOT NULL UNIQUE, display_name text NOT NULL,
  password_hash text NOT NULL, security_question text NOT NULL, security_answer_hash text NOT NULL,
  password_changed_at timestamptz NOT NULL DEFAULT now(), created_at timestamptz NOT NULL DEFAULT now(), disabled_at timestamptz
);
CREATE TABLE IF NOT EXISTS devices (
  id uuid PRIMARY KEY, user_id uuid NOT NULL REFERENCES users(id), name text NOT NULL,
  trusted_at timestamptz NOT NULL, last_seen_at timestamptz NOT NULL, revoked_at timestamptz
);
CREATE INDEX IF NOT EXISTS devices_user_idx ON devices(user_id) WHERE revoked_at IS NULL;
CREATE TABLE IF NOT EXISTS device_challenges (
  id uuid PRIMARY KEY, user_id uuid NOT NULL REFERENCES users(id), device_name text NOT NULL,
  expires_at timestamptz NOT NULL, used_at timestamptz, created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS password_reset_otps (
  id bigserial PRIMARY KEY, phone_e164 text NOT NULL, otp_digest text NOT NULL,
  attempts smallint NOT NULL DEFAULT 0, expires_at timestamptz NOT NULL, used_at timestamptz, created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS otp_lookup_idx ON password_reset_otps(phone_e164, created_at DESC) WHERE used_at IS NULL;
CREATE TABLE IF NOT EXISTS refresh_tokens (
  id bigserial PRIMARY KEY, user_id uuid NOT NULL REFERENCES users(id), device_id uuid NOT NULL REFERENCES devices(id),
  token_digest text NOT NULL UNIQUE, expires_at timestamptz NOT NULL, revoked_at timestamptz, created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS messages (
  id bigint PRIMARY KEY, conversation_id uuid NOT NULL, sender_id uuid NOT NULL REFERENCES users(id),
  recipient_id uuid NOT NULL REFERENCES users(id), client_message_id uuid NOT NULL, body text NOT NULL,
  created_at timestamptz NOT NULL, UNIQUE(sender_id, client_message_id)
);
CREATE INDEX IF NOT EXISTS message_history_idx ON messages(conversation_id, id DESC);

