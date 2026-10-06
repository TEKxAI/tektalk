#pragma once
#include <windows.h>
#include <cstdint>
namespace tektalk {
class RustCore final { public: RustCore(); ~RustCore(); RustCore(RustCore const&)=delete; bool Available() const noexcept; int64_t NextSnowflake() const noexcept; private: using Create=void*(__cdecl*)(uint16_t);using Next=int64_t(__cdecl*)(void*);using Destroy=void(__cdecl*)(void*);HMODULE module_{};void* generator_{};Next next_{};Destroy destroy_{}; };
}
