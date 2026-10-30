package vn.tektalk.data

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.encodeToJsonElement
import vn.tektalk.NativeCore
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
@Serializable private data class DeviceLinkProofDto(val link_id:String,val secret:String)
@Serializable private data class StatusDto(val status:String)
@Serializable private data class SendDto(val conversation_id:String,val recipient_id:String,val client_message_id:String,val text:String)
@Serializable private data class HistoryDto(val conversation_id:String,val before_message_id:Long?=null,val limit:Long=50)
@Serializable private data class MessageDto(val id:Long,val conversation_id:String,val sender_id:String,val recipient_id:String,val client_message_id:String,val body:String,val created_at:String)
@Serializable private data class SendResponseDto(val message:MessageDto,val deduplicated:Boolean)
@Serializable private data class CoreBundleDto(val user_id:String,val device_id:String,val x25519_public:List<Int>,val ml_kem_768_public:List<Int>,val suites:List<String>)
@Serializable private data class RegisterSecretBundleDto(val suite:String,val x25519_public:List<Int>,val ml_kem_768_public:List<Int>)
@Serializable private data class EnableSecretDto(val conversation_id:String,val kind:String,val member_ids:List<String>)
@Serializable private data class ConversationDto(val conversation_id:String)
@Serializable private data class SecretPolicyDto(val conversation_id:String,val kind:String,val suite:String,val epoch:Long)
@Serializable private data class ServerBundleDto(val user_id:String,val device_id:String,val suite:String,val x25519_public:List<Int>,val ml_kem_768_public:List<Int>)
@Serializable private data class CoreWrappedKeyDto(val recipient_device_id:String,val envelope:JsonElement,val nonce:List<Int>,val ciphertext:List<Int>,val epoch:Long,val generation:Long=1)
@Serializable private data class PublishPacketDto(val recipient_device_id:String,val packet:JsonElement)
@Serializable private data class PublishKeysDto(val conversation_id:String,val epoch:Long,val packets:List<PublishPacketDto>)
@Serializable private data class KeyPacketDto(val conversation_id:String,val epoch:Long,val sender_device_id:String,val recipient_device_id:String,val packet:JsonElement,val created_at:String)
@Serializable private data class CoreEncryptedDto(val conversation_id:String,val epoch:Long,val sender_device_id:String,val counter:Long,val nonce:List<Int>,val ciphertext:List<Int>,val generation:Long=1)
@Serializable private data class SendSecretDto(val conversation_id:String,val client_message_id:String,val epoch:Long,val counter:Long,val nonce:List<Int>,val ciphertext:List<Int>,val generation:Long=1)
@Serializable private data class SecretMessageDto(val id:Long,val conversation_id:String,val epoch:Long,val sender_id:String,val sender_device_id:String,val client_message_id:String,val counter:Long,val nonce:List<Int>,val ciphertext:List<Int>,val created_at:String,val generation:Long=1)
@Serializable private data class SendSecretResponseDto(val message:SecretMessageDto,val deduplicated:Boolean)
@Serializable private data class CachedSecretDto(val wire:SecretMessageDto,val body:String)
@Serializable private data class StoredSecretDto(val core:String,val messages:List<CachedSecretDto>)

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

class HttpDeviceLinkRepository internal constructor(private val api:HttpApi=HttpApi()) {
    fun approve(accessToken:String,linkId:String,secret:String) { api.post<DeviceLinkProofDto,StatusDto>("/v1/auth/device-link/approve",DeviceLinkProofDto(linkId,secret),accessToken) }
}

class HttpChatRepository internal constructor(private val api:HttpApi=HttpApi()):ChatRepository {
    override fun history(accessToken:String,conversationId:String)=api.post<HistoryDto,List<MessageDto>>("/v1/chat/messages/history",HistoryDto(conversationId),accessToken).map{it.domain()}
    override fun send(accessToken:String,draft:ChatDraft)=api.post<SendDto,SendResponseDto>("/v1/chat/messages/send",SendDto(draft.conversationId,draft.recipientId,UUID.randomUUID().toString(),draft.text),accessToken).message.domain()
}

class HttpSecretChatRepository internal constructor(private val api:HttpApi=HttpApi(),private val storage:SecretStateStorage?=null):SecretChatRepository {
    // Level 3 generation is authenticated wire metadata. Generation 1 equals
    // the Kotlin default, but it must still be present in published packets.
    private val json=Json{ignoreUnknownKeys=true;encodeDefaults=true}
    private var core:NativeCore.SecretDevice?=null
    private val policies=mutableMapOf<String,SecretPolicyDto>()
    private val decrypted=mutableMapOf<Long,Pair<SecretMessageDto,Message>>()
    private var owner:String?=null

