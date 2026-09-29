@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.tektalk

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class MainActivity:ComponentActivity(){override fun onCreate(state:Bundle?){super.onCreate(state);setContent{TEKtalkTheme{TEKtalkApp()}}}}
private val TekBlue=Color(0xFF1769E0)
@Composable private fun TEKtalkTheme(content:@Composable ()->Unit){MaterialTheme(colorScheme=lightColorScheme(primary=TekBlue),content=content)}
@Composable fun TEKtalkApp(){var tokens by remember{mutableStateOf<Tokens?>(null)};if(tokens==null)AuthFlow{tokens=it}else HostTabs(tokens!!){tokens=null}}

private enum class AuthMode{Login,Register,Verify}
@Composable private fun AuthFlow(onAuthenticated:(Tokens)->Unit){
 var mode by remember{mutableStateOf(AuthMode.Login)};var phone by remember{mutableStateOf("+84")};var password by remember{mutableStateOf("")};var displayName by remember{mutableStateOf("")};var question by remember{mutableStateOf("Tên trường tiểu học của bạn?")};var answer by remember{mutableStateOf("")};var challenge by remember{mutableStateOf<String?>(null)};var serverQuestion by remember{mutableStateOf("")};var busy by remember{mutableStateOf(false)};var error by remember{mutableStateOf<String?>(null)};val scope=rememberCoroutineScope();val api=remember{Api()}
 Surface(Modifier.fillMaxSize()){Column(Modifier.fillMaxSize().padding(horizontal=28.dp),verticalArrangement=Arrangement.Center){
  Text("TEKtalk",style=MaterialTheme.typography.displaySmall,fontWeight=FontWeight.Bold,color=TekBlue);Text(if(mode==AuthMode.Verify)serverQuestion else if(mode==AuthMode.Register)"Tạo tài khoản Core Platform" else "Kết nối an toàn, trò chuyện tức thời",color=MaterialTheme.colorScheme.onSurfaceVariant);Spacer(Modifier.height(28.dp))
  if(mode!=AuthMode.Verify){OutlinedTextField(phone,{phone=it},Modifier.fillMaxWidth(),label={Text("Số điện thoại E.164")},singleLine=true);Spacer(Modifier.height(10.dp));if(mode==AuthMode.Register){OutlinedTextField(displayName,{displayName=it},Modifier.fillMaxWidth(),label={Text("Tên hiển thị")},singleLine=true);Spacer(Modifier.height(10.dp))};OutlinedTextField(password,{password=it},Modifier.fillMaxWidth(),label={Text("Mật khẩu")},visualTransformation=PasswordVisualTransformation(),singleLine=true);if(mode==AuthMode.Register){Spacer(Modifier.height(10.dp));OutlinedTextField(question,{question=it},Modifier.fillMaxWidth(),label={Text("Câu hỏi bảo mật")});Spacer(Modifier.height(10.dp));OutlinedTextField(answer,{answer=it},Modifier.fillMaxWidth(),label={Text("Câu trả lời")})}}else OutlinedTextField(answer,{answer=it},Modifier.fillMaxWidth(),label={Text("Câu trả lời bảo mật")},singleLine=true)
  error?.let{Text(it,color=MaterialTheme.colorScheme.error,modifier=Modifier.padding(top=10.dp))};Spacer(Modifier.height(18.dp));Button(enabled=!busy,onClick={scope.launch{busy=true;error=null;runCatching{withContext(Dispatchers.IO){when(mode){AuthMode.Login->api.login(LoginRequest(phone,password,null,android.os.Build.MODEL)).let{if(it.tokens!=null)it.tokens else{challenge=it.challenge_id;serverQuestion=it.question.orEmpty();mode=AuthMode.Verify;null}};AuthMode.Register->api.register(RegisterRequest(phone,displayName,password,question,answer,android.os.Build.MODEL));AuthMode.Verify->api.verifyDevice(challenge!!,answer)}}}.onSuccess{if(it!=null)onAuthenticated(it)}.onFailure{error=it.message};busy=false}},modifier=Modifier.fillMaxWidth()){Text(if(busy)"Đang xử lý…" else when(mode){AuthMode.Login->"Đăng nhập";AuthMode.Register->"Đăng ký";AuthMode.Verify->"Xác minh thiết bị"})}
  if(mode!=AuthMode.Verify)TextButton(onClick={mode=if(mode==AuthMode.Login)AuthMode.Register else AuthMode.Login;error=null}){Text(if(mode==AuthMode.Login)"Chưa có tài khoản? Đăng ký" else "Đã có tài khoản? Đăng nhập")}
 }}
}

