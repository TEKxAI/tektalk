CREATE KEYSPACE IF NOT EXISTS chat WITH replication = {'class':'NetworkTopologyStrategy','dc1':3};
CREATE TABLE IF NOT EXISTS chat.messages_by_conversation_bucket (
  conversation_id uuid, bucket date, message_id bigint, sender_id uuid, recipient_id uuid,
  client_message_id uuid, ciphertext blob, created_at timestamp,
  PRIMARY KEY ((conversation_id, bucket), message_id)
) WITH CLUSTERING ORDER BY (message_id DESC)
  AND compaction = {'class':'TimeWindowCompactionStrategy'};
CREATE TABLE IF NOT EXISTS chat.dedup_by_sender (
  sender_id uuid, day date, client_message_id uuid, server_message_id bigint,
  PRIMARY KEY ((sender_id, day), client_message_id)
) WITH default_time_to_live = 604800;

