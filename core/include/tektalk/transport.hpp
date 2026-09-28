#pragma once

#include <cstdint>
#include <functional>
#include <span>
#include <string>
#include <vector>

namespace tektalk {

struct TransportError {
  std::int32_t code;
  std::string message;
  bool retryable;
};

class Transport {
 public:
  using FrameHandler = std::function<void(std::vector<std::uint8_t>)>;
  using ErrorHandler = std::function<void(TransportError)>;

  virtual ~Transport() = default;
  virtual void connect(FrameHandler on_frame, ErrorHandler on_error) = 0;
  virtual void send(std::span<const std::uint8_t> frame) = 0;
  virtual void close() noexcept = 0;
};

}  // namespace tektalk
