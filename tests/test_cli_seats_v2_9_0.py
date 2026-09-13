"""v2.9.0: subscription CLI seats + Braid-grade markdown.

Source-level checks (no build needed) plus a live pass against a running binary when
AMNI_CODE_URL is set (e.g. http://127.0.0.1:3131). Run: python tests/test_cli_seats_v2_9_0.py
"""
import os,sys,json,re,subprocess,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
RS=(ROOT/'src'/'cli_seats.rs').read_text(encoding='utf-8')
MAIN=(ROOT/'src'/'main.rs').read_text(encoding='utf-8')
UI=(ROOT/'static'/'index.html').read_text(encoding='utf-8')
SEATS=('claude-code','codex','grok-cli','antigravity','copilot','cursor','opencode')
FAILS=[]
def ck(name,cond,detail=''):
    FAILS.append(name+((' — '+str(detail)) if detail else '')) if not cond else None
    print(('  ok   ' if cond else '  FAIL ')+name+(('  '+str(detail)) if (detail and not cond) else ''))
def test_seat_table():
    for s in SEATS:ck('seat '+s+' in table','id:"'+s+'"' in RS)
    ck('main.rs mounts the module','mod cli_seats;' in MAIN)
    ck('/api/seats route','.route("/api/seats", get(handle_seats))' in MAIN)
    ck('models endpoint answers for seats','cli_seats::seat(&cfg.provider)' in MAIN)
    ck('agent loop branches to run_cli_seat','cli_seats::run(seat,&config.model,&config.mode,config.auto_approve,&cwd_path' in MAIN)
    ck('seat provider defaults need no key','p if cli_seats::is_seat(p) => ("default".to_string(), String::new())' in MAIN)
def test_safety_flags():
    PROD=RS.split('#[cfg(test)]')[0]
    ck('no yolo anywhere','--yolo' not in PROD and '"-f"' not in PROD)
    ck('claude plan mode','"plan"=>"plan","auto"=>"bypassPermissions",_=>"acceptEdits"' in RS)
    ck('codex plan is read-only sandbox','"read-only"' in RS)
    ck('prompt never on argv for stdin seats','Feed::Stdin' in RS and 'write_all(body.as_bytes())' in RS)
    ck('long argv prompts go to a task file','feed==Feed::Argv&&long' in RS)
    ck('env override must be a file','.filter(|p|p.is_file())' in RS)
    ck('windows .cmd wrapped in cmd /c','c.arg("/c").arg(&ps)' in RS)
    ck('interrupt kills the child','child.kill().await' in RS)
def test_ui():
    ck('provider picker has the subscription group','Subscription CLIs' in UI)
    for s in SEATS:ck('picker option '+s,'<option value="'+s+'">' in UI)
    for s in SEATS:ck('CORE_PROVIDERS has '+s,re.search(r"[\'\"]?"+re.escape(s)+r"[\'\"]?:\{url:'',key:false,cli:true",UI) is not None)
    ck('seat status line','async function seatStatus(p)' in UI and "fetch('/api/seats')" in UI)
    ck('cli hides the key field','const showKey=!!c.key&&!c.cli' in UI)
    ck('thinking lines stream to the bubble',"case 'thinking':\n              if(data.text)updateThinking(data.text);" in UI)
    ck('renderMd is the ported engine','window.renderMd=t=>{try{return md(t)}' in UI and 'function renderMd(text){' not in UI)
    ck('fences get a copy button','class="cb-copy" onclick="copyCode(this)"' in UI and 'window.copyCode=' in UI)
    ck('tables/blockquote/heading CSS on theme vars','.msg.assistant table{' in UI and '.msg.assistant blockquote{' in UI and '.msg.assistant h1{' in UI)
    ck('inline sentinels survived',UI.count('')==2 and UI.count('')==2)
