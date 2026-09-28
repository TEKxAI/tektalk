#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct tektalk_session tektalk_session;

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

#ifdef __cplusplus
}
#endif
