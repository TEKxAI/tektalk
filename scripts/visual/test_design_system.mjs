import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { chromium } from 'playwright';

const baseURL = process.env.TEKTALK_WEB_URL || 'http://127.0.0.1:8080';
const browser = await chromium.launch({ headless: true });
try {
  for (const locale of ['en', 'vi']) {
    for (const theme of ['light', 'dark']) {
      const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, colorScheme: theme });
      const page = await context.newPage();
      await page.addInitScript(({ locale, theme }) => {
        localStorage.setItem('tektalk.locale', locale);
        if (!localStorage.getItem('tektalk.theme')) localStorage.setItem('tektalk.theme', theme);
        sessionStorage.setItem('tektalk.demo', '1');
      }, { locale, theme });
      await page.goto(baseURL);
      await page.waitForFunction(expected => document.documentElement.dataset.theme === expected, theme);
      await fs.mkdir('recordings/design-system', { recursive: true });
      await page.screenshot({ path: `recordings/design-system/web-${locale}-${theme}-initial.png` });
      const tabs = page.locator('.rail .nav-item');
      assert.deepEqual(await tabs.evaluateAll(items => items.map(item => item.dataset.tab)), ['messages', 'ai', 'me']);
      assert.deepEqual((await tabs.allTextContents()).map(text => text.replace(/\d/g, '').trim()), locale === 'en' ? ['Messages', 'AI', 'Me'] : ['Tin nhắn', 'AI', 'Cá nhân']);
      const desktop = await page.locator('.rail').boundingBox();
      const workspace = await page.locator('#messages').boundingBox();
      const sidebar = await page.locator('.sidebar').boundingBox();
      const history = await page.locator('.conversation-list').boundingBox();
      assert.ok(desktop && workspace && sidebar && history);
      // The sidebar includes its 1px border; the navigation uses the content box.
      assert.ok(Math.abs(desktop.x - sidebar.x) <= 1 && Math.abs(desktop.width - sidebar.width) <= 1, 'Tabs must fill the left pane');
      assert.ok(Math.abs(desktop.y + desktop.height - sidebar.y - sidebar.height) < 1, 'Tabs must be pinned to the bottom of the left pane');
      assert.ok(history.y + history.height <= desktop.y, 'History must be above Message tabs');
      assert.ok(Math.abs(sidebar.x + sidebar.width - workspace.x) < 1, 'Web must have exactly two adjacent panes');
      assert.equal(await page.locator('#messages .conversation-list').count(), 0, 'History must not create a third pane');
      const desktopTabBoxes = await tabs.evaluateAll(items => items.map(item => item.getBoundingClientRect()).map(({ x, y, width, height }) => ({ x, y, width, height })));
      assert.ok(desktopTabBoxes.every(box => Math.abs(box.y - desktopTabBoxes[0].y) < 1), 'Desktop tabs must share one horizontal row');
      assert.ok(desktopTabBoxes.every((box, index) => index === 0 || box.x > desktopTabBoxes[index - 1].x), 'Desktop tabs must run left to right');
      await page.locator('#messageInput').fill('Unsent draft');
      for (const [index, name] of ['messages', 'ai', 'me'].entries()) {
        await tabs.nth(index).click();
        assert.equal(await tabs.nth(index).getAttribute('aria-current'), 'page');
        assert.equal(await page.locator(`#${name}`).isVisible(), true);
        assert.equal(await page.locator(`[data-sidebar="${name}"]`).isVisible(), true);
        assert.equal(await page.locator('.sidebar-page.active').count(), 1);
        if (name !== 'messages') assert.equal(await page.locator('.conversation-list').isVisible(), false);
      }
      await tabs.nth(0).click();
      assert.equal(await page.locator('#messageInput').inputValue(), 'Unsent draft', 'Switching tabs must preserve drafts');
      await page.locator('#messageInput').fill('');
      await page.setViewportSize({ width: 390, height: 844 });
      await tabs.nth(0).click();
      const mobile = await page.locator('.rail').boundingBox();
      const content = await page.locator('#messages').boundingBox();
      assert.ok(mobile && content && mobile.y >= content.y + content.height - 1, 'Mobile navigation must be below content');
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Mobile page must not overflow horizontally');
      const primary = await page.locator('html').evaluate(element => getComputedStyle(element).getPropertyValue('--tek-primary').trim());
      assert.equal(primary, theme === 'light' ? '#0089FF' : '#63B5FF');
      await assertChatLayout(page, 'mobile');
      await fs.mkdir('recordings/design-system', { recursive: true });
      await page.screenshot({ path: `recordings/design-system/web-${locale}-${theme}-mobile.png` });
      await page.setViewportSize({ width: 1280, height: 900 });
      await assertChatLayout(page, 'desktop');
      await page.screenshot({ path: `recordings/design-system/web-${locale}-${theme}-desktop.png` });
      // System mode must react to OS preference changes without reloading.
      await page.evaluate(() => { localStorage.setItem('tektalk.theme', 'system'); });
      await page.reload();
      await page.emulateMedia({ colorScheme: theme === 'light' ? 'dark' : 'light' });
      await page.waitForFunction(expected => document.documentElement.dataset.theme === expected, theme === 'light' ? 'dark' : 'light');
      await context.close();
      console.log(`${locale}/${theme}: two panes, pinned bottom tabs, history, draft preservation and appearance passed`);
    }
  }
} finally {
  await browser.close();
}

async function assertChatLayout(page, viewport) {
  // The toolbar is hidden in demo mode and visible after login. Both layouts
  // must reserve the remaining space for messages, never for the composer.
  for (const toolbarVisible of [false, true]) {
    await page.locator('#chatControls').evaluate((element, visible) => { element.hidden = !visible; }, toolbarVisible);
    const composer = await page.locator('.composer').boundingBox();
    const messages = await page.locator('.message-list').boundingBox();
    const panel = await page.locator('.chat-panel').boundingBox();
    assert.ok(composer && messages && panel);
    assert.ok(composer.height < 140, `${viewport}: composer must remain compact (toolbar=${toolbarVisible})`);
    assert.ok(messages.height > panel.height / 2, `${viewport}: messages must fill available space`);
    assert.ok(composer.y + composer.height <= panel.y + panel.height, `${viewport}: composer must fit inside chat`);
  }
  await page.locator('#chatControls').evaluate(element => { element.hidden = true; });
}
