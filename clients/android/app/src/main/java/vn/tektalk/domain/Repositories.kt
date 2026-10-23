package vn.tektalk.domain

interface AuthRepository {
    fun register(phone: String, displayName: String, password: String, question: String, answer: String, deviceName: String): SessionTokens
    fun login(phone: String, password: String, deviceName: String): LoginResult
    fun verifyDevice(challengeId: String, answer: String): SessionTokens
}

interface ChatRepository {
    fun history(accessToken: String, conversationId: String): List<Message>
    fun send(accessToken: String, draft: ChatDraft): Message
}

interface SecretChatRepository : AutoCloseable {
    fun enable(accessToken:String,tokens:SessionTokens,conversationId:String,kind:String,memberIds:List<String>)
    fun history(accessToken:String,tokens:SessionTokens,conversationId:String):List<Message>
    fun send(accessToken:String,tokens:SessionTokens,draft:ChatDraft):Message
}

