import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { spawn } from 'node:child_process';
const DIR = process.argv[2];
const STATIC = path.resolve(path.dirname(new URL(import.meta.url).pathname), '../../static');
const files = { '/work/a.py': 'print("a")\nx = 1\ny = 2\n', '/work/b.js': 'const b = 2;\nconsole.log(b);\n' };
const log = { writes: [], reads: [], ls: [], fsClients: [] };
const mime = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.ttf': 'font/ttf', '.map': 'application/json' };
const server = http.createServer((req, res) => {
  const u = new URL(req.url, 'http://x');
  if (u.pathname === '/api/read_file') {
    const p = u.searchParams.get('path');
    log.reads.push(p);
    if (p in files) { res.setHeader('content-type', 'application/json'); res.end(JSON.stringify({ content: files[p] })); } else { res.statusCode = 500; res.end('boom'); }
    return;
  }
  if (u.pathname === '/api/ls') {
    log.ls.push(u.searchParams.get('path'));
    res.setHeader('content-type', 'application/json');
    res.end(JSON.stringify({ dirs: ['sub'], files: ['a.py', 'b.js', 'Readme.md', 'notes.txt'] }));
    return;
  }
  if (u.pathname === '/api/write_file') {
    let b = ''; req.on('data', d => b += d); req.on('end', () => { log.writes.push({ body: b, ct: req.headers['content-type'] }); res.setHeader('content-type', 'application/json'); res.end('{"ok":true}'); });
    return;
  }
  if (u.pathname === '/api/fs-events') {
    res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-cache' });
    res.write(': hi\n\n');
    log.fsClients.push(res);
    req.on('close', () => { log.fsClients = log.fsClients.filter(r => r !== res); });
    return;
  }
  if (u.pathname === '/__log') { res.setHeader('content-type', 'application/json'); res.end(JSON.stringify({ writes: log.writes, reads: log.reads, ls: log.ls, fs: log.fsClients.length })); return; }
  if (u.pathname === '/__fsevent') { for (const r of log.fsClients) r.write('event: fs\ndata: {}\n\n'); res.end('ok'); return; }
  if (u.pathname === '/__fsend') { for (const r of log.fsClients) r.end(); res.end('ok'); return; }
  let fp = u.pathname === '/' ? path.join(path.dirname(STATIC), 'ui', 'test', 'editor_page.html') : u.pathname.startsWith('/vendor/') ? path.join(STATIC, u.pathname) : path.join(DIR, u.pathname);
  fs.readFile(fp, (e, d) => { if (e) { res.statusCode = 404; res.end('nf'); } else { res.setHeader('content-type', mime[path.extname(fp)] || 'application/octet-stream'); res.end(d); } });
});
await new Promise(r => server.listen(0, '127.0.0.1', r));
const port = server.address().port;
const prof = fs.mkdtempSync(path.join(os.tmpdir(), 'edtest-prof-'));
const chrome = spawn('/usr/bin/google-chrome', ['--headless=new', '--remote-debugging-port=0', '--user-data-dir=' + prof, '--no-first-run', '--no-default-browser-check', '--disable-gpu', '--window-size=1280,900', 'about:blank'], { stdio: ['ignore', 'ignore', 'pipe'] });
const dbg = await new Promise((res, rej) => { let b = ''; chrome.stderr.on('data', d => { b += d; const m = b.match(/ws:\/\/[^\s]+/); if (m) res(m[0]); }); setTimeout(() => rej(new Error('no devtools url')), 20000); });
const cleanup = () => { try { chrome.kill('SIGKILL'); } catch (e) {} try { server.close(); } catch (e) {} for (const r of log.fsClients) try { r.destroy(); } catch (e) {} try { fs.rmSync(prof, { recursive: true, force: true }); } catch (e) {} };
process.on('exit', cleanup);
const base = dbg.replace(/^ws:\/\//, 'http://').replace(/\/devtools\/browser.*$/, '');
const tabs = await (await fetch(base + '/json/list')).json();
const ws = new WebSocket(tabs.find(t => t.type === 'page').webSocketDebuggerUrl);
await new Promise(r => ws.addEventListener('open', r));
let id = 0; const pend = new Map(); const consoleMsgs = [];
ws.addEventListener('message', m => {
  const d = JSON.parse(m.data);
  if (d.id && pend.has(d.id)) { const { res, rej } = pend.get(d.id); pend.delete(d.id); d.error ? rej(new Error(JSON.stringify(d.error))) : res(d.result); }
  else if (d.method === 'Runtime.consoleAPICalled') consoleMsgs.push(d.params.type + ': ' + d.params.args.map(a => a.value ?? a.description).join(' '));
  else if (d.method === 'Runtime.exceptionThrown') consoleMsgs.push('EXC: ' + JSON.stringify(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text));
});
const send = (method, params = {}) => new Promise((res, rej) => { const i = ++id; pend.set(i, { res, rej }); ws.send(JSON.stringify({ id: i, method, params })); });
const ev = async expr => { const r = await send('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true }); if (r.exceptionDetails) throw new Error('eval: ' + JSON.stringify(r.exceptionDetails.exception?.description || r.exceptionDetails.text) + ' in ' + expr); return r.result.value; };
const sleep = ms => new Promise(r => setTimeout(r, ms));
const until = async (expr, ms = 8000) => { const t = Date.now(); for (;;) { const v = await ev(expr); if (v) return v; if (Date.now() - t > ms) throw new Error('timeout: ' + expr); await sleep(50); } };
const key = async (k, code, vk, mods = 0, text) => { await send('Input.dispatchKeyEvent', { type: 'keyDown', key: k, code, windowsVirtualKeyCode: vk, modifiers: mods, text: text ?? (k.length === 1 && !(mods & 2) ? k : undefined) }); await send('Input.dispatchKeyEvent', { type: 'keyUp', key: k, code, windowsVirtualKeyCode: vk, modifiers: mods }); };
let pass = 0, fail = 0;
const ok = (name, cond, extra) => { cond ? pass++ : fail++; console.log((cond ? 'PASS ' : 'FAIL ') + name + (cond ? '' : '  -> ' + JSON.stringify(extra))); };
const getlog = async () => (await fetch(`http://127.0.0.1:${port}/__log`)).json();
try {
  await send('Runtime.enable'); await send('Page.enable');
  await send('Page.navigate', { url: `http://127.0.0.1:${port}/` });
  await until('document.readyState === "complete" && window.__app !== undefined');
  await sleep(800);
  const T = await import(path.resolve(process.argv[3]));
  await T.run({ ev, until, key, sleep, ok, send, getlog, port, files, log, consoleMsgs });
} catch (e) { fail++; console.log('FATAL', e.message); }
console.log('console:', consoleMsgs.slice(0, 15));
console.log(`passed ${pass} failed ${fail}`);
cleanup();
process.exit(fail ? 1 : 0);
