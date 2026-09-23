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

class MainActivity:ComponentActivity(){override fun onCreate(state:Bundle?){super.onCreate(state);setContent{MaterialTheme{AuthScreen()}}}}
@Composable fun AuthScreen(){var phone by remember{mutableStateOf("+84")};var password by remember{mutableStateOf("")};var status by remember{mutableStateOf("Đăng nhập bằng mật khẩu")};val scope=rememberCoroutineScope();Column(Modifier.padding(24.dp).fillMaxWidth(),verticalArrangement=Arrangement.spacedBy(12.dp)){Text("TEKtalk",style=MaterialTheme.typography.headlineLarge);OutlinedTextField(phone,{phone=it},label={Text("Số điện thoại")});OutlinedTextField(password,{password=it},label={Text("Mật khẩu")});Button(onClick={scope.launch(Dispatchers.IO){runCatching{Api().login(LoginRequest(phone,password,null,android.os.Build.MODEL))}.onSuccess{status=if(it.status=="authenticated")"Đăng nhập thành công" else "Thiết bị lạ: ${it.question}"}.onFailure{status=it.message?:"Lỗi"}}}){Text("Đăng nhập")};Text(status)}}
