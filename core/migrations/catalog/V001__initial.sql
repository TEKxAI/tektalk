CREATE TABLE IF NOT EXISTS profiles(
  profile_id INTEGER NOT NULL, profile_type INTEGER NOT NULL, display_name TEXT NOT NULL,
  username TEXT, avatar_url TEXT, cover_url TEXT, cover_type INTEGER,
  profile_version INTEGER NOT NULL, verification_state INTEGER NOT NULL DEFAULT 0,
  relationship_state INTEGER NOT NULL DEFAULT 0, updated_at_ms INTEGER NOT NULL,
  metadata_codec INTEGER NOT NULL, metadata_version INTEGER NOT NULL, metadata BLOB NOT NULL,
  PRIMARY KEY(profile_type,profile_id)
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS conversations(
  conversation_id INTEGER PRIMARY KEY, conversation_type INTEGER NOT NULL,
  peer_id INTEGER NOT NULL, peer_type INTEGER NOT NULL, display_name TEXT NOT NULL,
  display_avatar_url TEXT, display_badge INTEGER NOT NULL DEFAULT 0,
  last_message_id INTEGER, last_message_sender_id INTEGER, last_message_sender_type INTEGER,
  last_message_sender_name TEXT, last_message_type INTEGER, last_message_text TEXT,
  last_message_at_ms INTEGER, last_message_flags INTEGER NOT NULL DEFAULT 0,
  last_message_state INTEGER, draft_text TEXT, draft_updated_at_ms INTEGER,
  unread_count INTEGER NOT NULL DEFAULT 0, mention_count INTEGER NOT NULL DEFAULT 0,
  pinned_order INTEGER, muted_until_ms INTEGER, archived INTEGER NOT NULL DEFAULT 0,
  local_db_state INTEGER NOT NULL DEFAULT 0, local_db_size INTEGER NOT NULL DEFAULT 0,
  local_schema_version INTEGER, last_opened_at_ms INTEGER,
  metadata_codec INTEGER NOT NULL, metadata_version INTEGER NOT NULL, metadata BLOB NOT NULL
);

CREATE INDEX IF NOT EXISTS conversations_activity
  ON conversations(archived,pinned_order,last_message_at_ms DESC,conversation_id DESC);
CREATE INDEX IF NOT EXISTS conversations_peer ON conversations(peer_type,peer_id);
