#pragma once
#include <algorithm>
#include <cwctype>
#include <regex>
#include <string>

namespace tektalk {
inline bool ValidE164(std::wstring const& value) { return std::regex_match(value, std::wregex(LR"(^\+[1-9][0-9]{7,14}$)")); }
inline bool ValidPassword(std::wstring const& value) {
    return value.size() >= 10 && std::any_of(value.begin(), value.end(), [](wchar_t c) { return std::iswupper(c) != 0; }) && std::any_of(value.begin(), value.end(), [](wchar_t c) { return std::iswdigit(c) != 0; });
}
inline bool LooksLikeUuid(std::wstring const& value) { return std::regex_match(value, std::wregex(LR"(^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$)")); }
}
