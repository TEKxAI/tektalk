#include "pch.h"
#include "RustCore.h"
namespace tektalk {
RustCore::RustCore(){module_=LoadLibraryW(L"tektalk_client_core.dll");if(!module_)return;auto create=reinterpret_cast<Create>(GetProcAddress(module_,"tektalk_snowflake_create"));next_=reinterpret_cast<Next>(GetProcAddress(module_,"tektalk_snowflake_next"));destroy_=reinterpret_cast<Destroy>(GetProcAddress(module_,"tektalk_snowflake_destroy"));if(create&&next_&&destroy_)generator_=create(18);}
RustCore::~RustCore(){if(generator_&&destroy_)destroy_(generator_);if(module_)FreeLibrary(module_);}
bool RustCore::Available()const noexcept{return generator_&&next_;}int64_t RustCore::NextSnowflake()const noexcept{return Available()?next_(generator_):-1;}
}
