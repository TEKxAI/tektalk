import { translations } from './locales.js';

const $ = selector => document.querySelector(selector);
const $$ = selector => [...document.querySelectorAll(selector)];
const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const state = {
  locale: localStorage.getItem('tektalk.locale') === 'en' ? 'en' : 'vi',
  theme: localStorage.getItem('tektalk.theme') || 'system',
  mode: 'login', challengeId: null, challengeQuestion: '', tokens: null, busy: false, sending: false, online: false, qr: false,
};
const t = key => translations[state.locale][key] || key;
const recentConversations = new Map();
let sessionEpoch = 0;
let authController;
let qrPollTimer;
const queryApi = new URLSearchParams(location.search).get('api');
const apiBase = (queryApi || localStorage.getItem('tektalk.api') || (location.port === '5173' ? 'http://localhost:8080' : location.origin)).replace(/\/$/, '');
const systemTheme = matchMedia('(prefers-color-scheme: dark)');

function translate() {
  document.documentElement.lang = state.locale;
  $$('[data-i18n]').forEach(e => { e.textContent = t(e.dataset.i18n); });
  $$('[data-i18n-placeholder]').forEach(e => { e.placeholder = t(e.dataset.i18nPlaceholder); });
  $$('[data-i18n-aria]').forEach(e => { e.setAttribute('aria-label', t(e.dataset.i18nAria)); });
  $('#languageButton').textContent = state.locale.toUpperCase();
  renderAuthForm(); renderHistory(); setConnection(state.online);
}
function setTheme(theme) {
  state.theme = ['light', 'dark', 'system'].includes(theme) ? theme : 'system';
  document.documentElement.dataset.theme = state.theme === 'system' ? (systemTheme.matches ? 'dark' : 'light') : state.theme;
  localStorage.setItem('tektalk.theme', state.theme);
}
systemTheme.addEventListener('change', () => { if (state.theme === 'system') setTheme('system'); });
function toast(message) {
  const el = $('#toast'); el.textContent = message; el.classList.add('show');
  clearTimeout(toast.timer); toast.timer = setTimeout(() => el.classList.remove('show'), 2400);
}
function setConnection(ok) {
  state.online = ok;
  $('#connection').classList.toggle('online', ok);
  $('#connection span').textContent = t(ok ? 'backendReady' : 'connecting');
  $('#coreStatus').textContent = t(ok ? 'backendReady' : 'connecting');
}
async function health() {
  try { setConnection((await fetch(apiBase + '/healthz')).ok); } catch { setConnection(false); }
}
async function request(path, options = {}) {
  const token = state.tokens?.access_token;
  const headers = { 'content-type': 'application/json', ...(token ? {authorization: 'Bearer ' + token} : {}), ...options.headers };
  const response = await fetch(apiBase + path, {...options, headers});
  if (!response.ok) {
    if (response.status === 401 && token && token === state.tokens?.access_token) signOut(t('sessionExpired'));
    let detail = {};
    try { detail = await response.json(); } catch { /* The server may return an empty error body. */ }
    throw new Error(detail.message || detail.error || 'HTTP ' + response.status);
  }
  return response.status === 204 ? null : response.json();
}
function activateTab(name) {
  if (!state.tokens) return;
  if (!['messages', 'ai', 'me'].includes(name)) name = 'messages';
  $$('.nav-item').forEach(e => {
    e.classList.toggle('active', e.dataset.tab === name);
    e.setAttribute('aria-current', e.dataset.tab === name ? 'page' : 'false');
  });
  $$('.tab-panel').forEach(e => e.classList.toggle('active', e.id === name));
  $$('[data-sidebar]').forEach(e => e.classList.toggle('active', e.dataset.sidebar === name));
  history.replaceState(null, '', '#' + name);
}
function renderAuthForm() {
  const register = state.mode === 'register', verify = state.mode === 'verify';
  $('#authTitle').textContent = t(verify ? 'verifyDevice' : register ? 'register' : 'login');
  $('#authTitle').removeAttribute('data-i18n');
  $('#authIntro').textContent = t(verify ? 'verifyIntro' : register ? 'registerIntro' : 'authIntro');
  $('#authIntro').removeAttribute('data-i18n');
  $('#authCredentials').hidden = verify || state.qr;
  $('#authCredentials').disabled = verify || state.busy;
  for (const id of ['displayNameField', 'questionField', 'answerField']) {
    const field = $('#' + id); field.hidden = !register;
    field.querySelector('input').disabled = !register || state.busy;
    field.querySelector('input').required = register;
  }
  $('[name="password"]').autocomplete = register ? 'new-password' : 'current-password';
  $('#challengeField').hidden = !verify;
  $('[name="challenge_answer"]').disabled = !verify || state.busy;
  $('[name="challenge_answer"]').required = verify;
  $('#challengeQuestion').textContent = state.challengeQuestion || t('securityAnswer');
  $('#challengeQuestion').removeAttribute('data-i18n');
  $('#authSubmit').textContent = t(state.busy ? 'working' : verify ? 'verifyDevice' : register ? 'register' : 'login');
  $('#authSubmit').removeAttribute('data-i18n');
  $('#authSubmit').disabled = state.busy;
  $('#authSubmit').hidden = state.qr;
  $('#qrLoginButton').hidden = state.qr || register || verify;
  $('#qrLogin').hidden = !state.qr;
  $('#authForm').setAttribute('aria-busy', String(state.busy));
  $('[data-auth-mode="register"]').hidden = register || verify;
  $('[data-auth-mode="login"]').hidden = !register && !verify;
}
function cancelAuthentication() {
  sessionEpoch++; authController?.abort(); authController = null; state.busy = false;
  clearTimeout(qrPollTimer); qrPollTimer = null;
}

