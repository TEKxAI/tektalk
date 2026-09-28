#include "tektalk/ffi.h"

#include <new>

#include "tektalk/session.hpp"

struct tektalk_session {
  explicit tektalk_session(int64_t id) : state(id) {}
  tektalk::SessionState state;
};

tektalk_session* tektalk_session_create(int64_t session_id) {
  return new (std::nothrow) tektalk_session(session_id);
}

void tektalk_session_destroy(tektalk_session* session) { delete session; }

int32_t tektalk_session_next_content_sequence(tektalk_session* session) {
  return session == nullptr ? -1 : session->state.next_content_sequence();
}

enum tektalk_accept_result tektalk_session_accept_message_id(
    tektalk_session* session,
    int64_t message_id,
    int client_to_server) {
  if (session == nullptr) {
    return TEKTALK_REPLAYED;
  }
  const auto direction = client_to_server != 0
                             ? tektalk::Direction::client_to_server
                             : tektalk::Direction::server_to_client;
  return static_cast<enum tektalk_accept_result>(
      session->state.accept_message_id(message_id, direction));
}