    private fun device(tokens:SessionTokens):NativeCore.SecretDevice {
        val identity=tokens.userId+":"+tokens.deviceId
        if(owner!=identity){
            close()
            try{storage?.load(identity)?.let{value->
                val stored=json.decodeFromString<StoredSecretDto>(value)
                core=checkNotNull(NativeCore.restoreSecretDevice(stored.core)){"Stored Secret Chat identity is invalid"}
                val bundle=json.decodeFromString<CoreBundleDto>(core!!.publicBundleJson)
                check(bundle.user_id==tokens.userId&&bundle.device_id==tokens.deviceId){"Secret Chat identity does not match this account"}
                stored.messages.forEach{cached->val row=cached.wire;decrypted[row.id]=row to Message(row.id,row.conversation_id,row.sender_id,row.sender_id,row.client_message_id,cached.body,row.created_at)}
            };owner=identity}catch(error:Throwable){close();throw error}
        }
        return core ?: checkNotNull(NativeCore.secretDevice(tokens.userId,tokens.deviceId)){"Rust Secret Chat core is unavailable"}.also{core=it}
    }
    private fun save(){
        val storage=storage ?: return
        val handle=core ?: return
        storage.save(checkNotNull(owner),json.encodeToString(StoredSecretDto(handle.snapshotJson,decrypted.values.map{(row,message)->CachedSecretDto(row,message.body)})))
    }
    private fun register(accessToken:String,tokens:SessionTokens){
        val bundle=json.decodeFromString<CoreBundleDto>(device(tokens).publicBundleJson)
        save()
        api.post<RegisterSecretBundleDto,ServerBundleDto>("/v1/secret/devices/register",RegisterSecretBundleDto(bundle.suites.single(),bundle.x25519_public,bundle.ml_kem_768_public),accessToken)
    }
    @Synchronized override fun registerDevice(accessToken:String,tokens:SessionTokens){register(accessToken,tokens)}
    @Synchronized override fun enable(accessToken:String,tokens:SessionTokens,conversationId:String,kind:String,memberIds:List<String>){
        register(accessToken,tokens)
        val policy=api.post<EnableSecretDto,SecretPolicyDto>("/v1/secret/conversations/enable",EnableSecretDto(conversationId,kind,memberIds),accessToken)
        publish(accessToken,tokens,conversationId,policy)
    }
    private fun publish(accessToken:String,tokens:SessionTokens,conversationId:String,policy:SecretPolicyDto){
        val serverBundles=api.post<ConversationDto,List<ServerBundleDto>>("/v1/secret/conversations/bundles",ConversationDto(conversationId),accessToken)
        check(serverBundles.any{it.user_id!=tokens.userId}){"Every peer must register a Secret Chat device before opt-in"}
        val coreBundles=serverBundles.map{CoreBundleDto(it.user_id,it.device_id,it.x25519_public,it.ml_kem_768_public,listOf(it.suite))}
        val wrappedJson=device(tokens).beginEpoch(conversationId,policy.kind,policy.epoch,json.encodeToString(coreBundles))
        save()
        val wrapped=json.decodeFromString<List<CoreWrappedKeyDto>>(wrappedJson)
        api.post<PublishKeysDto,JsonElement>("/v1/secret/keys/publish",PublishKeysDto(conversationId,policy.epoch,wrapped.map{PublishPacketDto(it.recipient_device_id,json.encodeToJsonElement(it))}),accessToken)
        policies[conversationId]=policy
    }
    private fun prepare(accessToken:String,tokens:SessionTokens,conversationId:String):SecretPolicyDto{
        register(accessToken,tokens)
        val policy=api.post<ConversationDto,SecretPolicyDto>("/v1/secret/conversations/policy",ConversationDto(conversationId),accessToken)
        val inbox=api.post<ConversationDto,List<KeyPacketDto>>("/v1/secret/keys/inbox",ConversationDto(conversationId),accessToken)
        inbox.forEach{check(device(tokens).acceptKey(conversationId,it.sender_device_id,policy.kind,it.epoch,json.encodeToString(it.packet))){"Secret Chat key authentication failed"}}
        publish(accessToken,tokens,conversationId,policy)
        policies[conversationId]=policy
        return policy
    }
    @Synchronized override fun history(accessToken:String,tokens:SessionTokens,conversationId:String):List<Message>{
        prepare(accessToken,tokens,conversationId)
        return api.post<ConversationDto,List<SecretMessageDto>>("/v1/secret/messages/history",ConversationDto(conversationId),accessToken).map{row->
            decrypted[row.id]?.let{(wire,message)->check(wire==row){"Secret Chat history changed"};return@map message}
            val encrypted=CoreEncryptedDto(row.conversation_id,row.epoch,row.sender_device_id,row.counter,row.nonce,row.ciphertext,row.generation)
            val body=device(tokens).decrypt(json.encodeToString(encrypted))
            Message(row.id,row.conversation_id,row.sender_id,row.sender_id,row.client_message_id,body,row.created_at).also{decrypted[row.id]=row to it;save()}
        }
    }
    @Synchronized override fun send(accessToken:String,tokens:SessionTokens,draft:ChatDraft):Message{
        val policy=prepare(accessToken,tokens,draft.conversationId)
        val encrypted=json.decodeFromString<CoreEncryptedDto>(device(tokens).encrypt(draft.conversationId,policy.epoch,draft.text))
        save()
        val result=api.post<SendSecretDto,SendSecretResponseDto>("/v1/secret/messages/send",SendSecretDto(draft.conversationId,UUID.randomUUID().toString(),encrypted.epoch,encrypted.counter,encrypted.nonce,encrypted.ciphertext,encrypted.generation),accessToken).message
        return Message(result.id,result.conversation_id,result.sender_id,result.sender_id,result.client_message_id,draft.text,result.created_at)
    }
    @Synchronized override fun close(){core?.close();core=null;policies.clear();decrypted.clear();owner=null}
}

object ClientDependencies {
    private lateinit var context:android.content.Context
    fun initialize(context:android.content.Context){this.context=context.applicationContext}
    val auth: AuthRepository by lazy { HttpAuthRepository() }
    val chat: ChatRepository by lazy { HttpChatRepository() }
    val deviceLink by lazy { HttpDeviceLinkRepository() }
    val secretChat: SecretChatRepository by lazy { HttpSecretChatRepository(storage=AndroidSecretStateStorage(context)) }
}
