#pragma once
#include <windows.h>
#include <array>
#include <cstdint>
#include <span>
namespace tektalk {
class RustCore final { public: RustCore(); ~RustCore(); RustCore(RustCore const&)=delete; bool Available() const noexcept; int64_t NextSnowflake() const noexcept; bool VerifyArtifact(std::span<uint8_t const>,std::array<uint8_t,32> const&,std::array<uint8_t,64> const&,std::array<uint8_t,32> const&) const noexcept; private: using Create=void*(__cdecl*)(uint16_t);using Next=int64_t(__cdecl*)(void*);using Destroy=void(__cdecl*)(void*);using Verify=int32_t(__cdecl*)(uint8_t const*,uintptr_t,uint8_t const*,uint8_t const*,uint8_t const*);HMODULE module_{};void* generator_{};Next next_{};Destroy destroy_{};Verify verify_{}; };
}
