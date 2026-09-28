package vn.tektalk
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.*

class MainActivity:ComponentActivity(){override fun onCreate(state:Bundle?){super.onCreate(state);setContent{MaterialTheme{TEKtalkHost()}}}}

@Composable fun TEKtalkHost(){var authenticated by remember{mutableStateOf(false)};if(authenticated)HostTabs()else AuthScreen{authenticated=true}}

@Composable fun AuthScreen(onAuthenticated:()->Unit){var phone by remember{mutableStateOf("+84")};var password by remember{mutableStateOf("")};var status by remember{mutableStateOf("Đăng nhập bằng mật khẩu")};val scope=rememberCoroutineScope();Column(Modifier.padding(24.dp).fillMaxWidth(),verticalArrangement=Arrangement.spacedBy(12.dp)){Text("TEKtalk",style=MaterialTheme.typography.headlineLarge);OutlinedTextField(phone,{phone=it},label={Text("Số điện thoại")});OutlinedTextField(password,{password=it},label={Text("Mật khẩu")});Button(onClick={scope.launch{runCatching{withContext(Dispatchers.IO){Api().login(LoginRequest(phone,password,null,android.os.Build.MODEL))}}.onSuccess{if(it.status=="authenticated")onAuthenticated()else status="Thiết bị lạ: ${it.question}"}.onFailure{status=it.message?:"Lỗi"}}}){Text("Đăng nhập")};Text(status)}}

private data class HostTab(val id:String,val title:String,val icon:String)
private val bundledTabs=listOf(HostTab("tektalk.message","Message","message"),HostTab("tektalk.ai","AI","sparkles"),HostTab("tektalk.me","Me","person"))

@Composable fun HostTabs(){var selected by remember{mutableIntStateOf(0)};Scaffold(bottomBar={NavigationBar{bundledTabs.forEachIndexed{i,tab->NavigationBarItem(selected=selected==i,onClick={selected=i},icon={Text(tab.icon.take(1).uppercase())},label={Text(tab.title)})}}}){padding->Box(Modifier.padding(padding).fillMaxSize()){PluginMount(bundledTabs[selected])}}}

@Composable private fun PluginMount(tab:HostTab){Column(Modifier.padding(24.dp),verticalArrangement=Arrangement.spacedBy(8.dp)){Text(tab.title,style=MaterialTheme.typography.headlineMedium);Text("Valdi plugin mount: ${tab.id}")}}
