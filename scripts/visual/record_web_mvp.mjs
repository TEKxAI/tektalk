import { chromium, request } from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const apiUrl='http://127.0.0.1:8080';
const api=await request.newContext({baseURL:apiUrl});
const f=JSON.parse(await fs.readFile('build/visual-fixtures/web.json','utf8'));
const browser=await chromium.launch({headless:true});
const context=await browser.newContext({viewport:{width:1440,height:960},recordVideo:{dir:'recordings/raw-web',size:{width:1440,height:960}}});
const page=await context.newPage();const video=page.video();page.setDefaultTimeout(30000);
const pause=()=>page.waitForTimeout(1400);
async function type(selector,value){const e=page.locator(selector);await e.fill('');await e.pressSequentially(value,{delay:45});}
async function submit(path){const response=page.waitForResponse(r=>r.url().endsWith(path)&&r.request().method()==='POST');await page.locator('#authSubmit').click();const r=await response;assert.ok(r.ok(),await r.text());return r.json();}
let complete=false;
try{
  await page.goto(apiUrl);await page.locator('#authDialog').waitFor({state:'visible'});await pause();
  await page.locator('[data-auth-mode="register"]').click();
  await type('[name="phone"]',f.phone);await type('[name="display_name"]','MVP Learner');await type('[name="password"]',f.password);await type('[name="security_question"]',f.question);await type('[name="security_answer"]',f.answer);await pause();
  const registered=await submit('/v1/auth/register');assert.ok(registered.user_id);await page.locator('#authDialog').waitFor({state:'hidden'});await pause();
  await page.locator('#signOutButton').click();await page.locator('#authDialog').waitFor({state:'visible'});await pause();
  await type('[name="phone"]',f.phone);await type('[name="password"]',f.password);await pause();
  const login=await submit('/v1/auth/login');assert.equal(login.status,'device_verification_required');await page.locator('#challengeField').waitFor({state:'visible'});await pause();
  await type('[name="challenge_answer"]',f.answer);const verified=await submit('/v1/auth/device/verify');assert.equal(verified.user_id,registered.user_id);await page.locator('#authDialog').waitFor({state:'hidden'});await pause();
  await page.locator('#configureChat').click();await type('#conversationId',f.conversation);await type('#recipientId',f.peer_id);await page.locator('#openConversation').click();await page.locator('#chatDialog').waitFor({state:'hidden'});await pause();
  await type('#messageInput',f.outgoing);await pause();
  const sentResponse=page.waitForResponse(r=>r.url().endsWith('/v1/chat/messages/send')&&r.request().method()==='POST');await page.locator('#messageForm .send').click();const sent=await sentResponse;assert.ok(sent.ok(),await sent.text());await page.locator('.message').getByText(f.outgoing,{exact:true}).waitFor();await pause();
  const history=await api.post('/v1/chat/messages/history',{headers:{authorization:`Bearer ${f.peer_token}`},data:{conversation_id:f.conversation,limit:50}});assert.ok(history.ok());const rows=await history.json();const outgoing=rows.find(x=>x.body===f.outgoing);assert.ok(outgoing,'Message must exist in recipient history');
  const reply=await api.post('/v1/chat/messages/send',{headers:{authorization:`Bearer ${f.peer_token}`},data:{conversation_id:f.conversation,recipient_id:outgoing.sender_id,client_message_id:crypto.randomUUID(),text:f.incoming}});assert.ok(reply.ok());
  await page.locator('#syncHistory').click();await page.locator('.message').getByText(f.incoming,{exact:true}).waitFor();await pause();
  await type('#messageInput','Draft preserved across tabs');
  for(const tab of ['ai','me','messages']){await page.locator(`[data-tab="${tab}"]`).click();await pause();}
  assert.equal(await page.locator('#messageInput').inputValue(),'Draft preserved across tabs');
  await page.locator('.message').getByText(f.incoming,{exact:true}).waitFor();
  const nav=await page.locator('.rail').boundingBox(), sidebar=await page.locator('.sidebar').boundingBox();
  assert.ok(nav && sidebar && Math.abs(nav.y+nav.height-sidebar.y-sidebar.height)<1);
  await page.reload();await page.locator('#syncHistory').click();await page.locator('.message').getByText(f.outgoing,{exact:true}).waitFor();await page.locator('.message').getByText(f.incoming,{exact:true}).waitFor();await pause();
  await fs.writeFile('recordings/web-backend.json',JSON.stringify({platform:'web',registration:true,logout:true,login:true,device_verification:true,outgoing_persisted:true,reply_visible_after_reload:true},null,2));complete=true;
}finally{
  await context.close();await video.saveAs('recordings/web.webm');await api.dispose();await browser.close();
  if(!complete)console.error('Recording retained for diagnostics; flow is incomplete.');
}