private data class HostTab(val title:String,val mark:String)
@Composable private fun HostTabs(tokens:Tokens,onLogout:()->Unit){val tabs=listOf(HostTab("Message","M"),HostTab("AI","AI"),HostTab("Me","Me"));var selected by remember{mutableIntStateOf(0)};Scaffold(bottomBar={NavigationBar{tabs.forEachIndexed{i,t->NavigationBarItem(selected=selected==i,onClick={selected=i},icon={Text(t.mark,fontWeight=FontWeight.Bold)},label={Text(t.title)})}}}){p->Box(Modifier.padding(p)){when(selected){0->MessagesHome(tokens);1->EmptyFeature("AI","Trợ lý AI sẽ được phân phối như một plugin đã ký.");else->Profile(tokens,onLogout)}}}}
@Composable private fun MessagesHome(tokens:Tokens){var open by remember{mutableStateOf(false)};if(open){ChatRoom(tokens){open=false};return};Column(Modifier.fillMaxSize().padding(20.dp)){Row(verticalAlignment=Alignment.CenterVertically){Column(Modifier.weight(1f)){Text("Tin nhắn",style=MaterialTheme.typography.headlineMedium,fontWeight=FontWeight.Bold);Text("OTT Platform",color=MaterialTheme.colorScheme.onSurfaceVariant)};FilledIconButton(onClick={open=true}){Text("+")}};Spacer(Modifier.height(24.dp));Card(onClick={open=true},modifier=Modifier.fillMaxWidth()){Row(Modifier.padding(18.dp),verticalAlignment=Alignment.CenterVertically){Surface(shape=RoundedCornerShape(18.dp),color=TekBlue){Text("T",Modifier.padding(14.dp),color=Color.White,fontWeight=FontWeight.Bold)};Spacer(Modifier.width(14.dp));Column{Text("Cuộc trò chuyện trực tiếp",fontWeight=FontWeight.SemiBold);Text("Nhập conversation ID và recipient ID",color=MaterialTheme.colorScheme.onSurfaceVariant)}}}}}

@Composable private fun ChatRoom(tokens:Tokens,onBack:()->Unit){var conversation by remember{mutableStateOf("")};var recipient by remember{mutableStateOf("")};var draft by remember{mutableStateOf("")};var messages by remember{mutableStateOf<List<ChatMessage>>(emptyList())};var status by remember{mutableStateOf("Nhập hai UUID để bắt đầu")};val api=remember{Api()};val scope=rememberCoroutineScope();fun load(){scope.launch{runCatching{withContext(Dispatchers.IO){api.history(tokens.access_token,conversation)}}.onSuccess{messages=it;status="Đã đồng bộ ${it.size} tin"}.onFailure{status=it.message.orEmpty()}}}
 Column(Modifier.fillMaxSize()){TopAppBar(title={Text("TEKtalk Chat")},navigationIcon={TextButton(onClick=onBack){Text("‹")}},actions={TextButton(onClick={status="Call cần signaling/SFU production"}){Text("Call")}});Column(Modifier.padding(horizontal=14.dp)){OutlinedTextField(conversation,{conversation=it},Modifier.fillMaxWidth(),label={Text("Conversation UUID")},singleLine=true);OutlinedTextField(recipient,{recipient=it},Modifier.fillMaxWidth(),label={Text("Recipient UUID")},singleLine=true);TextButton(onClick={load()}){Text("Đồng bộ lịch sử")};Text(status,style=MaterialTheme.typography.bodySmall,color=MaterialTheme.colorScheme.onSurfaceVariant)};LazyColumn(Modifier.weight(1f).padding(horizontal=12.dp)){items(messages,key={it.id}){m->val mine=m.sender_id==tokens.user_id;Row(Modifier.fillMaxWidth(),horizontalArrangement=if(mine)Arrangement.End else Arrangement.Start){Surface(color=if(mine)TekBlue else Color(0xFFE9EEF6),shape=RoundedCornerShape(18.dp),modifier=Modifier.widthIn(max=290.dp).padding(vertical=3.dp)){Text(m.body,Modifier.padding(12.dp),color=if(mine)Color.White else Color.Black)}}}};Row(Modifier.padding(12.dp),verticalAlignment=Alignment.CenterVertically){OutlinedTextField(draft,{draft=it},Modifier.weight(1f),placeholder={Text("Tin nhắn")});Spacer(Modifier.width(8.dp));Button(enabled=draft.isNotBlank()&&conversation.isNotBlank()&&recipient.isNotBlank(),onClick={val sent=draft;draft="";scope.launch{runCatching{withContext(Dispatchers.IO){api.send(tokens.access_token,conversation,recipient,sent)}}.onSuccess{messages=messages+it.message}.onFailure{status=it.message.orEmpty()}}}){Text("Gửi")}}}
}
@Composable private fun EmptyFeature(title:String,subtitle:String){Box(Modifier.fillMaxSize(),contentAlignment=Alignment.Center){Column(horizontalAlignment=Alignment.CenterHorizontally){Text(title,style=MaterialTheme.typography.headlineMedium,fontWeight=FontWeight.Bold);Text(subtitle,Modifier.padding(24.dp),color=MaterialTheme.colorScheme.onSurfaceVariant)}}}
@Composable private fun Profile(tokens:Tokens,onLogout:()->Unit){Column(Modifier.fillMaxSize().padding(24.dp)){Text("Tôi",style=MaterialTheme.typography.headlineMedium,fontWeight=FontWeight.Bold);Spacer(Modifier.height(24.dp));Text("User ID");Text(tokens.user_id,color=MaterialTheme.colorScheme.onSurfaceVariant);Spacer(Modifier.height(12.dp));Text("Device ID");Text(tokens.device_id,color=MaterialTheme.colorScheme.onSurfaceVariant);Spacer(Modifier.height(28.dp));OutlinedButton(onClick=onLogout){Text("Đăng xuất")}}}
