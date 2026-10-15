#pragma once
#include "../Infrastructure/RustCore.h"
#include <array>
#include <cstdint>
#include <filesystem>

namespace tektalk {
class EncryptedDatabase final {
public:
    EncryptedDatabase(RustCore const& core, int64_t accountId);
    ~EncryptedDatabase();
    EncryptedDatabase(EncryptedDatabase const&)=delete;
    bool Available() const noexcept { return database_ != nullptr; }
    bool OpenConversation(int64_t conversationId) const noexcept;
private:
    static std::array<uint8_t,32> LoadOrCreateKey(int64_t accountId);
    static std::filesystem::path AccountRoot(int64_t accountId);
    RustCore const& core_;
    void* database_{};
};
}
