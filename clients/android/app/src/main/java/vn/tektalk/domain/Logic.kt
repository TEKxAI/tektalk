package vn.tektalk.domain

object AuthValidator {
    private val e164 = Regex("^\\+[1-9]\\d{7,14}$")

    fun validate(form: AuthForm): String? = when (form.mode) {
        AuthMode.Verify -> when {
            form.challengeId.isNullOrBlank() -> "Phiên xác minh không hợp lệ"
            form.securityAnswer.trim().length < 2 -> "Vui lòng nhập câu trả lời bảo mật"
            else -> null
        }
        AuthMode.Login, AuthMode.Register -> when {
            !e164.matches(form.phone.trim()) -> "Số điện thoại phải theo chuẩn E.164"
            form.password.length < 10 || form.password.none(Char::isUpperCase) || form.password.none(Char::isDigit) -> "Mật khẩu cần ít nhất 10 ký tự, một chữ hoa và một chữ số"
            form.mode == AuthMode.Register && form.displayName.trim().length < 2 -> "Tên hiển thị phải có ít nhất 2 ký tự"
            form.mode == AuthMode.Register && form.securityQuestion.trim().length < 5 -> "Câu hỏi bảo mật quá ngắn"
            form.mode == AuthMode.Register && form.securityAnswer.trim().length < 2 -> "Câu trả lời bảo mật quá ngắn"
            else -> null
        }
    }
}

object ChatLogic {
    fun validationError(draft: ChatDraft): String? = when {
        draft.conversationId.isBlank() -> "Thiếu conversation ID"
        !isUuid(draft.conversationId) -> "Conversation ID không đúng định dạng UUID"
        draft.recipientId.isBlank() -> "Thiếu recipient ID"
        !isUuid(draft.recipientId) -> "Recipient ID không đúng định dạng UUID"
        draft.text.isBlank() -> "Tin nhắn không được để trống"
        draft.text.length > 16_384 -> "Tin nhắn vượt quá 16384 ký tự"
        else -> null
    }

    fun merge(current: List<Message>, incoming: List<Message>): List<Message> =
        (current + incoming).associateBy(Message::id).values.sortedBy(Message::id)

    private fun isUuid(value: String) = runCatching { java.util.UUID.fromString(value.trim()) }.isSuccess
}

