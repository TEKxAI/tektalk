package vn.tektalk

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody

@Serializable data class RegisterRequest(val phone:String,val display_name:String,val password:String,val security_question:String,val security_answer:String,val device_name:String)
@Serializable data class LoginRequest(val phone:String,val password:String,val device_id:String?=null,val device_name:String)
@Serializable data class VerifyDeviceRequest(val challenge_id:String,val answer:String)
@Serializable data class Tokens(val access_token:String,val refresh_token:String,val user_id:String,val device_id:String)
@Serializable data class LoginResponse(val status:String,val tokens:Tokens?=null,val challenge_id:String?=null,val question:String?=null)
@Serializable data class BootstrapRequest(val access_token:String)
@Serializable data class Bootstrap(val ticket:String,val server_public_key:String,val expires_in_seconds:Int,val protocol_version:Int)
@Serializable data class SendMessageRequest(val conversation_id:String,val recipient_id:String,val client_message_id:String,val text:String)
@Serializable data class HistoryRequest(val conversation_id:String,val before_message_id:Long?=null,val limit:Long=50)
@Serializable data class ChatMessage(val id:Long,val conversation_id:String,val sender_id:String,val recipient_id:String,val client_message_id:String,val body:String,val created_at:String)
@Serializable data class SendMessageResponse(val message:ChatMessage,val deduplicated:Boolean)

class Api(private val base:String=BuildConfig.API_BASE_URL,private val http:OkHttpClient=OkHttpClient()){
    private val json=Json{ignoreUnknownKeys=true};private val media="application/json".toMediaType()
    private inline fun <reified I,reified O> post(path:String,body:I,token:String?=null):O{
        val builder=Request.Builder().url(base+path).post(json.encodeToString(body).toRequestBody(media))
        if(token!=null)builder.header("Authorization","Bearer $token")
        http.newCall(builder.build()).execute().use{response->val payload=response.body?.string().orEmpty();if(!response.isSuccessful)error("HTTP ${response.code}: $payload");return json.decodeFromString(payload)}
    }
    fun register(r:RegisterRequest)=post<RegisterRequest,Tokens>("/v1/auth/register",r)
    fun login(r:LoginRequest)=post<LoginRequest,LoginResponse>("/v1/auth/login",r)
    fun verifyDevice(challengeId:String,answer:String)=post<VerifyDeviceRequest,Tokens>("/v1/auth/device/verify",VerifyDeviceRequest(challengeId,answer))
    fun bootstrap(token:String)=post<BootstrapRequest,Bootstrap>("/v1/realtime/bootstrap",BootstrapRequest(token))
    fun history(token:String,conversationId:String)=post<HistoryRequest,List<ChatMessage>>("/v1/chat/messages/history",HistoryRequest(conversationId),token)
    fun send(token:String,conversationId:String,recipientId:String,text:String)=post<SendMessageRequest,SendMessageResponse>("/v1/chat/messages/send",SendMessageRequest(conversationId,recipientId,java.util.UUID.randomUUID().toString(),text),token)
}
