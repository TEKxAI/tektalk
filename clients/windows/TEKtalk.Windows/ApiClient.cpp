#include "pch.h"
#include "ApiClient.h"
#include "ClientLogic.h"
using namespace winrt; using namespace Windows::Foundation; using namespace Windows::Data::Json; using namespace Windows::Web::Http;
namespace tektalk {
ApiClient::ApiClient(hstring base) : base_(std::move(base)) {}
IAsyncOperation<JsonValue> ApiClient::Post(hstring const& path, JsonObject const& body, hstring const& token) {
    HttpRequestMessage request(HttpMethod::Post(), Windows::Foundation::Uri(base_ + path));
    HttpStringContent content(body.Stringify());
    content.Headers().ContentType(Windows::Web::Http::Headers::HttpMediaTypeHeaderValue(L"application/json"));
    request.Content(content);
    if (!token.empty()) request.Headers().Authorization(Windows::Web::Http::Headers::HttpCredentialsHeaderValue(L"Bearer", token));
    auto response = co_await http_.SendRequestAsync(request); auto text = co_await response.Content().ReadAsStringAsync();
    if (!response.IsSuccessStatusCode()) throw hresult_error(E_FAIL, text);
    co_return JsonValue::Parse(text);
}
Tokens ApiClient::ParseTokens(JsonObject const& o) { return {o.GetNamedString(L"access_token"), o.GetNamedString(L"refresh_token"), o.GetNamedString(L"user_id"), o.GetNamedString(L"device_id")}; }
Message ApiClient::ParseMessage(JsonObject const& m){return {static_cast<int64_t>(m.GetNamedNumber(L"id")),m.GetNamedString(L"sender_id"),m.GetNamedString(L"body"),m.GetNamedString(L"created_at")};}
IAsyncOperation<JsonObject> ApiClient::Register(hstring const& phone,hstring const& name,hstring const& password,hstring const& question,hstring const& answer) { JsonObject o; o.SetNamedValue(L"phone",JsonValue::CreateStringValue(phone));o.SetNamedValue(L"display_name",JsonValue::CreateStringValue(name));o.SetNamedValue(L"password",JsonValue::CreateStringValue(password));o.SetNamedValue(L"security_question",JsonValue::CreateStringValue(question));o.SetNamedValue(L"security_answer",JsonValue::CreateStringValue(answer));o.SetNamedValue(L"device_name",JsonValue::CreateStringValue(L"Windows PC")); auto value=co_await Post(L"/v1/auth/register",o);co_return value.GetObject(); }
IAsyncOperation<JsonObject> ApiClient::Login(hstring const& phone,hstring const& password) { JsonObject o;o.SetNamedValue(L"phone",JsonValue::CreateStringValue(phone));o.SetNamedValue(L"password",JsonValue::CreateStringValue(password));o.SetNamedValue(L"device_id",JsonValue::CreateNullValue());o.SetNamedValue(L"device_name",JsonValue::CreateStringValue(L"Windows PC"));auto value=co_await Post(L"/v1/auth/login",o);co_return value.GetObject(); }
IAsyncOperation<JsonObject> ApiClient::Verify(hstring const& challenge,hstring const& answer){JsonObject o;o.SetNamedValue(L"challenge_id",JsonValue::CreateStringValue(challenge));o.SetNamedValue(L"answer",JsonValue::CreateStringValue(answer));auto value=co_await Post(L"/v1/auth/device/verify",o);co_return value.GetObject();}
IAsyncOperation<JsonArray> ApiClient::History(hstring const& token,hstring const& conversation){JsonObject o;o.SetNamedValue(L"conversation_id",JsonValue::CreateStringValue(conversation));o.SetNamedValue(L"before_message_id",JsonValue::CreateNullValue());o.SetNamedValue(L"limit",JsonValue::CreateNumberValue(50));auto value=co_await Post(L"/v1/chat/messages/history",o,token);co_return value.GetArray();}
IAsyncOperation<JsonObject> ApiClient::Send(hstring const& token,hstring const& conversation,hstring const& recipient,hstring const& text){JsonObject o;o.SetNamedValue(L"conversation_id",JsonValue::CreateStringValue(conversation));o.SetNamedValue(L"recipient_id",JsonValue::CreateStringValue(recipient));o.SetNamedValue(L"client_message_id",JsonValue::CreateStringValue(to_hstring(GuidHelper::CreateNewGuid())));o.SetNamedValue(L"text",JsonValue::CreateStringValue(text));auto value=co_await Post(L"/v1/chat/messages/send",o,token);co_return value.GetObject().GetNamedObject(L"message");}
}
