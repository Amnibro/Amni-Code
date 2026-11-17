"""v3.0.0: current model ids, offline-first assets, offline end-to-end run."""
import os, re, shutil, subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HTML = (ROOT / 'static' / 'index.html').read_text(encoding='utf-8')
TABLES = (ROOT / 'ui' / 'data' / 'tables.json').read_text(encoding='utf-8')
MAIN = (ROOT / 'src' / 'main.rs').read_text(encoding='utf-8')
PROV = (ROOT / 'src' / 'providers.rs').read_text(encoding='utf-8')
STALE = ['claude-sonnet-4-20250514', 'claude-opus-4-20250514', 'claude-haiku-4-20250514', "'gpt-4o'", 'gemini-1.5-pro',
         'grok-beta', 'qwen3.5', 'grok-4-1-fast-reasoning', 'gemini-2.0-flash', 'Granite 4.1']


def test_no_remote_assets_in_ui():
    for src in re.findall(r'<(?:script|link)[^>]+(?:src|href)="([^"]+)"', HTML):
        assert not src.startswith(('http://', 'https://', '//')), f'remote asset {src}'
    assert 'cdnjs.cloudflare.com' not in HTML and 'fonts.googleapis' not in HTML
    assert '"paths\\":{\\"vs\\":\\"/vendor/monaco/vs\\"}' in (ROOT / 'ui' / 'editor.he').read_text(encoding='utf-8')
    assert '<script src="/vendor/monaco/vs/loader.js"></script>' in HTML
    assert (ROOT / 'static' / 'vendor' / 'monaco' / 'vs' / 'loader.js').exists()
    assert (ROOT / 'static' / 'vendor' / 'monaco' / 'vs' / 'editor' / 'editor.main.js').exists()
    assert '/assets/ui.css' in HTML and '/assets/start.js' in HTML and '/assets/ui.js' not in HTML
    for a in ('/assets/start.js', '/assets/helium.js', '/assets/ui.wasm'):
        assert f'.route("{a}"' in MAIN, a
    assert (ROOT / 'static' / 'ui.wasm').read_bytes()[:4] == b'\0asm'


def test_no_stale_model_ids():
    for s in STALE:
        assert s not in HTML, f'stale {s} in index.html'
        assert s not in TABLES, f'stale {s} in tables.json'
        assert s not in MAIN, f'stale {s} in main.rs'
    for s in ['claude-opus-5-5', 'claude-sonnet-5-5', 'claude-haiku-5-5', 'claude-fable-5-2', 'gpt-6-astra', 'gemini-3.8-flash', 'grok-4.7']:
        assert s in PROV and s in TABLES, s


def test_suggested_local_models():
    for s in ['ibm-granite/granite-4.2-3b-GGUF', 'unsloth/Qwen3.8-27B-GGUF', 'ctx: 131072', 'ctx: 262144']:
        assert s in PROV, s
    assert 'hf.co/ibm-granite/granite-4.2-3b-GGUF' in TABLES


def test_offline_switches():
    assert '"--offline"' in MAIN and 'AMNI_OFFLINE' in MAIN
    assert 'is disabled in Offline mode' in MAIN
    assert '"--serve"' in MAIN and 'AMNI_NO_SELF_UPDATE' in MAIN
    assert '/api/local-servers' in MAIN and '/api/providers' in MAIN


def test_readme_links():
    readme = (ROOT / 'README.md').read_text(encoding='utf-8')
    assert 'github.com/anmire/Amni-Code' not in readme
    assert 'github.com/Amnibro/Amni-Code' in readme
    assert 'amni-scient.com/amni-code' in readme


def test_offline_e2e():
    """Full UI turn against local stub servers inside a loopback-only network namespace."""
    binary = ROOT / 'target' / 'debug' / 'amni'
    if os.geteuid() != 0 or not shutil.which('unshare') or not shutil.which('node') or not binary.exists():
        import pytest
        pytest.skip('needs root, unshare, node+playwright and a built target/debug/amni')
    r = subprocess.run([str(ROOT / 'tests' / 'e2e' / 'offline_e2e.sh'), str(binary)], capture_output=True, text=True, timeout=180)
    print(r.stdout, r.stderr)
    assert r.returncode == 0 and 'PASS' in r.stdout and 'external requests: 0' in r.stdout
