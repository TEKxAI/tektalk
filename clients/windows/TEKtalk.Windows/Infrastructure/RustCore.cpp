#include "pch.h"
#include "RustCore.h"
namespace tektalk {
RustCore::RustCore(){module_=LoadLibraryW(L"tektalk_client_core.dll");if(!module_)return;auto create=reinterpret_cast<Create>(GetProcAddress(module_,"tektalk_snowflake_create"));next_=reinterpret_cast<Next>(GetProcAddress(module_,"tektalk_snowflake_next"));destroy_=reinterpret_cast<Destroy>(GetProcAddress(module_,"tektalk_snowflake_destroy"));verify_=reinterpret_cast<Verify>(GetProcAddress(module_,"tektalk_plugin_verify_artifact"));databaseOpen_=reinterpret_cast<DatabaseOpen>(GetProcAddress(module_,"tektalk_database_open"));databaseClose_=reinterpret_cast<DatabaseClose>(GetProcAddress(module_,"tektalk_database_close"));conversationOpen_=reinterpret_cast<ConversationOpen>(GetProcAddress(module_,"tektalk_database_open_conversation"));if(create&&next_&&destroy_&&verify_&&databaseOpen_&&databaseClose_&&conversationOpen_)generator_=create(18);}
RustCore::~RustCore(){if(generator_&&destroy_)destroy_(generator_);if(module_)FreeLibrary(module_);}
bool RustCore::Available()const noexcept{return generator_&&next_;}int64_t RustCore::NextSnowflake()const noexcept{return Available()?next_(generator_):-1;}
bool RustCore::VerifyArtifact(std::span<uint8_t const>b,std::array<uint8_t,32>const&d,std::array<uint8_t,64>const&s,std::array<uint8_t,32>const&k)const noexcept{return verify_&&verify_(b.data(),b.size(),d.data(),s.data(),k.data())==1;}
void* RustCore::OpenDatabase(std::string const&root,std::span<uint8_t const,32>key,uint32_t maxOpen)const noexcept{return databaseOpen_?databaseOpen_(root.c_str(),key.data(),key.size(),maxOpen):nullptr;}
void RustCore::CloseDatabase(void*database)const noexcept{if(database&&databaseClose_)databaseClose_(database);}
bool RustCore::OpenConversation(void*database,int64_t conversationId)const noexcept{return database&&conversationId>0&&conversationOpen_&&conversationOpen_(database,conversationId)==1;}
}
