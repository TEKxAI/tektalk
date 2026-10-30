CREATE TABLE IF NOT EXISTS device_link_challenges (
  id uuid PRIMARY KEY,
  secret_digest text NOT NULL,
  device_name text NOT NULL,
  platform text NOT NULL CHECK (platform IN ('web', 'macos', 'windows')),
  qr_payload text NOT NULL,
  expires_at timestamptz NOT NULL,
  approved_user_id uuid REFERENCES users(id),
  approved_device_id uuid REFERENCES devices(id),
  approved_at timestamptz,
  claimed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS device_link_expiry_idx ON device_link_challenges(expires_at) WHERE claimed_at IS NULL;

ALTER TABLE secret_key_packets ADD COLUMN IF NOT EXISTS generation bigint NOT NULL DEFAULT 1 CHECK (generation > 0);
ALTER TABLE secret_key_packets DROP CONSTRAINT IF EXISTS secret_key_packets_pkey;
ALTER TABLE secret_key_packets ADD PRIMARY KEY (conversation_id, epoch, generation, sender_device_id, recipient_device_id);
ALTER TABLE secret_messages ADD COLUMN IF NOT EXISTS generation bigint NOT NULL DEFAULT 1 CHECK (generation > 0);
DO $$ DECLARE c text; BEGIN
  SELECT conname INTO c FROM pg_constraint
  WHERE conrelid='secret_messages'::regclass AND contype='u'
    AND pg_get_constraintdef(oid) LIKE '%conversation_id%epoch%sender_device_id%counter%';
  IF c IS NOT NULL THEN EXECUTE format('ALTER TABLE secret_messages DROP CONSTRAINT %I',c); END IF;
END $$;
CREATE UNIQUE INDEX IF NOT EXISTS secret_message_counter_generation_idx ON secret_messages(conversation_id,epoch,generation,sender_device_id,counter);
