import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { chromium } from 'playwright';

const baseURL = process.env.TEKTALK_WEB_URL || 'http://127.0.0.1:8080';
const browser = await chromium.launch({ headless: true });
const tokens = { access_token: 'controlled-test-token', user_id: '11111111-1111-4111-8111-111111111111', device_id: '22222222-2222-4222-8222-222222222222' };
const errors = [];
const json = (route, value, status = 200) => route.fulfill({ status, contentType: 'application/json', body: JSON.stringify(value) });
async function signedOut(page) {
  assert.equal(await page.locator('#authDialog').isVisible(), true);
  assert.equal(await page.locator('#appShell').isVisible(), false);
  assert.equal(await page.locator('#appShell').getAttribute('inert'), '');
  assert.deepEqual(await page.evaluate(() => window.__hostLeaks || []), []);
}
async function credentials(page) {
  await page.locator('[name="phone"]').fill('+84912345678');
  await page.locator('[name="password"]').fill('AuthRegression123');
}
async function contextFor(seed = {}) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
  await context.addInitScript(seed => {
    localStorage.setItem('tektalk.locale', 'en');
    for (const [key, value] of Object.entries(seed)) sessionStorage.setItem(key, value);
    window.__hostLeaks = [];
    new MutationObserver(() => {
      const shell = document.querySelector('#appShell');
      if (shell && !shell.hidden && !window.__allowHost) window.__hostLeaks.push(location.hash);
    }).observe(document, { subtree: true, childList: true, attributes: true, attributeFilter: ['hidden'] });
  }, seed);
  const page = await context.newPage();
  page.on('pageerror', error => errors.push(error.message));
  return { context, page };
}
try {
  await fs.mkdir('recordings/design-system', { recursive: true });
  // HTML itself must never paint a signed-in application before scripts start.
  const noJS = await browser.newContext({ javaScriptEnabled: false });
  const initial = await noJS.newPage();
  await initial.goto(baseURL + '/#me');
  assert.equal(await initial.locator('#appShell').isVisible(), false);
  assert.equal(await initial.locator('#authDialog').isVisible(), true);
  await noJS.close();

  for (const locale of ['en', 'vi']) for (const theme of ['light', 'dark']) {
    const { context, page } = await contextFor({ 'tektalk.demo': '1', 'tektalk.tokens': '{invalid JSON' });
    await context.addInitScript(({ locale, theme }) => {
      localStorage.setItem('tektalk.locale', locale); localStorage.setItem('tektalk.theme', theme);
    }, { locale, theme });
    await page.goto(baseURL + '/#me');
    await page.locator('#languageButton').filter({ hasText: locale.toUpperCase() }).waitFor();
    await signedOut(page);
    assert.equal(await page.locator('#demoButton,.dialog-close').count(), 0);
    await page.keyboard.press('Escape'); await signedOut(page);
    await page.screenshot({ path: `recordings/design-system/web-${locale}-${theme}-login.png` });
    await page.locator('[data-auth-mode="register"]').click();
    await signedOut(page);
    await page.screenshot({ path: `recordings/design-system/web-${locale}-${theme}-register.png` });
    await page.setViewportSize({ width: 390, height: 844 });
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
    await page.screenshot({ path: `recordings/design-system/web-${locale}-${theme}-auth-mobile.png`, fullPage: true });
    await context.close();
  }

  // Hold and reject login; the host must stay hidden throughout the request.
  {
    const { context, page } = await contextFor();
    let release;
    const held = new Promise(resolve => { release = resolve; });
    await context.route('**/v1/auth/login', async route => { await held; await json(route, { message: 'Incorrect credentials' }, 401); });
    await page.goto(baseURL); await credentials(page);
    await page.locator('#authSubmit').click();
    assert.equal(await page.locator('#authSubmit').isDisabled(), true);
    await signedOut(page); release();
    await page.getByText('Incorrect credentials', { exact: true }).waitFor(); await signedOut(page);
    await context.close();
  }
  // Registration rejection and new-device verification are not signed-in states.
  {
    const { context, page } = await contextFor();
    await context.route('**/v1/auth/register', route => json(route, { message: 'Account already exists' }, 409));
    await context.route('**/v1/auth/login', route => json(route, { status: 'device_verification_required', challenge_id: 'challenge', question: 'Favourite city?' }));
    let accept = false;
    await context.route('**/v1/auth/device/verify', route => json(route, accept ? tokens : { message: 'Wrong answer' }, accept ? 200 : 401));
    await context.route('**/v1/chat/messages/history', route => json(route, { message: 'Expired token' }, 401));
    await page.goto(baseURL); await page.locator('[data-auth-mode="register"]').click(); await credentials(page);
    await page.locator('[name="display_name"]').fill('Test User');
    await page.locator('[name="security_question"]').fill('Favourite city?');
    await page.locator('[name="security_answer"]').fill('hanoi'); await page.locator('#authSubmit').click();
    await page.getByText('Account already exists', { exact: true }).waitFor(); await signedOut(page);
    await page.locator('[data-auth-mode="login"]').click(); await credentials(page); await page.locator('#authSubmit').click();
    await page.locator('#challengeField').waitFor({ state: 'visible' }); await signedOut(page);
    assert.equal(await page.locator('#authCredentials').isVisible(), false);
    await page.locator('[name="challenge_answer"]').fill('wrong'); await page.locator('#authSubmit').click();
    await page.getByText('Wrong answer', { exact: true }).waitFor(); await signedOut(page);
    accept = true; await page.evaluate(() => { window.__allowHost = true; });
    await page.locator('[name="challenge_answer"]').fill('hanoi'); await page.locator('#authSubmit').click();
    await page.locator('#appShell').waitFor({ state: 'visible' });
    assert.equal(await page.locator('#authDialog').isVisible(), false);
    await page.locator('#conversationId').fill('33333333-3333-4333-8333-333333333333');
    await page.locator('#recipientId').fill('44444444-4444-4444-8444-444444444444');
    await page.locator('#messageInput').fill('Private draft');
    await page.locator('#syncHistory').click();
    await page.locator('#authDialog').waitFor({ state: 'visible' });
    await signedOut(page);
    assert.equal(await page.locator('#messageInput').inputValue(), '');
    assert.equal(await page.evaluate(() => sessionStorage.getItem('tektalk.tokens')), null);
    await context.close();
  }
  // Cached tokens require a successful backend check, and logout clears the host.
  for (const status of [200, 401, 503]) {
    const { context, page } = await contextFor({ 'tektalk.tokens': JSON.stringify(tokens) });
    let release;
    const held = new Promise(resolve => { release = resolve; });
    await context.route('**/v1/chat/messages/history', async route => { await held; await json(route, status === 200 ? [] : { message: 'Unavailable' }, status); });
    await page.goto(baseURL + '/#me'); await signedOut(page);
    if (status === 200) await page.evaluate(() => { window.__allowHost = true; });
    release();
    if (status === 200) {
      await page.locator('#appShell').waitFor({ state: 'visible' });
      await page.locator('#signOutButton').click();
      await page.evaluate(() => { window.__allowHost = false; });
    } else await page.locator('#authSubmit:not(:disabled)').waitFor();
    await signedOut(page);
    await context.close();
  }
  // An abandoned pending login must not sign the next screen in when it completes.
  {
    const { context, page } = await contextFor();
    let release;
    const held = new Promise(resolve => { release = resolve; });
    await context.route('**/v1/auth/login', async route => { await held; await json(route, {status:'authenticated', tokens}).catch(() => {}); });
    await page.goto(baseURL); await credentials(page); await page.locator('#authSubmit').click();
    await page.locator('[data-auth-mode="register"]').click(); release();
    await page.waitForTimeout(150); await signedOut(page);
    assert.equal(await page.locator('#displayNameField').isVisible(), true);
    await context.close();
  }
  assert.deepEqual(errors, []);
  console.log('Standalone auth, no host flash/demo bypass, rejected/pending auth, device verification, cached-session checks, expiry and logout passed.');
} finally { await browser.close(); }
