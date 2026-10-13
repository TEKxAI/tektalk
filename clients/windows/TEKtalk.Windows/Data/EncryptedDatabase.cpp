#include "pch.h"
#include "EncryptedDatabase.h"
#include <algorithm>
#include <bcrypt.h>
#include <cstdlib>
#include <fstream>
#include <iterator>
#include <stdexcept>
#include <vector>
#pragma comment(lib,"bcrypt.lib")

namespace tektalk {
namespace {
std::filesystem::path LocalRoot(){
    wchar_t* value{}; size_t ignored{};
    _wdupenv_s(&value,&ignored,L"LOCALAPPDATA");
    if(!value) throw std::runtime_error("LOCALAPPDATA is unavailable");
    std::filesystem::path root(value); free(value);
    return root/L"TEKtalk";
}
}

EncryptedDatabase::EncryptedDatabase(RustCore const&core,int64_t accountId):core_(core){
    if(accountId<=0)return;
    auto root=AccountRoot(accountId);std::filesystem::create_directories(root);
    auto key=LoadOrCreateKey(accountId);
    auto utf8=root.u8string();
    database_=core_.OpenDatabase(std::string(reinterpret_cast<char const*>(utf8.c_str()),utf8.size()),key,12);
    SecureZeroMemory(key.data(),key.size());
}
EncryptedDatabase::~EncryptedDatabase(){core_.CloseDatabase(database_);}
bool EncryptedDatabase::OpenConversation(int64_t id)const noexcept{return core_.OpenConversation(database_,id);}
std::filesystem::path EncryptedDatabase::AccountRoot(int64_t id){return LocalRoot()/L"accounts"/std::to_wstring(id);}

std::array<uint8_t,32> EncryptedDatabase::LoadOrCreateKey(int64_t accountId){
    auto directory=LocalRoot()/L"keys";std::filesystem::create_directories(directory);
    auto path=directory/(std::to_wstring(accountId)+L".dpapi");
    if(std::filesystem::exists(path)){
        std::ifstream input(path,std::ios::binary);std::vector<uint8_t> encrypted((std::istreambuf_iterator<char>(input)),{});
        DATA_BLOB source{static_cast<DWORD>(encrypted.size()),encrypted.data()},plain{};
        if(!CryptUnprotectData(&source,nullptr,nullptr,nullptr,nullptr,CRYPTPROTECT_UI_FORBIDDEN,&plain))throw std::runtime_error("DPAPI decrypt failed");
        if(plain.cbData!=32){LocalFree(plain.pbData);throw std::runtime_error("Invalid database key");}
        std::array<uint8_t,32> key{};std::copy_n(plain.pbData,32,key.data());SecureZeroMemory(plain.pbData,plain.cbData);LocalFree(plain.pbData);return key;
    }
    std::array<uint8_t,32> key{};
    if(BCryptGenRandom(nullptr,key.data(),static_cast<ULONG>(key.size()),BCRYPT_USE_SYSTEM_PREFERRED_RNG)!=0)throw std::runtime_error("Secure RNG failed");
    DATA_BLOB plain{static_cast<DWORD>(key.size()),key.data()},encrypted{};
    if(!CryptProtectData(&plain,L"TEKtalk database key",nullptr,nullptr,nullptr,CRYPTPROTECT_UI_FORBIDDEN,&encrypted))throw std::runtime_error("DPAPI encrypt failed");
    std::ofstream output(path,std::ios::binary|std::ios::trunc);output.write(reinterpret_cast<char const*>(encrypted.pbData),encrypted.cbData);output.close();
    SecureZeroMemory(encrypted.pbData,encrypted.cbData);LocalFree(encrypted.pbData);return key;
}
}
