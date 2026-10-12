import { chromium, request } from 'playwright';

const appUrl = 'http://127.0.0.1:5173/?api=http://127.0.0.1:8080';
const apiUrl = 'http://127.0.0.1:8080';
const suffix = String(Date.now()).slice(-8);
const phone = `+849${suffix}`;
const peerPhone = `+848${suffix}`;
const password = 'Learning1234';
const answer = 'hanoi';
const conversation = crypto.randomUUID();

const browser = await chromium.launch({ headless: true });
const api = await request.newContext({ baseURL: apiUrl });

async function newPage() {
  const context = await browser.newContext({
    viewport: { width: 1440, height: 960 },
    deviceScaleFactor: 1,
  });
  const page = await context.newPage();
  await page.goto(appUrl, { waitUntil: 'networkidle' });
  await page.locator('#authDialog').waitFor({ state: 'visible' });
  return { context, page };
}

const registration = await newPage();
await registration.page.locator('[data-auth-mode="register"]').click();
await registration.page.locator('[name="phone"]').fill(phone);
await registration.page.locator('[name="display_name"]').fill('MVP Learner');
await registration.page.locator('[name="password"]').fill(password);
await registration.page.locator('[name="security_question"]').fill('Thành phố yêu thích?');
await registration.page.locator('[name="security_answer"]').fill(answer);
await registration.page.screenshot({ path: 'screenshots/web-01-register.png', fullPage: true });
await registration.page.locator('#authSubmit').click();
await registration.page.locator('#authDialog').waitFor({ state: 'hidden' });
await registration.page.screenshot({ path: 'screenshots/web-02-registered.png', fullPage: true });
await registration.context.close();

const peerResponse = await api.post('/v1/auth/register', {
  data: {
    phone: peerPhone,
    display_name: 'MVP Peer',
    password,
    security_question: 'Thành phố yêu thích?',
    security_answer: answer,
    device_name: 'visual-e2e-peer',
  },
});
if (!peerResponse.ok()) throw new Error(`Peer registration failed: ${peerResponse.status()}`);
const peer = await peerResponse.json();

const login = await newPage();
await login.page.locator('[name="phone"]').fill(phone);
await login.page.locator('[name="password"]').fill(password);
await login.page.screenshot({ path: 'screenshots/web-03-login.png', fullPage: true });
await login.page.locator('#authSubmit').click();
await login.page.locator('#challengeField').waitFor({ state: 'visible' });
await login.page.screenshot({ path: 'screenshots/web-04-device-verification.png', fullPage: true });
await login.page.locator('[name="challenge_answer"]').fill(answer);
await login.page.locator('#authSubmit').click();
await login.page.locator('#authDialog').waitFor({ state: 'hidden' });
await login.page.evaluate(({ recipient, conversationId }) => {
  localStorage.setItem('tektalk.recipient', recipient);
  localStorage.setItem('tektalk.conversation', conversationId);
}, { recipient: peer.user_id, conversationId: conversation });
await login.page.locator('#messageInput').fill('Xin chào từ luồng MVP TEKtalk!');
await login.page.locator('#messageForm').evaluate(form => form.requestSubmit());
await login.page.getByText('Xin chào từ luồng MVP TEKtalk!').waitFor();
await login.page.waitForTimeout(500);
await login.page.screenshot({ path: 'screenshots/web-05-chat.png', fullPage: true });

await login.context.close();
await api.dispose();
await browser.close();
