#include "pch.h"
#include "RustCore.h"
namespace tektalk {
RustCore::RustCore(){module_=LoadLibraryW(L"tektalk_client_core.dll");if(!module_)return;auto create=reinterpret_cast<Create>(GetProcAddress(module_,"tektalk_snowflake_create"));next_=reinterpret_cast<Next>(GetProcAddress(module_,"tektalk_snowflake_next"));destroy_=reinterpret_cast<Destroy>(GetProcAddress(module_,"tektalk_snowflake_destroy"));verify_=reinterpret_cast<Verify>(GetProcAddress(module_,"tektalk_plugin_verify_artifact"));if(create&&next_&&destroy_&&verify_)generator_=create(18);}
RustCore::~RustCore(){if(generator_&&destroy_)destroy_(generator_);if(module_)FreeLibrary(module_);}
bool RustCore::Available()const noexcept{return generator_&&next_;}int64_t RustCore::NextSnowflake()const noexcept{return Available()?next_(generator_):-1;}
bool RustCore::VerifyArtifact(std::span<uint8_t const>b,std::array<uint8_t,32>const&d,std::array<uint8_t,64>const&s,std::array<uint8_t,32>const&k)const noexcept{return verify_&&verify_(b.data(),b.size(),d.data(),s.data(),k.data())==1;}
}
