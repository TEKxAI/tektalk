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
      const tabs = page.locator('.rail .nav-item');
      assert.deepEqual(await tabs.evaluateAll(items => items.map(item => item.dataset.tab)), ['messages', 'ai', 'me']);
      assert.deepEqual((await tabs.allTextContents()).map(text => text.replace(/\d/g, '').trim()), locale === 'en' ? ['Messages', 'AI', 'Me'] : ['Tin nhắn', 'AI', 'Cá nhân']);
      const desktop = await page.locator('.rail').boundingBox();
      const workspace = await page.locator('#messages').boundingBox();
      assert.ok(desktop && workspace && desktop.x + desktop.width <= workspace.x + 1, 'Desktop navigation must be left of content');
      for (const [index, name] of ['messages', 'ai', 'me'].entries()) {
        await tabs.nth(index).click();
        assert.equal(await tabs.nth(index).getAttribute('aria-current'), 'page');
        assert.equal(await page.locator(`#${name}`).isVisible(), true);
      }
      await page.setViewportSize({ width: 390, height: 844 });
      await tabs.nth(0).click();
      const mobile = await page.locator('.rail').boundingBox();
      const content = await page.locator('#messages').boundingBox();
      assert.ok(mobile && content && mobile.y >= content.y + content.height - 1, 'Mobile navigation must be below content');
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Mobile page must not overflow horizontally');
      const primary = await page.locator('html').evaluate(element => getComputedStyle(element).getPropertyValue('--tek-primary').trim());
      assert.equal(primary, theme === 'light' ? '#6558F5' : '#B1A6FF');
      await fs.mkdir('recordings/design-system', { recursive: true });
      await page.screenshot({ path: `recordings/design-system/web-${locale}-${theme}-mobile.png` });
      await page.setViewportSize({ width: 1280, height: 900 });
      await page.screenshot({ path: `recordings/design-system/web-${locale}-${theme}-desktop.png` });
      // System mode must react to OS preference changes without reloading.
      await page.evaluate(() => { localStorage.setItem('tektalk.theme', 'system'); });
      await page.reload();
      await page.emulateMedia({ colorScheme: theme === 'light' ? 'dark' : 'light' });
      await page.waitForFunction(expected => document.documentElement.dataset.theme === expected, theme === 'light' ? 'dark' : 'light');
      await context.close();
      console.log(`${locale}/${theme}: desktop sidebar, mobile tabs, navigation, palette and system appearance passed`);
    }
  }
} finally {
  await browser.close();
}