async function startQrLogin() {
  cancelAuthentication(); const epoch = sessionEpoch; state.qr = true; state.busy = true; renderAuthForm(); $('#authError').textContent='';
  try {
    const link = await request('/v1/auth/device-link/start', {method:'POST', body:JSON.stringify({device_name:'TEKtalk Web · '+(navigator.platform||'Browser'), platform:'web'})});
    if (epoch !== sessionEpoch) return;
    $('#qrCode').innerHTML = link.qr_svg;
    const poll = async () => {
      if (epoch !== sessionEpoch || !state.qr) return;
      try {
        const result = await request('/v1/auth/device-link/poll', {method:'POST', body:JSON.stringify({link_id:link.link_id,secret:link.secret})});
        if (result.status === 'approved') { state.qr=false; enterSession(result.tokens); return; }
      } catch (error) { if (epoch === sessionEpoch) $('#authError').textContent = error.message; return; }
      qrPollTimer=setTimeout(poll,1500);
    }; poll();
  } catch(error) { if(epoch===sessionEpoch){state.qr=false;$('#authError').textContent=error.message;} }
  finally { if(epoch===sessionEpoch){state.busy=false;renderAuthForm();} }
}

function stopQrLogin(){ cancelAuthentication(); state.qr=false; $('#qrCode').replaceChildren(); renderAuthForm(); }
function setAuthMode(mode) {
  cancelAuthentication();
  state.qr = false;
  state.mode = mode === 'register' ? 'register' : 'login';
  state.challengeId = null; state.challengeQuestion = '';
  $('[name="password"]').value = ''; $('[name="challenge_answer"]').value = '';
  $('#authError').textContent = ''; renderAuthForm();
}
function validTokens(tokens) {
  return !!(tokens && typeof tokens.access_token === 'string' && tokens.access_token && uuid.test(tokens.user_id) && uuid.test(tokens.device_id));
}
function renderSession() {
  const signedIn = !!state.tokens;
  // Visibility and keyboard access change together; no modal overlay or demo bypass.
  $('#appShell').hidden = !signedIn; $('#appShell').inert = !signedIn;
  $('#authDialog').hidden = signedIn;
  $(signedIn ? '#sessionPreferences' : '#authPreferences').append($('#preferences'));
  $('#profileId').textContent = state.tokens?.user_id || '';
  $('#sidebarUserId').textContent = state.tokens?.user_id || '';
  $('#deviceId').textContent = state.tokens?.device_id || '';
  renderHistory(); updateSendButton();
}
function enterSession(tokens) {
  if (!validTokens(tokens)) throw new Error(t('invalidSession'));
  cancelAuthentication(); state.tokens = tokens;
  sessionStorage.setItem('tektalk.tokens', JSON.stringify(tokens));
  $('#authForm').reset(); state.mode = 'login'; state.challengeId = null;
  renderAuthForm(); renderSession(); activateTab('messages');
  $('#conversationId').value = localStorage.getItem('tektalk.conversation') || '';
  $('#recipientId').value = localStorage.getItem('tektalk.recipient') || '';
}
async function authenticate(event) {
  event.preventDefault();
  if (state.busy) return;
  const form = new FormData(event.currentTarget), epoch = sessionEpoch;
  state.busy = true; renderAuthForm(); $('#authError').textContent = '';
  authController = new AbortController();
  try {
    const options = {method: 'POST', signal: authController.signal};
    if (state.challengeId) {
      const tokens = await request('/v1/auth/device/verify', {...options, body: JSON.stringify({challenge_id: state.challengeId, answer: form.get('challenge_answer')})});
      if (epoch === sessionEpoch) enterSession(tokens);
      return;
    }
    const common = {phone: form.get('phone').trim(), password: form.get('password'), device_name: 'TEKtalk Web · ' + (navigator.platform || 'Browser')};
    if (state.mode === 'register') {
      const tokens = await request('/v1/auth/register', {...options, body: JSON.stringify({...common, display_name: form.get('display_name').trim(), security_question: form.get('security_question').trim(), security_answer: form.get('security_answer')})});
      if (epoch === sessionEpoch) enterSession(tokens);
    } else {
      const result = await request('/v1/auth/login', {...options, body: JSON.stringify(common)});
      if (epoch !== sessionEpoch) return;
      if (result.status === 'authenticated') enterSession(result.tokens);
      else if (result.status === 'device_verification_required' && result.challenge_id) {
        state.mode = 'verify'; state.challengeId = result.challenge_id; state.challengeQuestion = result.question;
        $('[name="password"]').value = ''; $('[name="challenge_answer"]').value = '';
      } else throw new Error(t('invalidSession'));
    }
  } catch (error) {
    if (epoch === sessionEpoch && error.name !== 'AbortError') $('#authError').textContent = error.message;
  } finally {
    if (epoch === sessionEpoch) {
      state.busy = false; authController = null; renderAuthForm();
      if (state.mode === 'verify') $('[name="challenge_answer"]').focus();
    }
  }
}
function signOut(message = '') {
  cancelAuthentication(); state.tokens = null; state.sending = false;
  recentConversations.clear();
  sessionStorage.removeItem('tektalk.tokens');
  localStorage.removeItem('tektalk.conversation'); localStorage.removeItem('tektalk.recipient');
  $('#authForm').reset(); $('#chatForm').reset(); $('#messageForm').reset();
  $('#messageList').replaceChildren(); $('#chatStatus').textContent = t('startChat');
  state.mode = 'login'; state.challengeId = null; state.challengeQuestion = '';
  renderAuthForm(); renderSession(); $('#authError').textContent = message;
  history.replaceState(null, '', '#login'); $('#phone').focus();
}
function appendMessage(text, kind) {
  const item = document.createElement('div'); item.className = 'message ' + kind;
  const p = document.createElement('p'); p.textContent = text; item.append(p);
  $('#messageList').append(item); item.scrollIntoView({block: 'nearest'});
}
function rememberConversation(preview = '') {
  const id = localStorage.getItem('tektalk.conversation');
  if (!state.tokens || !id) return;
  recentConversations.delete(id);
  recentConversations.set(id, {id, recipient: localStorage.getItem('tektalk.recipient'), preview});
  renderHistory();
}
function renderHistory() {
  const list = $('.conversation-list'); list.replaceChildren();
  if (!state.tokens) return;
  if (!recentConversations.size) {
    const hint = document.createElement('p'); hint.className = 'empty-history'; hint.textContent = t('startChat'); list.append(hint);
  }
  for (const entry of [...recentConversations.values()].reverse()) {
    const button = document.createElement('button'); button.type = 'button';
    button.className = 'conversation' + (entry.id === localStorage.getItem('tektalk.conversation') ? ' active' : '');
    button.dataset.conversation = entry.id;
    const title = document.createElement('strong'); title.textContent = t('directConversation');
    const preview = document.createElement('span'); preview.textContent = entry.preview || entry.id;
    button.append(title, preview);
    button.addEventListener('click', () => {
      $('#conversationId').value = entry.id; $('#recipientId').value = entry.recipient;
      selectConversation(); activateTab('messages'); syncMessages().catch(showChatError);
    });
    list.append(button);
  }
}
function selectConversation() {
  const id = $('#conversationId').value.trim(), recipient = $('#recipientId').value.trim();
  if (!uuid.test(id) || !uuid.test(recipient)) throw new Error(t('invalidConversation'));
  if (id !== localStorage.getItem('tektalk.conversation')) $('#messageList').replaceChildren();
  localStorage.setItem('tektalk.conversation', id); localStorage.setItem('tektalk.recipient', recipient);
  renderHistory(); return {id, recipient};
}
function showChatError(error) {
  if (state.tokens) $('#chatStatus').textContent = error.message;
}
async function syncMessages() {
  if (!state.tokens) return;
  const {id} = selectConversation(), epoch = sessionEpoch;
  const rows = await request('/v1/chat/messages/history', {method: 'POST', body: JSON.stringify({conversation_id: id, limit: 50})});
  if (epoch !== sessionEpoch || id !== localStorage.getItem('tektalk.conversation')) return;
  $('#messageList').replaceChildren();
  for (const row of rows) appendMessage(row.body, row.sender_id === state.tokens.user_id ? 'outgoing' : 'incoming');
  $('#chatStatus').textContent = t('syncedMessages') + ' ' + rows.length;
  rememberConversation(rows.at(-1)?.body || '');
}
function updateSendButton() { $('#sendButton').disabled = !state.tokens || state.sending || !$('#messageInput').value.trim(); }
async function sendMessage(event) {
  event.preventDefault();
  if (!state.tokens || state.sending) return;
  const input = $('#messageInput'), text = input.value.trim();
  if (!text) return;
  let selection;
  try { selection = selectConversation(); } catch (error) { showChatError(error); $('#conversationId').focus(); return; }
  const {id, recipient} = selection, epoch = sessionEpoch;
  state.sending = true; updateSendButton();
  try {
    const result = await request('/v1/chat/messages/send', {method: 'POST', body: JSON.stringify({conversation_id: id, recipient_id: recipient, client_message_id: crypto.randomUUID(), text})});
    if (epoch !== sessionEpoch || id !== localStorage.getItem('tektalk.conversation')) return;
    appendMessage(result.message.body, 'outgoing');
    if (input.value.trim() === text) input.value = '';
    rememberConversation(result.message.body);
  } catch (error) { if (epoch === sessionEpoch) showChatError(error); }
  finally { if (epoch === sessionEpoch) { state.sending = false; updateSendButton(); } }
}
async function restoreSession() {
  let cached;
  try { cached = JSON.parse(sessionStorage.getItem('tektalk.tokens') || 'null'); } catch { /* Invalid storage must keep auth visible. */ }
  sessionStorage.removeItem('tektalk.demo');
  if (!validTokens(cached)) { sessionStorage.removeItem('tektalk.tokens'); return; }
  const epoch = sessionEpoch, destination = location.hash.slice(1);
  state.busy = true; renderAuthForm();
  const controller = new AbortController(); authController = controller;
  const timeout = setTimeout(() => controller.abort(), 10000);
  try {
    // This authenticated read verifies the cached access token before exposing the host.
    const stored = localStorage.getItem('tektalk.conversation');
    await request('/v1/chat/messages/history', {method: 'POST', headers: {authorization: 'Bearer ' + cached.access_token}, signal: controller.signal, body: JSON.stringify({conversation_id: uuid.test(stored) ? stored : crypto.randomUUID(), limit: 1})});
    if (epoch === sessionEpoch) { enterSession(cached); activateTab(destination); }
  } catch {
    if (epoch === sessionEpoch) signOut(t('sessionExpired'));
  } finally {
    clearTimeout(timeout);
    if (epoch === sessionEpoch) { state.busy = false; authController = null; renderAuthForm(); }
  }
}

