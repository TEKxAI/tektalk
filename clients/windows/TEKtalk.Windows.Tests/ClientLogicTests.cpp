#include "../TEKtalk.Windows/ClientLogic.h"
#include <cassert>
int main() {
    assert(tektalk::ValidE164(L"+84912345678"));
    assert(!tektalk::ValidE164(L"0912345678"));
    assert(tektalk::ValidPassword(L"StrongPass1"));
    assert(!tektalk::ValidPassword(L"weak"));
    assert(tektalk::LooksLikeUuid(L"550e8400-e29b-41d4-a716-446655440000"));
    assert(!tektalk::LooksLikeUuid(L"not-a-uuid"));
    return 0;
}
