#pragma once
#include "RustCore.h"
#include <array>
#include <filesystem>
#include <memory>
#include <span>
#include <string>

namespace tektalk {
class NativeDesktopPlugin final {
public:
    ~NativeDesktopPlugin();
    NativeDesktopPlugin(NativeDesktopPlugin const&) = delete;
    static std::unique_ptr<NativeDesktopPlugin> Load(std::filesystem::path const& root, std::filesystem::path const& library, std::string const& expected_id);
    winrt::Microsoft::UI::Xaml::FrameworkElement CreateView();
private:
    explicit NativeDesktopPlugin(HMODULE module) : module_(module) {}
    using Create = void*(__cdecl*)(void const*); using Destroy = void(__cdecl*)(void*);
    HMODULE module_{}; Create create_{}; Destroy destroy_{}; void* raw_view_{};
};

class DesktopPluginInstaller final {
public:
    DesktopPluginInstaller(std::filesystem::path root, RustCore const& core) : root_(std::move(root)), core_(core) {}
    std::filesystem::path Install(std::string const& id, std::string const& version, std::span<uint8_t const> bytes,
        std::array<uint8_t,32> const& digest, std::array<uint8_t,64> const& signature, std::array<uint8_t,32> const& public_key) const;
private:
    std::filesystem::path root_; RustCore const& core_;
};
}
