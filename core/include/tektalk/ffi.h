#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct tektalk_session tektalk_session;
typedef struct tektalk_snowflake tektalk_snowflake;

enum tektalk_accept_result {
  TEKTALK_ACCEPTED = 0,
  TEKTALK_INVALID_PARITY = 1,
  TEKTALK_REPLAYED = 2,
};

tektalk_session* tektalk_session_create(int64_t session_id);
void tektalk_session_destroy(tektalk_session* session);
int32_t tektalk_session_next_content_sequence(tektalk_session* session);
enum tektalk_accept_result tektalk_session_accept_message_id(
    tektalk_session* session,
    int64_t message_id,
    int client_to_server);

/* 41-bit time, 10-bit assigned node, 12-bit sequence. Returns NULL for an
 * invalid node and -1 if generation fails (for example, clock regression). */
tektalk_snowflake* tektalk_snowflake_create(uint16_t node_id);
void tektalk_snowflake_destroy(tektalk_snowflake* generator);
int64_t tektalk_snowflake_next(tektalk_snowflake* generator);

/* Generates a monotonically increasing MTProto 2.0 wire message ID. This is
 * intentionally distinct from a persisted Snowflake message ID. */
int64_t tektalk_session_next_mtproto_message_id(
    tektalk_session* session,
    int client_to_server);

int32_t tektalk_plugin_verify_artifact(
    const uint8_t* bytes,
    uintptr_t length,
    const uint8_t expected_sha256[32],
    const uint8_t signature[64],
    const uint8_t public_key[32]);

#ifdef __cplusplus
}
#endif
