import http from 'node:http'; import fs from 'node:fs'; import os from 'node:os'; import path from 'node:path'; import { execFileSync } from 'node:child_process'; import { fileURLToPath } from 'node:url'; import { createRequire } from 'node:module';
const { chromium } = createRequire(import.meta.url)('playwright-core');
const here = path.dirname(fileURLToPath(import.meta.url)), ui = path.join(here, '..'), hc = process.env.GOLD_HC || path.join(os.homedir(), 'ai/amni-lang/helium-webgpu/build/hc');
const work = fs.mkdtempSync(path.join(os.tmpdir(), 'chat-smoke-'));
for (const f of ['core.he', 'md.he', 'chat.he']) fs.copyFileSync(path.join(ui, f), path.join(work, f));
fs.cpSync(path.join(ui, 'data'), path.join(work, 'data'), { recursive: true });
fs.copyFileSync(path.join(here, 'chat_smoke_root.he'), path.join(work, 'root.he'));
execFileSync(hc, ['build', '--web', 'root.he', '-o', 'ui.wasm'], { cwd: work, env: { ...process.env, HC_LIB: process.env.HC_LIB || path.join(os.homedir(), 'ai/amni-lang/helium-stream/hc/lib') }, stdio: 'inherit' });
const staticDir = path.join(ui, '..', 'static');
const page = fs.readFileSync(path.join(staticDir, 'index.html'), 'utf8').replace(/<script[^>]*>[\s\S]*?<\/script>/g, '').replace(/\/assets\//g, '/').replace('</body>', '<script type="module">import("/helium.js").then(m=>m.start({wasm:"/ui.wasm"}))</script></body>');
const T = { html: 'text/html', js: 'text/javascript', wasm: 'application/wasm', css: 'text/css' };
const sleep = ms => new Promise(r => setTimeout(r, ms));
let calls = [];
const edit = { path: 'src/a.js', old_string: 'one\ntwo\nthree\nfour\nfive\nsix\nseven', new_string: 'one\ntwo\nTHREE\nfour\nfive\nsix\nseven' };
const srv = http.createServer(async (q, r) => {
  let body = ''; for await (const c of q) body += c;
  if (q.url === '/__reset') { calls = []; return r.end('ok'); }
  if (q.url === '/__calls') return r.end(JSON.stringify(calls));
  if (q.url.startsWith('/api/')) {
    calls.push([q.url, body]);
    if (q.url !== '/api/chat') { r.writeHead(200, { 'content-type': 'application/json' }); return r.end(JSON.stringify(q.url === '/api/undo' ? { reverted: ['a.js', 'b.py'] } : { status: 'ok' })); }
    const m = JSON.parse(body).message || '', ev = (t, d) => r.write(`event: ${t}\ndata: ${JSON.stringify(d)}\n\n`);
    r.writeHead(200, { 'content-type': 'text/event-stream' });
    ev('session', { session_id: 'sess-1' }); await sleep(30); ev('thinking', { text: 'planning...' });
    /SLOW/.test(m) && await sleep(2500);
    await sleep(30); ev('tool_start', { tool: 'read_file', input: { path: 'notes.md' } });
    await sleep(30); ev('tool_result', { tool: 'read_file', input: { path: 'notes.md' }, output: '# Demo\nline two', status: 'success' });
    if (/ERR/.test(m)) { ev('error', { error: 'Error: boom' }); return r.end(); }
    if (/DROP/.test(m)) return r.destroy();
    await sleep(30); ev('tool_result', { tool: 'edit_file', input: edit, output: 'ok', status: 'success' });
    await sleep(30); ev('tool_result', { tool: 'write_file', input: { path: 'b.py', content: 'x = 1\n# hi' }, output: 'wrote', status: 'success' });
    await sleep(30); ev('widget', { type: 'time', title: 'Clock', data: { time_human: '12:00' } });
    await sleep(30); ev('message', { message: 'Starts with: # Demo\n\n```js\nconst a = 1;\n```\n\n**bold**' });
    await sleep(30); ev('done', {}); return r.end();
  }
  const f = q.url === '/' ? null : q.url === '/helium.js' || q.url === '/ui.wasm' ? path.join(work, q.url.slice(1)) : path.join(staticDir, q.url.slice(1));
  if (!f) { r.writeHead(200, { 'content-type': 'text/html' }); return r.end(page); }
  fs.existsSync(f) && fs.statSync(f).isFile() ? (r.writeHead(200, { 'content-type': T[f.split('.').pop()] || 'application/octet-stream' }), r.end(fs.readFileSync(f))) : (r.writeHead(404), r.end());
}).listen(0, '127.0.0.1', async () => {
  const port = srv.address().port, base = `http://127.0.0.1:${port}/`;
  const b = await chromium.launch({ executablePath: '/usr/bin/google-chrome', headless: true, args: ['--no-first-run'] });
  const ctx = await b.newContext({ viewport: { width: 1600, height: 900 }, permissions: ['clipboard-read', 'clipboard-write'] }); const p = await ctx.newPage(); p.setDefaultTimeout(8000);
  let fails = 0; const ok = (n, c, x) => { console.log(c ? 'PASS' : 'FAIL', n, c ? '' : JSON.stringify(x)); if (!c) fails++; };
  p.on('pageerror', e => console.log('pageerror', e.message));
  const dialogs = []; let dmode = 'accept'; p.on('dialog', async d => { dialogs.push(d.message()); await (dmode === 'accept' ? d.accept() : d.dismiss()).catch(() => 0); });
  const calls_ = async () => (await fetch(base + '__calls')).json();
  const act = (v, a) => p.evaluate(([v, a]) => { const e = document.createElement('div'); e.setAttribute('data-act', v); e.setAttribute('data-arg', a); document.body.appendChild(e); e.click(); e.remove(); }, [v, a]);
  const fresh = async () => { await p.goto(base); await p.waitForSelector('#messages'); await sleep(1000); await p.evaluate(() => localStorage.clear()); await p.reload(); await sleep(1200); await fetch(base + '__reset'); dialogs.length = 0; dmode = 'accept'; };
  const idle = async () => { await sleep(300); await p.waitForFunction(() => !document.querySelector('#thinking') && document.getElementById('interrupt').disabled, null, { timeout: 12000 }); await sleep(500); };
  const html = s => p.innerHTML(s);
  try {
    await fresh();
    await p.click('.key[data-act="chat.fill"]');
    ok('fill', (await p.inputValue('#input')).startsWith('Create a Python') && await p.evaluate(() => document.activeElement.id === 'input'));
    await p.fill('#input', 'hello world'); await p.click('#send');
    await p.waitForSelector('#thinking .live-steps .step.ok');
    await idle();
    ok('tool badge, no thinking', await p.evaluate(() => !document.querySelector('#thinking') && !!document.querySelector('.msg.assistant .tool-badge')));
    const m = await html('#messages');
    ok('markdown, widget, copy button', /cb-copy/.test(m) && /amni-widget/.test(m) && /<strong>bold/.test(m));
    ok('chat body', JSON.parse((await calls_())[0][1]).message === 'hello world');
    const hist = JSON.parse(await p.evaluate(() => localStorage.getItem('amniHistory')));
    ok('history saved', hist.length === 1 && hist[0].title === 'hello world...' && hist[0].sessionId === 'sess-1' && hist[0].diff.length === 4, hist);
    ok('diff panel open with files', await p.evaluate(() => document.getElementById('diff-col').classList.contains('open')) && /src\/a\.js/.test(await html('#diff-body')) && /b\.py/.test(await html('#diff-body')));
    await p.click('.tool-toggle'); ok('tool toggle', await p.evaluate(() => document.querySelector('.tool-detail').style.display === 'block'));
    await p.click('.cb-copy'); await sleep(150); const c1 = await p.textContent('.cb-copy'); await sleep(1400);
    ok('copy feedback', c1 !== 'copy' && await p.textContent('.cb-copy') === 'copy' && await p.evaluate(() => navigator.clipboard.readText()) === 'const a = 1;', c1);
    await p.fill('#input', 'second'); await p.click('#send'); await idle();
    ok('session id sent', JSON.parse((await calls_()).filter(c => c[0] === '/api/chat')[1][1]).session_id === 'sess-1');
    await sleep(400); dmode = 'dismiss'; await p.click('.diff-action-btn.undo'); await sleep(500);
    ok('undo cancelled', dialogs.length === 1 && !(await calls_()).some(c => c[0] === '/api/undo'));
    dmode = 'accept'; await p.click('.diff-action-btn.undo'); await sleep(900);
    ok('undo done', (await calls_()).some(c => c[0] === '/api/undo') && /Reverted 2 file/.test(await html('#messages')) && /No changes yet/.test(await html('#diff-body')));
    await act('chat.system', 'Hello `code`'); ok('system', /msg system[\s\S]*<code>code<\/code>/.test(await html('#messages')));
    await act('chat.parse_diff', JSON.stringify({ tool: 'write_file', input: { path: 'z.txt', content: 'hi' }, output: 'ok', status: 'success' })); await sleep(200);
    ok('parse_diff', (await html('#diff-body')).includes('z.txt'));
    await p.click('.diff-action-btn.accept'); await sleep(400); ok('accept', /Changes accepted/.test(await html('#messages')) && (await calls_()).some(c => c[0] === '/api/accept'));
    await act('chat.history', ''); await sleep(400);
    ok('history list', /hello world\.\.\./.test(await html('#history-list')) && /hist-time[^>]*>[^<]{6}/.test(await html('#history-list')));
    await p.reload(); await sleep(1200); await act('chat.history', ''); await sleep(400);
    await p.click('#history-list .hist-item'); await sleep(700);
    ok('load chat', /hello world/.test(await html('#messages')) && !/onclick/.test(await html('#messages')) && (await html('#diff-body')).includes('z.txt'), (await html('#diff-body')).slice(0, 200));
    await p.evaluate(() => localStorage.setItem('amniHistory', JSON.stringify([{ id: 5, title: 'old...', date: 1700000000000, html: '<div class="msg user">q</div><div class="msg assistant"><div class="tool-group"><button class="tool-toggle" onclick="x()">d</button><div class="tool-detail" style="display:none">x</div></div><button class="cb-copy" onclick="copyCode(this)">copy</button></div>' }])));
    await act('chat.history', ''); await p.click('#history-list .hist-item'); await sleep(600);
    ok('old chat: no onclick, toggle by class', !/onclick/.test(await html('#messages')) && (await p.click('.tool-toggle'), await p.evaluate(() => document.querySelector('.tool-detail').style.display === 'block')));
    await p.fill('#input', 'continue'); await p.click('#send'); await idle();
    const h2 = JSON.parse(await p.evaluate(() => localStorage.getItem('amniHistory')));
    ok('continue updates entry', h2.length === 1 && h2[0].id === 5 && /continue/.test(h2[0].html), h2.map(x => x.id));
    await act('chat.delete', '5'); await sleep(300); ok('delete', await p.evaluate(() => localStorage.getItem('amniHistory')) === '[]');
    await fresh(); await p.fill('#input', 'ERR x'); await p.click('#send'); await idle();
    ok('error event', /msg system error[^>]*><p>boom<\/p>/.test(await html('#messages')) && await p.evaluate(() => document.getElementById('send').disabled === false), [(await html('#messages')).slice(-300), await p.evaluate(() => document.getElementById('send').disabled)]);
    await fresh(); await p.fill('#input', 'DROP x'); await p.click('#send'); await idle();
    ok('dropped stream', /msg system/.test(await html('#messages')) && await p.evaluate(() => document.getElementById('send').disabled === false));
    await fresh(); await p.fill('#input', 'SLOW x'); await p.click('#send'); await p.waitForSelector('#thinking'); await sleep(300);
    await p.fill('#input', 'also y'); await p.click('#steer'); await sleep(500);
    ok('steer', (await calls_()).some(c => c[0] === '/api/steer' && /also y/.test(c[1])) && (await p.inputValue('#input')) === '');
    await p.click('#interrupt'); await sleep(500);
    ok('interrupt', (await calls_()).some(c => c[0] === '/api/interrupt'), await calls_());
    await sleep(3000); await act('chat.new', ''); await sleep(1800);
    ok('new chat', (await html('#messages')).includes('chat.fill'));
  } catch (e) { fails++; console.log('FATAL', e.message.split('\n')[0]); }
  await b.close(); srv.close(); fs.rmSync(work, { recursive: true, force: true });
  console.log(fails ? `${fails} failed` : 'all passed'); process.exit(fails ? 1 : 0);
});