$('#signOutButton').addEventListener('click', () => signOut());
$('#themeButton').addEventListener('click', () => setTheme({system: 'light', light: 'dark', dark: 'system'}[state.theme]));
$('#languageButton').addEventListener('click', () => { state.locale = state.locale === 'vi' ? 'en' : 'vi'; localStorage.setItem('tektalk.locale', state.locale); translate(); });
$$('[data-auth-mode]').forEach(button => button.addEventListener('click', () => setAuthMode(button.dataset.authMode)));
$('#authForm').addEventListener('submit', authenticate);
$('#qrLoginButton').addEventListener('click', startQrLogin);
$('#cancelQrButton').addEventListener('click', stopQrLogin);
$('#chatForm').addEventListener('submit', event => { event.preventDefault(); syncMessages().catch(showChatError); });
$$('.nav-item').forEach(button => button.addEventListener('click', () => activateTab(button.dataset.tab)));
$('#messageForm').addEventListener('submit', sendMessage);
$('#messageInput').addEventListener('input', updateSendButton);
$('#messageInput').addEventListener('keydown', event => {
  if (event.key === 'Enter' && !event.shiftKey) { event.preventDefault(); $('#messageForm').requestSubmit(); }
});
window.addEventListener('hashchange', () => activateTab(location.hash.slice(1)));
setTheme(state.theme); translate(); renderSession(); health(); restoreSession();
