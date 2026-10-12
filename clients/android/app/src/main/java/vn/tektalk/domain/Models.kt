package vn.tektalk.domain

enum class AuthMode { Login, Register, Verify }

data class AuthForm(
    val mode: AuthMode,
    val phone: String = "",
    val password: String = "",
    val displayName: String = "",
    val securityQuestion: String = "",
    val securityAnswer: String = "",
    val challengeId: String? = null,
)

data class SessionTokens(
    val accessToken: String,
    val refreshToken: String,
    val userId: String,
    val deviceId: String,
)

data class LoginResult(
    val tokens: SessionTokens? = null,
    val challengeId: String? = null,
    val question: String? = null,
)

data class Message(
    val id: Long,
    val conversationId: String,
    val senderId: String,
    val recipientId: String,
    val clientMessageId: String,
    val body: String,
    val createdAt: String,
)

data class ChatDraft(val conversationId: String, val recipientId: String, val text: String)

