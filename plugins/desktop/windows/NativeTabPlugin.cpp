#include "../include/tektalk_plugin.h"
#include <unknwn.h>
#include <winrt/base.h>
#include <winrt/Microsoft.UI.Xaml.h>
#include <winrt/Microsoft.UI.Xaml.Controls.h>

#if defined(TEKTALK_MESSAGE_PLUGIN)
static constexpr char ID[]="tektalk.message";static constexpr wchar_t TITLE[]=L"Message";
#elif defined(TEKTALK_AI_PLUGIN)
static constexpr char ID[]="tektalk.ai";static constexpr wchar_t TITLE[]=L"AI";
#else
static constexpr char ID[]="tektalk.me";static constexpr wchar_t TITLE[]=L"Me";
#endif

extern "C" uint32_t tektalk_plugin_abi_version(){return TEKTALK_DESKTOP_PLUGIN_ABI;}
extern "C" char const* tektalk_plugin_id(){return ID;}
extern "C" int32_t tektalk_plugin_health_check(){return 1;}
extern "C" void* tektalk_plugin_create_view(tektalk_host_api const*){
    winrt::Microsoft::UI::Xaml::Controls::StackPanel panel;panel.Spacing(12);
    winrt::Microsoft::UI::Xaml::Controls::TextBlock heading;heading.Text(TITLE);heading.FontSize(30);panel.Children().Append(heading);
    winrt::Microsoft::UI::Xaml::Controls::TextBlock detail;detail.Text(L"Native TEKtalk plugin • ABI 1");panel.Children().Append(detail);
    return winrt::detach_abi(panel.as<winrt::IInspectable>());
}
extern "C" void tektalk_plugin_destroy_view(void* view){if(view)reinterpret_cast<::IUnknown*>(view)->Release();}
