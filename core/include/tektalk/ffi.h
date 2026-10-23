#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct tektalk_session tektalk_session;
typedef struct tektalk_snowflake tektalk_snowflake;
typedef struct tektalk_account_store tektalk_account_store;
typedef struct tektalk_secret_device tektalk_secret_device;

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

/* Opens an account-scoped SQLCipher store. The key must contain exactly 32
 * bytes. NULL means SQLCipher is unavailable or the database cannot decrypt. */
tektalk_account_store* tektalk_database_open(
    const char* root_utf8,
    const uint8_t* key,
    uintptr_t key_length,
    uint32_t max_open_conversations);
void tektalk_database_close(tektalk_account_store* database);
int32_t tektalk_database_open_conversation(
    tektalk_account_store* database,
    int64_t conversation_id);
int32_t tektalk_database_open_handle_count(
    const tektalk_account_store* database);

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

/* Opt-in Secret Chat. All returned strings are UTF-8 JSON owned by Rust and
 * must be released with tektalk_string_destroy. A NULL result fails closed. */
tektalk_secret_device* tektalk_secret_device_create(const char* user_id, const char* device_id);
void tektalk_secret_device_destroy(tektalk_secret_device* device);
char* tektalk_secret_public_bundle_json(const tektalk_secret_device* device);
char* tektalk_secret_begin_epoch_json(tektalk_secret_device* device, const char* conversation_id, const char* kind, uint64_t epoch, const char* bundles_json);
int32_t tektalk_secret_accept_key_json(tektalk_secret_device* device, const char* conversation_id, const char* sender_device_id, const char* kind, uint64_t epoch, const char* wrapped_json);
char* tektalk_secret_encrypt_json(tektalk_secret_device* device, const char* conversation_id, uint64_t epoch, const char* plaintext);
char* tektalk_secret_decrypt_json(tektalk_secret_device* device, const char* message_json);
void tektalk_string_destroy(char* value);

#ifdef __cplusplus
}
#endif
