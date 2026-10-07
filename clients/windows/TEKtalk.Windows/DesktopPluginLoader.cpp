#include "pch.h"
#include "DesktopPluginLoader.h"
#include <fstream>
#include <regex>
#include <stdexcept>
#include <softpub.h>
#include <wintrust.h>

namespace {
bool AuthenticodeValid(std::filesystem::path const& path) {
    WINTRUST_FILE_INFO file{}; file.cbStruct=sizeof(file); file.pcwszFilePath=path.c_str();
    WINTRUST_DATA data{}; data.cbStruct=sizeof(data);data.dwUIChoice=WTD_UI_NONE;data.fdwRevocationChecks=WTD_REVOKE_NONE;data.dwUnionChoice=WTD_CHOICE_FILE;data.pFile=&file;data.dwStateAction=WTD_STATEACTION_VERIFY;data.dwProvFlags=WTD_CACHE_ONLY_URL_RETRIEVAL;
    GUID policy=WINTRUST_ACTION_GENERIC_VERIFY_V2;auto status=WinVerifyTrust(nullptr,&policy,&data);data.dwStateAction=WTD_STATEACTION_CLOSE;WinVerifyTrust(nullptr,&policy,&data);return status==ERROR_SUCCESS;
}
bool Below(std::filesystem::path const& root,std::filesystem::path const& value){auto r=std::filesystem::weakly_canonical(root).wstring();auto v=std::filesystem::weakly_canonical(value).wstring();return v.starts_with(r+L"\\");}
}
namespace tektalk {
NativeDesktopPlugin::~NativeDesktopPlugin(){if(raw_view_&&destroy_)destroy_(raw_view_);if(module_)FreeLibrary(module_);}
std::unique_ptr<NativeDesktopPlugin> NativeDesktopPlugin::Load(std::filesystem::path const& root,std::filesystem::path const& library,std::string const& expected_id){
    if(!Below(root,library)||!AuthenticodeValid(library))throw std::runtime_error("plugin path or Authenticode signature rejected");
    auto module=LoadLibraryExW(library.c_str(),nullptr,LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR|LOAD_LIBRARY_SEARCH_SYSTEM32);if(!module)throw std::runtime_error("plugin load failed");
    using ABI=uint32_t(__cdecl*)();using ID=char const*(__cdecl*)();using Health=int32_t(__cdecl*)();
    auto abi=reinterpret_cast<ABI>(GetProcAddress(module,"tektalk_plugin_abi_version"));auto id=reinterpret_cast<ID>(GetProcAddress(module,"tektalk_plugin_id"));auto health=reinterpret_cast<Health>(GetProcAddress(module,"tektalk_plugin_health_check"));
    auto create=reinterpret_cast<Create>(GetProcAddress(module,"tektalk_plugin_create_view"));auto destroy=reinterpret_cast<Destroy>(GetProcAddress(module,"tektalk_plugin_destroy_view"));
    if(!abi||!id||!health||!create||!destroy||abi()!=1||expected_id!=id()||health()!=1){FreeLibrary(module);throw std::runtime_error("plugin ABI, identity or health check rejected");}
    auto plugin=std::unique_ptr<NativeDesktopPlugin>(new NativeDesktopPlugin(module));plugin->create_=create;plugin->destroy_=destroy;return plugin;
}
winrt::Microsoft::UI::Xaml::FrameworkElement NativeDesktopPlugin::CreateView(){if(!raw_view_)raw_view_=create_(nullptr);if(!raw_view_)return nullptr;winrt::Windows::Foundation::IInspectable value{nullptr};winrt::copy_from_abi(value,reinterpret_cast<::IUnknown*>(raw_view_));return value.as<winrt::Microsoft::UI::Xaml::FrameworkElement>();}
winrt::Windows::Foundation::IAsyncOperation<std::filesystem::path> DesktopPluginInstaller::DownloadAndInstall(winrt::hstring const&url,std::string const&id,std::string const&version,std::array<uint8_t,32>const&digest,std::array<uint8_t,64>const&signature,std::array<uint8_t,32>const&key)const{
    auto uri=winrt::Windows::Foundation::Uri(url);if(uri.SchemeName()!=L"https")throw std::runtime_error("desktop plugins require HTTPS");
    winrt::Windows::Web::Http::HttpClient client;auto response=co_await client.GetAsync(uri);if(!response.IsSuccessStatusCode())throw std::runtime_error("plugin download failed");
    auto buffer=co_await response.Content().ReadAsBufferAsync();std::vector<uint8_t>bytes(buffer.Length());auto reader=winrt::Windows::Storage::Streams::DataReader::FromBuffer(buffer);reader.ReadBytes(bytes);co_return Install(id,version,bytes,digest,signature,key);
}
std::filesystem::path DesktopPluginInstaller::Install(std::string const&id,std::string const&version,std::span<uint8_t const>bytes,std::array<uint8_t,32>const&digest,std::array<uint8_t,64>const&signature,std::array<uint8_t,32>const&key)const{
    if(!std::regex_match(id,std::regex(R"(^[a-z][a-z0-9.-]{2,127}$)"))||!std::regex_match(version,std::regex(R"(^[0-9]+\.[0-9]+\.[0-9]+$)")))throw std::runtime_error("invalid plugin identity");
    if(!core_.VerifyArtifact(bytes,digest,signature,key))throw std::runtime_error("plugin Ed25519 signature rejected");
    auto directory=root_/std::filesystem::path(winrt::to_hstring(id).c_str())/std::filesystem::path(winrt::to_hstring(version).c_str());std::filesystem::create_directories(directory);auto staged=directory/L"plugin.dll.staged";auto active=directory/L"plugin.dll";
    {std::ofstream stream(staged,std::ios::binary|std::ios::trunc);stream.write(reinterpret_cast<char const*>(bytes.data()),static_cast<std::streamsize>(bytes.size()));if(!stream)throw std::runtime_error("plugin staging failed");}
    if(!AuthenticodeValid(staged)){std::filesystem::remove(staged);throw std::runtime_error("plugin Authenticode signature rejected");}
    if(!MoveFileExW(staged.c_str(),active.c_str(),MOVEFILE_REPLACE_EXISTING|MOVEFILE_WRITE_THROUGH))throw std::runtime_error("plugin activation failed");return active;
}
}
