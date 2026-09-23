package vn.tektalk
import kotlinx.serialization.*
import kotlinx.serialization.json.Json
import okhttp3.*
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody

@Serializable data class RegisterRequest(val phone:String,val display_name:String,val password:String,val security_question:String,val security_answer:String,val device_name:String)
@Serializable data class LoginRequest(val phone:String,val password:String,val device_id:String?=null,val device_name:String)
@Serializable data class Tokens(val access_token:String,val refresh_token:String,val user_id:String,val device_id:String)
@Serializable data class LoginResponse(val status:String,val tokens:Tokens?=null,val challenge_id:String?=null,val question:String?=null)
@Serializable data class BootstrapRequest(val access_token:String)
@Serializable data class Bootstrap(val ticket:String,val server_public_key:String,val expires_in_seconds:Int,val protocol_version:Int)

class Api(private val base:String=BuildConfig.API_BASE_URL,private val http:OkHttpClient=OkHttpClient()){
 private val json=Json{ignoreUnknownKeys=true};private val media="application/json".toMediaType()
 private inline fun <reified I,reified O> post(path:String,body:I):O{val raw=json.encodeToString(body);val req=Request.Builder().url(base+path).post(raw.toRequestBody(media)).build();http.newCall(req).execute().use{if(!it.isSuccessful)error("HTTP ${it.code}: ${it.body?.string()}");return json.decodeFromString(it.body!!.string())}}
 fun register(r:RegisterRequest)=post<RegisterRequest,Tokens>("/v1/auth/register",r)
 fun login(r:LoginRequest)=post<LoginRequest,LoginResponse>("/v1/auth/login",r)
 fun bootstrap(token:String)=post<BootstrapRequest,Bootstrap>("/v1/realtime/bootstrap",BootstrapRequest(token))
}
