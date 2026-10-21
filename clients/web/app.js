import { translations } from './locales.js';
const state={locale:localStorage.getItem('tektalk.locale')||'vi',theme:localStorage.getItem('tektalk.theme')||'system',mode:'login',challengeId:null,tokens:JSON.parse(sessionStorage.getItem('tektalk.tokens')||'null'),demo:sessionStorage.getItem('tektalk.demo')==='1'};
const queryApi=new URLSearchParams(location.search).get('api');
const apiBase=(queryApi||localStorage.getItem('tektalk.api')||(location.port==='5173'?'http://localhost:8080':location.origin)).replace(/\/$/,'');
const $=selector=>document.querySelector(selector); const $$=selector=>[...document.querySelectorAll(selector)];
// Recent conversations live only in this signed-in page; logout clears previews.
const recentConversations = new Map();
function rememberConversation(preview = '') {
  const id = localStorage.getItem('tektalk.conversation');
  if (!state.tokens || !id) return;
  recentConversations.set(id, {id, recipient:localStorage.getItem('tektalk.recipient'), preview});
  renderHistory();
}
function renderHistory() {
  if (!state.tokens) return;
  const list = $('.conversation-list');
  list.replaceChildren();
  for (const entry of [...recentConversations.values()].reverse()) {
    const button = document.createElement('button');
    button.className = 'conversation' + (entry.id === localStorage.getItem('tektalk.conversation') ? ' active' : '');
    button.dataset.conversation = entry.id;
    const avatar = document.createElement('span'); avatar.className='avatar gradient'; avatar.textContent='T';
    const copy = document.createElement('span'); copy.className='conversation-copy';
    const title = document.createElement('strong'); title.textContent=translations[state.locale].directConversation;
    const preview = document.createElement('span'); preview.textContent=entry.preview || entry.id;
    copy.append(title,preview); button.append(avatar,copy);
    button.addEventListener('click',()=> {
      localStorage.setItem('tektalk.conversation',entry.id); localStorage.setItem('tektalk.recipient',entry.recipient);
      activateTab('messages'); renderHistory(); syncMessages().catch(e=>toast(e.message));
    });
    list.append(button);
  }
}
function translate(){document.documentElement.lang=state.locale;$$('[data-i18n]').forEach(e=>e.textContent=translations[state.locale][e.dataset.i18n]||e.dataset.i18n);$$('[data-i18n-placeholder]').forEach(e=>e.placeholder=translations[state.locale][e.dataset.i18nPlaceholder]||'');$('#languageButton').textContent=state.locale.toUpperCase()}
const systemTheme=matchMedia('(prefers-color-scheme: dark)');
function setTheme(theme){state.theme=['light','dark','system'].includes(theme)?theme:'system';document.documentElement.dataset.theme=state.theme==='system'?(systemTheme.matches?'dark':'light'):state.theme;localStorage.setItem('tektalk.theme',state.theme)}
systemTheme.addEventListener('change',()=>{if(state.theme==='system')setTheme('system')});
function nextTheme(){setTheme({system:'light',light:'dark',dark:'system'}[state.theme])}
function toast(message){const el=$('#toast');el.textContent=message;el.classList.add('show');clearTimeout(toast.timer);toast.timer=setTimeout(()=>el.classList.remove('show'),2400)}
function setConnection(ok){const el=$('#connection');el.classList.toggle('online',ok);el.querySelector('span').textContent=translations[state.locale][ok?'backendReady':'connecting']}
async function request(path,options={}){const headers={'content-type':'application/json',...(options.headers||{})};if(state.tokens?.access_token)headers.authorization=`Bearer ${state.tokens.access_token}`;const response=await fetch(`${apiBase}${path}`,{...options,headers});if(!response.ok){let detail;try{detail=await response.json()}catch{detail={}}throw new Error(detail.message||detail.error||`HTTP ${response.status}`)}return response.status===204?null:response.json()}
async function health(){try{const response=await fetch(`${apiBase}/healthz`);setConnection(response.ok)}catch{setConnection(false)}}
function activateTab(name){if(!['messages','ai','me'].includes(name))name='messages';$$('.nav-item').forEach(e=>{e.classList.toggle('active',e.dataset.tab===name);e.setAttribute('aria-current',e.dataset.tab===name?'page':'false')});$$('.tab-panel').forEach(e=>e.classList.toggle('active',e.id===name));$$('[data-sidebar]').forEach(e=>e.classList.toggle('active',e.dataset.sidebar===name));history.replaceState(null,'',`#${name}`)}
function setAuthMode(mode){state.mode=mode;state.challengeId=null;$$('[data-auth-mode]').forEach(e=>e.classList.toggle('active',e.dataset.authMode===mode));const register=mode==='register';$('#displayNameField').hidden=!register;$('#questionField').hidden=!register;$('#answerField').hidden=!register;$('#challengeField').hidden=true;$('#authError').textContent=''}
function storeTokens(tokens){state.tokens=tokens;state.demo=false;sessionStorage.setItem('tektalk.tokens',JSON.stringify(tokens));sessionStorage.removeItem('tektalk.demo');$('#profileId').textContent=tokens.user_id;$('#authHint').textContent=translations[state.locale].connectedBackend;$('#authDialog').close();renderSession();toast(translations[state.locale].signedIn)}
async function authenticate(event){event.preventDefault();const form=new FormData(event.currentTarget);const error=$('#authError');error.textContent='';try{if(state.challengeId){const tokens=await request('/v1/auth/device/verify',{method:'POST',body:JSON.stringify({challenge_id:state.challengeId,answer:form.get('challenge_answer')})});storeTokens(tokens);return}const common={phone:form.get('phone'),password:form.get('password'),device_name:`TEKtalk Web · ${navigator.platform||'Browser'}`};if(state.mode==='register'){const tokens=await request('/v1/auth/register',{method:'POST',body:JSON.stringify({...common,display_name:form.get('display_name'),security_question:form.get('security_question'),security_answer:form.get('security_answer')})});$('#profileName').textContent=form.get('display_name');storeTokens(tokens)}else{const result=await request('/v1/auth/login',{method:'POST',body:JSON.stringify(common)});if(result.status==='authenticated')storeTokens(result.tokens);else{state.challengeId=result.challenge_id;$('#challengeQuestion').textContent=result.question;$('#challengeField').hidden=false;$('#authSubmit').textContent=translations[state.locale].verifyDevice}}}catch(e){error.textContent=e.message}}
function enableDemo(){state.demo=true;sessionStorage.setItem('tektalk.demo','1');$('#profileName').textContent='TEKtalk Explorer';$('#profileId').textContent=translations[state.locale].demoWorkspace;$('#authDialog').close();setConnection(true);toast(translations[state.locale].demoEnabled)}
function appendMessage(text,kind='outgoing',time=new Date()){const item=document.createElement('div');item.className=`message ${kind}`;const p=document.createElement('p');p.textContent=text;const clock=document.createElement('time');clock.textContent=time.toLocaleTimeString(state.locale,{hour:'2-digit',minute:'2-digit'});item.append(p,clock);$('#messageList').append(item);item.scrollIntoView({behavior:'smooth'})}
function renderSession(){
  $('#signOutButton').hidden=!state.tokens;
  $('#chatControls').hidden=!state.tokens;
  if(state.tokens){
    $('#messageList').replaceChildren();
    document.querySelector('.chat-header strong').textContent=translations[state.locale].directConversation;
    $$('.conversation').forEach((item,index)=>{item.hidden=index>0;if(index===0){item.querySelector('strong').textContent=translations[state.locale].directConversation;item.querySelector('.conversation-copy span').textContent='';}});
    document.querySelector('.nav-item b').hidden=true;
  }
}
function signOut(){
  recentConversations.clear();
  $('.conversation-list').replaceChildren();
  $('#messageInput').value='';
  localStorage.removeItem('tektalk.conversation');localStorage.removeItem('tektalk.recipient');
  state.tokens=null;state.demo=false;state.challengeId=null;
  sessionStorage.removeItem('tektalk.tokens');sessionStorage.removeItem('tektalk.demo');
  $('#authForm').reset();setAuthMode('login');$('#authSubmit').textContent=translations[state.locale].continue;
  $('#messageList').replaceChildren();$('#profileId').textContent='';renderSession();
  $('#authDialog').showModal();toast(translations[state.locale].signedOut);
}
async function syncMessages(){
  if(!state.tokens)return;
  const conversation=localStorage.getItem('tektalk.conversation');if(!conversation)return;
  const rows=await request('/v1/chat/messages/history',{method:'POST',body:JSON.stringify({conversation_id:conversation,limit:50})});
  if(conversation!==localStorage.getItem('tektalk.conversation')||!state.tokens)return;
  $('#messageList').replaceChildren();
  for(const row of rows)appendMessage(row.body,row.sender_id===state.tokens.user_id?'outgoing':'incoming',new Date(row.created_at));
  rememberConversation(rows.at(-1)?.body || '');
  return rows;
}
async function sendMessage(event){
  event.preventDefault();const input=$('#messageInput');const text=input.value.trim();if(!text)return;
  if(state.demo){appendMessage(text);input.value='';setTimeout(()=>appendMessage(translations[state.locale].demoReply,'incoming'),450);return}
  if(!state.tokens){$('#authDialog').showModal();return}
  const recipient=localStorage.getItem('tektalk.recipient'),conversation=localStorage.getItem('tektalk.conversation');
  if(!recipient||!conversation){$('#chatDialog').showModal();return}
  const button=$('#messageForm .send');button.disabled=true;
  try{
    const result=await request('/v1/chat/messages/send',{method:'POST',body:JSON.stringify({conversation_id:conversation,recipient_id:recipient,client_message_id:crypto.randomUUID(),text})});
    if(!state.tokens || conversation!==localStorage.getItem('tektalk.conversation'))return;
    appendMessage(result.message.body,'outgoing',new Date(result.message.created_at));input.value='';
    rememberConversation(result.message.body);
  }catch(e){toast(e.message)}finally{button.disabled=false}
}
$('#signOutButton').addEventListener('click',signOut);
$('#configureChat').addEventListener('click',()=>$('#chatDialog').showModal());
$('.compose').addEventListener('click',()=>$('#chatDialog').showModal());
$('#syncHistory').addEventListener('click',()=>syncMessages().catch(e=>toast(e.message)));
$('#chatForm').addEventListener('submit',async event=>{
  event.preventDefault();const conversation=$('#conversationId').value.trim(),recipient=$('#recipientId').value.trim();
  const uuid=/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  if(!uuid.test(conversation)||!uuid.test(recipient)){toast(translations[state.locale].invalidConversation);return}
  localStorage.setItem('tektalk.conversation',conversation);localStorage.setItem('tektalk.recipient',recipient);
  $('#chatDialog').close();toast(translations[state.locale].chatReady);await syncMessages().catch(e=>toast(e.message));
});
$$('.nav-item').forEach(button=>button.addEventListener('click',()=>activateTab(button.dataset.tab)));$('#themeButton').addEventListener('click',nextTheme);$('#profileTheme').addEventListener('click',nextTheme);$('#languageButton').addEventListener('click',()=>{state.locale=state.locale==='vi'?'en':'vi';localStorage.setItem('tektalk.locale',state.locale);translate();health()});$('#authButton').addEventListener('click',()=>$('#authDialog').showModal());$$('[data-auth-mode]').forEach(button=>button.addEventListener('click',()=>setAuthMode(button.dataset.authMode)));$('#authForm').addEventListener('submit',authenticate);$('#demoButton').addEventListener('click',enableDemo);$('#messageForm').addEventListener('submit',sendMessage);$('#messageInput').addEventListener('keydown',event=>{if(event.key==='Enter'&&!event.shiftKey){event.preventDefault();$('#messageForm').requestSubmit()}});$$('[data-prompt]').forEach(button=>button.addEventListener('click',()=>{activateTab('messages');appendMessage(button.textContent);setTimeout(()=>appendMessage(translations[state.locale].aiSimulated,'incoming'),350)}));
renderSession();setTheme(state.theme);translate();activateTab(location.hash.slice(1)||'messages');health();if(!state.tokens&&!state.demo)setTimeout(()=>$('#authDialog').showModal(),500);
