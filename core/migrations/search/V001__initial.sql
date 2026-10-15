CREATE TABLE IF NOT EXISTS message_search(
  conversation_id INTEGER NOT NULL, message_id INTEGER NOT NULL, sender_id INTEGER NOT NULL,
  sent_at_ms INTEGER NOT NULL, normalized_text TEXT NOT NULL, content_type INTEGER NOT NULL,
  PRIMARY KEY(conversation_id,message_id)
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS message_search_timeline
  ON message_search(conversation_id,sent_at_ms DESC,message_id DESC);
CREATE INDEX IF NOT EXISTS message_search_text ON message_search(normalized_text);
