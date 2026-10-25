CREATE TABLE IF NOT EXISTS secret_device_bundles (
  device_id uuid PRIMARY KEY REFERENCES devices(id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  suite text NOT NULL,
  x25519_public bytea NOT NULL,
  ml_kem_768_public bytea NOT NULL,
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS secret_device_bundles_user_idx ON secret_device_bundles(user_id);

CREATE TABLE IF NOT EXISTS secret_conversations (
  conversation_id uuid PRIMARY KEY,
  kind text NOT NULL CHECK (kind IN ('one_to_one', 'group')),
  created_by uuid NOT NULL REFERENCES users(id),
  suite text NOT NULL,
  epoch bigint NOT NULL DEFAULT 1 CHECK (epoch > 0),
  enabled_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS secret_conversation_members (
  conversation_id uuid NOT NULL REFERENCES secret_conversations(conversation_id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  joined_epoch bigint NOT NULL,
  left_epoch bigint,
  PRIMARY KEY (conversation_id, user_id)
);
CREATE INDEX IF NOT EXISTS secret_members_active_idx
  ON secret_conversation_members(user_id, conversation_id) WHERE left_epoch IS NULL;

CREATE TABLE IF NOT EXISTS secret_key_packets (
  conversation_id uuid NOT NULL REFERENCES secret_conversations(conversation_id) ON DELETE CASCADE,
  epoch bigint NOT NULL,
  sender_device_id uuid NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  recipient_device_id uuid NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  packet jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (conversation_id, epoch, sender_device_id, recipient_device_id)
);
CREATE INDEX IF NOT EXISTS secret_key_packets_recipient_idx
  ON secret_key_packets(recipient_device_id, conversation_id, epoch);

CREATE TABLE IF NOT EXISTS secret_messages (
  id bigint PRIMARY KEY,
  conversation_id uuid NOT NULL REFERENCES secret_conversations(conversation_id) ON DELETE CASCADE,
  epoch bigint NOT NULL,
  sender_id uuid NOT NULL REFERENCES users(id),
  sender_device_id uuid NOT NULL REFERENCES devices(id),
  client_message_id uuid NOT NULL,
  counter bigint NOT NULL CHECK (counter >= 0),
  nonce bytea NOT NULL,
  ciphertext bytea NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (sender_device_id, client_message_id),
  UNIQUE (conversation_id, epoch, sender_device_id, counter)
);
CREATE INDEX IF NOT EXISTS secret_message_history_idx ON secret_messages(conversation_id, id DESC);
