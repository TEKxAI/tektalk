package vn.tektalk.data

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import vn.tektalk.BuildConfig
import vn.tektalk.domain.*
import java.util.UUID

@Serializable private data class RegisterDto(val phone:String,val display_name:String,val password:String,val security_question:String,val security_answer:String,val device_name:String)
@Serializable private data class LoginDto(val phone:String,val password:String,val device_id:String?=null,val device_name:String)
@Serializable private data class VerifyDto(val challenge_id:String,val answer:String)
@Serializable private data class TokensDto(val access_token:String,val refresh_token:String,val user_id:String,val device_id:String)
@Serializable private data class LoginResponseDto(val status:String,val tokens:TokensDto?=null,val challenge_id:String?=null,val question:String?=null)
@Serializable private data class SendDto(val conversation_id:String,val recipient_id:String,val client_message_id:String,val text:String)
@Serializable private data class HistoryDto(val conversation_id:String,val before_message_id:Long?=null,val limit:Long=50)
@Serializable private data class MessageDto(val id:Long,val conversation_id:String,val sender_id:String,val recipient_id:String,val client_message_id:String,val body:String,val created_at:String)
@Serializable private data class SendResponseDto(val message:MessageDto,val deduplicated:Boolean)

private fun TokensDto.domain() = SessionTokens(access_token, refresh_token, user_id, device_id)
private fun MessageDto.domain() = Message(id, conversation_id, sender_id, recipient_id, client_message_id, body, created_at)

internal class HttpApi(
    @PublishedApi internal val base:String=BuildConfig.API_BASE_URL,
    @PublishedApi internal val http:OkHttpClient=OkHttpClient(),
) {
    @PublishedApi internal val json=Json{ignoreUnknownKeys=true}
    @PublishedApi internal val media="application/json".toMediaType()
    internal inline fun <reified I,reified O> post(path:String,body:I,token:String?=null):O {
        val builder=Request.Builder().url(base+path).post(json.encodeToString(body).toRequestBody(media))
        token?.let { builder.header("Authorization", "Bearer $it") }
        http.newCall(builder.build()).execute().use { response ->
            val payload=response.body?.string().orEmpty()
            if(!response.isSuccessful) error("HTTP ${response.code}: $payload")
            return json.decodeFromString(payload)
        }
    }
}

class HttpAuthRepository internal constructor(private val api:HttpApi=HttpApi()):AuthRepository {
    override fun register(phone:String,displayName:String,password:String,question:String,answer:String,deviceName:String)=
        api.post<RegisterDto,TokensDto>("/v1/auth/register",RegisterDto(phone,displayName,password,question,answer,deviceName)).domain()
    override fun login(phone:String,password:String,deviceName:String):LoginResult {
        val r=api.post<LoginDto,LoginResponseDto>("/v1/auth/login",LoginDto(phone,password,null,deviceName))
        return LoginResult(r.tokens?.domain(),r.challenge_id,r.question)
    }
    override fun verifyDevice(challengeId:String,answer:String)=api.post<VerifyDto,TokensDto>("/v1/auth/device/verify",VerifyDto(challengeId,answer)).domain()
}

class HttpChatRepository internal constructor(private val api:HttpApi=HttpApi()):ChatRepository {
    override fun history(accessToken:String,conversationId:String)=api.post<HistoryDto,List<MessageDto>>("/v1/chat/messages/history",HistoryDto(conversationId),accessToken).map{it.domain()}
    override fun send(accessToken:String,draft:ChatDraft)=api.post<SendDto,SendResponseDto>("/v1/chat/messages/send",SendDto(draft.conversationId,draft.recipientId,UUID.randomUUID().toString(),draft.text),accessToken).message.domain()
}

object ClientDependencies {
    val auth: AuthRepository by lazy { HttpAuthRepository() }
    val chat: ChatRepository by lazy { HttpChatRepository() }
}
