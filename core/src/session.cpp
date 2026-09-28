#include "tektalk/session.hpp"

namespace tektalk {

SessionState::SessionState(std::int64_t session_id) noexcept
    : session_id_(session_id) {}

std::int64_t SessionState::session_id() const noexcept { return session_id_; }

std::int32_t SessionState::next_content_sequence() noexcept {
  const auto sequence = content_sequence_ * 2 + 1;
  ++content_sequence_;
  return sequence;
}

AcceptResult SessionState::accept_message_id(
    std::int64_t message_id,
    Direction direction) noexcept {
  const auto modulo = ((message_id % 4) + 4) % 4;
  const auto valid = direction == Direction::client_to_server
                         ? modulo == 0
                         : modulo == 1 || modulo == 3;
  if (!valid) {
    return AcceptResult::invalid_parity;
  }

  auto& last = direction == Direction::client_to_server
                   ? last_client_message_id_
                   : last_server_message_id_;
  if (last.has_value() && message_id <= *last) {
    return AcceptResult::replayed;
  }
  last = message_id;
  return AcceptResult::accepted;
}

}  // namespace tektalk
