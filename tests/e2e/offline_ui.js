// Drives one full agent turn per local server through the real UI, and fails on any non-loopback request.
// usage: node offline_ui.js <base-url> <screenshot-dir>
const { chromium } = require('playwright');
(async () => {
  const [base, outDir] = process.argv.slice(2);
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 820 }, colorScheme: 'dark' });
  const external = [];
  page.on('request', r => { const u = r.url(); if (!/^(https?:\/\/(127\.0\.0\.1|localhost)(:\d+)?\/|data:|blob:)/.test(u)) external.push(u); });
  page.on('pageerror', e => console.log('pageerror:', e.message));
  await page.goto(base, { waitUntil: 'networkidle' });
  await page.waitForSelector('#amni-bar .ab-prov');
  const turn = async (prompt, n, liveShot) => {
    await page.fill('#input', prompt);
    await page.keyboard.press('Enter');
    if (liveShot) {
      await page.waitForSelector('#thinking .live-steps .step.ok', { timeout: 15000 });
      await page.screenshot({ path: liveShot });
    }
    await page.waitForFunction(k => document.querySelectorAll('.msg.assistant .tool-badge').length >= k && !document.querySelector('#thinking'), n, { timeout: 30000 });
    const last = await page.$$eval('.msg.assistant', els => els[els.length - 1].innerText);
    if (!/The file starts with: # Demo workspace/.test(last)) throw new Error('unexpected answer: ' + last);
    if (!/read_file/.test(last)) throw new Error('no read_file badge: ' + last);
    console.log('turn', n, 'ok:', last.replace(/\s+/g, ' ').slice(0, 120));
  };
  // Turn 1: Ollama-shape server with a small model -> text tool protocol.
  await turn('Read notes.md and tell me its first line.', 1);
  // Switch to Adam via the picker (Ctrl+K), which lists the detected local servers.
  await page.keyboard.press('Control+k');
  await page.waitForSelector('#amni-picker.open .pk-row[data-provider="amni"]');
  await page.screenshot({ path: outDir + '/offline_picker.png' });
  await page.click('#amni-picker .pk-row[data-provider="amni"]');
  await page.waitForFunction(() => /Adam/.test(document.querySelector('#amni-bar .ab-prov').textContent));
  // Turn 2: Adam (OpenAI-shape) with native tool calls.
  await turn('Now read notes.md again via Adam.', 2, outDir + '/offline_live_timeline.png');
  await page.screenshot({ path: outDir + '/offline_turns.png' });
  await browser.close();
  if (external.length) { console.log('EXTERNAL REQUESTS:', external); process.exit(2); }
  console.log('external requests: 0');
})().catch(e => { console.error(e); process.exit(1); });
