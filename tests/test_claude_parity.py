import re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
html=(ROOT/'static'/'index.html').read_text(encoding='utf-8')
start=(ROOT/'static'/'start.js').read_text(encoding='utf-8')
main=(ROOT/'src'/'main.rs').read_text(encoding='utf-8')
app=(ROOT/'ui'/'app.he').read_text(encoding='utf-8')
look=(ROOT/'ui'/'look.he').read_text(encoding='utf-8')
def test_default_amniscient_ivory():
    assert "s === 'braid' ? 'braid' : 'amniscient'" in start
    assert "t === 'dark' ? 'dark' : 'light'" in start
    assert 'return "amniscient"' in look and 'return "braid"' in look
    assert 'return "light"' in look
def test_side_panel_layout():
    for s in ['id="tool-panel"','data-act="tools.tab" data-arg="files"','data-arg="changes"','data-arg="term"','data-arg="editor"','data-arg="tasks"','id="tool-split"','id="tool-pop"','id="context-snow"','id="amni-logo"','/assets/amniscient-logo.png','id="sb-ctx"','id="sb-atex"','id="sidebar-tree"','id="editor-col"','id="diff-col"']:
        assert s in html, s
    assert 'tools::init' in app and 'owner == 7' in app
def test_backend_routes_and_slash():
    for s in ['/api/session','/api/session/reset','/api/approve','/api/tasks','/api/doctor','/api/term','/api/changes','amniscient-logo.png','SlashCmd::Ship','SlashCmd::Tasks','SlashCmd::Doctor','SlashCmd::CodeMode','CODE_MODE_PROMPT','parse_early','has_upstream']:
        assert s in main, s
    raw=(ROOT/'static'/'amniscient-logo.png').read_bytes()
    assert len(raw) > 1000 and raw[:8] == b'\x89PNG\r\n\x1a\n'
def test_friction_fixes():
    tools=(ROOT/'ui'/'tools.he').read_text(encoding='utf-8')
    chat=(ROOT/'ui'/'chat.he').read_text(encoding='utf-8')
    seats=(ROOT/'src'/'cli_seats.rs').read_text(encoding='utf-8')
    assert "class='task-row'" in tools and "tools.stop" in tools and "No tasks yet." in tools
    assert "approve-banner" in chat and "/api/approve" in chat and "resume_from_session" in chat and "on_boot" in chat
    assert "--permission-prompt-tool" in seats and "Feed::Jsonl" in seats and "EDIT_ALLOW" in seats and "Bash(hcn *)" in seats
    assert '"approval"' in seats and 'StreamEv::Permit' in seats
    assert ".approve-banner" in html
    assert "unknown option" in main and "has_upstream" in main and "open_desktop" in main
if __name__=='__main__':
    test_default_amniscient_ivory(); test_side_panel_layout(); test_backend_routes_and_slash(); test_friction_fixes(); print('ok')
