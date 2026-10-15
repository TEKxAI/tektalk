CREATE TABLE IF NOT EXISTS messages(
  local_id INTEGER PRIMARY KEY, message_id INTEGER, client_nonce INTEGER NOT NULL,
  sender_id INTEGER NOT NULL, server_seq INTEGER, sent_at_ms INTEGER NOT NULL,
  edited_at_ms INTEGER, message_type INTEGER NOT NULL, flags INTEGER NOT NULL DEFAULT 0,
  payload_codec INTEGER NOT NULL, payload_version INTEGER NOT NULL, payload BLOB NOT NULL,
  UNIQUE(message_id), UNIQUE(client_nonce)
);

CREATE INDEX IF NOT EXISTS messages_timeline ON messages(sent_at_ms DESC,local_id DESC);
CREATE UNIQUE INDEX IF NOT EXISTS messages_server_seq
  ON messages(server_seq) WHERE server_seq IS NOT NULL;

CREATE TABLE IF NOT EXISTS message_local_state(
  local_id INTEGER PRIMARY KEY REFERENCES messages(local_id) ON DELETE CASCADE,
  delivery_state INTEGER NOT NULL, retry_count INTEGER NOT NULL DEFAULT 0,
  next_retry_at_ms INTEGER, upload_progress INTEGER, download_progress INTEGER,
  last_error_code INTEGER, updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS reactions(
  message_id INTEGER NOT NULL, actor_id INTEGER NOT NULL, reaction_type INTEGER NOT NULL,
  reacted_at_ms INTEGER NOT NULL, metadata_codec INTEGER NOT NULL,
  metadata_version INTEGER NOT NULL, metadata BLOB NOT NULL,
  PRIMARY KEY(message_id,actor_id,reaction_type)
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS projection_outbox(
  event_id INTEGER PRIMARY KEY, event_type INTEGER NOT NULL, created_at_ms INTEGER NOT NULL,
  processed_at_ms INTEGER, payload_codec INTEGER NOT NULL,
  payload_version INTEGER NOT NULL, payload BLOB NOT NULL
);