def test_control_system():
    ck('control tokens',':root{--ctl-h:32px;--ctl-r:var(--radius,8px);--eyebrow:' in UI)
    ck('control block is last in the sheet',UI.rfind('/* v2.9.0 control system:')>UI.rfind('.interrupt-btn{background:#ff4444') and UI.rfind('/* v2.9.0 control system:')>UI.rfind('.settings-panel input,.settings-panel select{width:100%'))
    ck('header is one row','.header{padding:0 18px;height:54px;gap:10px;flex-wrap:nowrap;overflow:hidden}' in UI and '.header-actions{gap:6px;flex-wrap:nowrap' in UI)
    ck('wordmark never wraps','.header h1{font:800 .98rem/1 var(--site-sans);font-stretch:112%;letter-spacing:.06em;text-transform:uppercase;white-space:nowrap' in UI)
    ck('duplicate theme toggle hidden beside the skin switch','.header-actions .skin-switch~#theme-toggle{display:none}' in UI)
    ck('composer buttons share one size','.interrupt-btn,.steer-btn,.send-btn{width:34px;height:34px;border-radius:var(--ctl-r)' in UI and '.interrupt-btn,.steer-btn{background:transparent;border:1px solid var(--border)' in UI)
    ck('status bar quiet and in flow','.status-bar{background:var(--panel);border-top:1px solid var(--border)' in UI and 'body{display:flex;flex-direction:column}body>.app{flex:1;min-height:0;height:auto}' in UI)
    ck('selects own their chrome','.settings-panel select,.pane-header select{appearance:none' in UI)
    ck('agent mode labels translated','agent_mode:"Agent mode",agent_mode_desc:' in UI)
def test_renderer_in_node():
    m=re.search(r'\(function\(\)\{\nconst E=\{"&":"&amp;".*?window\.renderMd=t=>\{try\{return md\(t\)\}catch\(e\)\{return "<p>"\+esc\(t\)\+"</p>"\}\};\n\}\)\(\);',UI,re.S)
    ck('renderer IIFE found',m is not None)
    if not m:return
    js=m.group(0)+'''
const out=renderMd("# Title\\n\\nA **bold** line with `code` and 3 numbers 12 45.\\n\\n| a | b |\\n|---|:-:|\\n| 1 | 2 |\\n\\n- one\\n  - two\\n- [x] done\\n\\n```py\\ndef f(x):\\n    return x # hi\\n```\\n\\n> quoted\\n\\nhttps://example.com ok");
console.log(JSON.stringify(out));'''
    try:r=subprocess.run(['node','-e','globalThis.window=globalThis;'+js],capture_output=True,text=True,timeout=30)
    except Exception as e:ck('node available',False,e);return
    out=json.loads(r.stdout.strip() or '""') if r.returncode==0 else ''
    ck('node ran the renderer',r.returncode==0,r.stderr[-300:])
    ck('heading',"<h1>Title</h1>" in out)
    ck('bold + inline code',"<strong>bold</strong>" in out and "<code>code</code>" in out)
    ck('digits untouched (sentinel bug guard)',"3 numbers 12 45." in out and out.count("<code>")==1)
    ck('table with alignment','<table>' in out and 'style="text-align:center"' in out)
    ck('nested list + task','<ul><li>one<ul><li>two</li></ul></li>' in out and 'type="checkbox" disabled checked' in out)
    ck('fence with copy + highlight','class="cb-copy"' in out and 'hl-kw' in out and 'hl-com' in out)
    ck('blockquote','<blockquote><p>quoted</p></blockquote>' in out)
    ck('autolink','<a href="https://example.com"' in out)
def test_live():
    base=os.environ.get('AMNI_CODE_URL')
    if not base:print('  skip live (set AMNI_CODE_URL)');return
    j=json.loads(urllib.request.urlopen(base+'/api/seats',timeout=5).read())
    ck('live /api/seats lists 7',len(j.get('seats',[]))==7)
    ck('live seat shape',all(k in j['seats'][0] for k in ('id','label','installed','path','hint','get','models')))
    html=urllib.request.urlopen(base+'/',timeout=5).read().decode('utf-8')
    ck('live UI carries the picker group','Subscription CLIs' in html and 'cb-copy' in html)
if __name__=='__main__':
    for fn in (test_seat_table,test_safety_flags,test_ui,test_control_system,test_renderer_in_node,test_live):
        print('=== '+fn.__name__);fn()
    print('\n'+('ALL OK' if not FAILS else str(len(FAILS))+' FAILED:\n  '+'\n  '.join(FAILS)))
    sys.exit(1 if FAILS else 0)
