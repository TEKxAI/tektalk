#include <cassert>

#include "tektalk/session.hpp"

int main() {
  tektalk::SessionState session(42);
  assert(session.session_id() == 42);
  assert(session.next_content_sequence() == 1);
  assert(session.next_content_sequence() == 3);
  assert(session.accept_message_id(0x100000000, tektalk::Direction::client_to_server) ==
         tektalk::AcceptResult::accepted);
  assert(session.accept_message_id(0x100000000, tektalk::Direction::client_to_server) ==
         tektalk::AcceptResult::replayed);
  assert(session.accept_message_id(0x100000002, tektalk::Direction::client_to_server) ==
         tektalk::AcceptResult::invalid_parity);
  assert(session.accept_message_id(0x100000001, tektalk::Direction::server_to_client) ==
         tektalk::AcceptResult::accepted);
  assert(session.accept_message_id(0x100000005, tektalk::Direction::server_to_client) ==
         tektalk::AcceptResult::accepted);
  assert(session.accept_message_id(0x100000003, tektalk::Direction::server_to_client) ==
         tektalk::AcceptResult::replayed);

  tektalk::SessionState directional(7);
  assert(directional.accept_message_id(4, tektalk::Direction::client_to_server) ==
         tektalk::AcceptResult::accepted);
  assert(directional.accept_message_id(1, tektalk::Direction::server_to_client) ==
         tektalk::AcceptResult::accepted);
  assert(directional.accept_message_id(8, tektalk::Direction::server_to_client) ==
         tektalk::AcceptResult::invalid_parity);

  tektalk::SessionState negative(8);
  assert(negative.accept_message_id(-4, tektalk::Direction::client_to_server) ==
         tektalk::AcceptResult::accepted);
  assert(negative.accept_message_id(-3, tektalk::Direction::server_to_client) ==
         tektalk::AcceptResult::accepted);
  return 0;
}
