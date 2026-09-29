package vn.tektalk

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class DomainLogicTest {
    @Test fun loginAcceptsValidE164AndPassword() {
        assertNull(AuthValidator.validate(AuthForm(AuthMode.Login, "+84901234567", "Correct1234")))
    }

    @Test fun loginRejectsLocalPhoneFormat() {
        assertEquals(
            "Số điện thoại phải theo chuẩn E.164",
            AuthValidator.validate(AuthForm(AuthMode.Login, "0901234567", "Correct1234")),
        )
    }

    @Test fun registrationRequiresProfileAndSecurityRecoveryData() {
        val base = AuthForm(AuthMode.Register, "+84901234567", "Password123")
        assertEquals("Tên hiển thị phải có ít nhất 2 ký tự", AuthValidator.validate(base))
        assertEquals(
            "Câu hỏi bảo mật quá ngắn",
            AuthValidator.validate(base.copy(displayName = "An", securityQuestion = "Tên?", securityAnswer = "TEK")),
        )
        assertNull(base.copy(displayName = "An", securityQuestion = "Trường đầu tiên?", securityAnswer = "TEK").let(AuthValidator::validate))
    }

    @Test fun deviceVerificationRequiresChallengeAndAnswer() {
        assertEquals("Phiên xác minh không hợp lệ", AuthValidator.validate(AuthForm(AuthMode.Verify, securityAnswer = "TEK")))
        assertEquals("Vui lòng nhập câu trả lời bảo mật", AuthValidator.validate(AuthForm(AuthMode.Verify, securityAnswer = " ", challengeId = "challenge")))
        assertNull(AuthValidator.validate(AuthForm(AuthMode.Verify, securityAnswer = "TEK", challengeId = "challenge")))
    }

    @Test fun chatDraftValidatesUuidBlankAndSizeBoundaries() {
        val valid = ChatDraft("6ba7b810-9dad-11d1-80b4-00c04fd430c8", "6ba7b811-9dad-11d1-80b4-00c04fd430c8", "hello")
        assertNull(ChatLogic.validationError(valid))
        assertEquals("Conversation ID không đúng định dạng UUID", ChatLogic.validationError(valid.copy(conversationId = "room-1")))
        assertEquals("Tin nhắn không được để trống", ChatLogic.validationError(valid.copy(text = "   ")))
        assertEquals("Tin nhắn vượt quá 16384 ký tự", ChatLogic.validationError(valid.copy(text = "x".repeat(16385))))
    }

    @Test fun mergeDeduplicatesServerReplayAndOrdersMessages() {
        val newer = message(20, "new")
        val older = message(10, "old")
        val replayWithCanonicalBody = message(20, "new from server")

        assertEquals(listOf(older, replayWithCanonicalBody), ChatLogic.merge(listOf(newer), listOf(older, replayWithCanonicalBody)))
    }

    private fun message(id: Long, body: String) = ChatMessage(
        id, "conversation", "sender", "recipient", "client-$id", body, "2026-09-29T00:00:00Z",
    )
}
