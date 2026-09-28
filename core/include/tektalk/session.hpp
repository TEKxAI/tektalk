#pragma once

#include <cstdint>
#include <optional>

namespace tektalk {

enum class Direction : std::uint8_t {
  client_to_server,
  server_to_client,
};

enum class AcceptResult : std::uint8_t {
  accepted,
  invalid_parity,
  replayed,
};

class SessionState final {
 public:
  explicit SessionState(std::int64_t session_id) noexcept;

  [[nodiscard]] std::int64_t session_id() const noexcept;
  [[nodiscard]] std::int32_t next_content_sequence() noexcept;
  [[nodiscard]] AcceptResult accept_message_id(
      std::int64_t message_id,
      Direction direction) noexcept;

 private:
  std::int64_t session_id_;
  std::int32_t content_sequence_{0};
  std::optional<std::int64_t> last_client_message_id_;
  std::optional<std::int64_t> last_server_message_id_;
};

}  // namespace tektalk
