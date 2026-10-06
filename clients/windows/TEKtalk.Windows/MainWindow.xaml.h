#pragma once
#include "MainWindow.g.h"
#include "ApiClient.h"
#include "RustCore.h"
namespace winrt::TEKtalk::implementation {
struct MainWindow : MainWindowT<MainWindow> {
    MainWindow();
private:
    tektalk::ApiClient api_; tektalk::RustCore core_; std::optional<tektalk::Tokens> tokens_; bool register_mode_{}; winrt::hstring challenge_;
    Microsoft::UI::Xaml::Controls::Grid root_{nullptr}; Microsoft::UI::Xaml::Controls::TextBox phone_{nullptr},name_{nullptr},question_{nullptr},answer_{nullptr},conversation_{nullptr},recipient_{nullptr},draft_{nullptr}; Microsoft::UI::Xaml::Controls::PasswordBox password_{nullptr}; Microsoft::UI::Xaml::Controls::TextBlock status_{nullptr}; Microsoft::UI::Xaml::Controls::ListView messages_{nullptr};
    void ShowAuth();void ShowHost();void ShowMessage();void ShowAI();void ShowMe();
    winrt::Windows::Foundation::IAsyncAction SubmitAuth();winrt::Windows::Foundation::IAsyncAction LoadMessages();winrt::Windows::Foundation::IAsyncAction SendMessage();
    static Microsoft::UI::Xaml::Controls::TextBlock Text(winrt::hstring const&,double=14,bool=false);
};
}
namespace winrt::TEKtalk::factory_implementation {
struct MainWindow : MainWindowT<MainWindow, implementation::MainWindow> {};
}
