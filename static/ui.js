// Amni-Code 3.0 UI layer: model picker, offline mode, live tool timeline, shortcuts.
// Dependency-free; served from the binary at /assets/ui.js.
(function () {
  'use strict';
  const $ = (s, r) => (r || document).querySelector(s);
  const esc = s => String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const state = { cfg: null, providers: [], local: [], servers: [], models: [], modelsSource: '', modelsError: '' };
  const LABEL = { amni: 'Adam', ollama: 'Ollama', local: 'Local', openai: 'OpenAI', anthropic: 'Anthropic', google: 'Gemini', xai: 'xAI' };
  const PICKER_CLOUD = ['anthropic', 'openai', 'google', 'xai'];

  async function j(url, opts) {
    const r = await fetch(url, opts);
    if (!r.ok) throw new Error(url + ' ' + r.status);
    return r.json();
  }
  async function saveCfg(patch) {
    state.cfg = await j('/api/config', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(patch) });
    if (typeof window.loadConfig === 'function') { try { await window.loadConfig(); } catch (e) {} }
    render();
    return state.cfg;
  }
  async function refresh(full) {
    try { state.cfg = await j('/api/config'); } catch (e) { return; }
    if (full || !state.providers.length) {
      try { const p = await j('/api/providers'); state.providers = p.providers; state.local = p.suggested_local; } catch (e) {}
      try { state.servers = (await j('/api/local-servers')).servers || []; } catch (e) { state.servers = []; }
    }
    try { const m = await j('/api/models'); state.models = m.models || []; state.modelsSource = m.source || ''; state.modelsError = m.error || ''; } catch (e) {}
    render();
  }
  const isCloud = id => { const p = state.providers.find(x => x.id === id); return p ? p.cloud : false; };
  const shortModel = m => (typeof MODEL_LABELS !== 'undefined' && MODEL_LABELS[m]) || String(m || '').replace(/^hf\.co\//, '');
  const coreModels = id => (typeof CORE_PROVIDERS !== 'undefined' && CORE_PROVIDERS[id] && CORE_PROVIDERS[id].models) || [];

  // ---------- composer bar ----------
  function ensureBar() {
    let bar = $('#amni-bar');
    if (bar) return bar;
    const area = $('#pane-0 .input-area') || $('.input-area');
    if (!area) return null;
    bar = document.createElement('div');
    bar.id = 'amni-bar';
    bar.className = 'amni-bar';
    bar.innerHTML = '<button type="button" class="ab-pill" id="ab-model" title="Choose provider and model (Ctrl+K)" aria-haspopup="dialog"><span class="ab-dot"></span><span class="ab-prov"></span><span class="ab-model"></span><span class="ab-caret">▾</span></button>' +
      '<button type="button" class="ab-chip" id="ab-offline" title="Offline mode: local models only, no web tools (Ctrl+Shift+O)" aria-pressed="false"><span class="ab-led"></span><span class="ab-off-label">Online</span></button>' +
      '<div class="ab-mode" role="group" aria-label="Agent mode"><button type="button" data-mode="plan" title="Plan: propose, never change files">Plan</button><button type="button" data-mode="edit" title="Edit: apply changes">Edit</button><button type="button" data-mode="autonomous" title="Autonomous: run everything">Auto</button></div>' +
      '<span class="ab-spacer"></span><button type="button" class="ab-help" id="ab-help" title="Keyboard shortcuts (?)">Shortcuts <kbd>?</kbd></button>';
    area.appendChild(bar);
    $('#ab-model', bar).addEventListener('click', togglePicker);
    $('#ab-offline', bar).addEventListener('click', toggleOffline);
    $('#ab-help', bar).addEventListener('click', showShortcuts);
    bar.querySelectorAll('.ab-mode button').forEach(b => b.addEventListener('click', () => saveCfg({ mode: b.dataset.mode }).then(() => { const s = $('#cfg-mode'); if (s) s.value = b.dataset.mode; })));
    return bar;
  }
  function render() {
    const bar = ensureBar();
    const c = state.cfg;
    if (!bar || !c) return;
    const cloud = isCloud(c.provider);
    const seat = state.providers.length && !state.providers.find(x => x.id === c.provider);
    $('.ab-dot', bar).className = 'ab-dot ' + (seat ? 'seat' : cloud ? 'cloud' : 'local');
    $('.ab-prov', bar).textContent = LABEL[c.provider] || c.provider;
    $('.ab-model', bar).textContent = shortModel(c.model) || 'choose a model';
    const off = $('#ab-offline', bar);
    off.classList.toggle('on', !!c.offline);
    off.setAttribute('aria-pressed', c.offline ? 'true' : 'false');
    $('.ab-off-label', off).textContent = c.offline ? 'Offline' : 'Online';
    bar.querySelectorAll('.ab-mode button').forEach(b => b.classList.toggle('on', (c.mode || 'edit') === b.dataset.mode));
    document.body.classList.toggle('amni-offline', !!c.offline);
    const warn = c.offline && cloud;
    bar.classList.toggle('warn', !!warn);
    $('#ab-model', bar).title = warn ? 'Offline mode is on but a cloud provider is selected: pick a local model' : 'Choose provider and model (Ctrl+K)';
    renderWelcome();
    renderSettingsToggle();
    if ($('#amni-picker.open')) renderPicker();
  }

  // ---------- picker ----------
  function togglePicker() { const p = $('#amni-picker'); if (p && p.classList.contains('open')) closePicker(); else openPicker(); }
  function openPicker() {
    let p = $('#amni-picker');
    if (!p) {
      p = document.createElement('div');
      p.id = 'amni-picker';
      p.className = 'amni-picker';
      p.setAttribute('role', 'dialog');
      p.setAttribute('aria-label', 'Choose a model');
      document.body.appendChild(p);
      p.addEventListener('click', onPickerClick);
    }
    p.classList.add('open');
    renderPicker();
    refresh(true);
    setTimeout(() => { const f = $('#pk-filter'); f && f.focus(); }, 30);
  }
  function closePicker() { const p = $('#amni-picker'); if (p) p.classList.remove('open'); }
  function row(provider, model, opts) {
    const c = state.cfg || {};
    const active = c.provider === provider && c.model === model;
    return `<button type="button" class="pk-row${active ? ' active' : ''}" data-provider="${esc(provider)}" data-model="${esc(model)}" data-base="${esc(opts.base || '')}"${opts.disabled ? ' disabled' : ''} title="${esc(opts.title || model)}">` +
      `<span class="pk-name">${esc(shortModel(model))}</span><span class="pk-meta">${esc(opts.meta || '')}</span>${opts.badge ? `<span class="pk-badge ${esc(opts.badgeClass || '')}">${esc(opts.badge)}</span>` : ''}${active ? '<span class="pk-check">✓</span>' : ''}</button>`;
  }
  function renderPicker() {
    const p = $('#amni-picker');
    if (!p) return;
    const c = state.cfg || {};
    const prev = $('#pk-filter');
    const q = ((prev && prev.value) || '').toLowerCase();
    const ok = m => !q || String(m).toLowerCase().includes(q);
    let h = `<div class="pk-head"><input id="pk-filter" placeholder="Filter models…" value="${esc(q)}" autocomplete="off" aria-label="Filter models"><button type="button" class="pk-close" data-act="close" title="Close (Esc)">✕</button></div><div class="pk-body">`;
    h += '<div class="pk-sec">Local · works offline</div>';
    const up = state.servers.filter(s => s.up);
    if (!state.servers.length) h += '<div class="pk-note">Scanning for local servers…</div>';
    state.servers.forEach(s => {
      h += `<div class="pk-srv ${s.up ? 'up' : 'down'}"><span class="ab-dot ${s.up ? 'local' : 'off'}"></span><span>${esc(s.label)}</span><span class="pk-url">${esc(s.base_url.replace('http://', ''))}</span><span class="pk-state">${s.up ? (s.models.length + ' model' + (s.models.length === 1 ? '' : 's')) : 'not running'}</span></div>`;
      if (s.up) (s.models.length ? s.models : (s.provider === 'amni' ? ['adam:3b', 'adam:8b'] : [])).filter(ok).forEach(m => { h += row(s.provider, m, { base: s.base_url, meta: '', badge: 'offline', badgeClass: 'ok' }); });
    });
    if (state.servers.length && !up.length) h += '<div class="pk-note">No local model server is running. Start Adam (Amni-AI, :7700) or <code>ollama serve</code> (:11434).</div>';
    h += '<div class="pk-sec">Suggested local models</div>';
    state.local.forEach(m => {
      h += `<div class="pk-sugg"><div class="pk-sugg-t"><b>${esc(m.label)}</b><span class="pk-meta">${(m.context / 1024) | 0}K context · ${esc(m.gguf)}</span></div><div class="pk-cmd"><code>ollama pull ${esc(m.ollama)}</code><button type="button" class="pk-copy" data-copy="ollama pull ${esc(m.ollama)}">Copy</button><button type="button" class="pk-use" data-provider="ollama" data-model="${esc(m.ollama)}" data-base="http://127.0.0.1:11434">Use</button></div></div>`;
    });
    h += '<div class="pk-sec">Cloud · needs a key and internet</div>';
    PICKER_CLOUD.forEach(id => {
      const pr = state.providers.find(x => x.id === id);
      if (!pr) return;
      const models = (c.provider === id && state.models.length) ? state.models : (coreModels(id).length ? coreModels(id) : [pr.default_model]);
      const blocked = !!c.offline;
      h += `<div class="pk-prov">${esc(pr.label)}${pr.has_env_key ? ' <span class="pk-badge ok">key found</span>' : ''}${blocked ? ' <span class="pk-badge warn">off while offline</span>' : ''}</div>`;
      models.filter(ok).slice(0, 8).forEach(m => { h += row(id, m, { base: pr.default_base, disabled: blocked, title: blocked ? 'Turn Offline mode off to use cloud models' : m }); });
    });
    h += `<div class="pk-foot">${state.modelsSource ? 'Model list: ' + esc(state.modelsSource) : ''}${state.modelsError ? ' (' + esc(state.modelsError) + ')' : ''} · <a href="#" data-act="settings">All providers and keys</a></div></div>`;
    p.innerHTML = h;
    const f = $('#pk-filter', p);
    if (f) f.addEventListener('input', () => { const pos = f.selectionStart; renderPicker(); const g = $('#pk-filter'); if (g) { g.focus(); g.setSelectionRange(pos, pos); } });
  }
  async function onPickerClick(e) {
    const t = e.target.closest('button,a');
    if (!t) return;
    if (t.dataset.act === 'close') return closePicker();
    if (t.dataset.act === 'settings') { e.preventDefault(); closePicker(); const b = $('#settings-btn'); b && b.click(); return; }
    if (t.dataset.copy) { try { await navigator.clipboard.writeText(t.dataset.copy); t.textContent = 'Copied'; } catch (_) { t.textContent = 'Select it'; } return; }
    if (t.dataset.provider && !t.disabled) {
      const patch = { provider: t.dataset.provider, model: t.dataset.model };
      if (t.dataset.base) patch.base_url = t.dataset.base;
      await saveCfg(patch);
      const sel = $('#cfg-model');
      if (sel && ![...sel.options].some(o => o.value === t.dataset.model)) { const o = document.createElement('option'); o.value = o.textContent = t.dataset.model; sel.appendChild(o); }
      if (sel) sel.value = t.dataset.model;
      closePicker();
      toast(`Using ${LABEL[t.dataset.provider] || t.dataset.provider} · ${shortModel(t.dataset.model)}`);
      refresh(false);
    }
  }

  // ---------- offline ----------
  async function toggleOffline() {
    const on = !(state.cfg && state.cfg.offline);
    await saveCfg({ offline: on });
    toast(on ? 'Offline mode on: cloud providers and web tools are disabled' : 'Offline mode off');
  }
  function renderSettingsToggle() {
    const anchor = $('#cfg-provider');
    if (!anchor) return;
    let r = $('#cfg-offline-row');
    if (!r) {
      r = document.createElement('div');
      r.id = 'cfg-offline-row';
      r.className = 'cfg-offline-row';
      r.innerHTML = '<label class="cfg-off-label"><input type="checkbox" id="cfg-offline"> Offline mode</label><p class="toggle-desc">Local models only. Cloud providers, web_search, web_fetch, model downloads and update checks are switched off, with a clear message instead of a timeout.</p>';
      anchor.parentNode.insertBefore(r, anchor.previousElementSibling);
      $('#cfg-offline', r).addEventListener('change', e => saveCfg({ offline: e.target.checked }));
    }
    $('#cfg-offline', r).checked = !!(state.cfg && state.cfg.offline);
  }

  // ---------- welcome / empty state ----------
  function renderWelcome() {
    const w = $('#welcome');
    if (!w) return;
    let card = $('#amni-status-card');
    if (!card) { card = document.createElement('div'); card.id = 'amni-status-card'; card.className = 'amni-status-card'; w.appendChild(card); }
    const c = state.cfg || {};
    const up = state.servers.filter(s => s.up);
    let h = '<div class="sc-line">';
    if (!state.servers.length) h += '<span class="ab-dot off"></span> Checking for local model servers…';
    else if (up.length) h += `<span class="ab-dot local"></span> Local model server${up.length > 1 ? 's' : ''}: ${up.map(s => `<b>${esc(s.label)}</b> (${s.models.length})`).join(' · ')}`;
    else h += '<span class="ab-dot off"></span> No local model server found. Start Adam (:7700) or Ollama (:11434), or <a href="#" data-open-picker>pick a cloud model</a>.';
    h += `</div><div class="sc-sub">Using <b>${esc(LABEL[c.provider] || c.provider || '…')}</b> · ${esc(shortModel(c.model) || 'no model')}${c.offline ? ' · <span class="sc-off">offline</span>' : ''} · <a href="#" data-open-picker>change</a> <kbd>Ctrl</kbd><kbd>K</kbd></div>`;
    card.innerHTML = h;
    card.querySelectorAll('[data-open-picker]').forEach(a => a.addEventListener('click', ev => { ev.preventDefault(); openPicker(); }));
  }

  // ---------- live tool timeline ----------
  const TOOL_ICON = { read_file: '▤', write_file: '✎', edit_file: '✎', multi_edit: '✎', run_command: '›', list_directory: '☰', search_files: '⌕', web_fetch: '⇣', web_search: '⌕', run_tests: '✓', run_lint: '≋', run_format: '¶', git_status: '⎇', git_diff: '±', git_add: '+', git_commit: '●', git_log: '≡', find_definition: '⌖' };
  function argSummary(d) { const i = d.input || {}; return i.path || i.command || i.query || i.url || i.name || i.message || ''; }
  function timeline() {
    const th = $('#thinking');
    if (!th) return null;
    let tl = $('.live-steps', th);
    if (!tl) {
      th.innerHTML = '<div class="live-head"><div class="thinking"><span></span><span></span><span></span></div><span class="live-title">Working…</span></div><ol class="live-steps"></ol>';
      tl = $('.live-steps', th);
    }
    return tl;
  }
  window.AmniLive = {
    start(d) {
      const tl = timeline();
      if (!tl) return null;
      const li = document.createElement('li');
      li.className = 'step run';
      li.dataset.tool = d.tool;
      li.innerHTML = `<span class="st-ico">${esc(TOOL_ICON[d.tool] || '⚙')}</span><span class="st-name">${esc(d.tool)}</span><span class="st-arg">${esc(argSummary(d))}</span><span class="st-state">running</span>`;
      tl.appendChild(li);
      const t = $('#thinking .live-title'); if (t) t.textContent = 'Running ' + d.tool + '…';
      const m = $('#messages'); if (m) m.scrollTop = m.scrollHeight;
      return li;
    },
    end(d) {
      const tl = timeline();
      if (!tl) return;
      const el = [...tl.querySelectorAll('li.step.run')].find(x => x.dataset.tool === d.tool) || this.start(d);
      if (!el) return;
      el.classList.remove('run');
      el.classList.add(d.status === 'success' ? 'ok' : d.status === 'planned' ? 'plan' : 'err');
      $('.st-state', el).textContent = d.status === 'success' ? 'done' : d.status;
      const out = String(d.output || '');
      if (out) { const pre = document.createElement('div'); pre.className = 'st-out'; pre.textContent = out.length > 240 ? out.slice(0, 240) + '…' : out; el.appendChild(pre); }
      const t = $('#thinking .live-title'); if (t) t.textContent = 'Thinking…';
    }
  };

  // ---------- shortcuts ----------
  const SHORTCUTS = [['Ctrl/⌘ K', 'Choose provider and model'], ['Ctrl/⌘ Shift O', 'Toggle Offline mode'], ['Ctrl/⌘ Shift L', 'Toggle light / dark'], ['Ctrl/⌘ Shift P', 'Command palette'], ['Ctrl/⌘ P', 'Quick open file'], ['Ctrl/⌘ ,', 'Settings'], ['Ctrl/⌘ Shift N', 'New chat'], ['Enter · Shift Enter', 'Send · new line'], ['/', 'Focus the prompt'], ['?', 'This list'], ['Esc', 'Close panels, stop generation']];
  function showShortcuts() {
    let o = $('#amni-keys');
    if (!o) {
      o = document.createElement('div');
      o.id = 'amni-keys';
      o.className = 'amni-keys';
      o.setAttribute('role', 'dialog');
      o.setAttribute('aria-label', 'Keyboard shortcuts');
      o.innerHTML = '<div class="ak-box"><div class="ak-head">Keyboard shortcuts<button type="button" class="pk-close" title="Close">✕</button></div>' + SHORTCUTS.map(([k, d]) => `<div class="ak-row"><kbd>${esc(k)}</kbd><span>${esc(d)}</span></div>`).join('') + '</div>';
      o.addEventListener('click', e => { if (e.target === o || e.target.closest('.pk-close')) o.classList.remove('open'); });
      document.body.appendChild(o);
    }
    o.classList.add('open');
  }
  function typing(e) { const t = e.target; return t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable); }
  document.addEventListener('keydown', e => {
    const mod = e.ctrlKey || e.metaKey;
    const k = (e.key || '').toLowerCase();
    if (mod && !e.shiftKey && k === 'k') { e.preventDefault(); togglePicker(); }
    else if (mod && e.shiftKey && k === 'o') { e.preventDefault(); toggleOffline(); }
    else if (mod && e.shiftKey && k === 'l') { e.preventDefault(); if (window.AmniSkin) AmniSkin.toggleTone(); }
    else if (mod && e.shiftKey && k === 'n') { e.preventDefault(); const b = $('#new-chat'); b && b.click(); }
    else if (mod && k === ',') { e.preventDefault(); const b = $('#settings-btn'); b && b.click(); }
    else if (e.key === 'Escape') {
      closePicker();
      const o = $('#amni-keys'); if (o) o.classList.remove('open');
      const ib = $('#interrupt'); if (ib && !ib.disabled) ib.click();
    } else if (!mod && !typing(e) && e.key === '?') { e.preventDefault(); showShortcuts(); }
    else if (!mod && !typing(e) && e.key === '/') { e.preventDefault(); const i = $('#input'); i && i.focus(); }
  });
  document.addEventListener('click', e => {
    const p = $('#amni-picker.open');
    if (p && !p.contains(e.target) && !e.target.closest('#ab-model') && !e.target.closest('[data-open-picker]')) closePicker();
  });

  function toast(msg) {
    if (typeof window.showToast === 'function') return window.showToast(msg);
    let t = $('#amni-toast');
    if (!t) { t = document.createElement('div'); t.id = 'amni-toast'; t.className = 'amni-toast'; document.body.appendChild(t); }
    t.textContent = msg; t.classList.add('show'); clearTimeout(t._h); t._h = setTimeout(() => t.classList.remove('show'), 2600);
  }

  window.AmniUI = { refresh, openPicker, closePicker, toggleOffline, showShortcuts, state };
  const boot = () => { ensureBar(); refresh(true); };
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot); else boot();
  const prov = $('#cfg-provider');
  if (prov) prov.addEventListener('change', () => setTimeout(() => refresh(false), 400));
  const mdl = $('#cfg-model');
  if (mdl) mdl.addEventListener('change', () => setTimeout(() => refresh(false), 400));
})();
