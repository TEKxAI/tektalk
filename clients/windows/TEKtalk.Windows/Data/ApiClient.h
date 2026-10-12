#pragma once
#include "pch.h"
namespace tektalk {
struct Tokens { winrt::hstring access_token, refresh_token, user_id, device_id; };
struct LoginResult { std::optional<Tokens> tokens; winrt::hstring challenge_id, question; };
struct Message { int64_t id{}; winrt::hstring sender_id, body, created_at; };

class ApiClient {
public:
    explicit ApiClient(winrt::hstring base = L"http://localhost:8080");
    winrt::Windows::Foundation::IAsyncOperation<winrt::Windows::Data::Json::JsonObject> Register(winrt::hstring const&, winrt::hstring const&, winrt::hstring const&, winrt::hstring const&, winrt::hstring const&);
    winrt::Windows::Foundation::IAsyncOperation<winrt::Windows::Data::Json::JsonObject> Login(winrt::hstring const&, winrt::hstring const&);
    winrt::Windows::Foundation::IAsyncOperation<winrt::Windows::Data::Json::JsonObject> Verify(winrt::hstring const&, winrt::hstring const&);
    winrt::Windows::Foundation::IAsyncOperation<winrt::Windows::Data::Json::JsonArray> History(winrt::hstring const&, winrt::hstring const&);
    winrt::Windows::Foundation::IAsyncOperation<winrt::Windows::Data::Json::JsonObject> Send(winrt::hstring const&, winrt::hstring const&, winrt::hstring const&, winrt::hstring const&);
    static Tokens ParseTokens(winrt::Windows::Data::Json::JsonObject const&);
    static Message ParseMessage(winrt::Windows::Data::Json::JsonObject const&);
private:
    winrt::Windows::Web::Http::HttpClient http_;
    winrt::hstring base_;
    winrt::Windows::Foundation::IAsyncOperation<winrt::Windows::Data::Json::JsonValue> Post(winrt::hstring const&, winrt::Windows::Data::Json::JsonObject const&, winrt::hstring const& token = L"");
};
}
